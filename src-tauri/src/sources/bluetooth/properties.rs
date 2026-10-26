use windows::Foundation::Collections::{IIterable, IMapView};
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

    pub fn from_properties(properties: &IMapView<HSTRING, IInspectable>) -> Self {
        Self {
            address: read_property(properties, ADDRESS_KEY).and_then(|value| value.GetString().ok()).map(normalize_address),
            connected: read_property(properties, CONNECTED_KEY).and_then(|value| value.GetBoolean().ok()),
            battery: read_property(properties, CLASSIC_BATTERY_KEY).and_then(|value| read_percent(&value)),
        }
    }
}

pub fn requested_property_keys() -> IIterable<HSTRING> {
    let keys = [CONNECTED_KEY, ADDRESS_KEY, CLASSIC_BATTERY_KEY].map(HSTRING::from).to_vec();
    IIterable::<HSTRING>::from(keys)
}

pub fn normalize_address(address: impl ToString) -> String {
    address
        .to_string()
        .chars()
        .filter(char::is_ascii_hexdigit)
        .collect::<String>()
        .to_lowercase()
}

fn read_property(properties: &IMapView<HSTRING, IInspectable>, key: &str) -> Option<IPropertyValue> {
    let key = HSTRING::from(key);
    if !properties.HasKey(&key).unwrap_or(false) {
        return None;
    }
    properties.Lookup(&key).ok()?.cast::<IPropertyValue>().ok()
}

fn read_percent(value: &IPropertyValue) -> Option<u8> {
    let raw = value.GetUInt8().map(i32::from).or_else(|_| value.GetInt32()).ok()?;
    u8::try_from(raw).ok().filter(|percent| *percent <= 100)
}
