mod apps;
pub(crate) mod desktop;
mod layout;
mod matching;
mod runner;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Deserialize;
use serde_json::Value;

use self::apps::InstalledApp;
use crate::core::activity::SourceCommand;
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct LauncherSource;

#[derive(Deserialize)]
struct LaunchRequest {
    id: String,
}

impl ActivitySource for LauncherSource {
    fn name(&self) -> &'static str {
        "launcher"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let busy: Arc<Mutex<()>> = Arc::default();
        let mut installed: Option<Vec<InstalledApp>> = None;
        while ctx.is_running() {
            match ctx.next_event(POLL_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => handle(&ctx, &busy, &mut installed, command),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
        }
    }
}

fn handle(ctx: &SourceContext, busy: &Arc<Mutex<()>>, installed: &mut Option<Vec<InstalledApp>>, command: SourceCommand) {
    let name = command.command.clone();
    match name.as_str() {
        "apps" => {
            let refresh = command.payload.get("refresh").and_then(Value::as_bool).unwrap_or(false);
            if refresh || installed.is_none() {
                match apps::installed() {
                    Ok(list) => *installed = Some(list),
                    Err(error) => return command.respond_err(error),
                }
            }
            command.respond_ok(serde_json::to_value(installed.as_deref().unwrap_or_default()).unwrap_or(Value::Null));
        }
        "launch" => match serde_json::from_value::<LaunchRequest>(command.payload.clone()) {
            Ok(request) => launch_group(ctx, busy, &request.id, command),
            Err(error) => command.respond_err(format!("bad launch request: {error}")),
        },
        other => command.respond_err(format!("unknown launcher command {other}")),
    }
}

fn launch_group(ctx: &SourceContext, busy: &Arc<Mutex<()>>, id: &str, command: SourceCommand) {
    let Some(group) = ctx.current_settings().shortcuts.groups.into_iter().find(|group| group.id == id) else {
        return command.respond_err("that shortcut no longer exists");
    };
    command.respond_ok(Value::Bool(true));
    let ctx = ctx.clone();
    let busy = Arc::clone(busy);
    let spawned = thread::Builder::new().name("launcher-run".into()).spawn(move || {
        let _one_group_at_a_time = busy.lock();
        runner::run(&ctx, &group);
    });
    if let Err(error) = spawned {
        log::error!("launcher thread failed to spawn: {error}");
    }
}
