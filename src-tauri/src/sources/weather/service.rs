use std::time::Duration;

use reqwest::Client;
use tauri::async_runtime::block_on;

use crate::core::settings::WeatherSettings;
use crate::core::source::SourceContext;

use super::alerts::{detect, AlertTracker};
use super::geolocate::{Location, Locator};
use super::open_meteo::{fetch_conditions, Conditions};
use super::presenter::{alert_activity, current_activity};
use super::units::Units;

const POLL_INTERVAL: Duration = Duration::from_secs(15 * 60);
const FAILURE_BASE_INTERVAL: Duration = Duration::from_secs(60);
const MAX_FAILURE_EXPONENT: u32 = 4;

pub struct WeatherService {
    ctx: SourceContext,
    client: Client,
    locator: Locator,
    alerts: AlertTracker,
    applied_settings: WeatherSettings,
    consecutive_failures: u32,
}

impl WeatherService {
    pub fn new(ctx: SourceContext, client: Client) -> Self {
        let applied_settings = ctx.current_settings().weather;
        Self {
            ctx,
            client,
            locator: Locator::default(),
            alerts: AlertTracker::default(),
            applied_settings,
            consecutive_failures: 0,
        }
    }

    pub fn settings_changed(&self) -> bool {
        self.ctx.current_settings().weather != self.applied_settings
    }

    pub fn refresh(&mut self) -> Duration {
        let settings = self.ctx.current_settings().weather;
        self.applied_settings = settings.clone();
        match block_on(self.fetch(&settings)) {
            Some((location, conditions)) => {
                self.publish(&location, &conditions, Units::parse(&settings.units));
                self.consecutive_failures = 0;
                POLL_INTERVAL
            }
            None => self.back_off(),
        }
    }

    async fn fetch(&mut self, settings: &WeatherSettings) -> Option<(Location, Conditions)> {
        let location = self.locator.locate(&self.client, settings).await?;
        match fetch_conditions(&self.client, &location).await {
            Ok(conditions) => Some((location, conditions?)),
            Err(error) => {
                log::warn!("weather: forecast request failed: {error}");
                None
            }
        }
    }

    fn publish(&mut self, location: &Location, conditions: &Conditions, units: Units) {
        self.ctx.upsert(current_activity(conditions, location, units));
        for kind in self.alerts.newly_raised(detect(conditions)) {
            self.ctx.upsert(alert_activity(kind, conditions, location, units));
        }
    }

    fn back_off(&mut self) -> Duration {
        let exponent = self.consecutive_failures.min(MAX_FAILURE_EXPONENT);
        self.consecutive_failures += 1;
        (FAILURE_BASE_INTERVAL * 2u32.pow(exponent)).min(POLL_INTERVAL)
    }
}
