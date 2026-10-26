use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

use super::events::BatteryEvent;
use super::reading::{BatteryReading, ChargeState};

pub const STATUS_KEY: &str = "status";
pub const EVENT_KEY: &str = "event";

const PLUG_EVENT_TTL_MS: u64 = 3_000;
const LOW_EVENT_TTL_MS: u64 = 6_000;
const ACCENT_CHARGING: &str = "#22c55e";
const ACCENT_LOW: &str = "#ef4444";
const ACCENT_NORMAL: &str = "#f4f4f5";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BatteryView {
    percent: u8,
    charging: bool,
    plugged_in: bool,
    full: bool,
    low: bool,
    minutes_remaining: Option<u32>,
    event: Option<&'static str>,
}

pub fn status_activity(ctx: &SourceContext, reading: BatteryReading) -> Activity {
    base_activity(ctx, STATUS_KEY, reading, None).priority(Priority::Ambient)
}

pub fn event_activity(ctx: &SourceContext, reading: BatteryReading, event: BatteryEvent) -> Activity {
    let activity = base_activity(ctx, EVENT_KEY, reading, Some(event)).priority(Priority::High);
    match event {
        BatteryEvent::Low => activity.alert(true).ttl_ms(LOW_EVENT_TTL_MS),
        BatteryEvent::PluggedIn | BatteryEvent::Unplugged => activity.ttl_ms(PLUG_EVENT_TTL_MS),
    }
}

fn base_activity(
    ctx: &SourceContext,
    key: &str,
    reading: BatteryReading,
    event: Option<BatteryEvent>,
) -> Activity {
    let compact = CompactView::new(icon_for(reading), format!("{}%", reading.percent))
        .subtitle(caption_for(reading, event))
        .accent(accent_for(reading));
    ctx.activity(key, ActivityKind::Battery, compact).expanded(json!(view_of(reading, event)))
}

fn view_of(reading: BatteryReading, event: Option<BatteryEvent>) -> BatteryView {
    BatteryView {
        percent: reading.percent,
        charging: reading.state == ChargeState::Charging,
        plugged_in: reading.is_plugged_in(),
        full: reading.state == ChargeState::Full,
        low: reading.is_low(),
        minutes_remaining: reading.minutes_remaining,
        event: event.map(event_name),
    }
}

fn event_name(event: BatteryEvent) -> &'static str {
    match event {
        BatteryEvent::PluggedIn => "pluggedIn",
        BatteryEvent::Unplugged => "unplugged",
        BatteryEvent::Low => "low",
    }
}

fn icon_for(reading: BatteryReading) -> &'static str {
    if reading.is_plugged_in() {
        "⚡"
    } else {
        "🔋"
    }
}

fn caption_for(reading: BatteryReading, event: Option<BatteryEvent>) -> String {
    match (event, reading.state) {
        (Some(BatteryEvent::Low), _) => "Battery low".into(),
        (Some(BatteryEvent::Unplugged), _) => "Unplugged".into(),
        (_, ChargeState::Charging) => "Charging".into(),
        (_, ChargeState::Full) => "Plugged in".into(),
        (_, ChargeState::Discharging) => remaining_caption(reading.minutes_remaining),
    }
}

fn remaining_caption(minutes_remaining: Option<u32>) -> String {
    match minutes_remaining {
        Some(minutes) => format!("{}h {:02}m left", minutes / 60, minutes % 60),
        None => "On battery".into(),
    }
}

fn accent_for(reading: BatteryReading) -> &'static str {
    if reading.is_low() {
        ACCENT_LOW
    } else if reading.is_plugged_in() {
        ACCENT_CHARGING
    } else {
        ACCENT_NORMAL
    }
}
