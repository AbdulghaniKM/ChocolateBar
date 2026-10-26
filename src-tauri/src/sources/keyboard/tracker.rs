use super::caps_lock;
use super::layout::{self, KeyboardLayout};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputChange {
    Layout,
    CapsLock,
}

#[derive(Default)]
pub struct InputTracker {
    language_id: Option<u16>,
    layout: Option<KeyboardLayout>,
    caps_lock_on: Option<bool>,
}

impl InputTracker {
    pub fn poll(&mut self) -> Option<InputChange> {
        let layout_changed = self.refresh_layout();
        let caps_changed = self.refresh_caps_lock();
        match (layout_changed, caps_changed) {
            (_, true) => Some(InputChange::CapsLock),
            (true, false) => Some(InputChange::Layout),
            (false, false) => None,
        }
    }

    pub fn layout(&self) -> Option<&KeyboardLayout> {
        self.layout.as_ref()
    }

    pub fn is_caps_lock_on(&self) -> bool {
        self.caps_lock_on.unwrap_or(false)
    }

    fn refresh_layout(&mut self) -> bool {
        let Some(language_id) = layout::foreground_language_id() else {
            return false;
        };
        if self.language_id == Some(language_id) {
            return false;
        }
        let is_first_reading = self.language_id.is_none();
        self.language_id = Some(language_id);
        self.layout = Some(layout::describe_layout(language_id));
        !is_first_reading
    }

    fn refresh_caps_lock(&mut self) -> bool {
        let caps_lock_on = caps_lock::is_caps_lock_on();
        let previous = self.caps_lock_on.replace(caps_lock_on);
        previous.is_some_and(|before| before != caps_lock_on)
    }
}
