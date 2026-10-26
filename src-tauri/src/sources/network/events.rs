use super::wifi::Link;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    pub link: Link,
    pub vpn_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEvent {
    WifiConnected { ssid: Option<String> },
    WifiDropped,
    VpnConnected,
    VpnDisconnected,
}

#[derive(Default)]
pub struct ConnectionTracker {
    committed: Option<Connection>,
    pending_drop: Option<Connection>,
}

impl ConnectionTracker {
    pub fn current(&self) -> Option<&Connection> {
        self.committed.as_ref()
    }

    pub fn is_confirming_drop(&self) -> bool {
        self.pending_drop.is_some()
    }

    pub fn observe(&mut self, observed: Connection) -> Option<NetworkEvent> {
        let Some(before) = self.committed.clone() else {
            self.committed = Some(observed);
            return None;
        };
        if self.is_unconfirmed_drop(&before, &observed) {
            self.pending_drop = Some(observed);
            return None;
        }
        self.pending_drop = None;
        let event = detect_event(&before, &observed);
        self.committed = Some(observed);
        event
    }

    fn is_unconfirmed_drop(&self, before: &Connection, observed: &Connection) -> bool {
        let drops_link = observed.link.is_offline() && !before.link.is_offline();
        drops_link && self.pending_drop.is_none()
    }
}

fn detect_event(before: &Connection, now: &Connection) -> Option<NetworkEvent> {
    wifi_event(&before.link, &now.link).or_else(|| vpn_event(before.vpn_active, now.vpn_active))
}

fn wifi_event(before: &Link, now: &Link) -> Option<NetworkEvent> {
    match (before, now) {
        (Link::Wifi { .. }, Link::Offline) => Some(NetworkEvent::WifiDropped),
        (_, Link::Wifi { ssid, .. }) if !before.is_same_connection(now) => {
            Some(NetworkEvent::WifiConnected { ssid: ssid.clone() })
        }
        _ => None,
    }
}

fn vpn_event(before: bool, now: bool) -> Option<NetworkEvent> {
    match (before, now) {
        (false, true) => Some(NetworkEvent::VpnConnected),
        (true, false) => Some(NetworkEvent::VpnDisconnected),
        _ => None,
    }
}
