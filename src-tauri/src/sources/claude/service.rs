use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::cards::{apply_change, permission_activity};
use super::payload::HookPayload;
use super::permission::{defer_to_terminal_reply, Decision, PermissionBroker};
use super::tracker::SessionTracker;
use crate::core::activity::Activity;
use crate::core::source::SourceContext;

const PERMISSION_EVENT: &str = "PermissionRequest";
const PERMISSION_TIMEOUT: Duration = Duration::from_secs(590);

#[derive(Clone)]
pub struct HookService {
    ctx: SourceContext,
    broker: Arc<PermissionBroker>,
    sessions: Arc<SessionTracker>,
}

impl HookService {
    pub fn new(ctx: SourceContext, broker: Arc<PermissionBroker>, sessions: Arc<SessionTracker>) -> Self {
        Self { ctx, broker, sessions }
    }

    pub async fn handle(&self, event: &str, body: &[u8]) -> Value {
        let payload = HookPayload::parse(body);
        if event == PERMISSION_EVENT {
            return self.await_permission(&payload).await;
        }
        self.track_session(event, &payload);
        json!({})
    }

    fn track_session(&self, event: &str, payload: &HookPayload) {
        if let Some(change) = self.sessions.record_event(event, payload, Instant::now()) {
            apply_change(&self.ctx, &change);
        }
    }

    async fn await_permission(&self, payload: &HookPayload) -> Value {
        let mut parked = self.broker.park();
        let card = permission_activity(&self.ctx, &parked.key, payload);
        let _visible = VisibleCard::show(&self.ctx, card, &parked.key);
        match parked.wait_for_decision(PERMISSION_TIMEOUT).await {
            Some(decision) => {
                self.reflect_switched_mode(payload.session_id(), decision);
                decision.reply()
            }
            None => defer_to_terminal_reply(),
        }
    }

    fn reflect_switched_mode(&self, session_id: &str, decision: Decision) {
        let Some(mode) = decision.switched_mode() else {
            return;
        };
        if let Some(change) = self.sessions.change_permission_mode(session_id, mode.as_str()) {
            apply_change(&self.ctx, &change);
        }
    }
}

struct VisibleCard {
    ctx: SourceContext,
    key: String,
}

impl VisibleCard {
    fn show(ctx: &SourceContext, activity: Activity, key: &str) -> Self {
        ctx.upsert(activity);
        Self { ctx: ctx.clone(), key: key.to_owned() }
    }
}

impl Drop for VisibleCard {
    fn drop(&mut self) {
        self.ctx.remove(&self.key);
    }
}
