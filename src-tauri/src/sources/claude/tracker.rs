use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::payload::HookPayload;
use super::sessions::{transition, SessionState, Transition};
use super::transcript::{TranscriptChange, TranscriptWatcher};

const SESSION_RETENTION: Duration = Duration::from_secs(12 * 60 * 60);
const NEW_TURN_EVENT: &str = "UserPromptSubmit";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SessionTraits {
    pub permission_mode: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardChange {
    Show { session_id: String, project: Option<String>, state: SessionState, traits: SessionTraits },
    Remove { session_id: String },
}

fn adopt(slot: &mut Option<String>, incoming: Option<&str>) -> bool {
    let Some(incoming) = incoming.filter(|incoming| slot.as_deref() != Some(*incoming)) else {
        return false;
    };
    *slot = Some(incoming.to_owned());
    true
}

struct TrackedSession {
    state: SessionState,
    project: Option<String>,
    permission_mode: Option<String>,
    model: Option<String>,
    watcher: Option<TranscriptWatcher>,
    last_signal: Instant,
    dismissed: bool,
}

impl TrackedSession {
    fn new(now: Instant) -> Self {
        Self {
            state: SessionState::Started,
            project: None,
            permission_mode: None,
            model: None,
            watcher: None,
            last_signal: now,
            dismissed: false,
        }
    }

    fn absorb(&mut self, event: &str, payload: &HookPayload, state: SessionState, now: Instant) {
        self.state = state;
        self.last_signal = now;
        self.dismissed = false;
        if let Some(project) = payload.project_name() {
            self.project = Some(project);
        }
        adopt(&mut self.permission_mode, payload.permission_mode());
        adopt(&mut self.model, payload.model());
        self.follow_transcript(event, payload.transcript_path());
        self.adopt_watched_model();
    }

    fn adopt_watched_model(&mut self) -> bool {
        let watched = self.watcher.as_ref().and_then(TranscriptWatcher::model);
        watched.is_some_and(|watched| adopt(&mut self.model, Some(watched)))
    }

    fn follow_transcript(&mut self, event: &str, path: Option<PathBuf>) {
        let Some(path) = path else {
            return;
        };
        match self.watcher.as_mut().filter(|watcher| watcher.path() == path) {
            Some(watcher) if event == NEW_TURN_EVENT => watcher.skip_to_end(),
            Some(_) => {}
            None => self.watcher = Some(TranscriptWatcher::starting_at_end(path)),
        }
    }

    fn settle_from_transcript(&mut self, now: Instant) -> bool {
        if self.dismissed || !self.state.can_be_interrupted() {
            return false;
        }
        let change = self.watcher.as_mut().map_or(TranscriptChange::Unchanged, TranscriptWatcher::poll);
        let model_changed = self.adopt_watched_model();
        self.apply_transcript_change(change, now) || model_changed
    }

    fn apply_transcript_change(&mut self, change: TranscriptChange, now: Instant) -> bool {
        match change {
            TranscriptChange::Interrupted => return self.become_state(SessionState::Interrupted),
            TranscriptChange::Grew => self.last_signal = now,
            TranscriptChange::Unchanged => {}
        }
        let silence = now.saturating_duration_since(self.last_signal);
        self.state.goes_stale_after(silence) && self.become_state(SessionState::Idle)
    }

    fn become_state(&mut self, state: SessionState) -> bool {
        self.state = state;
        true
    }

    fn card_change(&self, session_id: &str) -> CardChange {
        CardChange::Show {
            session_id: session_id.to_owned(),
            project: self.project.clone(),
            state: self.state.clone(),
            traits: SessionTraits { permission_mode: self.permission_mode.clone(), model: self.model.clone() },
        }
    }
}

#[derive(Default)]
pub struct SessionTracker {
    sessions: Mutex<HashMap<String, TrackedSession>>,
}

impl SessionTracker {
    pub fn record_event(&self, event: &str, payload: &HookPayload, now: Instant) -> Option<CardChange> {
        let session_id = payload.session_id();
        let mut sessions = self.lock();
        let previous = sessions.get(session_id).map(|tracked| &tracked.state);
        match transition(previous, event, payload) {
            Transition::Ignore => None,
            Transition::Remove => {
                sessions.remove(session_id);
                Some(CardChange::Remove { session_id: session_id.to_owned() })
            }
            Transition::Show(state) => {
                let tracked = sessions.entry(session_id.to_owned()).or_insert_with(|| TrackedSession::new(now));
                tracked.absorb(event, payload, state, now);
                Some(tracked.card_change(session_id))
            }
        }
    }

    pub fn poll(&self, now: Instant) -> Vec<CardChange> {
        let mut sessions = self.lock();
        let changes = sessions
            .iter_mut()
            .filter_map(|(session_id, tracked)| {
                tracked.settle_from_transcript(now).then(|| tracked.card_change(session_id))
            })
            .collect();
        sessions.retain(|_, tracked| now.saturating_duration_since(tracked.last_signal) < SESSION_RETENTION);
        changes
    }

    pub fn change_permission_mode(&self, session_id: &str, mode: &str) -> Option<CardChange> {
        let mut sessions = self.lock();
        let tracked = sessions.get_mut(session_id)?;
        adopt(&mut tracked.permission_mode, Some(mode));
        (!tracked.dismissed).then(|| tracked.card_change(session_id))
    }

    pub fn transcripts(&self) -> Vec<(String, PathBuf)> {
        self.lock()
            .iter()
            .filter(|(_, tracked)| !tracked.dismissed)
            .filter_map(|(id, tracked)| Some((id.clone(), tracked.watcher.as_ref()?.path().to_path_buf())))
            .collect()
    }

    pub fn transcript_of(&self, session_id: &str) -> Option<PathBuf> {
        self.lock().get(session_id)?.watcher.as_ref().map(|watcher| watcher.path().to_path_buf())
    }

    pub fn dismiss(&self, session_id: &str) -> Option<CardChange> {
        let mut sessions = self.lock();
        sessions.get_mut(session_id).map(|tracked| {
            tracked.dismissed = true;
            CardChange::Remove { session_id: session_id.to_owned() }
        })
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, TrackedSession>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    use super::*;
    use crate::sources::claude::sessions::STALE_AFTER;

    const INTERRUPT_LINE: &str = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#;

    fn body(session: &str, transcript: Option<&Path>) -> HookPayload {
        let transcript_path = transcript.map(|path| path.to_string_lossy().into_owned());
        let json = serde_json::json!({ "session_id": session, "cwd": r"C:\code\demo", "transcript_path": transcript_path });
        HookPayload::parse(json.to_string().as_bytes())
    }

    fn state_of(change: Option<CardChange>) -> Option<SessionState> {
        match change? {
            CardChange::Show { state, .. } => Some(state),
            CardChange::Remove { .. } => None,
        }
    }

    fn temp_transcript(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("chocolatebar-tracker-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("session.jsonl");
        fs::write(&path, "").unwrap();
        path
    }

    fn append_line(path: &Path, line: &str) {
        let mut file = fs::OpenOptions::new().append(true).open(path).unwrap();
        writeln!(file, "{line}").unwrap();
    }

    fn cleanup(path: &Path) {
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn interrupt_in_the_transcript_settles_a_thinking_session() {
        let path = temp_transcript("interrupt");
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        let shown = tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), now);
        assert_eq!(state_of(shown), Some(SessionState::Thinking));
        assert!(tracker.poll(now).is_empty());
        append_line(&path, INTERRUPT_LINE);
        let changes = tracker.poll(now);
        assert_eq!(changes.len(), 1);
        assert_eq!(state_of(changes.into_iter().next()), Some(SessionState::Interrupted));
        assert!(tracker.poll(now).is_empty());
        cleanup(&path);
    }

    #[test]
    fn interrupt_during_a_tool_run_is_detected() {
        let path = temp_transcript("tool");
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("PreToolUse", &body("s1", Some(&path)), now);
        append_line(&path, INTERRUPT_LINE);
        assert_eq!(state_of(tracker.poll(now).into_iter().next()), Some(SessionState::Interrupted));
        cleanup(&path);
    }

    #[test]
    fn a_new_prompt_forgets_the_previous_interrupt() {
        let path = temp_transcript("newturn");
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), now);
        append_line(&path, INTERRUPT_LINE);
        tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), now);
        assert!(tracker.poll(now).is_empty());
        cleanup(&path);
    }

    #[test]
    fn silent_working_session_goes_idle_only_after_the_stale_window() {
        let (tracker, start) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), start);
        assert!(tracker.poll(start + STALE_AFTER - Duration::from_secs(1)).is_empty());
        let changes = tracker.poll(start + STALE_AFTER);
        assert_eq!(state_of(changes.into_iter().next()), Some(SessionState::Idle));
        assert!(tracker.poll(start + STALE_AFTER * 2).is_empty());
    }

    #[test]
    fn transcript_growth_keeps_a_working_session_alive() {
        let path = temp_transcript("alive");
        let (tracker, start) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), start);
        let almost_stale = start + STALE_AFTER - Duration::from_secs(1);
        append_line(&path, r#"{"type":"assistant","message":{"role":"assistant","content":[]}}"#);
        assert!(tracker.poll(almost_stale).is_empty());
        assert!(tracker.poll(almost_stale + STALE_AFTER - Duration::from_secs(1)).is_empty());
        cleanup(&path);
    }

    #[test]
    fn notification_after_an_interrupt_does_not_revive_thinking() {
        let path = temp_transcript("notify");
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), now);
        append_line(&path, INTERRUPT_LINE);
        tracker.poll(now);
        let shown = tracker.record_event("Notification", &body("s1", Some(&path)), now);
        assert!(matches!(state_of(shown), Some(SessionState::AwaitingFollowUp { .. })));
        assert!(tracker.poll(now + STALE_AFTER * 2).is_empty());
        cleanup(&path);
    }

    #[test]
    fn session_end_removes_the_card_and_forgets_the_session() {
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), now);
        let removed = tracker.record_event("SessionEnd", &body("s1", None), now);
        assert_eq!(removed, Some(CardChange::Remove { session_id: "s1".into() }));
        assert!(tracker.poll(now + STALE_AFTER * 2).is_empty());
    }

    #[test]
    fn dismissed_session_stays_hidden_until_its_next_event() {
        let (tracker, start) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), start);
        assert_eq!(tracker.dismiss("s1"), Some(CardChange::Remove { session_id: "s1".into() }));
        assert!(tracker.poll(start + STALE_AFTER * 2).is_empty());
        let revived = tracker.record_event("PostToolUse", &body("s1", None), start + STALE_AFTER * 2);
        assert_eq!(state_of(revived), Some(SessionState::Thinking));
    }

    #[test]
    fn mode_and_model_ride_along_and_persist_across_sparse_events() {
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        let full = HookPayload::parse(br#"{"session_id":"s1","permission_mode":"plan","model":"claude-opus-5-5"}"#);
        tracker.record_event("SessionStart", &full, now);
        let sparse = HookPayload::parse(br#"{"session_id":"s1"}"#);
        match tracker.record_event("PostToolUse", &sparse, now) {
            Some(CardChange::Show { traits, .. }) => {
                assert_eq!(traits.permission_mode.as_deref(), Some("plan"));
                assert_eq!(traits.model.as_deref(), Some("claude-opus-5-5"));
            }
            other => panic!("unexpected change {other:?}"),
        }
    }

    #[test]
    fn switching_mode_updates_the_card_and_unknown_sessions_are_ignored() {
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), now);
        match tracker.change_permission_mode("s1", "acceptEdits") {
            Some(CardChange::Show { traits, state, .. }) => {
                assert_eq!(traits.permission_mode.as_deref(), Some("acceptEdits"));
                assert_eq!(state, SessionState::Thinking);
            }
            other => panic!("unexpected change {other:?}"),
        }
        assert_eq!(tracker.change_permission_mode("ghost", "auto"), None);
    }

    #[test]
    fn switching_mode_of_a_dismissed_session_stays_hidden() {
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), now);
        tracker.dismiss("s1");
        assert_eq!(tracker.change_permission_mode("s1", "auto"), None);
    }

    #[test]
    fn model_from_the_transcript_reaches_the_card_while_working() {
        let path = temp_transcript("model");
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", Some(&path)), now);
        append_line(&path, r#"{"type":"assistant","message":{"model":"claude-opus-5-5","content":[]}}"#);
        match tracker.poll(now).into_iter().next() {
            Some(CardChange::Show { traits, .. }) => assert_eq!(traits.model.as_deref(), Some("claude-opus-5-5")),
            other => panic!("unexpected change {other:?}"),
        }
        assert!(tracker.poll(now).is_empty());
        cleanup(&path);
    }

    #[test]
    fn dismissing_an_unknown_session_changes_nothing() {
        assert_eq!(SessionTracker::default().dismiss("ghost"), None);
    }

    #[test]
    fn project_name_survives_events_without_a_working_directory() {
        let (tracker, now) = (SessionTracker::default(), Instant::now());
        tracker.record_event("UserPromptSubmit", &body("s1", None), now);
        let bare = HookPayload::parse(br#"{"session_id":"s1"}"#);
        match tracker.record_event("PostToolUse", &bare, now) {
            Some(CardChange::Show { project, .. }) => assert_eq!(project.as_deref(), Some("demo")),
            other => panic!("unexpected change {other:?}"),
        }
    }
}
