use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use super::cards::{permission_activity, session_activity};
use super::payload::HookPayload;
use super::permission::{defer_to_terminal_reply, PermissionBroker};
use super::sessions::state_after;
use crate::core::activity::Activity;
use crate::core::source::SourceContext;

const PERMISSION_EVENT: &str = "PermissionRequest";
const PERMISSION_TIMEOUT: Duration = Duration::from_secs(590);

#[derive(Clone)]
pub struct HookService {
    ctx: SourceContext,
    broker: Arc<PermissionBroker>,
}

impl HookService {
    pub fn new(ctx: SourceContext, broker: Arc<PermissionBroker>) -> Self {
        Self { ctx, broker }
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
        let Some(state) = state_after(event, payload) else {
            return;
        };
        let project = payload.project_name();
        let activity = session_activity(&self.ctx, payload.session_id(), project.as_deref(), &state);
        self.ctx.upsert(activity);
    }

    async fn await_permission(&self, payload: &HookPayload) -> Value {
        let mut parked = self.broker.park();
        let card = permission_activity(&self.ctx, &parked.key, payload);
        let _visible = VisibleCard::show(&self.ctx, card, &parked.key);
        match parked.wait_for_decision(PERMISSION_TIMEOUT).await {
            Some(decision) => decision.reply(),
            None => defer_to_terminal_reply(),
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
