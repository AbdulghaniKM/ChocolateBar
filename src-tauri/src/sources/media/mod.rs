mod artwork;
mod lrc;
mod lyrics;
pub mod playback_flag;
mod publisher;
mod selection;
mod session;
mod view;

use std::time::Duration;

use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager;

use self::publisher::Publisher;
use self::selection::Selection;
use self::session::{Control, Reading};
use crate::core::activity::ActionRequest;
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const MANAGER_RETRY_DELAY: Duration = Duration::from_secs(5);
const ACTION_SETTLE_DELAY: Duration = Duration::from_millis(150);
const SELECT_SOURCE_ACTION: &str = "source";

#[derive(Default)]
pub struct MediaSource;

impl ActivitySource for MediaSource {
    fn name(&self) -> &'static str {
        "media"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let Some(manager) = open_manager_with_retry(&ctx) else { return };
        let mut publisher = Publisher::with_settings(&ctx.current_settings());
        let mut selection = Selection::default();
        while ctx.is_running() {
            match ctx.next_event(POLL_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Action(request) => {
                    handle_action(&manager, &mut selection, &request);
                    ctx.pause(ACTION_SETTLE_DELAY);
                }
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Idle => {}
            }
            if let Some(settings) = ctx.take_changed_settings() {
                publisher.apply_settings(&settings);
            }
            publisher.refresh(&ctx, read_selected(&manager, &mut selection));
        }
        publisher.withdraw(&ctx);
    }
}

fn open_manager_with_retry(ctx: &SourceContext) -> Option<Manager> {
    while ctx.is_running() {
        match session::open_manager() {
            Ok(manager) => return Some(manager),
            Err(error) => {
                log::warn!("media session manager unavailable: {error}");
                ctx.pause(MANAGER_RETRY_DELAY);
            }
        }
    }
    None
}

fn read_selected(manager: &Manager, selection: &mut Selection) -> Option<Reading> {
    let sessions = session::sessions(manager);
    let summaries: Vec<_> = sessions.iter().map(|(summary, _)| summary.clone()).collect();
    let chosen = selection.choose(&summaries, session::current_session_id(manager).as_deref())?;
    let (summary, chosen_session) = sessions.into_iter().find(|(summary, _)| summary.id == chosen)?;
    match session::read(&chosen_session, summary, summaries) {
        Ok(reading) if !reading.track.is_blank() => Some(reading),
        Ok(_) => None,
        Err(error) => {
            log::debug!("media session read failed: {error}");
            None
        }
    }
}

fn handle_action(manager: &Manager, selection: &mut Selection, request: &ActionRequest) {
    if request.action == SELECT_SOURCE_ACTION {
        if let Some(id) = request.value.as_ref().and_then(|value| value.as_str()) {
            selection.pick(id);
        }
        return;
    }
    let Some(control) = control_for(request) else {
        log::debug!("media ignored action {}", request.action);
        return;
    };
    let Some(id) = selection.active() else { return };
    let Some((_, target)) = session::sessions(manager).into_iter().find(|(summary, _)| summary.id == id) else {
        return;
    };
    if let Err(error) = session::send(&target, control) {
        log::warn!("media action {} failed: {error}", request.action);
    }
}

fn control_for(request: &ActionRequest) -> Option<Control> {
    match request.action.as_str() {
        "prev" => Some(Control::Previous),
        "toggle" => Some(Control::TogglePlayPause),
        "next" => Some(Control::Next),
        "seek" => {
            let position_ms = request.value.as_ref()?.as_f64()?;
            Some(Control::SeekTo(position_ms.max(0.0) as u64))
        }
        "shuffle" => Some(Control::SetShuffle(request.value.as_ref()?.as_bool()?)),
        "repeat" => Some(Control::SetRepeat(session::parse_repeat(request.value.as_ref()?.as_str()?)?)),
        _ => None,
    }
}
