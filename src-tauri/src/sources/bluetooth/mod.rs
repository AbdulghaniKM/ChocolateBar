mod battery;
mod events;
mod properties;
mod registry;
mod view;
mod watcher;

use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use battery::LowEnergyBatteryWatchers;
use events::EventDetector;
use registry::{DeviceDelta, DeviceRegistry, DeviceSummary};
use watcher::{BluetoothWatchers, DeviceMessage, WATCHER_COUNT};

pub(crate) use battery::read_classic_battery;

const WAKE_INTERVAL: Duration = Duration::from_millis(250);
const WATCHER_RETRY_DELAY: Duration = Duration::from_secs(15);
const BUDS_SOURCE_NAME: &str = "buds";
const GALAXY_BUDS_NAME: &str = "galaxy buds";

#[derive(Default)]
pub struct BluetoothSource;

impl ActivitySource for BluetoothSource {
    fn name(&self) -> &'static str {
        "bluetooth"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let (sender, receiver) = unbounded();
        let Some(_watchers) = start_watchers_with_retry(&ctx, &sender) else {
            return;
        };
        let mut monitor = BluetoothMonitor::new(sender);
        run_until_shutdown(&ctx, &receiver, &mut monitor);
    }
}

fn start_watchers_with_retry(ctx: &SourceContext, sender: &Sender<DeviceMessage>) -> Option<BluetoothWatchers> {
    loop {
        match BluetoothWatchers::start(sender) {
            Ok(watchers) => return Some(watchers),
            Err(error) => log::warn!("bluetooth: device watcher failed, retrying: {error}"),
        }
        if !ctx.pause(WATCHER_RETRY_DELAY) {
            return None;
        }
    }
}

fn run_until_shutdown(ctx: &SourceContext, receiver: &Receiver<DeviceMessage>, monitor: &mut BluetoothMonitor) {
    loop {
        match ctx.next_event(WAKE_INTERVAL) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => command.respond_err("command not supported"),
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
        for message in receiver.try_iter() {
            monitor.handle(ctx, message);
        }
    }
}

struct BluetoothMonitor {
    registry: DeviceRegistry,
    detector: EventDetector,
    low_energy_batteries: LowEnergyBatteryWatchers,
    messages: Sender<DeviceMessage>,
    enumerations_completed: usize,
}

impl BluetoothMonitor {
    fn new(messages: Sender<DeviceMessage>) -> Self {
        Self {
            registry: DeviceRegistry::default(),
            detector: EventDetector::default(),
            low_energy_batteries: LowEnergyBatteryWatchers::default(),
            messages,
            enumerations_completed: 0,
        }
    }

    fn handle(&mut self, ctx: &SourceContext, message: DeviceMessage) {
        if matches!(message, DeviceMessage::EnumerationCompleted) {
            self.enumerations_completed += 1;
            return;
        }
        let Some(delta) = self.registry.apply(message) else {
            return;
        };
        self.sync_low_energy_batteries(&delta.key);
        if delta.has_changed() {
            self.publish(ctx, &delta);
        }
    }

    fn sync_low_energy_batteries(&mut self, key: &str) {
        for (link_id, connected) in self.registry.low_energy_links(key) {
            let watching = self.low_energy_batteries.is_watching(&link_id);
            if connected && !watching {
                self.low_energy_batteries.watch(link_id, self.messages.clone());
            } else if !connected && watching {
                self.low_energy_batteries.stop(&link_id);
            }
        }
    }

    fn publish(&mut self, ctx: &SourceContext, delta: &DeviceDelta) {
        let shown = delta.after.as_ref().filter(|summary| is_shown(ctx, summary));
        match shown {
            Some(summary) if summary.connected && summary.battery.is_some() => {
                ctx.upsert(view::device_activity(ctx, &delta.key, summary));
            }
            _ => ctx.remove(&view::device_key(&delta.key)),
        }
        if self.enumerations_completed >= WATCHER_COUNT {
            self.publish_event(ctx, delta);
        }
    }

    fn publish_event(&mut self, ctx: &SourceContext, delta: &DeviceDelta) {
        let summary = delta.after.as_ref().or(delta.before.as_ref());
        let Some(summary) = summary.filter(|summary| is_shown(ctx, summary)) else {
            return;
        };
        if let Some(event) = self.detector.detect(delta) {
            ctx.upsert(view::event_activity(ctx, &delta.key, summary, event));
        }
    }
}

fn is_shown(ctx: &SourceContext, summary: &DeviceSummary) -> bool {
    let handled_by_buds_source = summary.name.to_lowercase().contains(GALAXY_BUDS_NAME)
        && ctx.current_settings().is_source_enabled(BUDS_SOURCE_NAME);
    !handled_by_buds_source
}
