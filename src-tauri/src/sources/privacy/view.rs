use serde_json::json;

use crate::core::activity::{ActionSpec, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

pub const MIC_KEY: &str = "mic";
pub const CAMERA_KEY: &str = "camera";
pub const MUTE_KEY: &str = "mute";
pub const MUTE_ACTION: &str = "mute";

const MUTE_NOTICE_TTL_MS: u64 = 2_000;
const LIVE_ACCENT: &str = "#ef4444";
const MUTED_ACCENT: &str = "#a1a1aa";
const CAMERA_ACCENT: &str = "#22c55e";

pub fn mic_activity(ctx: &SourceContext, apps: &[String], muted: bool) -> Activity {
    let title = if muted { "Microphone in use (muted)" } else { "Microphone in use" };
    let view = CompactView::new("🎙", title).subtitle(apps.join(", ")).accent(mic_accent(muted));
    ctx.activity(MIC_KEY, ActivityKind::Privacy, view)
        .priority(Priority::Ambient)
        .expanded(json!({ "kind": "mic", "apps": apps, "muted": muted }))
        .action(mute_toggle(muted))
}

pub fn camera_activity(ctx: &SourceContext, apps: &[String]) -> Activity {
    let view = CompactView::new("📷", "Camera in use").subtitle(apps.join(", ")).accent(CAMERA_ACCENT);
    ctx.activity(CAMERA_KEY, ActivityKind::Privacy, view)
        .priority(Priority::Ambient)
        .expanded(json!({ "kind": "camera", "apps": apps, "muted": false }))
}

pub fn mute_notice_activity(ctx: &SourceContext, muted: bool) -> Activity {
    let title = if muted { "Microphone muted" } else { "Microphone live" };
    let view = CompactView::new(if muted { "🔇" } else { "🎙" }, title).accent(mic_accent(muted));
    ctx.activity(MUTE_KEY, ActivityKind::Privacy, view)
        .priority(Priority::High)
        .ttl_ms(MUTE_NOTICE_TTL_MS)
        .expanded(json!({ "kind": "mute", "apps": Vec::<String>::new(), "muted": muted }))
        .action(mute_toggle(muted))
}

fn mute_toggle(muted: bool) -> ActionSpec {
    ActionSpec::toggle(MUTE_ACTION, "Mute microphone", muted).icon("🔇")
}

fn mic_accent(muted: bool) -> &'static str {
    if muted {
        MUTED_ACCENT
    } else {
        LIVE_ACCENT
    }
}
