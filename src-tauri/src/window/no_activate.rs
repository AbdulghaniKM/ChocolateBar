use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LRESULT};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, IsWindow, SetForegroundWindow, MA_ACTIVATE, MA_NOACTIVATE};

use super::{native, styles};

static LAST_EXTERNAL_FOREGROUND: AtomicIsize = AtomicIsize::new(0);
static KEYBOARD_WANTED: AtomicBool = AtomicBool::new(false);

pub fn mouse_activation() -> LRESULT {
    let answer = if keyboard_wanted() { MA_ACTIVATE } else { MA_NOACTIVATE };
    LRESULT(answer as isize)
}

pub fn keyboard_wanted() -> bool {
    KEYBOARD_WANTED.load(Ordering::Relaxed)
}

/// The island never takes focus, except while the user types into one of its text fields.
pub fn set_keyboard_wanted(wanted: bool) {
    if KEYBOARD_WANTED.swap(wanted, Ordering::Relaxed) == wanted {
        return;
    }
    let island = native::island();
    styles::set_no_activate(island, !wanted);
    if !wanted && unsafe { GetForegroundWindow() } == island {
        restore_external_foreground();
    }
}

pub fn on_foreground_changed(foreground: HWND) {
    if foreground == native::island() {
        if keyboard_wanted() {
            return;
        }
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
