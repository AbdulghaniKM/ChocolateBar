use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;

use super::events::DeviceEvent;
use super::registry::DeviceSummary;

const EVENT_TTL_MS: u64 = 3_000;
const ACCENT_CONNECTED: &str = "#3b82f6";
const ACCENT_DISCONNECTED: &str = "#a1a1aa";
const ACCENT_LOW: &str = "#ef4444";

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum DeviceKind {
    Headphones,
    Keyboard,
    Mouse,
    Gamepad,
    Speaker,
    Generic,
}

impl DeviceKind {
    fn of(name: &str) -> Self {
        let lowercase = name.to_lowercase();
        let matches_any = |words: &[&str]| words.iter().any(|word| lowercase.contains(word));
        if matches_any(&["buds", "airpods", "headphone", "headset", "earbud", "earphone", "wh-1000", "wf-1000"]) {
            Self::Headphones
        } else if matches_any(&["keyboard", "keychron", "k380"]) {
            Self::Keyboard
        } else if matches_any(&["mouse", "mx master", "trackpad", "mx anywhere"]) {
            Self::Mouse
        } else if matches_any(&["controller", "gamepad", "dualsense", "dualshock", "xbox", "joy-con"]) {
            Self::Gamepad
        } else if matches_any(&["speaker", "soundbar", "boombox", "jbl", "echo"]) {
            Self::Speaker
        } else {
            Self::Generic
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Headphones => "🎧",
            Self::Keyboard => "⌨",
            Self::Mouse => "🖱",
            Self::Gamepad => "🎮",
            Self::Speaker => "🔊",
            Self::Generic => "🔵",
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceView<'a> {
    name: &'a str,
    device_kind: DeviceKind,
    connected: bool,
    battery: Option<u8>,
    event: Option<&'static str>,
}

pub fn device_activity(ctx: &SourceContext, key: &str, summary: &DeviceSummary) -> Activity {
    let kind = DeviceKind::of(&summary.name);
    let mut compact = CompactView::new(kind.icon(), summary.name.clone()).accent(ACCENT_CONNECTED);
    if let Some(battery) = summary.battery {
        compact = compact.subtitle(format!("{battery}%")).progress(f32::from(battery) / 100.0);
    }
    ctx.activity(&device_key(key), ActivityKind::Bluetooth, compact)
        .priority(Priority::Ambient)
        .expanded(json!(view_of(summary, kind, None)))
}

pub fn event_activity(ctx: &SourceContext, key: &str, summary: &DeviceSummary, event: DeviceEvent) -> Activity {
    let kind = DeviceKind::of(&summary.name);
    let compact = CompactView::new(kind.icon(), summary.name.clone())
        .subtitle(event_caption(summary, event))
        .accent(event_accent(event));
    ctx.activity(&event_key(key), ActivityKind::Bluetooth, compact)
        .priority(Priority::High)
        .alert(event != DeviceEvent::Connected)
        .ttl_ms(EVENT_TTL_MS)
        .expanded(json!(view_of(summary, kind, Some(event))))
}

pub fn device_key(key: &str) -> String {
    format!("device-{key}")
}

fn event_key(key: &str) -> String {
    format!("event-{key}")
}

fn view_of<'a>(summary: &'a DeviceSummary, kind: DeviceKind, event: Option<DeviceEvent>) -> DeviceView<'a> {
    DeviceView {
        name: &summary.name,
        device_kind: kind,
        connected: summary.connected,
        battery: summary.battery,
        event: event.map(event_name),
    }
}

fn event_name(event: DeviceEvent) -> &'static str {
    match event {
        DeviceEvent::Connected => "connected",
        DeviceEvent::Disconnected => "disconnected",
        DeviceEvent::LowBattery => "lowBattery",
    }
}

fn event_caption(summary: &DeviceSummary, event: DeviceEvent) -> String {
    match (event, summary.battery) {
        (DeviceEvent::Connected, Some(battery)) => format!("Connected · {battery}%"),
        (DeviceEvent::Connected, None) => "Connected".into(),
        (DeviceEvent::Disconnected, _) => "Disconnected".into(),
        (DeviceEvent::LowBattery, Some(battery)) => format!("Battery low · {battery}%"),
        (DeviceEvent::LowBattery, None) => "Battery low".into(),
    }
}

fn event_accent(event: DeviceEvent) -> &'static str {
    match event {
        DeviceEvent::Connected => ACCENT_CONNECTED,
        DeviceEvent::Disconnected => ACCENT_DISCONNECTED,
        DeviceEvent::LowBattery => ACCENT_LOW,
    }
}
