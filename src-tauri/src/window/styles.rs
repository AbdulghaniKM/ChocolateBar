use std::ffi::c_void;
use std::mem::size_of;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_LAYERED,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
};

pub fn apply(hwnd: HWND) {
    apply_extended_styles(hwnd);
    remove_border(hwnd);
}

pub fn set_click_through(hwnd: HWND, enabled: bool) {
    let current = read_extended_styles(hwnd);
    let updated = with_click_through(current, enabled);
    if updated != current {
        write_extended_styles(hwnd, updated);
    }
}

pub fn set_no_activate(hwnd: HWND, enabled: bool) {
    let current = read_extended_styles(hwnd);
    let updated = if enabled { current | WS_EX_NOACTIVATE.0 } else { current & !WS_EX_NOACTIVATE.0 };
    if updated != current {
        write_extended_styles(hwnd, updated);
    }
}

fn apply_extended_styles(hwnd: HWND) {
    let current = read_extended_styles(hwnd);
    write_extended_styles(hwnd, with_overlay_styles(current));
}

fn read_extended_styles(hwnd: HWND) -> u32 {
    unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 }
}

fn write_extended_styles(hwnd: HWND, styles: u32) {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, styles as isize) };
}

fn with_overlay_styles(current: u32) -> u32 {
    let added = WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_TOPMOST.0;
    (current | added) & !WS_EX_APPWINDOW.0
}

fn with_click_through(current: u32, enabled: bool) -> u32 {
    if enabled {
        current | WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0
    } else {
        current & !WS_EX_TRANSPARENT.0
    }
}

fn remove_border(hwnd: HWND) {
    let color = DWMWA_COLOR_NONE;
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &color as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        )
    };
    if let Err(error) = result {
        log::debug!("border color attribute unsupported: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_styles_hide_from_taskbar_and_refuse_activation() {
        let styled = with_overlay_styles(WS_EX_APPWINDOW.0);
        assert_eq!(styled & WS_EX_APPWINDOW.0, 0);
        assert_eq!(styled & WS_EX_TOOLWINDOW.0, WS_EX_TOOLWINDOW.0);
        assert_eq!(styled & WS_EX_NOACTIVATE.0, WS_EX_NOACTIVATE.0);
        assert_eq!(styled & WS_EX_TOPMOST.0, WS_EX_TOPMOST.0);
    }

    #[test]
    fn click_through_toggles_transparency_and_keeps_other_styles() {
        let base = with_overlay_styles(0);
        let passing = with_click_through(base, true);
        assert_eq!(passing & WS_EX_TRANSPARENT.0, WS_EX_TRANSPARENT.0);
        assert_eq!(passing & WS_EX_LAYERED.0, WS_EX_LAYERED.0);
        let blocking = with_click_through(passing, false);
        assert_eq!(blocking & WS_EX_TRANSPARENT.0, 0);
        assert_eq!(blocking & WS_EX_LAYERED.0, WS_EX_LAYERED.0);
        assert_eq!(blocking & base, base);
    }
}
