use std::mem::size_of;
use std::sync::Mutex;

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{MonitorFromPoint, HMONITOR, MONITOR_DEFAULTTOPRIMARY};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETSTATE, ABS_AUTOHIDE, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SPI_SETWORKAREA, SWP_FRAMECHANGED, SWP_NOACTIVATE,
};

use super::native;
use crate::core::settings::{HorizontalAnchor, IslandPosition, VerticalAnchor};

const ISLAND_WIDTH_DIP: i32 = 480;
const ISLAND_HEIGHT_DIP: i32 = 360;
const EDGE_GAP_DIP: i32 = 8;
const BASE_DPI: i32 = 96;

static REQUESTED_SIZE: Mutex<Option<(i32, i32)>> = Mutex::new(None);
static POSITION: Mutex<IslandPosition> = Mutex::new(IslandPosition {
    vertical: VerticalAnchor::Bottom,
    horizontal: HorizontalAnchor::Center,
    offset_x: 0,
    offset_y: 0,
});

#[derive(Debug, PartialEq, Eq)]
struct Layout {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// The webview reports the physical size its largest layout needs, so the window grows with
/// Windows text scaling instead of clipping a fixed-size canvas.
pub fn request_size(hwnd: HWND, width: f64, height: f64) {
    let size = (width.ceil() as i32, height.ceil() as i32);
    if size.0 <= 0 || size.1 <= 0 {
        return;
    }
    if let Ok(mut requested) = REQUESTED_SIZE.lock() {
        if *requested == Some(size) {
            return;
        }
        *requested = Some(size);
    }
    place(hwnd);
}

pub fn set_position(hwnd: HWND, position: IslandPosition) {
    if let Ok(mut current) = POSITION.lock() {
        if *current == position {
            return;
        }
        *current = position;
    }
    place(hwnd);
}

pub fn place(hwnd: HWND) {
    let monitor = unsafe { MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY) };
    let Some(info) = native::monitor_info(monitor) else {
        return;
    };
    let anchor = if taskbar_autohides() { info.rcMonitor } else { info.rcWork };
    let dpi = monitor_dpi(monitor);
    let size = requested_size().unwrap_or_else(|| default_size(dpi));
    let position = POSITION.lock().map(|position| *position).unwrap_or_default();
    let layout = compute_layout(&anchor, &info.rcMonitor, dpi, size, position);
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

fn requested_size() -> Option<(i32, i32)> {
    REQUESTED_SIZE.lock().ok().and_then(|requested| *requested)
}

fn default_size(dpi: i32) -> (i32, i32) {
    (scale_to_physical(ISLAND_WIDTH_DIP, dpi), scale_to_physical(ISLAND_HEIGHT_DIP, dpi))
}

fn compute_layout(
    anchor: &RECT,
    monitor: &RECT,
    dpi: i32,
    (width, height): (i32, i32),
    position: IslandPosition,
) -> Layout {
    let gap = scale_to_physical(EDGE_GAP_DIP, dpi);
    let x = match position.horizontal {
        HorizontalAnchor::Left => anchor.left + gap,
        HorizontalAnchor::Center => anchor.left + (anchor.right - anchor.left - width) / 2,
        HorizontalAnchor::Right => anchor.right - gap - width,
    } + scale_to_physical(position.offset_x, dpi);
    let y = match position.vertical {
        VerticalAnchor::Top => anchor.top + gap,
        VerticalAnchor::Middle => anchor.top + (anchor.bottom - anchor.top - height) / 2,
        VerticalAnchor::Bottom => anchor.bottom - gap - height,
    } + scale_to_physical(position.offset_y, dpi);
    Layout {
        x: x.clamp(monitor.left, (monitor.right - width).max(monitor.left)),
        y: y.clamp(monitor.top, (monitor.bottom - height).max(monitor.top)),
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

    const WORK_AREA: RECT = RECT { left: 0, top: 0, right: 1920, bottom: 1040 };
    const MONITOR: RECT = RECT { left: 0, top: 0, right: 1920, bottom: 1080 };

    fn at(vertical: VerticalAnchor, horizontal: HorizontalAnchor) -> IslandPosition {
        IslandPosition { vertical, horizontal, offset_x: 0, offset_y: 0 }
    }

    #[test]
    fn centers_at_bottom_with_scaled_gap() {
        let layout = compute_layout(&WORK_AREA, &MONITOR, 96, default_size(96), IslandPosition::default());
        assert_eq!(layout, Layout { x: 720, y: 672, width: 480, height: 360 });
    }

    #[test]
    fn scales_dimensions_with_dpi() {
        let work_area = RECT { left: 0, top: 0, right: 3840, bottom: 2080 };
        let monitor = RECT { left: 0, top: 0, right: 3840, bottom: 2160 };
        let layout = compute_layout(&work_area, &monitor, 192, default_size(192), IslandPosition::default());
        assert_eq!((layout.width, layout.height), (960, 720));
        assert_eq!(layout.y, 2080 - 16 - 720);
    }

    #[test]
    fn uses_requested_size_and_stays_centered() {
        let layout = compute_layout(&WORK_AREA, &MONITOR, 96, (600, 440), IslandPosition::default());
        assert_eq!(layout, Layout { x: 660, y: 592, width: 600, height: 440 });
    }

    #[test]
    fn anchors_to_the_top_right_corner() {
        let position = at(VerticalAnchor::Top, HorizontalAnchor::Right);
        let layout = compute_layout(&WORK_AREA, &MONITOR, 96, (480, 360), position);
        assert_eq!((layout.x, layout.y), (1920 - 8 - 480, 8));
    }

    #[test]
    fn applies_offsets_but_never_leaves_the_monitor() {
        let position = IslandPosition { offset_x: -5000, offset_y: 40, ..at(VerticalAnchor::Middle, HorizontalAnchor::Left) };
        let layout = compute_layout(&WORK_AREA, &MONITOR, 96, (480, 360), position);
        assert_eq!((layout.x, layout.y), (0, 340 + 40));
    }
}
