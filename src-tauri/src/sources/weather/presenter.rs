use serde::Serialize;

use crate::core::activity::{Activity, ActivityKind, CompactView, Priority};

use super::alerts::AlertKind;
use super::codes::{description_for, icon_for, is_rain, is_storm};
use super::geolocate::Location;
use super::open_meteo::Conditions;
use super::units::Units;

const SOURCE_NAME: &str = "weather";
const CURRENT_KEY: &str = "current";
const ALERT_TTL_MS: u64 = 8_000;
const ALERT_ACCENT: &str = "#f97316";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WeatherView {
    icon: &'static str,
    description: &'static str,
    temperature: i32,
    temperature_unit: &'static str,
    feels_like: Option<i32>,
    humidity_percent: Option<u32>,
    wind: Option<i32>,
    gust: i32,
    speed_unit: &'static str,
    precipitation_mm: f64,
    high: Option<i32>,
    low: Option<i32>,
    city: Option<String>,
    alert: Option<String>,
}

fn view_of(conditions: &Conditions, location: &Location, units: Units) -> WeatherView {
    WeatherView {
        icon: icon_for(conditions.weather_code, conditions.is_day),
        description: description_for(conditions.weather_code),
        temperature: units.temperature(conditions.temperature_c),
        temperature_unit: units.temperature_unit(),
        feels_like: conditions.feels_like_c.map(|value| units.temperature(value)),
        humidity_percent: conditions.humidity_percent,
        wind: conditions.wind_kmh.map(|value| units.speed(value)),
        gust: units.speed(conditions.gust_kmh),
        speed_unit: units.speed_unit(),
        precipitation_mm: conditions.precipitation_mm,
        high: conditions.high_c.map(|value| units.temperature(value)),
        low: conditions.low_c.map(|value| units.temperature(value)),
        city: location.city.clone(),
        alert: None,
    }
}

fn compact_of(view: &WeatherView) -> CompactView {
    let place = view.city.as_deref().map(|city| format!(" \u{00B7} {city}")).unwrap_or_default();
    CompactView::new(view.icon, format!("{}\u{00B0}", view.temperature))
        .subtitle(format!("{}{place}", view.description))
}

fn activity_of(key: &str, compact: CompactView, view: &WeatherView) -> Activity {
    let expanded = serde_json::to_value(view).unwrap_or_default();
    Activity::new(SOURCE_NAME, key, ActivityKind::Weather, compact).expanded(expanded)
}

pub fn current_activity(conditions: &Conditions, location: &Location, units: Units) -> Activity {
    let view = view_of(conditions, location, units);
    activity_of(CURRENT_KEY, compact_of(&view), &view).priority(Priority::Ambient)
}

pub fn alert_activity(
    kind: AlertKind,
    conditions: &Conditions,
    location: &Location,
    units: Units,
) -> Activity {
    let (icon, title, message) = alert_text(kind, conditions, units);
    let mut view = view_of(conditions, location, units);
    view.alert = Some(format!("{title} \u{2014} {message}"));
    let compact = CompactView::new(icon, title).subtitle(message).accent(ALERT_ACCENT);
    activity_of(kind.key(), compact, &view).priority(Priority::High).ttl_ms(ALERT_TTL_MS)
}

fn alert_text(kind: AlertKind, conditions: &Conditions, units: Units) -> (&'static str, &'static str, String) {
    match kind {
        AlertKind::Storm => ("\u{26C8}\u{FE0F}", "Storm warning", storm_message(conditions)),
        AlertKind::RainSoon => ("\u{1F327}\u{FE0F}", "Rain soon", rain_message(conditions)),
        AlertKind::StrongGusts => {
            let gust = format!("Gusts up to {} {}", units.speed(conditions.gust_kmh), units.speed_unit());
            ("\u{1F4A8}", "Strong wind", gust)
        }
        AlertKind::ExtremeHeat => ("\u{1F975}", "Extreme heat", temperature_message(conditions, units)),
        AlertKind::ExtremeCold => ("\u{1F976}", "Freezing cold", temperature_message(conditions, units)),
    }
}

fn storm_message(conditions: &Conditions) -> String {
    match conditions.minutes_until(is_storm) {
        Some(0) | None => "Thunderstorm overhead".into(),
        Some(minutes) => format!("Thunderstorm in about {minutes} min"),
    }
}

fn rain_message(conditions: &Conditions) -> String {
    let minutes = conditions.minutes_until(is_rain).unwrap_or_default();
    format!("Rain in about {minutes} min")
}

fn temperature_message(conditions: &Conditions, units: Units) -> String {
    let degrees = units.temperature(conditions.temperature_c);
    format!("{degrees}{}", units.temperature_unit())
}
