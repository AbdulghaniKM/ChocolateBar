use serde_json::{json, Value};

use super::capture::{open_screen_clip, ArmedWindow};
use super::cards::{message_activity, recognized_activity};
use super::clipboard;
use super::dib::bgra_from_dib;
use super::recognize::{available_language_tags, recognize_text};
use crate::core::activity::{ActionRequest, SourceCommand};
use crate::core::source::SourceContext;

const ARABIC_TAG_PREFIX: &str = "ar";

pub struct OcrSession {
    ctx: SourceContext,
    armed: ArmedWindow,
    last_text: String,
    last_sequence: u32,
}

impl OcrSession {
    pub fn new(ctx: SourceContext) -> Self {
        Self { ctx, armed: ArmedWindow::default(), last_text: String::new(), last_sequence: 0 }
    }

    pub fn on_hotkey_pressed(&mut self) {
        self.armed.arm();
        open_screen_clip(&self.ctx.app);
    }

    pub fn on_image_copied(&mut self, sequence: u32) {
        if sequence == self.last_sequence {
            return;
        }
        self.last_sequence = sequence;
        let settings = self.ctx.current_settings();
        let from_hotkey = self.armed.consume();
        if !from_hotkey && !settings.ocr.auto_clipboard {
            return;
        }
        match read_clipboard_text(&settings.ocr.language) {
            Ok(text) if text.is_empty() => self.show_message("No text found", "Nothing readable in that image"),
            Ok(text) => self.show_recognized(text, from_hotkey),
            Err(reason) => {
                log::warn!("ocr failed: {reason}");
                self.show_message("OCR failed", &reason);
            }
        }
    }

    pub fn handle_action(&mut self, request: ActionRequest) {
        if request.action == "copy" && !self.last_text.is_empty() {
            let copied = clipboard::write_text(&self.last_text);
            self.ctx.upsert(recognized_activity(&self.ctx, &self.last_text, copied));
        }
    }

    pub fn handle_command(&self, command: SourceCommand) {
        match command.command.as_str() {
            "status" => command.respond_ok(self.status()),
            unknown => {
                let message = format!("unknown ocr command {unknown}");
                command.respond_err(message);
            }
        }
    }

    fn show_recognized(&mut self, text: String, from_hotkey: bool) {
        let copied = from_hotkey && clipboard::write_text(&text);
        self.ctx.upsert(recognized_activity(&self.ctx, &text, copied));
        self.last_text = text;
    }

    fn show_message(&self, title: &str, detail: &str) {
        self.ctx.upsert(message_activity(&self.ctx, title, detail));
    }

    fn status(&self) -> Value {
        let languages = available_language_tags();
        let arabic_available = languages.iter().any(|tag| tag.to_lowercase().starts_with(ARABIC_TAG_PREFIX));
        json!({ "languages": languages, "arabicAvailable": arabic_available, "armed": self.armed.is_armed() })
    }
}

fn read_clipboard_text(language_choice: &str) -> Result<String, String> {
    let bytes = clipboard::read_image_bytes().ok_or("the clipboard image could not be read")?;
    let image = bgra_from_dib(&bytes).ok_or("the clipboard image format is not supported")?;
    let text = recognize_text(image, language_choice).map_err(|error| error.to_string())?;
    Ok(text.trim().to_owned())
}
