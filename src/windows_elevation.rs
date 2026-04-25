use std::ffi::{OsStr, OsString};
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;

use anyhow::{Context, anyhow};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows_sys::Win32::Security::{
    GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONERROR, MB_OK, MessageBoxW, SW_SHOWNORMAL,
};

pub fn ensure_elevated_for_tun() -> anyhow::Result<bool> {
    if is_elevated_process()? {
        return Ok(true);
    }

    relaunch_as_administrator()?;
    Ok(false)
}

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

fn join_windows_args(args: impl Iterator<Item = OsString>) -> String {
    args.map(|arg| quote_windows_arg(&arg))
        .collect::<Vec<_>>()
        .join(" ")
}

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

fn wide_from_os(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    wide_from_os(OsStr::new(value))
}

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
