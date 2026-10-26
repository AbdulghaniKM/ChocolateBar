pub mod battery;
pub mod bluetooth;
pub mod buds;
pub mod claude;
pub mod download;
pub mod game;
pub mod keyboard;
pub mod media;
pub mod network;
pub mod notification;
pub mod ocr;
pub mod osd;
pub mod privacy;
pub mod shelf;
pub mod sports;
pub mod timer;
pub mod visualizer;
pub mod weather;

use tauri::{AppHandle, Manager};

use crate::core::settings::SettingsStore;
use crate::core::source::{spawn_source, ActivitySource, SourceContext, SourceFactory};

fn build<S: ActivitySource + Default + 'static>() -> Box<dyn ActivitySource> {
    Box::new(S::default())
}

pub fn all_sources() -> Vec<SourceFactory> {
    vec![
        build::<media::MediaSource>,
        build::<visualizer::VisualizerSource>,
        build::<osd::OsdSource>,
        build::<privacy::PrivacySource>,
        build::<buds::BudsSource>,
        build::<bluetooth::BluetoothSource>,
        build::<battery::BatterySource>,
        build::<keyboard::KeyboardSource>,
        build::<download::DownloadSource>,
        build::<network::NetworkSource>,
        build::<claude::ClaudeSource>,
        build::<notification::NotificationSource>,
        build::<ocr::OcrSource>,
        build::<shelf::ShelfSource>,
        build::<sports::SportsSource>,
        build::<timer::TimerSource>,
        build::<weather::WeatherSource>,
        build::<game::GameSource>,
    ]
}

pub fn spawn_all(app: &AppHandle) {
    let settings = app.state::<SettingsStore>().current();
    for build_source in all_sources() {
        let name = build_source().name();
        if settings.is_source_enabled(name) {
            spawn_source(build_source, SourceContext::for_source(app, name));
        } else {
            log::info!("source {name} disabled in settings");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn registers_eighteen_uniquely_named_sources() {
        let names: HashSet<&str> = all_sources().iter().map(|build| build().name()).collect();
        assert_eq!(names.len(), 18);
    }
}
