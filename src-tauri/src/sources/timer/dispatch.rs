use serde::Deserialize;
use serde_json::json;

use crate::core::activity::{ActionRequest, SourceCommand};
use crate::core::source::SourceContext;

use super::board::TimerBoard;
use super::contract::{Control, TimerKind};
use super::view::SOURCE_NAME;

const RESET: &str = "reset";
const DISMISS: &str = "dismiss";
const START: &str = "start";
const STATUS: &str = "status";

#[derive(Deserialize, Default)]
#[serde(default)]
struct TimerRequest {
    timer: Option<TimerKind>,
    seconds: Option<u64>,
}

pub fn handle_action(request: ActionRequest, board: &mut TimerBoard, ctx: &SourceContext) {
    let prefix = format!("{SOURCE_NAME}:");
    let Some(key) = request.activity_id.strip_prefix(&prefix) else {
        return;
    };
    match request.action.as_str() {
        DISMISS => ctx.remove(key),
        RESET => board.reset(key),
        other => {
            if let Some(control) = Control::parse(other) {
                board.control(key, control);
            }
        }
    }
}

pub fn handle_command(command: SourceCommand, board: &mut TimerBoard) {
    let request: TimerRequest = serde_json::from_value(command.payload.clone()).unwrap_or_default();
    match run_command(&command.command, &request, board) {
        Ok(()) => command.respond_ok(json!({ "active": board.active_keys() })),
        Err(message) => command.respond_err(message),
    }
}

fn run_command(name: &str, request: &TimerRequest, board: &mut TimerBoard) -> Result<(), String> {
    if name == STATUS {
        return Ok(());
    }
    let kind = request.timer.ok_or("missing timer kind")?;
    match name {
        START => board.start(kind, request.seconds),
        RESET => {
            board.reset(kind.key());
            Ok(())
        }
        other => {
            let control = Control::parse(other).ok_or("command not supported")?;
            board.control(kind.key(), control);
            Ok(())
        }
    }
}
