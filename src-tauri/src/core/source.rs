use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{select, Receiver};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio::sync::watch;

use super::activity::{ActionRequest, Activity, ActivityKind, CompactView, SourceCommand};
use super::com::ComGuard;
use super::hub::HubHandle;
use super::settings::{Settings, SettingsStore};

const RESTART_DELAY: Duration = Duration::from_secs(5);
const SHUTDOWN_POLL: Duration = Duration::from_millis(100);
const IDLE_POLL: Duration = Duration::from_millis(500);

pub trait ActivitySource: Send {
    fn name(&self) -> &'static str;
    fn start(self: Box<Self>, ctx: SourceContext);
}

pub type SourceFactory = fn() -> Box<dyn ActivitySource>;

#[derive(Clone, Default)]
pub struct Shutdown(Arc<AtomicBool>);

impl Shutdown {
    pub fn trigger(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.0)
    }
}

pub enum SourceEvent {
    Action(ActionRequest),
    Command(SourceCommand),
    Idle,
    Shutdown,
}

#[derive(Clone)]
pub struct SourceContext {
    pub name: &'static str,
    pub app: AppHandle,
    pub hub: HubHandle,
    pub actions: Receiver<ActionRequest>,
    pub commands: Receiver<SourceCommand>,
    pub settings: watch::Receiver<Settings>,
    pub shutdown: Arc<AtomicBool>,
}

impl SourceContext {
    pub fn for_source(app: &AppHandle, name: &'static str) -> Self {
        let hub = app.state::<HubHandle>().inner().clone();
        let (actions, commands) = hub.register_source(name);
        Self {
            name,
            app: app.clone(),
            hub,
            actions,
            commands,
            settings: app.state::<SettingsStore>().subscribe(),
            shutdown: app.state::<Shutdown>().flag(),
        }
    }

    pub fn is_running(&self) -> bool {
        !self.shutdown.load(Ordering::Relaxed)
    }

    pub fn activity(&self, key: &str, kind: ActivityKind, compact: CompactView) -> Activity {
        Activity::new(self.name, key, kind, compact)
    }

    pub fn upsert(&self, activity: Activity) {
        self.hub.upsert(activity);
    }

    pub fn remove(&self, key: &str) {
        self.hub.remove(Activity::id_for(self.name, key));
    }

    pub fn emit(&self, payload: &impl Serialize) {
        match serde_json::to_value(payload) {
            Ok(value) => self.hub.stream(self.name, value),
            Err(error) => log::warn!("source {} stream payload rejected: {error}", self.name),
        }
    }

    pub fn current_settings(&self) -> Settings {
        self.settings.borrow().clone()
    }

    pub fn take_changed_settings(&mut self) -> Option<Settings> {
        let changed = self.settings.has_changed().unwrap_or(false);
        changed.then(|| self.settings.borrow_and_update().clone())
    }

    pub fn recv_action(&self, timeout: Duration) -> Option<ActionRequest> {
        self.actions.recv_timeout(timeout).ok()
    }

    pub fn recv_command(&self, timeout: Duration) -> Option<SourceCommand> {
        self.commands.recv_timeout(timeout).ok()
    }

    pub fn next_event(&self, timeout: Duration) -> SourceEvent {
        if !self.is_running() {
            return SourceEvent::Shutdown;
        }
        select! {
            recv(self.actions) -> request => request.map_or(SourceEvent::Shutdown, SourceEvent::Action),
            recv(self.commands) -> command => command.map_or(SourceEvent::Shutdown, SourceEvent::Command),
            default(timeout) => SourceEvent::Idle,
        }
    }

    pub fn pause(&self, duration: Duration) -> bool {
        let deadline = Instant::now() + duration;
        while self.is_running() && Instant::now() < deadline {
            thread::sleep(SHUTDOWN_POLL.min(deadline.saturating_duration_since(Instant::now())));
        }
        self.is_running()
    }

    pub fn idle_until_shutdown(&self) {
        loop {
            match self.next_event(IDLE_POLL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
        }
    }
}

pub fn spawn_source(build: SourceFactory, ctx: SourceContext) {
    let name = ctx.name;
    let spawned = thread::Builder::new()
        .name(format!("source-{name}"))
        .spawn(move || run_with_one_restart(build, ctx));
    if let Err(error) = spawned {
        log::error!("source {name} failed to spawn: {error}");
    }
}

fn run_with_one_restart(build: SourceFactory, ctx: SourceContext) {
    if run_guarded(build, ctx.clone()) {
        return;
    }
    log::error!("source {} panicked, restarting in {RESTART_DELAY:?}", ctx.name);
    if ctx.pause(RESTART_DELAY) {
        run_guarded(build, ctx);
    }
}

fn run_guarded(build: SourceFactory, ctx: SourceContext) -> bool {
    let _com = ComGuard::initialize_multithreaded();
    let source = build();
    catch_unwind(AssertUnwindSafe(move || source.start(ctx))).is_ok()
}
