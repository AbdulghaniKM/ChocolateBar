use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

use super::layout::KeyboardLayout;
use super::tracker::{InputChange, InputTracker};

const INDICATOR_TTL_MS: u64 = 1_200;
const INDICATOR_KEY: &str = "indicator";
const CAPS_BADGE: &str = "CAPS";
const UNKNOWN_TAG: &str = "??";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IndicatorView<'a> {
    layout_tag: &'a str,
    layout_name: &'a str,
    caps_lock: bool,
    change: &'static str,
}

pub fn indicator_activity(ctx: &SourceContext, tracker: &InputTracker, change: InputChange) -> Activity {
    let layout = tracker.layout().cloned().unwrap_or_else(unknown_layout);
    let caps_lock = tracker.is_caps_lock_on();
    let mut compact = CompactView::new("⌨", layout.tag.clone()).subtitle(layout.name.clone());
    if caps_lock {
        compact = compact.badge(CAPS_BADGE);
    }
    let view = IndicatorView {
        layout_tag: &layout.tag,
        layout_name: &layout.name,
        caps_lock,
        change: change_name(change),
    };
    ctx.activity(INDICATOR_KEY, ActivityKind::Keyboard, compact)
        .priority(Priority::High)
        .ttl_ms(INDICATOR_TTL_MS)
        .expanded(json!(view))
}

fn unknown_layout() -> KeyboardLayout {
    KeyboardLayout { tag: UNKNOWN_TAG.into(), name: String::new() }
}

fn change_name(change: InputChange) -> &'static str {
    match change {
        InputChange::Layout => "layout",
        InputChange::CapsLock => "caps",
    }
}
