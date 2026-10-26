use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::cards::{apply_change, DISMISS_ACTION};
use super::conversation;
use super::hooks_install::{self, HookEndpoint, HooksStatus};
use super::permission::{Decision, PermissionBroker};
use super::server::RunningServer;
use super::session_defaults::{self, SessionDefaults};
use super::service::HookService;
use super::token::token_from_settings_or_new;
use super::tracker::SessionTracker;
use crate::core::activity::{ActionRequest, SourceCommand};
use crate::core::error::{AppError, AppResult};
use crate::core::settings::Settings;
use crate::core::source::SourceContext;

const SESSION_POLL_INTERVAL: Duration = Duration::from_secs(2);
const CONVERSATION_POLL_INTERVAL: Duration = Duration::from_millis(500);

pub struct ClaudeRuntime {
    ctx: SourceContext,
    broker: Arc<PermissionBroker>,
    sessions: Arc<SessionTracker>,
    next_session_poll: Instant,
    next_conversation_poll: Instant,
    conversation_sizes: HashMap<String, u64>,
    endpoint: HookEndpoint,
    server: Option<RunningServer>,
}

impl ClaudeRuntime {
    pub fn boot(ctx: SourceContext) -> Self {
        let endpoint = HookEndpoint {
            port: ctx.current_settings().claude.port,
            token: token_from_settings_or_new(&ctx),
        };
        let mut runtime = Self {
            ctx,
            broker: Arc::default(),
            sessions: Arc::default(),
            next_session_poll: Instant::now(),
            next_conversation_poll: Instant::now(),
            conversation_sizes: HashMap::new(),
            endpoint,
            server: None,
        };
        runtime.start_server();
        runtime
    }

    pub fn handle_action(&self, request: ActionRequest) {
        let Some(key) = request.activity_id.strip_prefix(&format!("{}:", self.ctx.name)) else {
            return;
        };
        if request.action == DISMISS_ACTION {
            self.dismiss_session(key);
        } else if let Some(decision) = Decision::from_action(&request.action) {
            self.broker.resolve(key, decision);
        }
    }

    /// Streams each session's conversation to the card whenever its transcript grows.
    pub fn poll_conversations(&mut self) {
        let now = Instant::now();
        if now < self.next_conversation_poll {
            return;
        }
        self.next_conversation_poll = now + CONVERSATION_POLL_INTERVAL;
        let transcripts = self.sessions.transcripts();
        self.conversation_sizes.retain(|id, _| transcripts.iter().any(|(session_id, _)| session_id == id));
        for (session_id, path) in transcripts {
            let size = fs::metadata(&path).map(|metadata| metadata.len()).unwrap_or(0);
            if self.conversation_sizes.insert(session_id.clone(), size) == Some(size) {
                continue;
            }
            self.ctx.emit(&json!({ "sessionId": session_id, "conversation": conversation::read(&path) }));
        }
    }

    pub fn poll_sessions(&mut self) {
        let now = Instant::now();
        if now < self.next_session_poll {
            return;
        }
        self.next_session_poll = now + SESSION_POLL_INTERVAL;
        for change in self.sessions.poll(now) {
            apply_change(&self.ctx, &change);
        }
    }

    pub fn handle_command(&self, command: SourceCommand) {
        let outcome = match command.command.as_str() {
            "install_hooks" => self.install_hooks(),
            "uninstall_hooks" => self.uninstall_hooks(),
            "hooks_status" => self.hooks_status(),
            "session_defaults" => self.session_defaults(),
            "set_default_permission_mode" => self.set_default_permission_mode(&command.payload),
            "set_default_model" => self.set_default_model(&command.payload),
            "conversation" => self.conversation(&command.payload),
            unknown => Err(AppError::Source(format!("unknown claude command {unknown}"))),
        };
        match outcome {
            Ok(report) => command.respond_ok(report),
            Err(error) => command.respond_err(error.to_string()),
        }
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        let token = if settings.claude.token.is_empty() {
            token_from_settings_or_new(&self.ctx)
        } else {
            settings.claude.token.clone()
        };
        if settings.claude.port == self.endpoint.port && token == self.endpoint.token {
            return;
        }
        self.endpoint = HookEndpoint { port: settings.claude.port, token };
        self.stop_server();
        self.start_server();
    }

    pub fn shutdown(mut self) {
        self.stop_server();
    }

    fn dismiss_session(&self, session_id: &str) {
        if let Some(change) = self.sessions.dismiss(session_id) {
            apply_change(&self.ctx, &change);
        }
    }

    fn conversation(&self, payload: &Value) -> AppResult<Value> {
        let session_id = payload.get("sessionId").and_then(Value::as_str).unwrap_or_default();
        let found = self.sessions.transcript_of(session_id).map(|path| conversation::read(&path)).unwrap_or_default();
        Ok(serde_json::to_value(found)?)
    }

    fn install_hooks(&self) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        self.report(&hooks_install::install(&path, &self.endpoint)?)
    }

    fn uninstall_hooks(&self) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        self.report(&hooks_install::uninstall(&path)?)
    }

    fn hooks_status(&self) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        self.report(&hooks_install::status(&path))
    }

    fn session_defaults(&self) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        defaults_report(&session_defaults::read_defaults(&path))
    }

    fn set_default_permission_mode(&self, payload: &Value) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        defaults_report(&session_defaults::set_default_permission_mode(&path, chosen_value(payload)?)?)
    }

    fn set_default_model(&self, payload: &Value) -> AppResult<Value> {
        let path = hooks_install::claude_settings_path()?;
        defaults_report(&session_defaults::set_default_model(&path, chosen_value(payload)?)?)
    }

    fn report(&self, status: &HooksStatus) -> AppResult<Value> {
        Ok(json!({
            "hooks": serde_json::to_value(status)?,
            "listening": self.server.as_ref().is_some_and(RunningServer::is_listening),
            "port": self.endpoint.port,
        }))
    }

    fn start_server(&mut self) {
        let service = HookService::new(self.ctx.clone(), Arc::clone(&self.broker), Arc::clone(&self.sessions));
        self.server = Some(RunningServer::start(service, self.endpoint.port, &self.endpoint.token));
    }

    fn stop_server(&mut self) {
        if let Some(server) = self.server.take() {
            server.stop();
        }
    }
}

fn chosen_value(payload: &Value) -> AppResult<&str> {
    payload
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Source("command needs a text \"value\"".into()))
}

fn defaults_report(defaults: &SessionDefaults) -> AppResult<Value> {
    Ok(serde_json::to_value(defaults)?)
}
