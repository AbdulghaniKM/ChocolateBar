use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use super::desktop::{self, WindowId};
use super::matching::match_score;
use crate::core::settings::{ShortcutApp, ShortcutGroup, WindowLayout};
use crate::core::source::SourceContext;

const ARRANGE_DEADLINE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(150);
const LAUNCH_GAP: Duration = Duration::from_millis(60);
const REARRANGE_AFTER: Duration = Duration::from_millis(900);
const SPLASH_GRACE: Duration = Duration::from_secs(4);
const ALREADY_RUNNING_AFTER: Duration = Duration::from_millis(2500);
const UNMATCHED_FALLBACK_AFTER: Duration = Duration::from_millis(1200);
const CONFIDENT_SCORE: u32 = 4;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> {
    group_id: &'a str,
    done: usize,
    total: usize,
    finished: bool,
}

/// Launches every app at once, then snaps each window into its slot as soon as it appears.
pub fn run(ctx: &SourceContext, group: &ShortcutGroup) {
    let before = desktop::snapshot();
    let total = group.apps.len();
    for (index, app) in group.apps.iter().enumerate() {
        if let Err(error) = desktop::launch(&app.app_id) {
            log::warn!("launching {} failed: {error}", app.name);
        }
        report(ctx, group, index + 1, false);
        thread::sleep(LAUNCH_GAP);
    }
    report(ctx, group, total, true);
    let to_arrange: Vec<&ShortcutApp> = group.apps.iter().filter(|app| app.layout != WindowLayout::None).collect();
    if !to_arrange.is_empty() {
        Arranger::new(to_arrange, before).run(ctx);
    }
}

fn report(ctx: &SourceContext, group: &ShortcutGroup, done: usize, finished: bool) {
    ctx.emit(&Progress { group_id: &group.id, done, total: group.apps.len(), finished });
}

struct Slot<'a> {
    app: &'a ShortcutApp,
    window: Option<WindowId>,
    placed_at: Option<Instant>,
    rearranged: bool,
}

struct Arranger<'a> {
    slots: Vec<Slot<'a>>,
    before: HashSet<WindowId>,
    claimed: HashSet<WindowId>,
    unmatched_since: HashMap<WindowId, Instant>,
    started: Instant,
}

impl<'a> Arranger<'a> {
    fn new(apps: Vec<&'a ShortcutApp>, before: HashSet<WindowId>) -> Self {
        let slots = apps.into_iter().map(|app| Slot { app, window: None, placed_at: None, rearranged: false }).collect();
        Self { slots, before, claimed: HashSet::new(), unmatched_since: HashMap::new(), started: Instant::now() }
    }

    fn run(mut self, ctx: &SourceContext) {
        while ctx.is_running() && self.started.elapsed() < ARRANGE_DEADLINE {
            self.reopen_closed_splashes();
            self.rearrange_settled();
            self.claim_new_windows();
            if self.started.elapsed() >= ALREADY_RUNNING_AFTER {
                self.claim_already_running();
            }
            if self.all_settled() {
                break;
            }
            thread::sleep(POLL);
        }
    }

    /// An updater splash (Discord's, say) closes and the real window opens; the slot waits for that one.
    fn reopen_closed_splashes(&mut self) {
        for slot in &mut self.slots {
            if slot.window.is_some_and(|window| !desktop::exists(window)) {
                slot.window = None;
                slot.placed_at = None;
                slot.rearranged = false;
            }
        }
    }

    /// Apps such as Chrome restore their saved bounds right after opening, so each window is snapped twice.
    fn rearrange_settled(&mut self) {
        for slot in &mut self.slots {
            let (Some(window), Some(placed_at)) = (slot.window, slot.placed_at) else { continue };
            if !slot.rearranged && placed_at.elapsed() >= REARRANGE_AFTER {
                desktop::arrange(window, slot.app.layout);
                slot.rearranged = true;
            }
        }
    }

    fn claim_new_windows(&mut self) {
        let fresh: Vec<WindowId> = desktop::app_windows()
            .into_iter()
            .filter(|window| !self.before.contains(window) && !self.claimed.contains(window))
            .collect();
        for window in fresh {
            let (title, process) = (desktop::title_of(window), desktop::process_name_of(window));
            let matched = self.best_open_slot(&title, &process, 1);
            let since = *self.unmatched_since.entry(window).or_insert_with(Instant::now);
            let fallback = || (since.elapsed() >= UNMATCHED_FALLBACK_AFTER).then(|| self.first_open_slot()).flatten();
            if let Some(index) = matched.or_else(fallback) {
                self.place(index, window);
            }
        }
    }

    /// An app that was already open brings its existing window forward instead of creating one.
    fn claim_already_running(&mut self) {
        let existing: Vec<WindowId> = desktop::app_windows()
            .into_iter()
            .filter(|window| self.before.contains(window) && !self.claimed.contains(window))
            .collect();
        for window in existing {
            let (title, process) = (desktop::title_of(window), desktop::process_name_of(window));
            if let Some(index) = self.best_open_slot(&title, &process, CONFIDENT_SCORE) {
                self.place(index, window);
            }
        }
    }

    fn best_open_slot(&self, title: &str, process: &str, minimum: u32) -> Option<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.window.is_none())
            .map(|(index, slot)| (index, match_score(&slot.app.name, &slot.app.app_id, title, process)))
            .filter(|(_, score)| *score >= minimum)
            .max_by_key(|(_, score)| *score)
            .map(|(index, _)| index)
    }

    fn first_open_slot(&self) -> Option<usize> {
        self.slots.iter().position(|slot| slot.window.is_none())
    }

    fn place(&mut self, index: usize, window: WindowId) {
        let slot = &mut self.slots[index];
        desktop::arrange(window, slot.app.layout);
        slot.window = Some(window);
        slot.placed_at = Some(Instant::now());
        slot.rearranged = false;
        self.claimed.insert(window);
    }

    fn all_settled(&self) -> bool {
        self.slots
            .iter()
            .all(|slot| slot.rearranged && slot.placed_at.is_some_and(|placed| placed.elapsed() >= SPLASH_GRACE))
    }
}
