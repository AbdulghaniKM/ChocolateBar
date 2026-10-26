mod alerts;
mod codes;
mod geolocate;
mod open_meteo;
mod presenter;
mod service;
mod units;

use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use service::WeatherService;

const EVENT_WAIT_SLICE: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct WeatherSource;

impl ActivitySource for WeatherSource {
    fn name(&self) -> &'static str {
        "weather"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        match open_meteo::build_http_client() {
            Ok(client) => run(ctx.clone(), WeatherService::new(ctx, client)),
            Err(error) => {
                log::error!("weather: http client unavailable: {error}");
                ctx.idle_until_shutdown();
            }
        }
    }
}

fn run(ctx: SourceContext, mut service: WeatherService) {
    let mut next_refresh = Instant::now();
    while ctx.is_running() {
        match ctx.next_event(wait_until(next_refresh)) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => command.respond_err("command not supported"),
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
        if service.settings_changed() || Instant::now() >= next_refresh {
            next_refresh = Instant::now() + service.refresh();
        }
    }
}

fn wait_until(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now()).min(EVENT_WAIT_SLICE)
}
