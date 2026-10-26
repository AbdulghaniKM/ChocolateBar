use std::collections::BTreeSet;

use super::codes::{is_rain, is_storm};
use super::open_meteo::Conditions;

const GUST_THRESHOLD_KMH: f64 = 60.0;
const HEAT_THRESHOLD_C: f64 = 42.0;
const COLD_THRESHOLD_C: f64 = -5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertKind {
    Storm,
    RainSoon,
    StrongGusts,
    ExtremeHeat,
    ExtremeCold,
}

impl AlertKind {
    pub fn key(self) -> &'static str {
        match self {
            Self::Storm => "alert:storm",
            Self::RainSoon => "alert:rain-soon",
            Self::StrongGusts => "alert:gusts",
            Self::ExtremeHeat => "alert:heat",
            Self::ExtremeCold => "alert:cold",
        }
    }
}

pub fn detect(conditions: &Conditions) -> BTreeSet<AlertKind> {
    let mut active = BTreeSet::new();
    if is_storm(conditions.weather_code) || conditions.minutes_until(is_storm).is_some() {
        active.insert(AlertKind::Storm);
    }
    if !is_rain(conditions.weather_code) && conditions.minutes_until(is_rain).is_some() {
        active.insert(AlertKind::RainSoon);
    }
    if conditions.gust_kmh >= GUST_THRESHOLD_KMH {
        active.insert(AlertKind::StrongGusts);
    }
    if conditions.temperature_c >= HEAT_THRESHOLD_C {
        active.insert(AlertKind::ExtremeHeat);
    }
    if conditions.temperature_c <= COLD_THRESHOLD_C {
        active.insert(AlertKind::ExtremeCold);
    }
    active
}

#[derive(Default)]
pub struct AlertTracker {
    active: BTreeSet<AlertKind>,
}

impl AlertTracker {
    pub fn newly_raised(&mut self, current: BTreeSet<AlertKind>) -> Vec<AlertKind> {
        let fresh = current.difference(&self.active).copied().collect();
        self.active = current;
        fresh
    }
}
