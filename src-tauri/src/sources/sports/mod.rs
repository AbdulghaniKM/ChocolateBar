mod catalog;
mod espn;
mod model;
mod poller;
mod presenter;
mod tracker;

use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use espn::EspnClient;
use poller::Poller;

const EVENT_WAIT_SLICE: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct SportsSource;

impl ActivitySource for SportsSource {
    fn name(&self) -> &'static str {
        "sports"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        match EspnClient::new() {
            Ok(client) => run(ctx.clone(), Poller::new(ctx, client)),
            Err(error) => {
                log::error!("sports: http client unavailable: {error}");
                ctx.idle_until_shutdown();
            }
        }
    }
}

fn run(ctx: SourceContext, mut poller: Poller) {
    let mut next_poll = Instant::now();
    while ctx.is_running() {
        match ctx.next_event(wait_until(next_poll)) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => {
                catalog::answer(command, poller.client(), poller.configured_leagues());
            }
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
        if poller.settings_changed() || Instant::now() >= next_poll {
            next_poll = Instant::now() + poller.poll();
        }
    }
}

fn wait_until(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now()).min(EVENT_WAIT_SLICE)
}
