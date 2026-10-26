use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use super::desktop::{self, WindowId};
use crate::core::settings::{ShortcutApp, ShortcutGroup, WindowLayout};
use crate::core::source::SourceContext;

const WINDOW_TIMEOUT: Duration = Duration::from_secs(15);
const FOREGROUND_FALLBACK_AFTER: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(200);
const SPLASH_WATCH: Duration = Duration::from_secs(8);
const SETTLE: Duration = Duration::from_millis(900);
const UNARRANGED_GAP: Duration = Duration::from_millis(300);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> {
    group_id: &'a str,
    done: usize,
    total: usize,
    finished: bool,
}

/// Opens every app in the group in order; apps with a layout are waited on so each window lands in its slot.
pub fn run(ctx: &SourceContext, group: &ShortcutGroup) {
    let total = group.apps.len();
    report(ctx, group, 0, false);
    for (index, app) in group.apps.iter().enumerate() {
        if !ctx.is_running() {
            return;
        }
        open(app);
        report(ctx, group, index + 1, index + 1 == total);
    }
    if total == 0 {
        report(ctx, group, 0, true);
    }
}

fn report(ctx: &SourceContext, group: &ShortcutGroup, done: usize, finished: bool) {
    ctx.emit(&Progress { group_id: &group.id, done, total: group.apps.len(), finished });
}

fn open(app: &ShortcutApp) {
    let before = desktop::snapshot();
    let foreground_before = desktop::foreground();
    if let Err(error) = desktop::launch(&app.app_id) {
        log::warn!("launching {} failed: {error}", app.name);
        return;
    }
    if app.layout == WindowLayout::None {
        thread::sleep(UNARRANGED_GAP);
        return;
    }
    let Some(window) = wait_for_window(&before, foreground_before) else {
        log::info!("{} opened no window to arrange", app.name);
        return;
    };
    place(window, app.layout);
    follow_splash_screens(window, before, app.layout);
}

fn wait_for_window(before: &HashSet<WindowId>, foreground_before: WindowId) -> Option<WindowId> {
    let started = Instant::now();
    while started.elapsed() < WINDOW_TIMEOUT {
        if let Some(window) = desktop::app_windows().into_iter().find(|window| !before.contains(window)) {
            return Some(window);
        }
        let foreground = desktop::foreground();
        let already_running_app_came_forward = started.elapsed() > FOREGROUND_FALLBACK_AFTER
            && foreground != foreground_before
            && desktop::is_app_window(foreground);
        if already_running_app_came_forward {
            return Some(foreground);
        }
        thread::sleep(POLL);
    }
    None
}

fn place(window: WindowId, layout: WindowLayout) {
    desktop::arrange(window, layout);
    thread::sleep(SETTLE);
    if desktop::exists(window) {
        desktop::arrange(window, layout);
    }
}

/// Launchers like Discord show an updater splash first; if the placed window closes, the next new one is arranged instead.
fn follow_splash_screens(mut window: WindowId, mut known: HashSet<WindowId>, layout: WindowLayout) {
    let started = Instant::now();
    while started.elapsed() < SPLASH_WATCH {
        thread::sleep(POLL);
        if desktop::exists(window) {
            continue;
        }
        known.insert(window);
        match desktop::app_windows().into_iter().find(|candidate| !known.contains(candidate)) {
            Some(next) => {
                window = next;
                place(window, layout);
            }
            None => continue,
        }
    }
}
