use std::time::{Duration, Instant};

use crossbeam_channel::Sender;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_opener::OpenerExt;

const ARMED_DURATION: Duration = Duration::from_secs(60);
const SCREEN_CLIP_URI: &str = "ms-screenclip:";

pub struct CaptureHotkey {
    app: AppHandle,
    presses: Sender<()>,
    registered: Option<String>,
}

impl CaptureHotkey {
    pub fn new(app: AppHandle, presses: Sender<()>) -> Self {
        Self { app, presses, registered: None }
    }

    pub fn follow(&mut self, shortcut: &str) {
        if self.registered.as_deref() == Some(shortcut) {
            return;
        }
        self.release();
        let presses = self.presses.clone();
        let registration = self.app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = presses.send(());
            }
        });
        match registration {
            Ok(()) => self.registered = Some(shortcut.to_owned()),
            Err(error) => log::warn!("ocr hotkey {shortcut} not registered: {error}"),
        }
    }

    pub fn release(&mut self) {
        if let Some(previous) = self.registered.take() {
            if let Err(error) = self.app.global_shortcut().unregister(previous.as_str()) {
                log::warn!("ocr hotkey {previous} not released: {error}");
            }
        }
    }
}

pub fn open_screen_clip(app: &AppHandle) {
    if let Err(error) = app.opener().open_url(SCREEN_CLIP_URI, None::<&str>) {
        log::warn!("could not open {SCREEN_CLIP_URI}: {error}");
    }
}

#[derive(Default)]
pub struct ArmedWindow {
    until: Option<Instant>,
}

impl ArmedWindow {
    pub fn arm(&mut self) {
        self.arm_at(Instant::now());
    }

    pub fn consume(&mut self) -> bool {
        self.consume_at(Instant::now())
    }

    pub fn is_armed(&self) -> bool {
        self.until.is_some_and(|deadline| Instant::now() < deadline)
    }

    fn arm_at(&mut self, now: Instant) {
        self.until = Some(now + ARMED_DURATION);
    }

    fn consume_at(&mut self, now: Instant) -> bool {
        let armed = self.until.is_some_and(|deadline| now < deadline);
        self.until = None;
        armed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn armed_window_is_consumed_once() {
        let now = Instant::now();
        let mut window = ArmedWindow::default();
        window.arm_at(now);
        assert!(window.consume_at(now + Duration::from_secs(30)));
        assert!(!window.consume_at(now + Duration::from_secs(31)));
    }

    #[test]
    fn armed_window_expires_after_sixty_seconds() {
        let now = Instant::now();
        let mut window = ArmedWindow::default();
        window.arm_at(now);
        assert!(!window.consume_at(now + Duration::from_secs(61)));
    }

    #[test]
    fn unarmed_window_never_consumes() {
        assert!(!ArmedWindow::default().consume_at(Instant::now()));
    }
}
