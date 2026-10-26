use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use tokio::sync::watch;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowRect};

use super::native;
use crate::core::hub::HubHandle;
use crate::core::settings::Settings;

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const SHELL_CLASSES: [&str; 3] = ["Progman", "WorkerW", "Shell_TrayWnd"];

pub fn spawn(hub: HubHandle, settings: watch::Receiver<Settings>, shutdown: Arc<AtomicBool>) {
    let spawned = thread::Builder::new()
        .name("fullscreen".into())
        .spawn(move || watch_fullscreen(&hub, &settings, &shutdown));
    if let Err(error) = spawned {
        log::error!("fullscreen thread failed to spawn: {error}");
    }
}

fn watch_fullscreen(hub: &HubHandle, settings: &watch::Receiver<Settings>, shutdown: &AtomicBool) {
    let mut reported: Option<bool> = None;
    while !shutdown.load(Ordering::Relaxed) {
        let hiding_enabled = settings.borrow().general.hide_in_fullscreen;
        let fullscreen = hiding_enabled && is_fullscreen_app_active();
        if reported != Some(fullscreen) {
            hub.set_fullscreen(fullscreen);
            reported = Some(fullscreen);
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn is_fullscreen_app_active() -> bool {
    notification_state_is_fullscreen() || foreground_covers_monitor()
}

fn notification_state_is_fullscreen() -> bool {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(state) => {
            state == QUNS_BUSY
                || state == QUNS_RUNNING_D3D_FULL_SCREEN
                || state == QUNS_PRESENTATION_MODE
        }
        Err(_) => false,
    }
}

fn foreground_covers_monitor() -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    if foreground.is_invalid() || foreground == native::island() || is_shell_window(foreground) {
        return false;
    }
    let mut window_rect = RECT::default();
    if unsafe { GetWindowRect(foreground, &mut window_rect) }.is_err() {
        return false;
    }
    let monitor = unsafe { MonitorFromWindow(foreground, MONITOR_DEFAULTTONEAREST) };
    native::monitor_info(monitor).is_some_and(|info| same_rect(&window_rect, &info.rcMonitor))
}

fn is_shell_window(hwnd: HWND) -> bool {
    let mut buffer = [0u16; 64];
    let length = unsafe { GetClassNameW(hwnd, &mut buffer) }.max(0) as usize;
    let class_name = String::from_utf16_lossy(&buffer[..length]);
    SHELL_CLASSES.contains(&class_name.as_str())
}

fn same_rect(first: &RECT, second: &RECT) -> bool {
    first.left == second.left
        && first.top == second.top
        && first.right == second.right
        && first.bottom == second.bottom
}
