use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    WA_INACTIVE, WM_ACTIVATE, WM_DISPLAYCHANGE, WM_DPICHANGED, WM_MOUSEACTIVATE, WM_NCDESTROY,
    WM_SETTINGCHANGE,
};

use super::{no_activate, placement};
use crate::core::error::AppResult;

const SUBCLASS_ID: usize = 1;

pub fn install(hwnd: HWND) -> AppResult<()> {
    unsafe { SetWindowSubclass(hwnd, Some(procedure), SUBCLASS_ID, 0) }
        .ok()
        .map_err(Into::into)
}

unsafe extern "system" fn procedure(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _reference_data: usize,
) -> LRESULT {
    match message {
        WM_MOUSEACTIVATE => no_activate::mouse_activation(),
        WM_ACTIVATE => {
            let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
            if (wparam.0 & 0xFFFF) as u32 != WA_INACTIVE && !no_activate::keyboard_wanted() {
                no_activate::restore_external_foreground();
            }
            result
        }
        WM_DISPLAYCHANGE | WM_DPICHANGED | WM_SETTINGCHANGE => {
            let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
            if message != WM_SETTINGCHANGE || placement::affects_placement(wparam, lparam) {
                placement::place(hwnd);
            }
            result
        }
        WM_NCDESTROY => {
            let _ = unsafe { RemoveWindowSubclass(hwnd, Some(procedure), SUBCLASS_ID) };
            unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefSubclassProc(hwnd, message, wparam, lparam) },
    }
}
