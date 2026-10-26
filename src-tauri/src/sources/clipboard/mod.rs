mod capture;
mod history;

use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use self::history::History;
use crate::core::activity::SourceCommand;
use crate::core::clipboard;
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const POLL_INTERVAL: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct ClipboardSource;

impl ActivitySource for ClipboardSource {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let mut session = Session { history: History::default(), seen_sequence: clipboard::current_sequence() };
        while ctx.is_running() {
            match ctx.next_event(POLL_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => session.handle(&ctx, command),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
            session.capture_changes(&ctx);
        }
    }
}

struct Session {
    history: History,
    seen_sequence: u32,
}

#[derive(Deserialize)]
struct EntryRequest {
    id: u64,
    #[serde(default)]
    pinned: bool,
}

impl Session {
    fn capture_changes(&mut self, ctx: &SourceContext) {
        let sequence = clipboard::current_sequence();
        if sequence == self.seen_sequence {
            return;
        }
        self.seen_sequence = sequence;
        if let Some(content) = capture::read_current() {
            self.history.record(content);
            self.publish(ctx);
        }
    }

    fn handle(&mut self, ctx: &SourceContext, command: SourceCommand) {
        let request = serde_json::from_value::<EntryRequest>(command.payload.clone()).ok();
        let name = command.command.clone();
        let changed = match (name.as_str(), request) {
            ("list", _) => false,
            ("copy", Some(request)) => self.copy(request.id),
            ("pin", Some(request)) => {
                self.history.set_pinned(request.id, request.pinned);
                true
            }
            ("remove", Some(request)) => {
                self.history.remove(request.id);
                true
            }
            ("clear", _) => {
                self.history.clear_unpinned();
                true
            }
            (other, _) => return command.respond_err(format!("unknown clipboard command {other}")),
        };
        if changed {
            self.publish(ctx);
        }
        command.respond_ok(self.snapshot());
    }

    fn copy(&mut self, id: u64) -> bool {
        let Some(entry) = self.history.get(id) else { return false };
        if !capture::restore(&entry.content) {
            log::warn!("clipboard history could not restore entry {id}");
            return false;
        }
        self.seen_sequence = clipboard::current_sequence();
        self.history.promote(id);
        true
    }

    fn snapshot(&self) -> Value {
        serde_json::to_value(self.history.view()).unwrap_or(Value::Null)
    }

    fn publish(&self, ctx: &SourceContext) {
        ctx.emit(&self.history.view());
    }
}
