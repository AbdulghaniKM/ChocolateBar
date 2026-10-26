mod identity;
mod listener;

use std::time::{Duration, Instant};

use serde_json::json;

use crate::core::activity::{ActivityKind, CompactView, Priority, SourceCommand};
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};
use listener::{Toast, ToastWatcher};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const FAILURE_BACKOFF: Duration = Duration::from_secs(15);
const TOAST_TTL_MS: u64 = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ListenerStatus {
    NeedsPackageIdentity,
    AccessDenied,
    Listening,
    Failed(String),
}

impl ListenerStatus {
    fn text(&self) -> String {
        match self {
            Self::NeedsPackageIdentity => "needs package identity".to_owned(),
            Self::AccessDenied => "notification access denied".to_owned(),
            Self::Listening => "listening".to_owned(),
            Self::Failed(reason) => format!("unavailable: {reason}"),
        }
    }

    fn reply(&self) -> serde_json::Value {
        json!({ "status": self.text(), "listening": *self == Self::Listening })
    }
}

#[derive(Default)]
pub struct NotificationSource;

impl ActivitySource for NotificationSource {
    fn name(&self) -> &'static str {
        "notification"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        if !identity::has_package_identity() {
            log::info!("notification mirror idle: app has no package identity");
            return answer_status_until_shutdown(&ctx, &ListenerStatus::NeedsPackageIdentity);
        }
        mirror_toasts(&ctx);
    }
}

fn answer_status_until_shutdown(ctx: &SourceContext, status: &ListenerStatus) {
    loop {
        match ctx.next_event(Duration::from_millis(500)) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => command.respond_ok(status.reply()),
            SourceEvent::Action(_) | SourceEvent::Idle => {}
        }
    }
}

fn mirror_toasts(ctx: &SourceContext) {
    let mut status = ListenerStatus::Listening;
    let mut watcher = open_watcher(&mut status);
    let mut next_poll = Instant::now() + POLL_INTERVAL;
    while ctx.is_running() {
        let wait = next_poll.saturating_duration_since(Instant::now());
        match ctx.next_event(wait) {
            SourceEvent::Shutdown => break,
            SourceEvent::Command(command) => reply_with_status(command, &status),
            SourceEvent::Action(_) => {}
            SourceEvent::Idle => {
                next_poll = Instant::now() + poll_once(ctx, &mut watcher, &mut status);
            }
        }
    }
}

fn reply_with_status(command: SourceCommand, status: &ListenerStatus) {
    command.respond_ok(status.reply());
}

fn open_watcher(status: &mut ListenerStatus) -> Option<ToastWatcher> {
    match ToastWatcher::request_access() {
        Ok(Some(watcher)) => Some(watcher),
        Ok(None) => {
            *status = ListenerStatus::AccessDenied;
            log::warn!("notification access was not granted");
            None
        }
        Err(error) => {
            *status = ListenerStatus::Failed(error.to_string());
            log::warn!("notification listener unavailable: {error}");
            None
        }
    }
}

fn poll_once(ctx: &SourceContext, watcher: &mut Option<ToastWatcher>, status: &mut ListenerStatus) -> Duration {
    let Some(active) = watcher.as_mut() else {
        *watcher = open_watcher(status);
        if watcher.is_some() {
            *status = ListenerStatus::Listening;
        }
        return FAILURE_BACKOFF;
    };
    match active.poll_new_toasts() {
        Ok(toasts) => {
            *status = ListenerStatus::Listening;
            toasts.iter().for_each(|toast| show_toast(ctx, toast));
            POLL_INTERVAL
        }
        Err(error) => {
            log::warn!("notification poll failed: {error}");
            *status = ListenerStatus::Failed(error.to_string());
            FAILURE_BACKOFF
        }
    }
}

fn show_toast(ctx: &SourceContext, toast: &Toast) {
    let title = if toast.title.is_empty() { "Notification" } else { toast.title.as_str() };
    let mut compact = CompactView::new("🔔", title);
    if !toast.body.is_empty() {
        compact = compact.subtitle(toast.body.clone());
    }
    let activity = ctx
        .activity(&format!("toast-{}", toast.id), ActivityKind::Notification, compact)
        .priority(Priority::High)
        .ttl_ms(TOAST_TTL_MS)
        .expanded(json!({ "title": title, "body": toast.body }));
    ctx.upsert(activity);
}
