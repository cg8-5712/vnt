use anyhow::{Context, anyhow};
use serde::Serialize;
use serde_json::Value;
use tauri::AppHandle;
use vnt_web::StartConfig;

use crate::mobile_vpn_plugin::MobileVpnExt;

#[cfg(target_os = "android")]
use tauri::Manager;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AndroidVpnCommandPayload {
    file_name: String,
    config_name: String,
    config_json: String,
}

#[tauri::command]
pub fn android_vpn_start(
    app: AppHandle,
    file_name: String,
    config_toml: String,
) -> Result<(), String> {
    let payload = build_android_vpn_payload(&app, file_name, config_toml).map_err(error_to_string)?;
    app.mobile_vpn().run_command::<()>("start", payload)
}

#[tauri::command]
pub fn android_vpn_restart(
    app: AppHandle,
    file_name: String,
    config_toml: String,
) -> Result<(), String> {
    let payload = build_android_vpn_payload(&app, file_name, config_toml).map_err(error_to_string)?;
    app.mobile_vpn().run_command::<()>("restart", payload)
}

#[tauri::command]
pub fn android_vpn_stop(app: AppHandle) -> Result<(), String> {
    app.mobile_vpn().run_command::<()>("stop", ())
}

#[tauri::command]
pub fn android_vpn_start_status(app: AppHandle) -> Result<Value, String> {
    app.mobile_vpn().run_command("startStatus", ())
}

#[tauri::command]
pub fn android_vpn_info(app: AppHandle) -> Result<Value, String> {
    app.mobile_vpn().run_command("info", ())
}

#[tauri::command]
pub fn android_vpn_peers(app: AppHandle) -> Result<Value, String> {
    app.mobile_vpn().run_command("peers", ())
}

#[tauri::command]
pub fn android_vpn_routes(app: AppHandle) -> Result<Value, String> {
    app.mobile_vpn().run_command("routes", ())
}

fn build_android_vpn_payload(
    app: &AppHandle,
    file_name: String,
    config_toml: String,
) -> anyhow::Result<AndroidVpnCommandPayload> {
    #[cfg(not(target_os = "android"))]
    let _ = app;

    if !is_valid_file_name(&file_name) {
        return Err(anyhow!("Invalid file name"));
    }

    let mut cfg: StartConfig =
        toml::from_str(&config_toml).context("Invalid TOML format in configuration")?;

    #[cfg(target_os = "android")]
    {
        let fallback_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data directory for device_id")?;
        vnt_core::utils::device_id::set_fallback_dir(fallback_dir);

        if cfg.device_id.as_deref().is_none_or(|value| value.trim().is_empty()) {
            cfg.device_id = Some(
                vnt_core::utils::device_id::get_device_id()
                    .context("failed to get device_id")?,
            );
        }
    }

    cfg.no_tun = false;

    if cfg.device_name.as_deref().is_some_and(|value| value.trim().is_empty()) {
        cfg.device_name = None;
    }

    let config_name = cfg
        .config_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| file_name.clone());

    let config_json =
        serde_json::to_string(&cfg).context("failed to serialize Android VPN configuration")?;

    Ok(AndroidVpnCommandPayload {
        file_name,
        config_name,
        config_json,
    })
}

fn is_valid_file_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && !file_name.contains("..")
        && !file_name.contains('/')
        && !file_name.contains('\\')
}

fn error_to_string(error: anyhow::Error) -> String {
    error.to_string()
}
