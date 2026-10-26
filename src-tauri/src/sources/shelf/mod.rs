mod cards;
mod drag_icon;
mod drop_events;
mod files;
mod session;

use std::time::Duration;

use crossbeam_channel::{select, unbounded};

use crate::core::source::{ActivitySource, SourceContext};
use drop_events::attach_to_main_window;
use session::ShelfSession;

const LOOP_TICK: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct ShelfSource;

impl ActivitySource for ShelfSource {
    fn name(&self) -> &'static str {
        "shelf"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let (signals, drop_signals) = unbounded();
        let mut session = ShelfSession::new(ctx.clone());
        let mut attached = false;
        while ctx.is_running() {
            if !attached {
                attached = attach_to_main_window(&ctx.app, signals.clone());
            }
            select! {
                recv(ctx.actions) -> request => match request {
                    Ok(request) => session.handle_action(request),
                    Err(_) => break,
                },
                recv(ctx.commands) -> command => match command {
                    Ok(command) => session.handle_command(command),
                    Err(_) => break,
                },
                recv(drop_signals) -> signal => if let Ok(signal) = signal {
                    session.on_drop_signal(signal);
                },
                default(LOOP_TICK) => {}
            }
            session.demote_expired_promotion();
        }
    }
}
