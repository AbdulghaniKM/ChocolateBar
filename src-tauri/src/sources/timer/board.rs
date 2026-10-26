use std::time::Duration;

use crate::core::source::SourceContext;

use super::contract::{Control, Timer, TimerKind};
use super::countdown::Countdown;
use super::pomodoro::Pomodoro;
use super::stopwatch::Stopwatch;

struct Slot {
    timer: Box<dyn Timer>,
    published_second: u64,
}

pub struct TimerBoard {
    ctx: SourceContext,
    slots: Vec<Slot>,
}

impl TimerBoard {
    pub fn new(ctx: SourceContext) -> Self {
        Self { ctx, slots: Vec::new() }
    }

    pub fn active_keys(&self) -> Vec<&'static str> {
        self.slots.iter().map(|slot| slot.timer.key()).collect()
    }

    pub fn start(&mut self, kind: TimerKind, seconds: Option<u64>) -> Result<(), String> {
        if seconds.is_none() && self.control(kind.key(), Control::Resume) {
            return Ok(());
        }
        let timer = self.create(kind, seconds)?;
        self.slots.retain(|slot| slot.timer.key() != kind.key());
        self.slots.push(Slot { timer, published_second: 0 });
        if let Some(slot) = self.slots.last_mut() {
            publish(&self.ctx, slot);
        }
        Ok(())
    }

    pub fn control(&mut self, key: &str, control: Control) -> bool {
        let Some(slot) = self.slots.iter_mut().find(|slot| slot.timer.key() == key) else {
            return false;
        };
        slot.timer.apply(control);
        publish(&self.ctx, slot);
        true
    }

    pub fn reset(&mut self, key: &str) {
        self.slots.retain(|slot| slot.timer.key() != key);
        self.ctx.remove(key);
    }

    pub fn tick(&mut self) {
        for slot in &mut self.slots {
            refresh(&self.ctx, slot);
        }
        let ctx = &self.ctx;
        self.slots.retain(|slot| {
            let keep = !slot.timer.is_done();
            if !keep {
                ctx.remove(slot.timer.key());
            }
            keep
        });
    }

    fn create(&self, kind: TimerKind, seconds: Option<u64>) -> Result<Box<dyn Timer>, String> {
        match kind {
            TimerKind::Countdown => match seconds.filter(|&amount| amount > 0) {
                Some(amount) => Ok(Box::new(Countdown::start(Duration::from_secs(amount)))),
                None => Err("countdown needs a positive number of seconds".into()),
            },
            TimerKind::Stopwatch => Ok(Box::new(Stopwatch::start())),
            TimerKind::Pomodoro => Ok(Box::new(Pomodoro::start(self.ctx.current_settings().timer))),
        }
    }
}

fn refresh(ctx: &SourceContext, slot: &mut Slot) {
    if let Some(alert) = slot.timer.take_finish_alert() {
        alert.announce(ctx);
        publish(ctx, slot);
    } else if slot.timer.displayed_second() != slot.published_second {
        publish(ctx, slot);
    }
}

fn publish(ctx: &SourceContext, slot: &mut Slot) {
    ctx.upsert(slot.timer.activity());
    slot.published_second = slot.timer.displayed_second();
}
