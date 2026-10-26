use std::ffi::c_void;
use std::mem::size_of;

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE, SW_HIDE, SW_SHOWNOACTIVATE,
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

pub fn apply(hwnd: HWND) {
    apply_extended_styles(hwnd);
    remove_border(hwnd);
}

pub fn show_without_activation(hwnd: HWND) {
    let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
}

pub fn hide(hwnd: HWND) {
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
}

fn apply_extended_styles(hwnd: HWND) {
    let added = WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_TOPMOST.0;
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let updated = (current | added) & !WS_EX_APPWINDOW.0;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated as isize);
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
