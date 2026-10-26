mod model;
mod protocol;
mod rfcomm;
mod session;
mod state;
mod view;

use std::time::Duration;

use crate::core::source::{ActivitySource, SourceContext};
use crate::sources::bluetooth::read_classic_battery;

use rfcomm::{OpenError, PairedBuds};
use session::SessionEnd;
use state::BudsState;
use view::{BudsSnapshot, LinkStatus};

const RESCAN_DELAY: Duration = Duration::from_secs(10);
const WAITING_FOR_CONNECTION_DELAY: Duration = Duration::from_secs(4);
const BATTERY_ONLY_DELAY: Duration = Duration::from_secs(20);
const BACKOFF_BASE: Duration = Duration::from_secs(2);
const BACKOFF_CEILING: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct BudsSource;

impl ActivitySource for BudsSource {
    fn name(&self) -> &'static str {
        "buds"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let mut supervisor = Supervisor::default();
        while ctx.is_running() {
            let delay = supervisor.step(&ctx);
            if !ctx.pause(delay) {
                break;
            }
        }
    }
}

#[derive(Default)]
struct Backoff {
    failures: u32,
}

impl Backoff {
    fn next_delay(&mut self) -> Duration {
        let delay = BACKOFF_BASE.saturating_mul(1u32 << self.failures.min(5));
        self.failures = self.failures.saturating_add(1);
        delay.min(BACKOFF_CEILING)
    }

    fn reset(&mut self) {
        self.failures = 0;
    }
}

struct Presence {
    address: String,
    name: String,
}

#[derive(Default)]
struct Supervisor {
    backoff: Backoff,
    present: Option<Presence>,
}

impl Supervisor {
    fn step(&mut self, ctx: &SourceContext) -> Duration {
        let paired = match rfcomm::find_paired_buds() {
            Ok(paired) => paired,
            Err(error) => {
                log::warn!("buds: paired device lookup failed: {error}");
                return self.backoff.next_delay();
            }
        };
        let connected = paired.into_iter().find(PairedBuds::is_connected);
        self.announce_presence(ctx, connected.as_ref());
        match connected {
            Some(buds) => self.serve(ctx, &buds),
            None if self.present.is_none() => RESCAN_DELAY,
            None => WAITING_FOR_CONNECTION_DELAY,
        }
    }

    fn serve(&mut self, ctx: &SourceContext, buds: &PairedBuds) -> Duration {
        if !buds.model.is_supported() {
            publish_without_link(ctx, buds, LinkStatus::Unsupported);
            return BATTERY_ONLY_DELAY;
        }
        match rfcomm::open_link(buds) {
            Ok(link) => self.run_session(ctx, buds, link),
            Err(OpenError::InUse) => {
                publish_without_link(ctx, buds, LinkStatus::InUse);
                self.backoff.next_delay()
            }
            Err(OpenError::Failed(reason)) => {
                log::debug!("buds: RFCOMM connect failed: {reason}");
                publish_without_link(ctx, buds, LinkStatus::Connecting);
                self.backoff.next_delay()
            }
        }
    }

    fn run_session(&mut self, ctx: &SourceContext, buds: &PairedBuds, link: rfcomm::BudsLink) -> Duration {
        self.backoff.reset();
        let end = session::run(ctx, buds, link);
        ctx.remove(&buds.address);
        match end {
            SessionEnd::Shutdown => Duration::ZERO,
            SessionEnd::Disconnected => self.backoff.next_delay(),
        }
    }

    fn announce_presence(&mut self, ctx: &SourceContext, connected: Option<&PairedBuds>) {
        let now_present = connected.map(|buds| Presence { address: buds.address.clone(), name: buds.name.clone() });
        let was_address = self.present.as_ref().map(|presence| presence.address.as_str());
        let is_address = now_present.as_ref().map(|presence| presence.address.as_str());
        if was_address == is_address {
            return;
        }
        if let Some(gone) = &self.present {
            ctx.remove(&gone.address);
            ctx.upsert(view::event_activity(ctx, &gone.address, &gone.name, false));
        }
        if let Some(arrived) = &now_present {
            ctx.upsert(view::event_activity(ctx, &arrived.address, &arrived.name, true));
        }
        self.present = now_present;
    }
}

fn publish_without_link(ctx: &SourceContext, buds: &PairedBuds, link: LinkStatus) {
    let snapshot = BudsSnapshot {
        address: buds.address.clone(),
        name: buds.name.clone(),
        link,
        state: BudsState::default(),
        noise_control: false,
        fallback_percent: read_classic_battery(&buds.device_id),
    };
    ctx.upsert(view::status_activity(ctx, &snapshot));
}
