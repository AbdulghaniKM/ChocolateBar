use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::os::windows::process::CommandExt;
use std::process::Command;

use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYEVENTF_KEYUP, VK_MENU};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow,
    SetWindowPos, ShowWindow, GWL_EXSTYLE, GW_OWNER, SWP_NOACTIVATE, SWP_NOZORDER, SW_MAXIMIZE, SW_RESTORE,
    WS_EX_TOOLWINDOW,
};

use super::layout::{target_rect, Rect};
use crate::core::settings::WindowLayout;
use crate::window::native;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub type WindowId = isize;

pub fn id_of(hwnd: HWND) -> WindowId {
    hwnd.0 as WindowId
}

fn handle(id: WindowId) -> HWND {
    HWND(id as *mut c_void)
}

/// Start-menu app ids (desktop or Store) and plain paths both open through the shell, exactly like
/// clicking them in Start or Explorer.
pub fn launch(app_id: &str) -> std::io::Result<()> {
    let target = if looks_like_path(app_id) { app_id.to_string() } else { format!("shell:AppsFolder\\{app_id}") };
    Command::new("explorer.exe").arg(target).creation_flags(CREATE_NO_WINDOW).spawn().map(drop)
}

fn looks_like_path(app_id: &str) -> bool {
    let lower = app_id.to_lowercase();
    (lower.len() > 2 && lower.as_bytes()[1] == b':') || lower.starts_with("\\\\")
}

pub fn app_windows() -> Vec<WindowId> {
    let mut all: Vec<WindowId> = Vec::new();
    unsafe extern "system" fn collect(hwnd: HWND, list: LPARAM) -> BOOL {
        let list = unsafe { &mut *(list.0 as *mut Vec<WindowId>) };
        list.push(id_of(hwnd));
        BOOL(1)
    }
    let _ = unsafe { EnumWindows(Some(collect), LPARAM(&mut all as *mut Vec<WindowId> as isize)) };
    all.into_iter().filter(|id| is_app_window(*id)).collect()
}

pub fn snapshot() -> HashSet<WindowId> {
    app_windows().into_iter().collect()
}

pub fn exists(id: WindowId) -> bool {
    let hwnd = handle(id);
    unsafe { IsWindow(Some(hwnd)) }.as_bool() && unsafe { IsWindowVisible(hwnd) }.as_bool()
}

pub fn is_app_window(id: WindowId) -> bool {
    let hwnd = handle(id);
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() || unsafe { GetWindowTextLengthW(hwnd) } == 0 {
        return false;
    }
    if unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok_and(|owner| !owner.is_invalid()) {
        return false;
    }
    let extended = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
    if extended & WS_EX_TOOLWINDOW.0 != 0 || is_cloaked(hwnd) {
        return false;
    }
    let mut process_id = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)) };
    process_id != std::process::id()
}

pub fn title_of(id: WindowId) -> String {
    let hwnd = handle(id);
    let length = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    let mut buffer = vec![0u16; length + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..copied])
}

/// The executable's file name, lower-cased (for example `code.exe`).
pub fn process_name_of(id: WindowId) -> String {
    let mut process_id = 0u32;
    unsafe { GetWindowThreadProcessId(handle(id), Some(&mut process_id)) };
    let Ok(process) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }) else {
        return String::new();
    };
    let mut buffer = vec![0u16; 1024];
    let mut length = buffer.len() as u32;
    let named = unsafe { QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length) };
    let _ = unsafe { CloseHandle(process) };
    if named.is_err() {
        return String::new();
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    path.rsplit(['\\', '/']).next().unwrap_or_default().to_lowercase()
}

/// Windows only lets the foreground app hand over focus; a synthetic Alt tap lifts that lock for our request.
pub fn bring_to_front(id: WindowId) {
    let hwnd = handle(id);
    if unsafe { IsIconic(hwnd) }.as_bool() {
        let _ = unsafe { ShowWindow(hwnd, SW_RESTORE) };
    }
    unsafe {
        keybd_event(VK_MENU.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
    let _ = unsafe { SetForegroundWindow(hwnd) };
}

fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    let result = unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&mut cloaked as *mut u32).cast(), size_of::<u32>() as u32)
    };
    result.is_ok() && cloaked != 0
}

pub fn arrange(id: WindowId, layout: WindowLayout) {
    let hwnd = handle(id);
    if layout == WindowLayout::Maximize {
        let _ = unsafe { ShowWindow(hwnd, SW_MAXIMIZE) };
        return;
    }
    let Some(target) = work_area(hwnd).and_then(|work| target_rect(work, layout)) else { return };
    let _ = unsafe { ShowWindow(hwnd, SW_RESTORE) };
    let (left, top, right, bottom) = invisible_borders(hwnd);
    let result = unsafe {
        SetWindowPos(
            hwnd,
            None,
            target.x - left,
            target.y - top,
            target.width + left + right,
            target.height + top + bottom,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
    if let Err(error) = result {
        log::warn!("arranging window failed: {error}");
    }
}

fn work_area(hwnd: HWND) -> Option<Rect> {
    let info = native::monitor_info(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) })?;
    let work = info.rcWork;
    Some(Rect { x: work.left, y: work.top, width: work.right - work.left, height: work.bottom - work.top })
}

/// Windows 10/11 frames include an invisible resize border; offsetting by it makes snapped windows meet edge to edge.
fn invisible_borders(hwnd: HWND) -> (i32, i32, i32, i32) {
    let mut outer = RECT::default();
    let mut visible = RECT::default();
    let measured = unsafe { GetWindowRect(hwnd, &mut outer) }.is_ok()
        && unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS,
                (&mut visible as *mut RECT).cast(),
                size_of::<RECT>() as u32,
            )
        }
        .is_ok();
    if !measured {
        return (0, 0, 0, 0);
    }
    (visible.left - outer.left, visible.top - outer.top, outer.right - visible.right, outer.bottom - visible.bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tells_paths_apart_from_start_menu_ids() {
        assert!(looks_like_path("C:\\Games\\game.exe"));
        assert!(looks_like_path("\\\\server\\share\\tool.exe"));
        assert!(!looks_like_path("Microsoft.MinecraftUWP_8wekyb3d8bbwe!App"));
        assert!(!looks_like_path("com.squirrel.Discord.Discord"));
    }
}
