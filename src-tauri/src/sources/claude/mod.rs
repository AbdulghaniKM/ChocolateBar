mod cards;
mod hooks_install;
mod payload;
mod permission;
mod runtime;
mod server;
mod service;
mod sessions;
mod token;
mod tracker;
mod transcript;

use std::time::Duration;

use runtime::ClaudeRuntime;

use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const LOOP_TICK: Duration = Duration::from_millis(250);

#[derive(Default)]
pub struct ClaudeSource;

impl ActivitySource for ClaudeSource {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let mut runtime = ClaudeRuntime::boot(ctx.clone());
        loop {
            match ctx.next_event(LOOP_TICK) {
                SourceEvent::Shutdown => break,
                SourceEvent::Action(request) => runtime.handle_action(request),
                SourceEvent::Command(command) => runtime.handle_command(command),
                SourceEvent::Idle => {}
            }
            runtime.poll_sessions();
            if let Some(settings) = ctx.take_changed_settings() {
                runtime.apply_settings(&settings);
            }
        }
        runtime.shutdown();
    }
}
