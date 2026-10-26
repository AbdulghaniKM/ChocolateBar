use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Ambient,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    Media,
    Osd,
    Privacy,
    Buds,
    Bluetooth,
    Battery,
    Keyboard,
    Download,
    Network,
    Claude,
    Notification,
    Ocr,
    Shelf,
    Sports,
    Timer,
    Weather,
    Game,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactView {
    pub icon: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub progress: Option<f32>,
    pub accent: Option<String>,
    pub badge: Option<String>,
}

impl CompactView {
    pub fn new(icon: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            subtitle: None,
            progress: None,
            accent: None,
            badge: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn progress(mut self, fraction: f32) -> Self {
        self.progress = Some(fraction.clamp(0.0, 1.0));
        self
    }

    pub fn accent(mut self, color: impl Into<String>) -> Self {
        self.accent = Some(color.into());
        self
    }

    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionStyle {
    Primary,
    Neutral,
    Danger,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ActionControl {
    Button,
    Toggle { on: bool },
    Slider { min: f64, max: f64, step: f64, value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionSpec {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub style: ActionStyle,
    pub control: ActionControl,
}

impl ActionSpec {
    fn with_control(id: impl Into<String>, label: impl Into<String>, control: ActionControl) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            style: ActionStyle::Neutral,
            control,
        }
    }

    pub fn button(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::with_control(id, label, ActionControl::Button)
    }

    pub fn toggle(id: impl Into<String>, label: impl Into<String>, on: bool) -> Self {
        Self::with_control(id, label, ActionControl::Toggle { on })
    }

    pub fn slider(
        id: impl Into<String>,
        label: impl Into<String>,
        range: (f64, f64),
        step: f64,
        value: f64,
    ) -> Self {
        let control = ActionControl::Slider { min: range.0, max: range.1, step, value };
        Self::with_control(id, label, control)
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn style(mut self, style: ActionStyle) -> Self {
        self.style = style;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub kind: ActivityKind,
    pub priority: Priority,
    pub compact: CompactView,
    pub expanded: Value,
    pub actions: Vec<ActionSpec>,
    pub alert: bool,
    pub ttl_ms: Option<u64>,
    pub updated_at_ms: u64,
}

impl Activity {
    pub fn new(source: &str, key: &str, kind: ActivityKind, compact: CompactView) -> Self {
        Self {
            id: Self::id_for(source, key),
            kind,
            priority: Priority::Normal,
            compact,
            expanded: Value::Null,
            actions: Vec::new(),
            alert: false,
            ttl_ms: None,
            updated_at_ms: now_ms(),
        }
    }

    pub fn id_for(source: &str, key: &str) -> String {
        format!("{source}:{key}")
    }

    pub fn source_of(id: &str) -> &str {
        id.split(':').next().unwrap_or(id)
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn expanded(mut self, expanded: Value) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn action(mut self, action: ActionSpec) -> Self {
        self.actions.push(action);
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = ActionSpec>) -> Self {
        self.actions.extend(actions);
        self
    }

    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    pub fn ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.ttl_ms = Some(ttl_ms);
        self
    }

    pub fn is_expired_at(&self, now: u64) -> bool {
        self.ttl_ms.is_some_and(|ttl| now >= self.updated_at_ms.saturating_add(ttl))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub activity_id: String,
    pub action: String,
    pub value: Option<Value>,
}

pub type CommandReply = Result<Value, String>;

pub struct SourceCommand {
    pub source: String,
    pub command: String,
    pub payload: Value,
    pub reply: oneshot::Sender<CommandReply>,
}

impl SourceCommand {
    pub fn respond_ok(self, value: Value) {
        let _ = self.reply.send(Ok(value));
    }

    pub fn respond_err(self, message: impl Into<String>) {
        let _ = self.reply.send(Err(message.into()));
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or_default()
}
