use windows::System::Power::{BatteryStatus, PowerManager};

const LOW_BATTERY_PERCENT: u8 = 15;
const TICKS_PER_MINUTE: i64 = 600_000_000;
const MAX_PLAUSIBLE_MINUTES: i64 = 48 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeState {
    Discharging,
    Charging,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryReading {
    pub percent: u8,
    pub state: ChargeState,
    pub minutes_remaining: Option<u32>,
}

impl BatteryReading {
    pub fn read() -> Option<Self> {
        let state = read_charge_state()?;
        let percent = PowerManager::RemainingChargePercent().ok()?.clamp(0, 100) as u8;
        Some(Self { percent, state, minutes_remaining: read_minutes_remaining(state) })
    }

    pub fn is_plugged_in(&self) -> bool {
        self.state != ChargeState::Discharging
    }

    pub fn is_low(&self) -> bool {
        !self.is_plugged_in() && self.percent <= LOW_BATTERY_PERCENT
    }
}

fn read_charge_state() -> Option<ChargeState> {
    match PowerManager::BatteryStatus().ok()? {
        BatteryStatus::Discharging => Some(ChargeState::Discharging),
        BatteryStatus::Charging => Some(ChargeState::Charging),
        BatteryStatus::Idle => Some(ChargeState::Full),
        _ => None,
    }
}

fn read_minutes_remaining(state: ChargeState) -> Option<u32> {
    if state != ChargeState::Discharging {
        return None;
    }
    let minutes = PowerManager::RemainingDischargeTime().ok()?.Duration / TICKS_PER_MINUTE;
    (1..=MAX_PLAUSIBLE_MINUTES).contains(&minutes).then_some(minutes as u32)
}
