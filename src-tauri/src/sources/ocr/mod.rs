mod capture;
mod cards;
mod clipboard_watch;
mod recognize;
mod session;

use std::time::Duration;

use crossbeam_channel::{select, unbounded};

use crate::core::source::{ActivitySource, SourceContext};
use capture::CaptureHotkey;
use clipboard_watch::ClipboardWatcher;
use session::OcrSession;

const LOOP_TICK: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct OcrSource;

impl ActivitySource for OcrSource {
    fn name(&self) -> &'static str {
        "ocr"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let (hotkey_presses, hotkey_events) = unbounded();
        let (clipboard_images, clipboard_events) = unbounded();
        let mut hotkey = CaptureHotkey::new(ctx.app.clone(), hotkey_presses);
        hotkey.follow(&ctx.current_settings().hotkeys.ocr);
        let watcher = ClipboardWatcher::start(clipboard_images);
        let mut session = OcrSession::new(ctx.clone());
        while ctx.is_running() {
            select! {
                recv(ctx.actions) -> request => match request {
                    Ok(request) => session.handle_action(request),
                    Err(_) => break,
                },
                recv(ctx.commands) -> command => match command {
                    Ok(command) => session.handle_command(command),
                    Err(_) => break,
                },
                recv(hotkey_events) -> _ => session.on_hotkey_pressed(),
                recv(clipboard_events) -> sequence => if let Ok(sequence) = sequence {
                    session.on_image_copied(sequence);
                },
                default(LOOP_TICK) => {}
            }
            if let Some(settings) = ctx.take_changed_settings() {
                hotkey.follow(&settings.hotkeys.ocr);
            }
        }
        hotkey.release();
        watcher.stop();
    }
}
