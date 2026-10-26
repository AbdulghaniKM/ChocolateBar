use serde_json::json;

use super::endpoint::VolumeLevel;
use crate::core::activity::{ActionSpec, Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

pub const VOLUME_KEY: &str = "volume";
pub const BRIGHTNESS_KEY: &str = "brightness";
pub const VOLUME_ACTION: &str = "volume";
pub const MUTE_ACTION: &str = "mute";
pub const BRIGHTNESS_ACTION: &str = "brightness";

const OSD_TTL_MS: u64 = 1_500;
const MUTED_ICON: &str = "🔇";
const BRIGHTNESS_ICON: &str = "🔆";
const MUTED_ACCENT: &str = "#a1a1aa";

pub fn volume_activity(ctx: &SourceContext, level: VolumeLevel) -> Activity {
    let percent = level.percent();
    let icon = volume_icon(level);
    let title = if level.muted { "Muted".to_string() } else { format!("{percent}%") };
    let mut view = CompactView::new(icon, title).subtitle("Volume");
    if level.muted {
        view = view.accent(MUTED_ACCENT);
    }
    ctx.activity(VOLUME_KEY, ActivityKind::Osd, view)
        .priority(Priority::High)
        .ttl_ms(OSD_TTL_MS)
        .expanded(json!({ "kind": "volume", "percent": percent, "muted": level.muted }))
        .action(ActionSpec::slider(VOLUME_ACTION, "Volume", (0.0, 100.0), 1.0, f64::from(percent)).icon(icon))
        .action(ActionSpec::toggle(MUTE_ACTION, "Mute", level.muted).icon(MUTED_ICON))
}

pub fn brightness_activity(ctx: &SourceContext, percent: u8) -> Activity {
    let view = CompactView::new(BRIGHTNESS_ICON, format!("{percent}%")).subtitle("Brightness");
    ctx.activity(BRIGHTNESS_KEY, ActivityKind::Osd, view)
        .priority(Priority::High)
        .ttl_ms(OSD_TTL_MS)
        .expanded(json!({ "kind": "brightness", "percent": percent, "muted": false }))
        .action(
            ActionSpec::slider(BRIGHTNESS_ACTION, "Brightness", (1.0, 100.0), 1.0, f64::from(percent))
                .icon(BRIGHTNESS_ICON),
        )
}

fn volume_icon(level: VolumeLevel) -> &'static str {
    match (level.muted, level.percent()) {
        (true, _) | (_, 0) => MUTED_ICON,
        (_, 1..=33) => "🔈",
        (_, 34..=66) => "🔉",
        _ => "🔊",
    }
}
