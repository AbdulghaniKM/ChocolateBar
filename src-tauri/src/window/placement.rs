use std::mem::size_of;

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, HMONITOR, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETSTATE, ABS_AUTOHIDE, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SPI_SETWORKAREA, SWP_FRAMECHANGED, SWP_NOACTIVATE,
};

use super::native;

const ISLAND_WIDTH_DIP: i32 = 480;
const ISLAND_HEIGHT_DIP: i32 = 360;
const TASKBAR_GAP_DIP: i32 = 8;
const BASE_DPI: i32 = 96;

#[derive(Debug, PartialEq, Eq)]
struct Layout {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub fn place(hwnd: HWND) {
    let monitor = unsafe { MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY) };
    let Some(info) = native::monitor_info(monitor) else {
        return;
    };
    let anchor = if taskbar_autohides() { info.rcMonitor } else { info.rcWork };
    let layout = compute_layout(&anchor, monitor_dpi(monitor));
    apply_layout(hwnd, &layout);
}

pub fn affects_placement(wparam: WPARAM, lparam: LPARAM) -> bool {
    wparam.0 == SPI_SETWORKAREA.0 as usize || is_tray_settings_broadcast(lparam)
}

fn is_tray_settings_broadcast(lparam: LPARAM) -> bool {
    let text = lparam.0 as *const u16;
    if text.is_null() {
        return false;
    }
    let name = unsafe { windows_core::PCWSTR(text).to_string() };
    name.is_ok_and(|name| name == "TraySettings")
}

fn taskbar_autohides() -> bool {
    let mut data = APPBARDATA { cbSize: size_of::<APPBARDATA>() as u32, ..Default::default() };
    let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) };
    state & ABS_AUTOHIDE as usize != 0
}

fn monitor_dpi(monitor: HMONITOR) -> i32 {
    let (mut dpi_x, mut dpi_y) = (0u32, 0u32);
    match unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) } {
        Ok(()) if dpi_x > 0 => dpi_x as i32,
        _ => BASE_DPI,
    }
}

fn scale_to_physical(dip: i32, dpi: i32) -> i32 {
    (dip * dpi + BASE_DPI / 2) / BASE_DPI
}

fn compute_layout(anchor: &RECT, dpi: i32) -> Layout {
    let width = scale_to_physical(ISLAND_WIDTH_DIP, dpi);
    let height = scale_to_physical(ISLAND_HEIGHT_DIP, dpi);
    let gap = scale_to_physical(TASKBAR_GAP_DIP, dpi);
    Layout {
        x: anchor.left + (anchor.right - anchor.left - width) / 2,
        y: anchor.bottom - gap - height,
        width,
        height,
    }
}

fn apply_layout(hwnd: HWND, layout: &Layout) {
    let result = unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            layout.x,
            layout.y,
            layout.width,
            layout.height,
            SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
    };
    if let Err(error) = result {
        log::warn!("island placement failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_at_bottom_with_scaled_gap() {
        let work_area = RECT { left: 0, top: 0, right: 1920, bottom: 1040 };
        let layout = compute_layout(&work_area, 96);
        assert_eq!(layout, Layout { x: 720, y: 672, width: 480, height: 360 });
    }

    #[test]
    fn scales_dimensions_with_dpi() {
        let work_area = RECT { left: 0, top: 0, right: 3840, bottom: 2080 };
        let layout = compute_layout(&work_area, 192);
        assert_eq!((layout.width, layout.height), (960, 720));
        assert_eq!(layout.y, 2080 - 16 - 720);
    }
}
