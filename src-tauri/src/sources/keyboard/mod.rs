mod caps_lock;
mod layout;
mod tracker;
mod view;

use std::time::Duration;

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use tracker::InputTracker;

const POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Default)]
pub struct KeyboardSource;

impl ActivitySource for KeyboardSource {
    fn name(&self) -> &'static str {
        "keyboard"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let mut tracker = InputTracker::default();
        loop {
            match ctx.next_event(POLL_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
            if let Some(change) = tracker.poll() {
                ctx.upsert(view::indicator_activity(&ctx, &tracker, change));
            }
        }
    }
}
