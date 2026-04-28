use anyhow::Context;
use std::fs;
#[cfg(target_os = "android")]
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

static DEVICE_ID_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_fallback_dir(path: PathBuf) {
    let _ = DEVICE_ID_DIR.set(path);
}

#[cfg(not(target_os = "android"))]
pub fn get_device_id() -> anyhow::Result<String> {
    match machine_uid::get() {
        Ok(id) => return Ok(id),
        Err(e) => {
            log::warn!("Failed to get system ID: {}. Using fallback.", e);
        }
    }

    get_fallback_id()
}

#[cfg(target_os = "android")]
pub fn get_device_id() -> anyhow::Result<String> {
    get_fallback_id()
}

fn get_fallback_id() -> anyhow::Result<String> {
    let path = fallback_device_id_path();

    if let Ok(content) = fs::read_to_string(&path) {
        let id = content.trim();
        if !id.is_empty() {
            return Ok(id.to_string());
        }
    }

    let new_id = uuid::Uuid::new_v4().to_string();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create device_id directory")?;
    }
    fs::write(&path, &new_id).context("Failed to write device_id file")?;

    Ok(new_id)
}

fn fallback_device_id_path() -> PathBuf {
    if let Some(path) = DEVICE_ID_DIR.get() {
        return path.join("device_id");
    }

    #[cfg(target_os = "android")]
    if let Some(home) = std::env::var_os("HOME") {
        return Path::new(&home).join("device_id");
    }

    PathBuf::from("device_id")
}
