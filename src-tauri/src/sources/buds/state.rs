use serde::Serialize;

use super::model::BudsModel;
use super::protocol::{EXTENDED_STATUS_UPDATED, NOISE_CONTROLS_UPDATE, STATUS_UPDATED};

const MAX_PERCENT: u8 = 100;
const PLACEMENT_BYTE_STATUS: usize = 5;
const PLACEMENT_BYTE_EXTENDED: usize = 6;
const CASE_BYTE_STATUS: usize = 6;
const CASE_BYTE_EXTENDED: usize = 7;
const CHARGING_BYTE_STATUS: usize = 7;
const NOISE_BYTE_EXTENDED: usize = 12;
const LEFT_CHARGING_BIT: u8 = 1 << 4;
const RIGHT_CHARGING_BIT: u8 = 1 << 2;
const CASE_CHARGING_BIT: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NoiseMode {
    Off,
    Anc,
    Ambient,
    Adaptive,
}

impl NoiseMode {
    pub fn wire_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Anc => 1,
            Self::Ambient => 2,
            Self::Adaptive => 3,
        }
    }

    pub fn from_wire_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Off),
            1 => Some(Self::Anc),
            2 => Some(Self::Ambient),
            3 => Some(Self::Adaptive),
            _ => None,
        }
    }

    pub fn from_action_value(value: &str) -> Option<Self> {
        match value {
            "off" => Some(Self::Off),
            "anc" => Some(Self::Anc),
            "ambient" => Some(Self::Ambient),
            "adaptive" => Some(Self::Adaptive),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Anc => "ANC",
            Self::Ambient => "Ambient",
            Self::Adaptive => "Adaptive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Placement {
    Disconnected,
    Wearing,
    Idle,
    InCase,
    ClosedCase,
}

impl Placement {
    fn from_nibble(nibble: u8) -> Self {
        match nibble {
            1 => Self::Wearing,
            2 => Self::Idle,
            3 => Self::InCase,
            4 => Self::ClosedCase,
            _ => Self::Disconnected,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bud {
    pub percent: Option<u8>,
    pub charging: bool,
    pub placement: Placement,
}

impl Default for Bud {
    fn default() -> Self {
        Self { percent: None, charging: false, placement: Placement::Disconnected }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Case {
    pub percent: Option<u8>,
    pub charging: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BudsState {
    pub left: Bud,
    pub right: Bud,
    pub case: Case,
    pub noise: Option<NoiseMode>,
    pub has_report: bool,
}

impl BudsState {
    pub fn apply(&mut self, model: BudsModel, id: u8, payload: &[u8]) {
        match id {
            STATUS_UPDATED => self.apply_status(payload),
            EXTENDED_STATUS_UPDATED => self.apply_extended_status(model, payload),
            NOISE_CONTROLS_UPDATE if model.supports_noise_control() => self.apply_noise_update(payload),
            _ => {}
        }
    }

    pub fn lowest_bud_percent(&self) -> Option<u8> {
        [self.left.percent, self.right.percent].into_iter().flatten().min()
    }

    fn apply_status(&mut self, payload: &[u8]) {
        if payload.len() <= CASE_BYTE_STATUS {
            return;
        }
        self.apply_levels(payload[1], payload[2], payload[CASE_BYTE_STATUS]);
        self.apply_placement(payload[PLACEMENT_BYTE_STATUS]);
        if let Some(charging) = payload.get(CHARGING_BYTE_STATUS) {
            self.apply_charging(*charging);
        }
        self.has_report = true;
    }

    fn apply_extended_status(&mut self, model: BudsModel, payload: &[u8]) {
        if payload.len() <= CASE_BYTE_EXTENDED {
            return;
        }
        self.apply_levels(payload[2], payload[3], payload[CASE_BYTE_EXTENDED]);
        self.apply_placement(payload[PLACEMENT_BYTE_EXTENDED]);
        if model.reports_noise_mode() {
            self.noise = payload.get(NOISE_BYTE_EXTENDED).and_then(|byte| read_noise_mode(model, *byte));
        }
        self.has_report = true;
    }

    fn apply_noise_update(&mut self, payload: &[u8]) {
        if let Some(mode) = payload.first().copied().and_then(NoiseMode::from_wire_byte) {
            self.noise = Some(mode);
        }
    }

    fn apply_levels(&mut self, left: u8, right: u8, case: u8) {
        self.left.percent = valid_percent(left);
        self.right.percent = valid_percent(right);
        self.case.percent = valid_percent(case);
    }

    fn apply_placement(&mut self, packed: u8) {
        self.left.placement = Placement::from_nibble(packed >> 4);
        self.right.placement = Placement::from_nibble(packed & 0x0F);
    }

    fn apply_charging(&mut self, flags: u8) {
        self.left.charging = flags & LEFT_CHARGING_BIT != 0;
        self.right.charging = flags & RIGHT_CHARGING_BIT != 0;
        self.case.charging = flags & CASE_CHARGING_BIT != 0;
    }
}

fn read_noise_mode(model: BudsModel, byte: u8) -> Option<NoiseMode> {
    if model == BudsModel::Live {
        Some(if byte == 1 { NoiseMode::Anc } else { NoiseMode::Off })
    } else {
        NoiseMode::from_wire_byte(byte)
    }
}

fn valid_percent(raw: u8) -> Option<u8> {
    (raw <= MAX_PERCENT).then_some(raw).filter(|percent| *percent > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_report_fills_levels_placement_and_charging() {
        let mut state = BudsState::default();
        let payload = [11, 80, 75, 1, 1, 0x11, 60, 0b0001_0001];
        state.apply(BudsModel::Buds2Pro, STATUS_UPDATED, &payload);
        assert_eq!(state.left.percent, Some(80));
        assert_eq!(state.right.percent, Some(75));
        assert_eq!(state.case.percent, Some(60));
        assert_eq!(state.left.placement, Placement::Wearing);
        assert!(state.left.charging && state.case.charging && !state.right.charging);
    }

    #[test]
    fn extended_report_reads_noise_mode_for_pro_models() {
        let mut state = BudsState::default();
        let mut payload = vec![0u8; 14];
        payload[2] = 50;
        payload[3] = 51;
        payload[7] = 90;
        payload[12] = 2;
        state.apply(BudsModel::Buds2Pro, EXTENDED_STATUS_UPDATED, &payload);
        assert_eq!(state.noise, Some(NoiseMode::Ambient));
    }

    #[test]
    fn short_payloads_are_ignored() {
        let mut state = BudsState::default();
        state.apply(BudsModel::Modern, STATUS_UPDATED, &[1, 2]);
        assert!(!state.has_report);
    }
}
