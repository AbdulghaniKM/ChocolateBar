use std::time::{Duration, Instant};

use reqwest::Client;
use serde::Deserialize;

use crate::core::settings::WeatherSettings;

const IP_LOCATION_URL: &str = "https://ipapi.co/json/";
const CACHE_LIFETIME: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Clone)]
pub struct Location {
    pub lat: f64,
    pub lon: f64,
    pub city: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct IpLocationDto {
    latitude: Option<f64>,
    longitude: Option<f64>,
    city: Option<String>,
}

impl IpLocationDto {
    fn into_location(self) -> Option<Location> {
        Some(Location { lat: self.latitude?, lon: self.longitude?, city: self.city })
    }
}

struct CachedLocation {
    location: Location,
    fetched_at: Instant,
}

impl CachedLocation {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < CACHE_LIFETIME
    }
}

#[derive(Default)]
pub struct Locator {
    cached: Option<CachedLocation>,
}

impl Locator {
    pub async fn locate(&mut self, client: &Client, settings: &WeatherSettings) -> Option<Location> {
        if let Some(configured) = configured_location(settings) {
            return Some(configured);
        }
        if let Some(cached) = self.cached.as_ref().filter(|cached| cached.is_fresh()) {
            return Some(cached.location.clone());
        }
        match fetch_ip_location(client).await {
            Ok(Some(location)) => Some(self.remember(location)),
            Ok(None) => self.stale_location(),
            Err(error) => {
                log::warn!("weather: ip geolocation failed: {error}");
                self.stale_location()
            }
        }
    }

    fn remember(&mut self, location: Location) -> Location {
        self.cached = Some(CachedLocation { location: location.clone(), fetched_at: Instant::now() });
        location
    }

    fn stale_location(&self) -> Option<Location> {
        self.cached.as_ref().map(|cached| cached.location.clone())
    }
}

fn configured_location(settings: &WeatherSettings) -> Option<Location> {
    Some(Location { lat: settings.lat?, lon: settings.lon?, city: settings.city.clone() })
}

async fn fetch_ip_location(client: &Client) -> reqwest::Result<Option<Location>> {
    let response = client.get(IP_LOCATION_URL).send().await?.error_for_status()?;
    let dto: IpLocationDto = response.json().await?;
    Ok(dto.into_location())
}
