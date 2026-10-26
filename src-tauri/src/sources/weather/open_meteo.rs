use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use super::geolocate::Location;

const FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const USER_AGENT: &str = concat!("ChocolateBar/", env!("CARGO_PKG_VERSION"));
const CURRENT_FIELDS: &str = "temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,precipitation,wind_gusts_10m,wind_speed_10m,is_day";
const FORECAST_STEPS: u32 = 8;
const MINUTES_PER_STEP: u32 = 15;

pub fn build_http_client() -> reqwest::Result<Client> {
    Client::builder().user_agent(USER_AGENT).timeout(REQUEST_TIMEOUT).build()
}

#[derive(Debug, Clone)]
pub struct Conditions {
    pub temperature_c: f64,
    pub feels_like_c: Option<f64>,
    pub humidity_percent: Option<u32>,
    pub weather_code: u32,
    pub precipitation_mm: f64,
    pub gust_kmh: f64,
    pub wind_kmh: Option<f64>,
    pub is_day: bool,
    pub upcoming_codes: Vec<u32>,
    pub high_c: Option<f64>,
    pub low_c: Option<f64>,
}

impl Conditions {
    pub fn minutes_until(&self, matches: impl Fn(u32) -> bool) -> Option<u32> {
        let step = self.upcoming_codes.iter().position(|&code| matches(code))?;
        Some(step as u32 * MINUTES_PER_STEP)
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct ForecastDto {
    current: Option<CurrentDto>,
    minutely_15: Option<MinutelyDto>,
    daily: Option<DailyDto>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct CurrentDto {
    temperature_2m: Option<f64>,
    apparent_temperature: Option<f64>,
    relative_humidity_2m: Option<f64>,
    weather_code: Option<f64>,
    precipitation: Option<f64>,
    wind_gusts_10m: Option<f64>,
    wind_speed_10m: Option<f64>,
    is_day: Option<f64>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct MinutelyDto {
    weather_code: Option<Vec<Option<f64>>>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct DailyDto {
    temperature_2m_max: Option<Vec<Option<f64>>>,
    temperature_2m_min: Option<Vec<Option<f64>>>,
}

fn first_value(series: Option<Vec<Option<f64>>>) -> Option<f64> {
    series?.into_iter().next()?
}

impl ForecastDto {
    fn into_conditions(self) -> Option<Conditions> {
        let current = self.current?;
        let daily = self.daily.unwrap_or_default();
        let upcoming_codes = self
            .minutely_15
            .and_then(|minutely| minutely.weather_code)
            .unwrap_or_default()
            .into_iter()
            .map(|code| code.unwrap_or_default() as u32)
            .collect();
        Some(Conditions {
            temperature_c: current.temperature_2m?,
            feels_like_c: current.apparent_temperature,
            humidity_percent: current.relative_humidity_2m.map(|value| value.round() as u32),
            weather_code: current.weather_code.unwrap_or_default() as u32,
            precipitation_mm: current.precipitation.unwrap_or_default(),
            gust_kmh: current.wind_gusts_10m.unwrap_or_default(),
            wind_kmh: current.wind_speed_10m,
            is_day: current.is_day.is_none_or(|flag| flag >= 1.0),
            upcoming_codes,
            high_c: first_value(daily.temperature_2m_max),
            low_c: first_value(daily.temperature_2m_min),
        })
    }
}

pub async fn fetch_conditions(client: &Client, location: &Location) -> reqwest::Result<Option<Conditions>> {
    let query = [
        ("latitude", format!("{:.4}", location.lat)),
        ("longitude", format!("{:.4}", location.lon)),
        ("current", CURRENT_FIELDS.to_string()),
        ("minutely_15", "weather_code".to_string()),
        ("forecast_minutely_15", FORECAST_STEPS.to_string()),
        ("daily", "temperature_2m_max,temperature_2m_min".to_string()),
        ("forecast_days", "1".to_string()),
        ("timezone", "auto".to_string()),
    ];
    let response = client.get(FORECAST_URL).query(&query).send().await?.error_for_status()?;
    let forecast: ForecastDto = response.json().await?;
    Ok(forecast.into_conditions())
}
