use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Chronometer {
    banked: Duration,
    running_since: Option<Instant>,
}

impl Chronometer {
    pub fn started() -> Self {
        Self { banked: Duration::ZERO, running_since: Some(Instant::now()) }
    }

    pub fn elapsed(&self) -> Duration {
        self.banked + self.running_since.map_or(Duration::ZERO, |since| since.elapsed())
    }

    pub fn is_running(&self) -> bool {
        self.running_since.is_some()
    }

    pub fn pause(&mut self) {
        self.banked = self.elapsed();
        self.running_since = None;
    }

    pub fn resume(&mut self) {
        if self.running_since.is_none() {
            self.running_since = Some(Instant::now());
        }
    }

    pub fn toggle(&mut self) {
        if self.is_running() {
            self.pause();
        } else {
            self.resume();
        }
    }

    pub fn restart(&mut self) {
        *self = Self::started();
    }
}
