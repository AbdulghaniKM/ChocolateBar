use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver};

use crate::core::activity::ActionRequest;
use crate::core::source::{SourceContext, SourceEvent};

use super::protocol::{encode, FrameDecoder, NOISE_CONTROLS};
use super::rfcomm::{BudsLink, LinkEvent, PairedBuds};
use super::state::{BudsState, NoiseMode};
use super::view::{self, BudsSnapshot, LinkStatus};

const WAKE_INTERVAL: Duration = Duration::from_millis(100);
const PRESENCE_CHECK_INTERVAL: Duration = Duration::from_secs(3);
const NOISE_ACTION: &str = "noise";

pub enum SessionEnd {
    Disconnected,
    Shutdown,
}

struct Session<'a> {
    ctx: &'a SourceContext,
    buds: &'a PairedBuds,
    link: BudsLink,
    events: Receiver<LinkEvent>,
    decoder: FrameDecoder,
    state: BudsState,
}

pub fn run(ctx: &SourceContext, buds: &PairedBuds, link: BudsLink) -> SessionEnd {
    let (sender, events) = unbounded();
    link.spawn_reader(sender);
    let mut session = Session { ctx, buds, link, events, decoder: FrameDecoder::default(), state: BudsState::default() };
    session.run_until_end()
}

impl Session<'_> {
    fn run_until_end(&mut self) -> SessionEnd {
        let mut last_presence_check = Instant::now();
        loop {
            let mut changed = match self.ctx.next_event(WAKE_INTERVAL) {
                SourceEvent::Shutdown => return SessionEnd::Shutdown,
                SourceEvent::Action(request) => self.handle_action(&request),
                SourceEvent::Command(command) => {
                    command.respond_err("command not supported");
                    false
                }
                SourceEvent::Idle => false,
            };
            match self.drain_link_events() {
                Some(link_changed) => changed |= link_changed,
                None => return SessionEnd::Disconnected,
            }
            if changed {
                self.publish();
            }
            if last_presence_check.elapsed() >= PRESENCE_CHECK_INTERVAL {
                last_presence_check = Instant::now();
                if !self.buds.is_connected() {
                    return SessionEnd::Disconnected;
                }
            }
        }
    }

    fn drain_link_events(&mut self) -> Option<bool> {
        let mut changed = false;
        for event in self.events.try_iter().collect::<Vec<_>>() {
            match event {
                LinkEvent::Closed => return None,
                LinkEvent::Data(bytes) => changed |= self.absorb(&bytes),
            }
        }
        Some(changed)
    }

    fn absorb(&mut self, bytes: &[u8]) -> bool {
        let before = self.state.clone();
        for frame in self.decoder.push(bytes) {
            self.state.apply(self.buds.model, frame.id, &frame.payload);
        }
        self.state != before
    }

    fn handle_action(&mut self, request: &ActionRequest) -> bool {
        let wanted = request.value.as_ref().and_then(|value| value.as_str()).and_then(NoiseMode::from_action_value);
        let Some(mode) = wanted.filter(|_| request.action == NOISE_ACTION) else {
            return false;
        };
        if !self.buds.model.supports_noise_control() {
            return false;
        }
        match self.link.send(&encode(NOISE_CONTROLS, &[mode.wire_byte()])) {
            Ok(()) => {
                self.state.noise = Some(mode);
                true
            }
            Err(error) => {
                log::warn!("buds: failed to send noise control: {error}");
                false
            }
        }
    }

    fn publish(&self) {
        if !self.state.has_report {
            return;
        }
        let snapshot = BudsSnapshot {
            address: self.buds.address.clone(),
            name: self.buds.name.clone(),
            link: LinkStatus::Connected,
            state: self.state.clone(),
            noise_control: self.buds.model.supports_noise_control(),
            fallback_percent: None,
        };
        self.ctx.upsert(view::status_activity(self.ctx, &snapshot));
    }
}
