use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;
use tokio::sync::watch;

use super::error::AppResult;
use super::events::SETTINGS_CHANGED;

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GeneralSettings {
    pub hide_in_fullscreen: bool,
    pub autostart: bool,
    pub position: IslandPosition,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self { hide_in_fullscreen: true, autostart: false, position: IslandPosition::default() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAnchor {
    Top,
    Middle,
    #[default]
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HorizontalAnchor {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct IslandPosition {
    pub vertical: VerticalAnchor,
    pub horizontal: HorizontalAnchor,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct HotkeySettings {
    pub mic_mute: String,
    pub ocr: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self { mic_mute: "Ctrl+Alt+M".into(), ocr: "Ctrl+Alt+O".into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MediaSettings {
    pub lyrics: bool,
    pub visualizer: bool,
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self { lyrics: true, visualizer: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OsdSettings {
    pub replace_volume: bool,
    pub brightness: bool,
}

impl Default for OsdSettings {
    fn default() -> Self {
        Self { replace_volume: true, brightness: true }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ClaudeSettings {
    pub port: u16,
    pub token: String,
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self { port: 47653, token: String::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRef {
    pub sport: String,
    pub league: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SportsSettings {
    pub leagues: Vec<String>,
    pub favorite_teams: Vec<TeamRef>,
}

impl Default for SportsSettings {
    fn default() -> Self {
        let leagues = ["eng.1", "esp.1", "uefa.champions", "nba"];
        Self {
            leagues: leagues.into_iter().map(String::from).collect(),
            favorite_teams: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TimerSettings {
    pub work_min: u32,
    pub short_min: u32,
    pub long_min: u32,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self { work_min: 25, short_min: 5, long_min: 15 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WeatherSettings {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub city: Option<String>,
    pub units: String,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self { lat: None, lon: None, city: None, units: "metric".into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OcrSettings {
    pub auto_clipboard: bool,
    pub language: String,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self { auto_clipboard: false, language: "auto".into() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NetworkSettings {
    pub show_speed: bool,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self { show_speed: true }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WindowLayout {
    #[default]
    None,
    Maximize,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutApp {
    pub app_id: String,
    pub name: String,
    #[serde(default)]
    pub layout: WindowLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub apps: Vec<ShortcutApp>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShortcutSettings {
    pub groups: Vec<ShortcutGroup>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub general: GeneralSettings,
    pub hotkeys: HotkeySettings,
    pub sources: BTreeMap<String, bool>,
    pub media: MediaSettings,
    pub osd: OsdSettings,
    pub claude: ClaudeSettings,
    pub sports: SportsSettings,
    pub timer: TimerSettings,
    pub weather: WeatherSettings,
    pub ocr: OcrSettings,
    pub network: NetworkSettings,
    pub shortcuts: ShortcutSettings,
}

impl Settings {
    pub fn is_source_enabled(&self, name: &str) -> bool {
        self.sources.get(name).copied().unwrap_or(true)
    }
}

pub struct SettingsStore {
    sender: watch::Sender<Settings>,
}

impl SettingsStore {
    pub fn load(app: &AppHandle) -> AppResult<Self> {
        let store = app.store(SETTINGS_FILE)?;
        let saved = store.get(SETTINGS_KEY).and_then(|value| serde_json::from_value(value).ok());
        let (sender, _) = watch::channel(saved.unwrap_or_default());
        Ok(Self { sender })
    }

    pub fn current(&self) -> Settings {
        self.sender.borrow().clone()
    }

    pub fn subscribe(&self) -> watch::Receiver<Settings> {
        self.sender.subscribe()
    }

    pub fn apply_patch(&self, app: &AppHandle, patch: Value) -> AppResult<Settings> {
        let mut merged = serde_json::to_value(self.current())?;
        deep_merge(&mut merged, patch);
        let updated: Settings = serde_json::from_value(merged)?;
        persist(app, &updated)?;
        self.sender.send_replace(updated.clone());
        app.emit(SETTINGS_CHANGED, &updated)?;
        Ok(updated)
    }
}

fn persist(app: &AppHandle, settings: &Settings) -> AppResult<()> {
    let store = app.store(SETTINGS_FILE)?;
    store.set(SETTINGS_KEY, serde_json::to_value(settings)?);
    store.save()?;
    Ok(())
}

fn deep_merge(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target_map), Value::Object(patch_map)) => {
            for (key, value) in patch_map {
                deep_merge(target_map.entry(key).or_insert(Value::Null), value);
            }
        }
        (target, patch) => *target = patch,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn deep_merge_overrides_only_patched_leaves() {
        let mut target = serde_json::to_value(Settings::default()).unwrap();
        deep_merge(&mut target, json!({ "timer": { "workMin": 50 }, "sources": { "media": false } }));
        let merged: Settings = serde_json::from_value(target).unwrap();
        assert_eq!(merged.timer.work_min, 50);
        assert_eq!(merged.timer.short_min, 5);
        assert!(!merged.is_source_enabled("media"));
        assert!(merged.is_source_enabled("osd"));
    }

    #[test]
    fn empty_json_yields_defaults() {
        let parsed: Settings = serde_json::from_value(json!({})).unwrap();
        assert_eq!(parsed, Settings::default());
    }
}
