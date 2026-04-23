use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, mpsc};

use anyhow::{Context, anyhow};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::oneshot;

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
