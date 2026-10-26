mod alert;
mod board;
mod chronometer;
mod contract;
mod countdown;
mod dispatch;
mod format;
mod pomodoro;
mod stopwatch;
mod view;

use std::time::Duration;

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

use board::TimerBoard;

const TICK: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct TimerSource;

impl ActivitySource for TimerSource {
    fn name(&self) -> &'static str {
        "timer"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        let mut board = TimerBoard::new(ctx.clone());
        while ctx.is_running() {
            match ctx.next_event(TICK) {
                SourceEvent::Shutdown => break,
                SourceEvent::Action(request) => dispatch::handle_action(request, &mut board, &ctx),
                SourceEvent::Command(command) => dispatch::handle_command(command, &mut board),
                SourceEvent::Idle => {}
            }
            board.tick();
        }
    }
}
