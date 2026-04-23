use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, mpsc};

use anyhow::{Context, anyhow};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

#[cfg(windows)]
use std::ffi::{OsStr, OsString};
#[cfg(windows)]
use std::mem::size_of;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND};
#[cfg(windows)]
use windows_sys::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::ShellExecuteW;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWNORMAL,
};

#[cfg(debug_assertions)]
const HTTP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 19099);

#[cfg(not(debug_assertions))]
const HTTP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

#[cfg(debug_assertions)]
const WINDOW_URL: &str = "http://127.0.0.1:5173";

struct HttpServerShutdown(Mutex<Option<oneshot::Sender<()>>>);

impl HttpServerShutdown {
    fn shutdown(&self) {
        if let Ok(mut sender) = self.0.lock() {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }
    }
}

pub fn run() {
    vnt2::log::log_init("vnt_app");

    #[cfg(windows)]
    match ensure_windows_elevated() {
        Ok(ElevationState::AlreadyElevated) => {}
        Ok(ElevationState::Relaunched) => return,
        Err(e) => {
            log::error!("failed to obtain administrator privileges: {e:?}");
            return;
        }
    }

    log::info!("version: {:?}", env!("CARGO_PKG_VERSION"));

    #[cfg(windows)]
    vnt2::extract_wintun_dll::extract_wintun();

    if let Err(e) = run_app() {
        log::error!("vnt-app startup failed: {e:?}");
    }
}

fn run_app() -> anyhow::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();

            app.manage(HttpServerShutdown(Mutex::new(Some(shutdown_tx))));
            let server_addr = start_http_server(app_handle.clone(), shutdown_rx)?;
            create_main_window(&app_handle, server_addr)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(shutdown) = window.try_state::<HttpServerShutdown>() {
                    shutdown.shutdown();
                }
            }
        })
        .run(tauri::generate_context!())
        .context("failed to run Tauri application")?;

    Ok(())
}

fn start_http_server(
    app: AppHandle,
    shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<SocketAddr> {
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let ready_tx = Arc::new(Mutex::new(Some(ready_tx)));
    let ready_tx_for_server = Arc::clone(&ready_tx);
    let ready_tx_for_error = Arc::clone(&ready_tx);

    tauri::async_runtime::spawn(async move {
        let result = vnt_web::run_http_server_with_shutdown(
            HTTP_ADDR,
            None,
            move |addr| {
                if let Ok(mut sender) = ready_tx_for_server.lock() {
                    if let Some(sender) = sender.take() {
                        let _ = sender.send(Ok(addr));
                    }
                }
            },
            async move {
                let _ = shutdown_rx.await;
            },
        )
        .await;

        if let Err(e) = result {
            let msg = e.to_string();
            if let Ok(mut sender) = ready_tx_for_error.lock() {
                if let Some(sender) = sender.take() {
                    let _ = sender.send(Err(msg.clone()));
                }
            }
            log::error!("HTTP server stopped: {msg}");
            let _ = app.emit("vnt-http-error", msg);
        }
    });

    ready_rx
        .recv()
        .context("HTTP server failed before reporting its local address")?
        .map_err(|msg| anyhow!(msg))
}

fn create_main_window(app: &AppHandle, server_addr: SocketAddr) -> anyhow::Result<()> {
    let url = window_url(server_addr);

    WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url.parse()?))
        .title("VNT2")
        .inner_size(1180.0, 820.0)
        .min_inner_size(960.0, 640.0)
        .resizable(true)
        .visible(true)
        .build()
        .context("failed to create main window")?;

    Ok(())
}

#[cfg(debug_assertions)]
fn window_url(_server_addr: SocketAddr) -> String {
    WINDOW_URL.to_string()
}

#[cfg(not(debug_assertions))]
fn window_url(server_addr: SocketAddr) -> String {
    format!("http://{server_addr}")
}

#[cfg(windows)]
enum ElevationState {
    AlreadyElevated,
    Relaunched,
}

#[cfg(windows)]
fn ensure_windows_elevated() -> anyhow::Result<ElevationState> {
    if is_elevated_process()? {
        return Ok(ElevationState::AlreadyElevated);
    }

    relaunch_as_administrator()?;
    Ok(ElevationState::Relaunched)
}

#[cfg(windows)]
fn is_elevated_process() -> anyhow::Result<bool> {
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(anyhow!("OpenProcessToken failed"));
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        ) != 0;
        let _ = CloseHandle(token);

        if !ok {
            return Err(anyhow!("GetTokenInformation(TokenElevation) failed"));
        }

        Ok(elevation.TokenIsElevated != 0)
    }
}

#[cfg(windows)]
fn relaunch_as_administrator() -> anyhow::Result<()> {
    let exe_path = std::env::current_exe().context("failed to resolve current executable")?;
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    let params = join_windows_args(args.into_iter());

    let verb = wide_from_str("runas");
    let exe = wide_from_os(exe_path.as_os_str());
    let params = if params.is_empty() {
        None
    } else {
        Some(wide_from_str(&params))
    };

    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut::<std::ffi::c_void>() as HWND,
            verb.as_ptr(),
            exe.as_ptr(),
            params
                .as_ref()
                .map_or(std::ptr::null(), |value| value.as_ptr()),
            std::ptr::null(),
            SW_SHOWNORMAL,
        ) as isize
    };

    if result <= 32 {
        let message = "VNT2 需要管理员权限才能创建 TUN 设备。\n请允许 UAC 提权后重新启动。";
        show_elevation_error(message);
        return Err(anyhow!("ShellExecuteW(runas) failed with code {result}"));
    }

    Ok(())
}

#[cfg(windows)]
fn join_windows_args(args: impl Iterator<Item = OsString>) -> String {
    args.map(|arg| quote_windows_arg(&arg))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(windows)]
fn quote_windows_arg(arg: &OsStr) -> String {
    let value = arg.to_string_lossy();
    if value.is_empty() {
        return "\"\"".to_string();
    }

    if !value.contains([' ', '\t', '"']) {
        return value.into_owned();
    }

    let mut quoted = String::from("\"");
    let mut backslashes = 0usize;

    for ch in value.chars() {
        match ch {
            '\\' => backslashes += 1,
            '"' => {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                if backslashes > 0 {
                    quoted.push_str(&"\\".repeat(backslashes));
                    backslashes = 0;
                }
                quoted.push(ch);
            }
        }
    }

    if backslashes > 0 {
        quoted.push_str(&"\\".repeat(backslashes * 2));
    }

    quoted.push('"');
    quoted
}

#[cfg(windows)]
fn wide_from_os(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn wide_from_str(value: &str) -> Vec<u16> {
    wide_from_os(OsStr::new(value))
}

#[cfg(windows)]
fn show_elevation_error(message: &str) {
    let title = wide_from_str("VNT2");
    let content = wide_from_str(message);

    unsafe {
        let _ = MessageBoxW(
            std::ptr::null_mut::<std::ffi::c_void>() as HWND,
            content.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
