use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

use super::state::{Bud, BudsState, Case};

const EVENT_TTL_MS: u64 = 3_000;
const LOW_BATTERY_PERCENT: u8 = 15;
const ACCENT_NORMAL: &str = "#818cf8";
const ACCENT_LOW: &str = "#ef4444";
const ACCENT_MUTED: &str = "#a1a1aa";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkStatus {
    Connected,
    Connecting,
    InUse,
    Unsupported,
}

pub struct BudsSnapshot {
    pub address: String,
    pub name: String,
    pub link: LinkStatus,
    pub state: BudsState,
    pub noise_control: bool,
    pub fallback_percent: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BudsView<'a> {
    name: &'a str,
    link: LinkStatus,
    left: Bud,
    right: Bud,
    case: Case,
    noise: Option<super::state::NoiseMode>,
    noise_control: bool,
    fallback_percent: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventView<'a> {
    name: &'a str,
    connected: bool,
}

pub fn status_activity(ctx: &SourceContext, snapshot: &BudsSnapshot) -> Activity {
    let lowest = snapshot.state.lowest_bud_percent().or(snapshot.fallback_percent);
    let mut compact = CompactView::new("🎧", short_name(&snapshot.name))
        .subtitle(subtitle_of(snapshot))
        .accent(accent_of(snapshot.link, lowest));
    if let Some(percent) = lowest {
        compact = compact.progress(f32::from(percent) / 100.0);
    }
    if let Some(mode) = snapshot.state.noise {
        compact = compact.badge(mode.label());
    }
    let priority = if snapshot.link == LinkStatus::Connected { Priority::Normal } else { Priority::Ambient };
    ctx.activity(&snapshot.address, ActivityKind::Buds, compact)
        .priority(priority)
        .expanded(json!(view_of(snapshot)))
}

pub fn event_activity(ctx: &SourceContext, address: &str, name: &str, connected: bool) -> Activity {
    let caption = if connected { "Connected" } else { "Disconnected" };
    let accent = if connected { ACCENT_NORMAL } else { ACCENT_MUTED };
    let compact = CompactView::new("🎧", short_name(name)).subtitle(caption).accent(accent);
    ctx.activity(&format!("event-{address}"), ActivityKind::Buds, compact)
        .priority(Priority::High)
        .alert(!connected)
        .ttl_ms(EVENT_TTL_MS)
        .expanded(json!(EventView { name, connected }))
}

fn view_of(snapshot: &BudsSnapshot) -> BudsView<'_> {
    BudsView {
        name: &snapshot.name,
        link: snapshot.link,
        left: snapshot.state.left,
        right: snapshot.state.right,
        case: snapshot.state.case,
        noise: snapshot.state.noise,
        noise_control: snapshot.noise_control && snapshot.link == LinkStatus::Connected,
        fallback_percent: snapshot.fallback_percent,
    }
}

fn subtitle_of(snapshot: &BudsSnapshot) -> String {
    match snapshot.link {
        LinkStatus::Connected => levels_caption(&snapshot.state),
        LinkStatus::Connecting => with_fallback("Connecting", snapshot.fallback_percent),
        LinkStatus::InUse => with_fallback("In use by another app", snapshot.fallback_percent),
        LinkStatus::Unsupported => with_fallback("Battery only", snapshot.fallback_percent),
    }
}

fn with_fallback(caption: &str, fallback_percent: Option<u8>) -> String {
    match fallback_percent {
        Some(percent) => format!("{caption} · {percent}%"),
        None => caption.to_owned(),
    }
}

fn levels_caption(state: &BudsState) -> String {
    let parts = [("L", state.left.percent), ("R", state.right.percent), ("Case", state.case.percent)];
    let shown: Vec<String> = parts
        .into_iter()
        .filter_map(|(label, percent)| percent.map(|value| format!("{label} {value}%")))
        .collect();
    if shown.is_empty() {
        "Waiting for status".into()
    } else {
        shown.join(" · ")
    }
}

fn accent_of(link: LinkStatus, lowest: Option<u8>) -> &'static str {
    match (link, lowest) {
        (_, Some(percent)) if percent <= LOW_BATTERY_PERCENT => ACCENT_LOW,
        (LinkStatus::Connected, _) => ACCENT_NORMAL,
        _ => ACCENT_MUTED,
    }
}

fn short_name(name: &str) -> String {
    let without_suffix = name.split(" (").next().unwrap_or(name);
    without_suffix.strip_prefix("Galaxy ").unwrap_or(without_suffix).to_owned()
}
