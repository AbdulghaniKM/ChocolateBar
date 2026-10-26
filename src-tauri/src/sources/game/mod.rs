mod epic;
mod model;
mod playtime;
mod presenter;
mod process_scan;
mod registry;
mod service;
mod steam;

use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use service::GameService;

const SCAN_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct GameSource;

impl ActivitySource for GameSource {
    fn name(&self) -> &'static str {
        "game"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let mut service = GameService::new(ctx.clone());
        let mut next_scan = Instant::now();
        while ctx.is_running() {
            match ctx.next_event(next_scan.saturating_duration_since(Instant::now())) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => service.answer(command),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
            if Instant::now() >= next_scan {
                service.scan();
                next_scan = Instant::now() + SCAN_INTERVAL;
            }
        }
    }
}
