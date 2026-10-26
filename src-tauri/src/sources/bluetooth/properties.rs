use windows::Foundation::IPropertyValue;
use windows_core::{IInspectable, Interface, HSTRING};

pub const CONNECTED_KEY: &str = "System.Devices.Aep.IsConnected";
pub const ADDRESS_KEY: &str = "System.Devices.Aep.DeviceAddress";
pub const CLASSIC_BATTERY_KEY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    Classic,
    LowEnergy,
}

#[derive(Debug, Clone, Default)]
pub struct PropertyPatch {
    pub address: Option<String>,
    pub connected: Option<bool>,
    pub battery: Option<u8>,
}

impl PropertyPatch {
    pub fn battery_only(percent: u8) -> Self {
        Self { battery: Some(percent), ..Self::default() }
    }

    pub fn from_lookup(lookup: impl Fn(&HSTRING) -> Option<IInspectable>) -> Self {
        let read = |key: &str| lookup(&HSTRING::from(key)).and_then(|value| value.cast::<IPropertyValue>().ok());
        Self {
            address: read(ADDRESS_KEY).and_then(|value| value.GetString().ok()).map(normalize_address),
            connected: read(CONNECTED_KEY).and_then(|value| value.GetBoolean().ok()),
            battery: read(CLASSIC_BATTERY_KEY).and_then(|value| read_percent(&value)),
        }
    }
}

pub fn property_key_names() -> [&'static str; 3] {
    [CONNECTED_KEY, ADDRESS_KEY, CLASSIC_BATTERY_KEY]
}

macro_rules! requested_property_keys {
    () => {{
        let keys = windows::Devices::Enumeration::DevicePicker::new()?.RequestedProperties()?;
        for name in $crate::sources::bluetooth::properties::property_key_names() {
            keys.Append(&windows_core::HSTRING::from(name))?;
        }
        keys
    }};
}

pub(crate) use requested_property_keys;

pub fn normalize_address(address: impl ToString) -> String {
    address
        .to_string()
        .chars()
        .filter(char::is_ascii_hexdigit)
        .collect::<String>()
        .to_lowercase()
}

fn read_percent(value: &IPropertyValue) -> Option<u8> {
    let raw = value.GetUInt8().map(i32::from).or_else(|_| value.GetInt32()).ok()?;
    u8::try_from(raw).ok().filter(|percent| *percent <= 100)
}
