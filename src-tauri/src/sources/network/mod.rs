mod events;
mod history;
mod signal;
mod throughput;
mod view;
mod vpn;
mod wifi;

use std::time::{Duration, Instant};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use events::{Connection, ConnectionTracker};
use history::SpeedHistory;
use signal::NetworkChangeSignal;
use throughput::{Throughput, ThroughputMeter};
use view::StatusInput;
use wifi::Link;

const WAKE_INTERVAL: Duration = Duration::from_millis(250);
const TICK_INTERVAL: Duration = Duration::from_secs(1);
const CONNECTION_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct NetworkSource;

impl ActivitySource for NetworkSource {
    fn name(&self) -> &'static str {
        "network"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let signal = NetworkChangeSignal::subscribe()
            .inspect_err(|error| log::warn!("network: status events unavailable, polling instead: {error}"))
            .ok();
        let mut monitor = NetworkMonitor::new(signal);
        let mut last_tick: Option<Instant> = None;
        loop {
            match ctx.next_event(WAKE_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
            if last_tick.is_none_or(|at| at.elapsed() >= TICK_INTERVAL) {
                last_tick = Some(Instant::now());
                monitor.tick(&ctx);
            }
        }
    }
}

struct NetworkMonitor {
    signal: Option<NetworkChangeSignal>,
    meter: ThroughputMeter,
    history: SpeedHistory,
    tracker: ConnectionTracker,
    throughput: Throughput,
    last_connection_read: Option<Instant>,
}

impl NetworkMonitor {
    fn new(signal: Option<NetworkChangeSignal>) -> Self {
        Self {
            signal,
            meter: ThroughputMeter::default(),
            history: SpeedHistory::default(),
            tracker: ConnectionTracker::default(),
            throughput: Throughput::default(),
            last_connection_read: None,
        }
    }

    fn tick(&mut self, ctx: &SourceContext) {
        self.throughput = self.meter.sample().unwrap_or_default();
        self.history.push(self.throughput);
        if self.connection_refresh_due() {
            self.refresh_connection(ctx);
        }
        self.publish_status(ctx);
    }

    fn connection_refresh_due(&self) -> bool {
        let signalled = self.signal.as_ref().is_some_and(NetworkChangeSignal::take_pending_change);
        let stale = self.last_connection_read.is_none_or(|at| at.elapsed() >= CONNECTION_REFRESH_INTERVAL);
        signalled || stale || self.tracker.is_confirming_drop()
    }

    fn refresh_connection(&mut self, ctx: &SourceContext) {
        self.last_connection_read = Some(Instant::now());
        let observed = Connection { link: Link::read(), vpn_active: vpn::is_vpn_active() };
        if let Some(event) = self.tracker.observe(observed) {
            ctx.upsert(view::event_activity(ctx, &event));
        }
    }

    fn publish_status(&self, ctx: &SourceContext) {
        let Some(connection) = self.tracker.current() else {
            return;
        };
        let input = StatusInput {
            connection,
            throughput: self.throughput,
            history: &self.history,
            show_speed: ctx.current_settings().network.show_speed,
        };
        ctx.upsert(view::status_activity(ctx, &input));
    }
}
