use windows_core::GUID;

const SPP_LEGACY: GUID = GUID::from_u128(0x00001102_0000_1000_8000_00805f9b34fd);
const SPP_STANDARD: GUID = GUID::from_u128(0x00001101_0000_1000_8000_00805f9b34fb);
const SPP_NEW: GUID = GUID::from_u128(0x2e73a4ad_332d_41fc_90e2_16bef06523f2);

const NAME_MARKER: &str = "galaxy buds";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudsModel {
    Legacy,
    Plus,
    Live,
    Pro,
    Buds2,
    Buds2Pro,
    Modern,
}

impl BudsModel {
    pub fn from_device_name(name: &str) -> Self {
        let lowercase = name.to_lowercase();
        let has = |fragment: &str| lowercase.contains(fragment);
        if has("buds live") {
            Self::Live
        } else if has("buds2 pro") {
            Self::Buds2Pro
        } else if has("buds pro") {
            Self::Pro
        } else if has("buds2") {
            Self::Buds2
        } else if has("buds+") {
            Self::Plus
        } else if has("buds (") {
            Self::Legacy
        } else {
            Self::Modern
        }
    }

    pub fn is_supported(self) -> bool {
        self != Self::Legacy
    }

    pub fn reports_noise_mode(self) -> bool {
        matches!(self, Self::Live | Self::Pro | Self::Buds2 | Self::Buds2Pro | Self::Modern)
    }

    pub fn supports_noise_control(self) -> bool {
        self.reports_noise_mode() && self != Self::Live
    }

    pub fn service_uuids(self) -> Vec<GUID> {
        match self {
            Self::Legacy => vec![SPP_LEGACY],
            Self::Plus | Self::Live | Self::Pro => vec![SPP_STANDARD],
            Self::Buds2 | Self::Buds2Pro | Self::Modern => vec![SPP_NEW, SPP_STANDARD],
        }
    }
}

pub fn is_galaxy_buds(name: &str) -> bool {
    name.to_lowercase().contains(NAME_MARKER)
}
