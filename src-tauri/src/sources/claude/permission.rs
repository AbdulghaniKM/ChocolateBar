use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::oneshot;

const SESSION_DESTINATION: &str = "session";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchableMode {
    AcceptEdits,
    Auto,
}

impl SwitchableMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AcceptEdits => "acceptEdits",
            Self::Auto => "auto",
        }
    }

    pub fn allow_action_id(self) -> &'static str {
        match self {
            Self::AcceptEdits => "allow_accept_edits",
            Self::Auto => "allow_auto",
        }
    }

    pub fn allow_label(self) -> &'static str {
        match self {
            Self::AcceptEdits => "Allow + accept edits",
            Self::Auto => "Allow + auto mode",
        }
    }
}

pub const SWITCHABLE_MODES: [SwitchableMode; 2] = [SwitchableMode::AcceptEdits, SwitchableMode::Auto];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    AllowAndSwitchTo(SwitchableMode),
    Deny,
}

impl Decision {
    pub fn from_action(action: &str) -> Option<Self> {
        match action {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            other => SWITCHABLE_MODES
                .into_iter()
                .find(|mode| mode.allow_action_id() == other)
                .map(Self::AllowAndSwitchTo),
        }
    }

    pub fn switched_mode(self) -> Option<SwitchableMode> {
        match self {
            Self::AllowAndSwitchTo(mode) => Some(mode),
            Self::Allow | Self::Deny => None,
        }
    }

    pub fn reply(self) -> Value {
        let decision = match self {
            Self::Allow => json!({ "behavior": "allow" }),
            Self::AllowAndSwitchTo(mode) => json!({
                "behavior": "allow",
                "updatedPermissions": [{ "type": "setMode", "mode": mode.as_str(), "destination": SESSION_DESTINATION }],
            }),
            Self::Deny => json!({ "behavior": "deny", "message": "Denied from ChocolateBar" }),
        };
        json!({ "hookSpecificOutput": { "hookEventName": "PermissionRequest", "decision": decision } })
    }
}

pub fn defer_to_terminal_reply() -> Value {
    json!({})
}

type PendingDecisions = HashMap<String, oneshot::Sender<Decision>>;

#[derive(Default)]
pub struct PermissionBroker {
    next_id: AtomicU64,
    pending: Mutex<PendingDecisions>,
}

impl PermissionBroker {
    pub fn park(self: &Arc<Self>) -> ParkedRequest {
        let key = format!("perm-{}", self.next_id.fetch_add(1, Ordering::Relaxed));
        let (sender, receiver) = oneshot::channel();
        self.lock_pending().insert(key.clone(), sender);
        ParkedRequest { key, receiver, broker: Arc::clone(self) }
    }

    pub fn resolve(&self, key: &str, decision: Decision) -> bool {
        let sender = self.lock_pending().remove(key);
        sender.is_some_and(|sender| sender.send(decision).is_ok())
    }

    fn forget(&self, key: &str) {
        self.lock_pending().remove(key);
    }

    fn lock_pending(&self) -> MutexGuard<'_, PendingDecisions> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct ParkedRequest {
    pub key: String,
    receiver: oneshot::Receiver<Decision>,
    broker: Arc<PermissionBroker>,
}

impl ParkedRequest {
    pub async fn wait_for_decision(&mut self, limit: Duration) -> Option<Decision> {
        tokio::time::timeout(limit, &mut self.receiver).await.ok()?.ok()
    }
}

impl Drop for ParkedRequest {
    fn drop(&mut self) {
        self.broker.forget(&self.key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_reply_matches_the_documented_schema() {
        let reply = Decision::Allow.reply();
        assert_eq!(reply["hookSpecificOutput"]["hookEventName"], "PermissionRequest");
        assert_eq!(reply["hookSpecificOutput"]["decision"]["behavior"], "allow");
    }

    #[test]
    fn deny_reply_carries_a_reason() {
        let reply = Decision::Deny.reply();
        assert_eq!(reply["hookSpecificOutput"]["decision"]["behavior"], "deny");
        assert!(reply["hookSpecificOutput"]["decision"]["message"].is_string());
    }

    #[test]
    fn accept_edits_reply_switches_the_session_mode() {
        let reply = Decision::AllowAndSwitchTo(SwitchableMode::AcceptEdits).reply();
        let decision = &reply["hookSpecificOutput"]["decision"];
        assert_eq!(decision["behavior"], "allow");
        assert_eq!(
            decision["updatedPermissions"],
            json!([{ "type": "setMode", "mode": "acceptEdits", "destination": "session" }])
        );
    }

    #[test]
    fn auto_reply_switches_the_session_mode() {
        let reply = Decision::AllowAndSwitchTo(SwitchableMode::Auto).reply();
        assert_eq!(reply["hookSpecificOutput"]["decision"]["updatedPermissions"][0]["mode"], "auto");
    }

    #[test]
    fn plain_allow_and_deny_carry_no_permission_update() {
        assert!(Decision::Allow.reply()["hookSpecificOutput"]["decision"].get("updatedPermissions").is_none());
        assert!(Decision::Deny.reply()["hookSpecificOutput"]["decision"].get("updatedPermissions").is_none());
    }

    #[test]
    fn every_action_id_maps_to_its_decision() {
        assert_eq!(Decision::from_action("allow"), Some(Decision::Allow));
        assert_eq!(Decision::from_action("deny"), Some(Decision::Deny));
        for mode in SWITCHABLE_MODES {
            let decision = Decision::from_action(mode.allow_action_id());
            assert_eq!(decision.and_then(Decision::switched_mode), Some(mode));
        }
        assert_eq!(Decision::from_action("dismiss"), None);
    }

    #[test]
    fn resolving_unknown_key_reports_false() {
        assert!(!PermissionBroker::default().resolve("perm-404", Decision::Allow));
    }

    #[tokio::test]
    async fn parked_request_receives_the_decision() {
        let broker = Arc::new(PermissionBroker::default());
        let mut parked = broker.park();
        assert!(broker.resolve(&parked.key, Decision::Deny));
        let decision = parked.wait_for_decision(Duration::from_secs(1)).await;
        assert_eq!(decision, Some(Decision::Deny));
    }

    #[tokio::test]
    async fn unresolved_request_times_out_with_no_decision() {
        let broker = Arc::new(PermissionBroker::default());
        let mut parked = broker.park();
        assert_eq!(parked.wait_for_decision(Duration::from_millis(20)).await, None);
    }

    #[test]
    fn dropping_a_parked_request_forgets_it() {
        let broker = Arc::new(PermissionBroker::default());
        let parked = broker.park();
        let key = parked.key.clone();
        drop(parked);
        assert!(!broker.resolve(&key, Decision::Allow));
    }
}
