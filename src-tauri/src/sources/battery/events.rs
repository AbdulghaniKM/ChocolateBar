use super::reading::BatteryReading;

const LOW_BATTERY_ALERT_LEVELS: [u8; 3] = [5, 10, 15];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryEvent {
    PluggedIn,
    Unplugged,
    Low,
}

#[derive(Default)]
pub struct EventDetector {
    previous: Option<BatteryReading>,
    deepest_alerted_level: Option<u8>,
}

impl EventDetector {
    pub fn observe(&mut self, current: BatteryReading) -> Option<BatteryEvent> {
        if current.is_plugged_in() {
            self.deepest_alerted_level = None;
        }
        let event = match self.previous {
            Some(before) => self.classify(before, current),
            None => self.low_battery_event(current),
        };
        self.previous = Some(current);
        event
    }

    fn classify(&mut self, before: BatteryReading, current: BatteryReading) -> Option<BatteryEvent> {
        match (before.is_plugged_in(), current.is_plugged_in()) {
            (false, true) => Some(BatteryEvent::PluggedIn),
            (true, false) => Some(BatteryEvent::Unplugged),
            _ => self.low_battery_event(current),
        }
    }

    fn low_battery_event(&mut self, current: BatteryReading) -> Option<BatteryEvent> {
        if !current.is_low() {
            return None;
        }
        let level = LOW_BATTERY_ALERT_LEVELS.into_iter().find(|level| current.percent <= *level)?;
        let is_new_low = self.deepest_alerted_level.is_none_or(|alerted| level < alerted);
        is_new_low.then(|| {
            self.deepest_alerted_level = Some(level);
            BatteryEvent::Low
        })
    }
}
