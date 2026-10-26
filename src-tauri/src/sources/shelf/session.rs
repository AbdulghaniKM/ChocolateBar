use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tauri_plugin_opener::OpenerExt;

use super::cards::{drop_hover_activity, files_activity, DROP_KEY, FILES_KEY};
use super::drag_icon::ensure_drag_icon;
use super::drop_events::DropSignal;
use super::files::ShelfFiles;
use crate::core::activity::{ActionRequest, SourceCommand};
use crate::core::source::SourceContext;

const PROMOTION_DURATION: Duration = Duration::from_secs(4);

pub struct ShelfSession {
    ctx: SourceContext,
    files: ShelfFiles,
    promoted_until: Option<Instant>,
}

impl ShelfSession {
    pub fn new(ctx: SourceContext) -> Self {
        Self { ctx, files: ShelfFiles::default(), promoted_until: None }
    }

    pub fn on_drop_signal(&mut self, signal: DropSignal) {
        match signal {
            DropSignal::Hover(true) => self.ctx.upsert(drop_hover_activity(&self.ctx)),
            DropSignal::Hover(false) => self.ctx.remove(DROP_KEY),
            DropSignal::Dropped(paths) => self.accept_dropped(paths),
        }
    }

    pub fn handle_action(&mut self, request: ActionRequest) {
        let path = request.value.as_ref().and_then(Value::as_str);
        match (request.action.as_str(), path) {
            ("open", Some(path)) => self.open(path),
            ("remove", Some(path)) => self.remove(path),
            ("clear", _) => self.clear(),
            (other, _) => log::debug!("shelf ignored action {other}"),
        }
    }

    pub fn handle_command(&self, command: SourceCommand) {
        match command.command.as_str() {
            "drag_icon" => match ensure_drag_icon() {
                Ok(path) => command.respond_ok(json!({ "path": path.display().to_string() })),
                Err(message) => command.respond_err(message),
            },
            unknown => {
                let message = format!("unknown shelf command {unknown}");
                command.respond_err(message);
            }
        }
    }

    pub fn demote_expired_promotion(&mut self) {
        if self.promoted_until.is_some_and(|deadline| Instant::now() >= deadline) {
            self.promoted_until = None;
            self.publish();
        }
    }

    fn accept_dropped(&mut self, paths: Vec<std::path::PathBuf>) {
        self.ctx.remove(DROP_KEY);
        if self.files.add_all(paths) > 0 {
            self.promoted_until = Some(Instant::now() + PROMOTION_DURATION);
            self.publish();
        }
    }

    fn open(&self, path: &str) {
        if !self.files.contains(path) {
            return;
        }
        if let Err(error) = self.ctx.app.opener().open_path(path, None::<&str>) {
            log::warn!("shelf could not open {path}: {error}");
        }
    }

    fn remove(&mut self, path: &str) {
        if self.files.remove(path) {
            self.publish();
        }
    }

    fn clear(&mut self) {
        self.files.clear();
        self.publish();
    }

    fn publish(&self) {
        if self.files.is_empty() {
            self.ctx.remove(FILES_KEY);
            return;
        }
        let just_dropped = self.promoted_until.is_some();
        self.ctx.upsert(files_activity(&self.ctx, &self.files, just_dropped));
    }
}
