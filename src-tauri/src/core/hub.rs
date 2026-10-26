use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, unbounded, Receiver, RecvTimeoutError, Sender};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::activity::{now_ms, ActionRequest, Activity, Priority, SourceCommand};
use super::cards::CardPrefs;
use super::events::{
    stream_event, ACTIVITY_REMOVE, ACTIVITY_UPSERT, ISLAND_FOCUS, ISLAND_VISIBILITY,
};
use super::settings::{Settings, SettingsStore};
use crate::window;

const TICK: Duration = Duration::from_millis(100);
const LIST_TIMEOUT: Duration = Duration::from_secs(2);

struct SourceRoute {
    actions: Sender<ActionRequest>,
    commands: Sender<SourceCommand>,
}

enum HubMessage {
    Upsert(Activity),
    Remove(String),
    Stream { source: String, payload: Value },
    RegisterSource { name: &'static str, route: SourceRoute },
    Action(ActionRequest),
    Command(SourceCommand),
    Cycle(i32),
    Focus(String),
    SetCardPrefs(CardPrefs),
    SetExpanded(bool),
    SetFullscreen(bool),
    SetPaused(bool),
    List(Sender<Vec<Activity>>),
}

#[derive(Clone)]
pub struct HubHandle {
    sender: Sender<HubMessage>,
}

impl HubHandle {
    fn post(&self, message: HubMessage) {
        if self.sender.send(message).is_err() {
            log::warn!("hub is gone, message dropped");
        }
    }

    pub fn upsert(&self, activity: Activity) {
        self.post(HubMessage::Upsert(activity));
    }

    pub fn remove(&self, id: impl Into<String>) {
        self.post(HubMessage::Remove(id.into()));
    }

    pub fn stream(&self, source: &str, payload: Value) {
        self.post(HubMessage::Stream { source: source.to_owned(), payload });
    }

    pub fn register_source(
        &self,
        name: &'static str,
    ) -> (Receiver<ActionRequest>, Receiver<SourceCommand>) {
        let (actions, action_receiver) = unbounded();
        let (commands, command_receiver) = unbounded();
        self.post(HubMessage::RegisterSource { name, route: SourceRoute { actions, commands } });
        (action_receiver, command_receiver)
    }

    pub fn dispatch_action(&self, request: ActionRequest) {
        self.post(HubMessage::Action(request));
    }

    pub fn dispatch_command(&self, command: SourceCommand) {
        self.post(HubMessage::Command(command));
    }

    pub fn cycle(&self, direction: i32) {
        self.post(HubMessage::Cycle(direction));
    }

    pub fn set_card_prefs(&self, prefs: CardPrefs) {
        self.post(HubMessage::SetCardPrefs(prefs));
    }

    pub fn focus(&self, id: String) {
        self.post(HubMessage::Focus(id));
    }

    pub fn set_expanded(&self, expanded: bool) {
        self.post(HubMessage::SetExpanded(expanded));
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.post(HubMessage::SetFullscreen(fullscreen));
    }

    pub fn set_paused(&self, paused: bool) {
        self.post(HubMessage::SetPaused(paused));
    }

    pub fn list(&self) -> Vec<Activity> {
        let (reply, receiver) = bounded(1);
        self.post(HubMessage::List(reply));
        receiver.recv_timeout(LIST_TIMEOUT).unwrap_or_default()
    }
}

pub fn spawn(app: AppHandle) -> HubHandle {
    let (sender, receiver) = unbounded();
    let mut hub = Hub::new(app.clone());
    let settings = app.state::<SettingsStore>();
    hub.prefs = settings.current().general.card_prefs();
    follow_card_settings(HubHandle { sender: sender.clone() }, settings.subscribe());
    let spawned = thread::Builder::new().name("hub".into()).spawn(move || hub.run(receiver));
    if let Err(error) = spawned {
        log::error!("hub failed to spawn: {error}");
    }
    HubHandle { sender }
}

fn follow_card_settings(hub: HubHandle, mut settings: tokio::sync::watch::Receiver<Settings>) {
    tauri::async_runtime::spawn(async move {
        while settings.changed().await.is_ok() {
            let prefs = settings.borrow_and_update().general.card_prefs();
            hub.set_card_prefs(prefs);
        }
    });
}

struct Hub {
    app: AppHandle,
    prefs: CardPrefs,
    clock_pinned: bool,
    activities: HashMap<String, Activity>,
    routes: HashMap<&'static str, SourceRoute>,
    focused: Option<String>,
    pinned: bool,
    fullscreen: bool,
    paused: bool,
    visible: bool,
}

impl Hub {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            prefs: CardPrefs::default(),
            clock_pinned: false,
            activities: HashMap::new(),
            routes: HashMap::new(),
            focused: None,
            pinned: false,
            fullscreen: false,
            paused: false,
            visible: true,
        }
    }

    fn run(mut self, receiver: Receiver<HubMessage>) {
        loop {
            match receiver.recv_timeout(TICK) {
                Ok(message) => self.handle(message),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            self.expire_stale();
        }
    }

    fn handle(&mut self, message: HubMessage) {
        match message {
            HubMessage::Upsert(activity) => self.upsert(activity),
            HubMessage::Remove(id) => self.remove(&id),
            HubMessage::Stream { source, payload } => self.emit(&stream_event(&source), &payload),
            HubMessage::RegisterSource { name, route } => {
                self.routes.insert(name, route);
            }
            HubMessage::Action(request) => self.route_action(request),
            HubMessage::Command(command) => self.route_command(command),
            HubMessage::Cycle(direction) => self.cycle(direction),
            HubMessage::Focus(id) => self.focus(&id),
            HubMessage::SetCardPrefs(prefs) => {
                self.prefs = prefs;
                self.refocus();
            }
            HubMessage::SetExpanded(expanded) => self.set_expanded(expanded),
            HubMessage::SetFullscreen(fullscreen) => self.set_fullscreen(fullscreen),
            HubMessage::SetPaused(paused) => self.set_paused(paused),
            HubMessage::List(reply) => {
                let _ = reply.send(self.ranked().into_iter().cloned().collect());
            }
        }
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: &S) {
        if let Err(error) = self.app.emit(event, payload) {
            log::warn!("emit {event} failed: {error}");
        }
    }

    fn upsert(&mut self, mut activity: Activity) {
        activity.updated_at_ms = now_ms();
        let urgent = activity.priority >= Priority::High;
        if self.outranks_focus(&activity) && (urgent || !self.clock_pinned) {
            self.pinned = false;
            self.clock_pinned = false;
        }
        self.emit(ACTIVITY_UPSERT, &activity);
        self.activities.insert(activity.id.clone(), activity);
        self.refocus();
    }

    fn remove(&mut self, id: &str) {
        if self.activities.remove(id).is_some() {
            self.emit(ACTIVITY_REMOVE, &id);
            self.refocus();
        }
    }

    fn expire_stale(&mut self) {
        let now = now_ms();
        let expired: Vec<String> = self
            .activities
            .values()
            .filter(|activity| activity.is_expired_at(now))
            .map(|activity| activity.id.clone())
            .collect();
        for id in expired {
            self.remove(&id);
        }
    }

    fn route_action(&self, request: ActionRequest) {
        let source = Activity::source_of(&request.activity_id);
        match self.routes.get(source) {
            Some(route) => {
                let _ = route.actions.send(request);
            }
            None => log::warn!("no source for action on {}", request.activity_id),
        }
    }

    fn route_command(&self, command: SourceCommand) {
        match self.routes.get(command.source.as_str()) {
            Some(route) => {
                if let Err(rejected) = route.commands.send(command) {
                    rejected.into_inner().respond_err("source is not running");
                }
            }
            None => command.respond_err("unknown source"),
        }
    }

    fn rank(prefs: &CardPrefs, activity: &Activity) -> (Priority, bool, usize, u64) {
        (activity.priority, activity.alert, prefs.order_score(activity.kind), activity.updated_at_ms)
    }

    fn ranked(&self) -> Vec<&Activity> {
        let mut ordered: Vec<&Activity> = self.activities.values().collect();
        ordered.sort_by(|a, b| {
            Self::rank(&self.prefs, b).cmp(&Self::rank(&self.prefs, a)).then_with(|| a.id.cmp(&b.id))
        });
        ordered
    }

    fn focusable_ids(&self) -> Vec<String> {
        self.ranked()
            .into_iter()
            .filter(|activity| !self.fullscreen || activity.priority == Priority::Critical)
            .map(|activity| activity.id.clone())
            .collect()
    }

    fn outranks_focus(&self, activity: &Activity) -> bool {
        let Some(focused) = self.focused.as_ref().and_then(|id| self.activities.get(id)) else {
            return true;
        };
        activity.id != focused.id
            && (activity.priority, activity.alert) > (focused.priority, focused.alert)
    }

    fn choose_focus(&mut self) -> Option<String> {
        if self.clock_pinned {
            return None;
        }
        let candidates = self.focusable_ids();
        let kept = self.focused.clone().filter(|id| self.pinned && candidates.contains(id));
        if kept.is_none() {
            self.pinned = false;
        }
        kept.or_else(|| self.idle_choice(&candidates))
    }

    /// Live activities always win; when only background cards remain, the user's default card is shown.
    fn idle_choice(&self, candidates: &[String]) -> Option<String> {
        let top = candidates.first()?;
        let live = self.activities.get(top).is_some_and(|activity| activity.priority > Priority::Ambient);
        if live {
            return Some(top.clone());
        }
        if self.prefs.shows_clock_when_idle() {
            return None;
        }
        candidates
            .iter()
            .find(|id| self.activities.get(*id).is_some_and(|activity| self.prefs.is_default(activity.kind)))
            .or(Some(top))
            .cloned()
    }

    /// Flipping goes through the clock (when it is the default card) and then the cards in the user's order.
    fn cycle_ring(&self) -> Vec<Option<String>> {
        let mut cards: Vec<&Activity> = self.focusable_ids().iter().filter_map(|id| self.activities.get(id)).collect();
        cards.sort_by_key(|activity| std::cmp::Reverse(self.prefs.order_score(activity.kind)));
        let clock = self.prefs.shows_clock_when_idle().then_some(None);
        clock.into_iter().chain(cards.into_iter().map(|activity| Some(activity.id.clone()))).collect()
    }

    fn refocus(&mut self) {
        let next = self.choose_focus();
        self.announce_focus(next);
        self.refresh_visibility();
    }

    fn announce_focus(&mut self, next: Option<String>) {
        if next != self.focused {
            self.emit(ISLAND_FOCUS, &next);
            self.focused = next;
        }
    }

    fn cycle(&mut self, direction: i32) {
        let ring = self.cycle_ring();
        if ring.len() < 2 {
            return;
        }
        let current = ring.iter().position(|entry| *entry == self.focused).unwrap_or(0);
        let step = (current as i32 + direction.signum()).rem_euclid(ring.len() as i32) as usize;
        let next = ring[step].clone();
        self.clock_pinned = next.is_none();
        self.pinned = next.is_some();
        self.announce_focus(next);
    }

    /// Something the user just started (a timer, say) takes over the island even while it is expanded and pinned.
    fn focus(&mut self, id: &str) {
        if !self.focusable_ids().iter().any(|candidate| candidate == id) {
            return;
        }
        self.pinned = true;
        self.announce_focus(Some(id.to_string()));
    }

    fn set_expanded(&mut self, expanded: bool) {
        self.pinned = expanded && self.focused.is_some();
        self.clock_pinned = expanded && self.clock_pinned;
        self.refocus();
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
        self.refocus();
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        self.refresh_visibility();
    }

    fn refresh_visibility(&mut self) {
        let visible = !self.paused && (!self.fullscreen || self.focused.is_some());
        if visible != self.visible {
            self.visible = visible;
            self.emit(ISLAND_VISIBILITY, &visible);
            window::apply_visibility(&self.app, visible);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Reverse;

    use super::*;
    use crate::core::activity::{ActivityKind, CompactView};

    fn activity(key: &str, priority: Priority, alert: bool, updated_at_ms: u64) -> Activity {
        let mut built = Activity::new("test", key, ActivityKind::Media, CompactView::new("x", key))
            .priority(priority)
            .alert(alert);
        built.updated_at_ms = updated_at_ms;
        built
    }

    #[test]
    fn rank_orders_by_priority_then_alert_then_recency() {
        let ambient = activity("a", Priority::Ambient, true, 9);
        let normal_old = activity("b", Priority::Normal, false, 1);
        let normal_alert = activity("c", Priority::Normal, true, 0);
        let normal_new = activity("d", Priority::Normal, false, 5);
        let mut ranked = [&ambient, &normal_old, &normal_alert, &normal_new];
        ranked.sort_by_key(|item| Reverse(Hub::rank(&CardPrefs::default(), item)));
        let order: Vec<&str> = ranked.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(order, ["test:c", "test:d", "test:b", "test:a"]);
    }

    #[test]
    fn expiry_respects_ttl() {
        let timed = activity("t", Priority::High, false, 100).ttl_ms(50);
        assert!(!timed.is_expired_at(149));
        assert!(timed.is_expired_at(150));
        assert!(!activity("u", Priority::High, false, 0).is_expired_at(u64::MAX));
    }

    #[test]
    fn source_prefix_is_text_before_first_colon() {
        assert_eq!(Activity::source_of("game:download:42"), "game");
    }
}
