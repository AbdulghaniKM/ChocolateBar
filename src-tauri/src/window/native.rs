use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicIsize, Ordering};

use tauri::WebviewWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};

use crate::core::error::AppResult;

static ISLAND_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub fn capture(window: &WebviewWindow) -> AppResult<HWND> {
    let hwnd = HWND(window.hwnd()?.0);
    ISLAND_HANDLE.store(hwnd.0 as isize, Ordering::Relaxed);
    Ok(hwnd)
}

pub fn island() -> HWND {
    HWND(ISLAND_HANDLE.load(Ordering::Relaxed) as *mut c_void)
}

pub fn monitor_info(monitor: HMONITOR) -> Option<MONITORINFO> {
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool().then_some(info)
}
