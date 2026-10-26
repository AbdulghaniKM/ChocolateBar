mod artwork;
mod lrc;
mod lyrics;
pub mod playback_flag;
mod publisher;
mod session;
mod view;

use std::time::Duration;

use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Manager;

use self::publisher::Publisher;
use self::session::{Control, Reading};
use crate::core::activity::ActionRequest;
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const MANAGER_RETRY_DELAY: Duration = Duration::from_secs(5);
const ACTION_SETTLE_DELAY: Duration = Duration::from_millis(150);

#[derive(Default)]
pub struct MediaSource;

impl ActivitySource for MediaSource {
    fn name(&self) -> &'static str {
        "media"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let Some(manager) = open_manager_with_retry(&ctx) else { return };
        let mut publisher = Publisher::with_settings(&ctx.current_settings());
        while ctx.is_running() {
            match ctx.next_event(POLL_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Action(request) => {
                    handle_action(&manager, &request);
                    ctx.pause(ACTION_SETTLE_DELAY);
                }
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Idle => {}
            }
            if let Some(settings) = ctx.take_changed_settings() {
                publisher.apply_settings(&settings);
            }
            publisher.refresh(&ctx, read_current(&manager));
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

fn read_current(manager: &Manager) -> Option<Reading> {
    let session = session::current_session(manager)?;
    match session::read(&session) {
        Ok(reading) if !reading.track.is_blank() => Some(reading),
        Ok(_) => None,
        Err(error) => {
            log::debug!("media session read failed: {error}");
            None
        }
    }
}

fn handle_action(manager: &Manager, request: &ActionRequest) {
    let Some(control) = control_for(request) else {
        log::debug!("media ignored action {}", request.action);
        return;
    };
    let Some(session) = session::current_session(manager) else { return };
    if let Err(error) = session::send(&session, control) {
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
        _ => None,
    }
}
