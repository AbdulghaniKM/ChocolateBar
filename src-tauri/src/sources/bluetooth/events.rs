use std::collections::HashSet;

use super::registry::DeviceDelta;

const LOW_BATTERY_PERCENT: u8 = 15;
const LOW_BATTERY_RECOVERY_PERCENT: u8 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEvent {
    Connected,
    Disconnected,
    LowBattery,
}

#[derive(Default)]
pub struct EventDetector {
    low_battery_alerted: HashSet<String>,
}

impl EventDetector {
    pub fn detect(&mut self, delta: &DeviceDelta) -> Option<DeviceEvent> {
        let was_connected = delta.before.as_ref().is_some_and(|summary| summary.connected);
        let is_connected = delta.after.as_ref().is_some_and(|summary| summary.connected);
        if was_connected == is_connected {
            return self.detect_low_battery(delta, is_connected);
        }
        if is_connected {
            return Some(DeviceEvent::Connected);
        }
        self.low_battery_alerted.remove(&delta.key);
        Some(DeviceEvent::Disconnected)
    }

    fn detect_low_battery(&mut self, delta: &DeviceDelta, is_connected: bool) -> Option<DeviceEvent> {
        let battery = delta.after.as_ref()?.battery?;
        if is_connected && battery <= LOW_BATTERY_PERCENT {
            return self.low_battery_alerted.insert(delta.key.clone()).then_some(DeviceEvent::LowBattery);
        }
        if battery >= LOW_BATTERY_RECOVERY_PERCENT {
            self.low_battery_alerted.remove(&delta.key);
        }
        None
    }
}
