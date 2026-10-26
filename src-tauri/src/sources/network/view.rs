use serde::Serialize;
use serde_json::json;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};
use crate::core::source::SourceContext;
use crate::sources::download::units::format_rate;

use super::events::{Connection, NetworkEvent};
use super::history::SpeedHistory;
use super::throughput::Throughput;
use super::wifi::Link;

const STATUS_KEY: &str = "status";
const EVENT_KEY: &str = "event";
const EVENT_TTL_MS: u64 = 3_000;
const ACCENT_ONLINE: &str = "#38bdf8";
const ACCENT_VPN: &str = "#a78bfa";
const ACCENT_OFFLINE: &str = "#ef4444";
const VPN_BADGE: &str = "VPN";

pub struct StatusInput<'a> {
    pub connection: &'a Connection,
    pub throughput: Throughput,
    pub history: &'a SpeedHistory,
    pub show_speed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusView {
    link: &'static str,
    label: String,
    ssid: Option<String>,
    signal_bars: Option<u8>,
    vpn: bool,
    show_speed: bool,
    download_text: String,
    upload_text: String,
    download_history: Vec<u64>,
    upload_history: Vec<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventView {
    kind: &'static str,
    message: String,
}

pub fn status_activity(ctx: &SourceContext, input: &StatusInput) -> Activity {
    let link = &input.connection.link;
    let download_text = format_rate(input.throughput.download_bytes_per_second);
    let upload_text = format_rate(input.throughput.upload_bytes_per_second);
    let mut compact = CompactView::new(icon_of(link), label_of(link)).accent(accent_of(input.connection));
    if input.show_speed && !link.is_offline() {
        compact = compact.subtitle(format!("↓ {download_text}  ↑ {upload_text}"));
    }
    if input.connection.vpn_active {
        compact = compact.badge(VPN_BADGE);
    }
    let view = StatusView {
        link: link_name(link),
        label: label_of(link),
        ssid: ssid_of(link),
        signal_bars: signal_bars_of(link),
        vpn: input.connection.vpn_active,
        show_speed: input.show_speed,
        download_text,
        upload_text,
        download_history: input.history.download_samples(),
        upload_history: input.history.upload_samples(),
    };
    ctx.activity(STATUS_KEY, ActivityKind::Network, compact)
        .priority(Priority::Ambient)
        .expanded(json!(view))
}

pub fn event_activity(ctx: &SourceContext, event: &NetworkEvent) -> Activity {
    let (icon, kind, message, accent, alert) = describe(event);
    let compact = CompactView::new(icon, message.clone()).accent(accent);
    ctx.activity(EVENT_KEY, ActivityKind::Network, compact)
        .priority(Priority::High)
        .alert(alert)
        .ttl_ms(EVENT_TTL_MS)
        .expanded(json!(EventView { kind, message }))
}

fn describe(event: &NetworkEvent) -> (&'static str, &'static str, String, &'static str, bool) {
    match event {
        NetworkEvent::WifiConnected { ssid: Some(ssid) } => ("📶", "wifiConnected", format!("Connected to {ssid}"), ACCENT_ONLINE, false),
        NetworkEvent::WifiConnected { ssid: None } => ("📶", "wifiConnected", "Wi-Fi connected".into(), ACCENT_ONLINE, false),
        NetworkEvent::WifiDropped => ("⚠", "wifiDropped", "Wi-Fi disconnected".into(), ACCENT_OFFLINE, true),
        NetworkEvent::VpnConnected => ("🔒", "vpnConnected", "VPN connected".into(), ACCENT_VPN, false),
        NetworkEvent::VpnDisconnected => ("🔓", "vpnDisconnected", "VPN disconnected".into(), ACCENT_OFFLINE, true),
    }
}

fn icon_of(link: &Link) -> &'static str {
    match link {
        Link::Wifi { .. } => "📶",
        Link::Wired | Link::Cellular => "🌐",
        Link::Offline => "⚠",
    }
}

fn link_name(link: &Link) -> &'static str {
    match link {
        Link::Wifi { .. } => "wifi",
        Link::Wired => "wired",
        Link::Cellular => "cellular",
        Link::Offline => "offline",
    }
}

fn label_of(link: &Link) -> String {
    match link {
        Link::Wifi { ssid: Some(ssid), .. } => ssid.clone(),
        Link::Wifi { ssid: None, .. } => "Wi-Fi".into(),
        Link::Wired => "Ethernet".into(),
        Link::Cellular => "Cellular".into(),
        Link::Offline => "Offline".into(),
    }
}

fn ssid_of(link: &Link) -> Option<String> {
    match link {
        Link::Wifi { ssid, .. } => ssid.clone(),
        _ => None,
    }
}

fn signal_bars_of(link: &Link) -> Option<u8> {
    match link {
        Link::Wifi { signal_bars, .. } => *signal_bars,
        _ => None,
    }
}

fn accent_of(connection: &Connection) -> &'static str {
    if connection.link.is_offline() {
        ACCENT_OFFLINE
    } else if connection.vpn_active {
        ACCENT_VPN
    } else {
        ACCENT_ONLINE
    }
}
