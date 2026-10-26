//! Small Windows-only helpers: elevation check, "restart as administrator", opening a URL.

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_ELEVATION: u32 = 20;
const SW_SHOWNORMAL: i32 = 1;

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: isize, access: u32, token: *mut isize) -> i32;
    fn GetTokenInformation(token: isize, class: u32, info: *mut u32, len: u32, returned: *mut u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> isize;
    fn CloseHandle(handle: isize) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn ShellExecuteW(hwnd: isize, op: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32)
    -> isize;
}

/// True when this process runs with an elevated (administrator) token.
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = 0isize;
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let (mut elevation, mut returned) = (0u32, 0u32);
        let ok = GetTokenInformation(token, TOKEN_ELEVATION, &mut elevation, 4, &mut returned) != 0;
        CloseHandle(token);
        ok && elevation != 0
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

/// Starts a new elevated copy of this exe (the UAC prompt appears). The caller closes the current
/// window only when this returns `Ok`.
pub fn relaunch_as_admin(args: &str) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let (verb, file, params) = (wide("runas"), wide(&exe.display().to_string()), wide(args));
    let code = unsafe { ShellExecuteW(0, verb.as_ptr(), file.as_ptr(), params.as_ptr(), std::ptr::null(), SW_SHOWNORMAL) };
    // Values above 32 mean success; otherwise the user declined the UAC prompt or it failed.
    if code > 32 { Ok(()) } else { Err("Administrator restart was cancelled or failed".into()) }
}

/// Opens a URL in the default browser.
pub fn open_url(url: &str) {
    let _ = Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}
