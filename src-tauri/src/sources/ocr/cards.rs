use serde_json::json;

use crate::core::activity::{ActionSpec, ActionStyle, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

const CARD_KEY: &str = "last";
const RESULT_TTL_MS: u64 = 8_000;
const MESSAGE_TTL_MS: u64 = 4_000;
const TITLE_LIMIT: usize = 36;
const TEXT_LIMIT: usize = 2_000;
const OCR_ACCENT: &str = "#38bdf8";

pub fn recognized_activity(ctx: &SourceContext, text: &str, copied: bool) -> Activity {
    let characters = text.chars().count();
    let subtitle = if copied { "Copied to clipboard".to_owned() } else { format!("{characters} characters") };
    let compact = CompactView::new("📝", truncate(first_line(text), TITLE_LIMIT))
        .subtitle(subtitle)
        .accent(OCR_ACCENT);
    let expanded = json!({
        "view": "text",
        "text": truncate(text, TEXT_LIMIT),
        "copied": copied,
        "characterCount": characters,
    });
    ctx.activity(CARD_KEY, ActivityKind::Ocr, compact)
        .priority(Priority::High)
        .ttl_ms(RESULT_TTL_MS)
        .expanded(expanded)
        .action(ActionSpec::button("copy", "Copy").icon("⧉").style(ActionStyle::Primary))
}

pub fn message_activity(ctx: &SourceContext, title: &str, detail: &str) -> Activity {
    let compact = CompactView::new("📝", title).subtitle(detail).accent(OCR_ACCENT);
    ctx.activity(CARD_KEY, ActivityKind::Ocr, compact)
        .priority(Priority::High)
        .ttl_ms(MESSAGE_TTL_MS)
        .expanded(json!({ "view": "message", "message": detail }))
}

fn first_line(text: &str) -> &str {
    text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or_default()
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_owned();
    }
    let kept: String = text.chars().take(limit).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_line_skips_blank_lines() {
        assert_eq!(first_line("\n  \n hello \nworld"), "hello");
    }

    #[test]
    fn truncate_keeps_short_text_and_cuts_long_text() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdef", 3), "abc…");
    }
}
