use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
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
use windows_sys::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
};
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

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "main-tray";
const TRAY_SHOW_ID: &str = "tray_show_main";
const TRAY_RESET_CLOSE_ID: &str = "tray_reset_close_behavior";
const TRAY_QUIT_ID: &str = "tray_quit";
const CLOSE_CONFIRM_EVENT: &str = "vnt://confirm-close";
const SHELL_CONFIG_FILE: &str = "desktop-shell.toml";

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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CloseBehavior {
    #[default]
    Ask,
    MinimizeToTray,
    Close,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CloseDecision {
    MinimizeToTray,
    Close,
}

impl From<CloseDecision> for CloseBehavior {
    fn from(value: CloseDecision) -> Self {
        match value {
            CloseDecision::MinimizeToTray => Self::MinimizeToTray,
            CloseDecision::Close => Self::Close,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DesktopShellConfig {
    #[serde(default)]
    close_behavior: CloseBehavior,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopShellInfo {
    custom_titlebar: bool,
}

struct DesktopShellState {
    config_path: Option<PathBuf>,
    close_behavior: Mutex<CloseBehavior>,
    close_prompt_open: AtomicBool,
    exiting: AtomicBool,
}

impl DesktopShellState {
    fn new(app: &AppHandle) -> Self {
        let config_path = resolve_shell_config_path(app).map_err(|e| {
            log::warn!("failed to resolve shell config path: {e:?}");
            e
        });

        let close_behavior = config_path
            .as_ref()
            .ok()
            .and_then(|path| load_shell_config(path).map(|cfg| cfg.close_behavior))
            .unwrap_or_default();

        Self {
            config_path: config_path.ok(),
            close_behavior: Mutex::new(close_behavior),
            close_prompt_open: AtomicBool::new(false),
            exiting: AtomicBool::new(false),
        }
    }

    fn close_behavior(&self) -> CloseBehavior {
        self.close_behavior
            .lock()
            .map(|guard| *guard)
            .unwrap_or_default()
    }

    fn update_close_behavior(&self, behavior: CloseBehavior) -> anyhow::Result<()> {
        if let Ok(mut current) = self.close_behavior.lock() {
            *current = behavior;
        }

        if let Some(path) = &self.config_path {
            save_shell_config(
                path,
                &DesktopShellConfig {
                    close_behavior: behavior,
                },
            )?;
        }

        Ok(())
    }

    fn begin_close_prompt(&self) -> bool {
        !self.close_prompt_open.swap(true, Ordering::SeqCst)
    }

    fn clear_close_prompt(&self) {
        self.close_prompt_open.store(false, Ordering::SeqCst);
    }

    fn mark_exiting(&self) {
        self.clear_close_prompt();
        self.exiting.store(true, Ordering::SeqCst);
    }

    fn is_exiting(&self) -> bool {
        self.exiting.load(Ordering::SeqCst)
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
        .invoke_handler(tauri::generate_handler![
            desktop_shell_info,
            window_minimize,
            window_is_maximized,
            window_toggle_maximize,
            request_close_window,
            dismiss_close_request,
            resolve_close_request
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();

            app.manage(HttpServerShutdown(Mutex::new(Some(shutdown_tx))));
            app.manage(DesktopShellState::new(&app_handle));
            create_tray(app)?;
            let server_addr = start_http_server(app_handle.clone(), shutdown_rx)?;
            create_main_window(&app_handle, server_addr)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();

                let Some(shell_state) = window.try_state::<DesktopShellState>() else {
                    return;
                };

                if let Err(e) =
                    handle_close_request(window, &window.app_handle(), shell_state.inner())
                {
                    log::error!("failed to handle close request: {e:?}");
                }
            }
            tauri::WindowEvent::Destroyed => {
                if let Some(shutdown) = window.try_state::<HttpServerShutdown>() {
                    shutdown.shutdown();
                }
            }
            _ => {}
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

    let mut builder =
        WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::External(url.parse()?))
            .title("VNT2")
            .inner_size(1180.0, 820.0)
            .min_inner_size(960.0, 640.0)
            .resizable(true)
            .visible(true);

    #[cfg(target_os = "windows")]
    {
        builder = builder.decorations(false).shadow(true);
    }

    builder.build().context("failed to create main window")?;

    Ok(())
}

fn create_tray(app: &tauri::App) -> anyhow::Result<()> {
    let show_item = MenuItem::with_id(app, TRAY_SHOW_ID, "显示主窗口", true, None::<&str>)?;
    let reset_close_item = MenuItem::with_id(
        app,
        TRAY_RESET_CLOSE_ID,
        "关闭时重新询问",
        true,
        None::<&str>,
    )?;
    let quit_item = MenuItem::with_id(app, TRAY_QUIT_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &PredefinedMenuItem::separator(app)?,
            &reset_close_item,
            &PredefinedMenuItem::separator(app)?,
            &quit_item,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("VNT2")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if event.id() == TRAY_SHOW_ID {
                if let Err(e) = show_main_window(app) {
                    log::error!("failed to restore main window from tray: {e:?}");
                }
                return;
            }

            if event.id() == TRAY_RESET_CLOSE_ID {
                let shell_state = app.state::<DesktopShellState>();
                if let Err(e) = shell_state.update_close_behavior(CloseBehavior::Ask) {
                    log::error!("failed to reset close behavior: {e:?}");
                }
                return;
            }

            if event.id() == TRAY_QUIT_ID {
                let shell_state = app.state::<DesktopShellState>();
                exit_application(app, shell_state.inner());
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(e) = show_main_window(tray.app_handle()) {
                    log::error!("failed to restore main window from tray click: {e:?}");
                }
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app).context("failed to create tray icon")?;

    Ok(())
}

fn handle_close_request(
    window: &tauri::Window,
    app: &AppHandle,
    shell_state: &DesktopShellState,
) -> anyhow::Result<()> {
    if shell_state.is_exiting() {
        return Ok(());
    }

    match shell_state.close_behavior() {
        CloseBehavior::Ask => emit_close_prompt(window, shell_state),
        CloseBehavior::MinimizeToTray => hide_window_to_tray(window),
        CloseBehavior::Close => {
            exit_application(app, shell_state);
            Ok(())
        }
    }
}

fn emit_close_prompt(
    window: &tauri::Window,
    shell_state: &DesktopShellState,
) -> anyhow::Result<()> {
    if !shell_state.begin_close_prompt() {
        return Ok(());
    }

    if let Err(e) = window.emit(CLOSE_CONFIRM_EVENT, ()) {
        shell_state.clear_close_prompt();
        return Err(e.into());
    }

    Ok(())
}

fn hide_window_to_tray(window: &tauri::Window) -> anyhow::Result<()> {
    window.hide().context("failed to hide main window")?;
    Ok(())
}

fn show_main_window(app: &AppHandle) -> anyhow::Result<()> {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .context("main window is not available")?;

    window.show().context("failed to show main window")?;
    let _ = window.unminimize();
    let _ = window.set_focus();

    Ok(())
}

fn exit_application(app: &AppHandle, shell_state: &DesktopShellState) {
    shell_state.mark_exiting();

    if let Some(shutdown) = app.try_state::<HttpServerShutdown>() {
        shutdown.shutdown();
    }

    app.exit(0);
}

fn resolve_shell_config_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    let mut config_dir = app
        .path()
        .app_config_dir()
        .context("failed to resolve app config directory")?;
    config_dir.push(SHELL_CONFIG_FILE);
    Ok(config_dir)
}

fn load_shell_config(path: &Path) -> Option<DesktopShellConfig> {
    if !path.exists() {
        return None;
    }

    match fs::read_to_string(path)
        .context("failed to read desktop shell config")
        .and_then(|content| {
            toml::from_str(&content).context("failed to parse desktop shell config")
        }) {
        Ok(config) => Some(config),
        Err(e) => {
            log::warn!("failed to load desktop shell config: {e:?}");
            None
        }
    }
}

fn save_shell_config(path: &Path, config: &DesktopShellConfig) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("failed to create shell config directory")?;
    }

    let content = toml::to_string_pretty(config).context("failed to serialize shell config")?;
    fs::write(path, content).context("failed to write shell config")?;

    Ok(())
}

#[tauri::command]
fn desktop_shell_info() -> DesktopShellInfo {
    DesktopShellInfo {
        custom_titlebar: cfg!(target_os = "windows"),
    }
}

#[tauri::command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn window_is_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

#[tauri::command]
fn window_toggle_maximize(window: tauri::Window) -> Result<bool, String> {
    let is_maximized = window.is_maximized().map_err(|e| e.to_string())?;

    if is_maximized {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
fn request_close_window(
    window: tauri::Window,
    app: AppHandle,
    shell_state: State<'_, DesktopShellState>,
) -> Result<(), String> {
    handle_close_request(&window, &app, shell_state.inner()).map_err(|e| e.to_string())
}

#[tauri::command]
fn dismiss_close_request(shell_state: State<'_, DesktopShellState>) {
    shell_state.clear_close_prompt();
}

#[tauri::command]
fn resolve_close_request(
    action: CloseDecision,
    remember: bool,
    window: tauri::Window,
    app: AppHandle,
    shell_state: State<'_, DesktopShellState>,
) -> Result<(), String> {
    let shell_state = shell_state.inner();
    shell_state.clear_close_prompt();

    if remember {
        shell_state
            .update_close_behavior(action.into())
            .map_err(|e| e.to_string())?;
    }

    match action {
        CloseDecision::MinimizeToTray => {
            hide_window_to_tray(&window).map_err(|e| e.to_string())?;
        }
        CloseDecision::Close => exit_application(&app, shell_state),
    }

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
