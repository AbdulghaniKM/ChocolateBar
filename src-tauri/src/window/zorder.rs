use std::thread;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, SetTimer, SetWindowPos, TranslateMessage, EVENT_SYSTEM_FOREGROUND,
    HWND_TOPMOST, MSG, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WINEVENT_OUTOFCONTEXT, WM_TIMER,
};

use super::{native, no_activate};

const REASSERT_INTERVAL_MS: u32 = 2000;

pub fn spawn() {
    let spawned = thread::Builder::new().name("zorder".into()).spawn(run_message_loop);
    if let Err(error) = spawned {
        log::error!("zorder thread failed to spawn: {error}");
    }
}

pub fn assert_topmost() {
    let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE;
    let _ = unsafe { SetWindowPos(native::island(), Some(HWND_TOPMOST), 0, 0, 0, 0, flags) };
}

fn run_message_loop() {
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(on_foreground_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    unsafe { SetTimer(None, 0, REASSERT_INTERVAL_MS, None) };
    pump_messages();
    let _ = unsafe { UnhookWinEvent(hook) };
}

fn pump_messages() {
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        if message.message == WM_TIMER {
            assert_topmost();
        } else {
            let _ = unsafe { TranslateMessage(&message) };
            unsafe { DispatchMessageW(&message) };
        }
    }
}

unsafe extern "system" fn on_foreground_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    foreground: HWND,
    _object_id: i32,
    _child_id: i32,
    _thread_id: u32,
    _timestamp: u32,
) {
    no_activate::on_foreground_changed(foreground);
    assert_topmost();
}
