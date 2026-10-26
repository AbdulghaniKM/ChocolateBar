mod events;
mod reading;
mod signals;
mod view;

use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use events::EventDetector;
use reading::BatteryReading;
use signals::PowerSignals;

const WAKE_INTERVAL: Duration = Duration::from_millis(500);
const HEARTBEAT_WITH_SIGNALS: Duration = Duration::from_secs(30);
const HEARTBEAT_POLLING_ONLY: Duration = Duration::from_secs(2);

#[derive(Default)]
pub struct BatterySource;

impl ActivitySource for BatterySource {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let signals = subscribe_to_power_changes();
        let heartbeat = if signals.is_some() { HEARTBEAT_WITH_SIGNALS } else { HEARTBEAT_POLLING_ONLY };
        run_until_shutdown(&ctx, signals.as_ref(), heartbeat);
    }
}

fn subscribe_to_power_changes() -> Option<PowerSignals> {
    PowerSignals::subscribe()
        .inspect_err(|error| log::warn!("battery: power events unavailable, polling instead: {error}"))
        .ok()
}

fn run_until_shutdown(ctx: &SourceContext, signals: Option<&PowerSignals>, heartbeat: Duration) {
    let mut publisher = BatteryPublisher::default();
    let mut last_refresh: Option<Instant> = None;
    loop {
        match ctx.next_event(WAKE_INTERVAL) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => command.respond_err("command not supported"),
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
        let signalled = signals.is_some_and(PowerSignals::take_pending_change);
        let heartbeat_due = last_refresh.is_none_or(|at| at.elapsed() >= heartbeat);
        if signalled || heartbeat_due {
            publisher.publish(ctx);
            last_refresh = Some(Instant::now());
        }
    }
}

#[derive(Default)]
struct BatteryPublisher {
    detector: EventDetector,
    last_published: Option<BatteryReading>,
}

impl BatteryPublisher {
    fn publish(&mut self, ctx: &SourceContext) {
        let Some(reading) = BatteryReading::read() else {
            self.clear(ctx);
            return;
        };
        if self.last_published == Some(reading) {
            return;
        }
        self.last_published = Some(reading);
        ctx.upsert(view::status_activity(ctx, reading));
        if let Some(event) = self.detector.observe(reading) {
            ctx.upsert(view::event_activity(ctx, reading, event));
        }
    }

    fn clear(&mut self, ctx: &SourceContext) {
        if self.last_published.take().is_some() {
            ctx.remove(view::STATUS_KEY);
            ctx.remove(view::EVENT_KEY);
        }
    }
}
