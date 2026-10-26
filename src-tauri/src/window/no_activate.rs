use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{IsWindow, SetForegroundWindow, MA_NOACTIVATE};

use super::native;

static LAST_EXTERNAL_FOREGROUND: AtomicIsize = AtomicIsize::new(0);

pub fn refuse_mouse_activation() -> LRESULT {
    LRESULT(MA_NOACTIVATE as isize)
}

pub fn on_foreground_changed(foreground: HWND) {
    if foreground == native::island() {
        restore_external_foreground();
    } else {
        LAST_EXTERNAL_FOREGROUND.store(foreground.0 as isize, Ordering::Relaxed);
    }
}

pub fn restore_external_foreground() {
    let last = HWND(LAST_EXTERNAL_FOREGROUND.load(Ordering::Relaxed) as *mut c_void);
    if !last.is_invalid() && unsafe { IsWindow(Some(last)) }.as_bool() {
        let _ = unsafe { SetForegroundWindow(last) };
    }
}
