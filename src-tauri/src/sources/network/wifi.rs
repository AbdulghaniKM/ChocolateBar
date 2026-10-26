use windows::Networking::Connectivity::{ConnectionProfile, NetworkInformation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    Offline,
    Wifi { ssid: Option<String>, signal_bars: Option<u8> },
    Wired,
    Cellular,
}

impl Link {
    pub fn read() -> Self {
        match NetworkInformation::GetInternetConnectionProfile() {
            Ok(profile) => classify(&profile),
            Err(_) => Link::Offline,
        }
    }

    pub fn is_offline(&self) -> bool {
        matches!(self, Link::Offline)
    }

    pub fn is_same_connection(&self, other: &Link) -> bool {
        match (self, other) {
            (Link::Wifi { ssid: mine, .. }, Link::Wifi { ssid: theirs, .. }) => mine == theirs,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

fn classify(profile: &ConnectionProfile) -> Link {
    if profile.IsWlanConnectionProfile().unwrap_or(false) {
        Link::Wifi { ssid: read_ssid(profile), signal_bars: read_signal_bars(profile) }
    } else if profile.IsWwanConnectionProfile().unwrap_or(false) {
        Link::Cellular
    } else {
        Link::Wired
    }
}

fn read_ssid(profile: &ConnectionProfile) -> Option<String> {
    let ssid = profile.WlanConnectionProfileDetails().ok()?.GetConnectedSsid().ok()?.to_string();
    (!ssid.is_empty()).then_some(ssid)
}

fn read_signal_bars(profile: &ConnectionProfile) -> Option<u8> {
    profile.GetSignalBars().ok()?.Value().ok()
}
