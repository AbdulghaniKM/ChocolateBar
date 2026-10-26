use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Map, Value};

use super::server::TOKEN_HEADER;
use crate::core::error::{AppError, AppResult};

pub const HOOK_EVENTS: [&str; 8] = [
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "Notification",
    "Stop",
    "SessionEnd",
    "PermissionRequest",
];
const PERMISSION_EVENT: &str = "PermissionRequest";
const PERMISSION_TIMEOUT_SECONDS: u32 = 600;
const QUICK_TIMEOUT_SECONDS: u32 = 10;
const ENDPOINT_URL_PREFIX: &str = "http://127.0.0.1:";
const ENDPOINT_PATH_MARKER: &str = "/hook/";
const BACKUP_EXTENSION: &str = "json.chocolatebar.bak";

#[derive(Debug, Clone)]
pub struct HookEndpoint {
    pub port: u16,
    pub token: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HooksStatus {
    pub installed: bool,
    pub events: Vec<String>,
    pub settings_path: String,
    pub backup_exists: bool,
}

pub fn claude_settings_path() -> AppResult<PathBuf> {
    let home = dirs::home_dir().ok_or(AppError::Missing("home directory"))?;
    Ok(home.join(".claude").join("settings.json"))
}

pub fn install(path: &Path, endpoint: &HookEndpoint) -> AppResult<HooksStatus> {
    let mut settings = read_settings(path)?;
    merge_hooks(&mut settings, endpoint)?;
    write_with_backup(path, &settings)?;
    Ok(status(path))
}

pub fn uninstall(path: &Path) -> AppResult<HooksStatus> {
    if !path.exists() {
        return Ok(status(path));
    }
    let mut settings = read_settings(path)?;
    strip_our_hooks(&mut settings);
    write_with_backup(path, &settings)?;
    Ok(status(path))
}

pub fn status(path: &Path) -> HooksStatus {
    let settings = read_settings(path).unwrap_or(Value::Null);
    let events = installed_events(&settings);
    HooksStatus {
        installed: events.len() == HOOK_EVENTS.len(),
        events,
        settings_path: path.display().to_string(),
        backup_exists: backup_path(path).exists(),
    }
}

pub fn merge_hooks(settings: &mut Value, endpoint: &HookEndpoint) -> AppResult<()> {
    strip_our_hooks(settings);
    let hooks = hooks_object(settings)?;
    for event in HOOK_EVENTS {
        let entries = hooks.entry(event).or_insert_with(|| json!([]));
        entries
            .as_array_mut()
            .ok_or_else(|| unexpected_shape(event))?
            .push(our_entry(event, endpoint));
    }
    Ok(())
}

pub fn strip_our_hooks(settings: &mut Value) {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };
    let touched: Vec<String> = hooks
        .iter_mut()
        .filter_map(|(event, entries)| remove_our_entries(entries).then(|| event.clone()))
        .collect();
    let emptied: Vec<&String> = touched.iter().filter(|event| is_empty_array(&hooks[event.as_str()])).collect();
    for event in emptied {
        hooks.remove(event);
    }
    if !touched.is_empty() && hooks.is_empty() {
        remove_key(settings, "hooks");
    }
}

pub fn installed_events(settings: &Value) -> Vec<String> {
    let Some(hooks) = settings.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    HOOK_EVENTS
        .iter()
        .filter(|event| hooks.get(**event).is_some_and(contains_our_hook))
        .map(|event| (*event).to_owned())
        .collect()
}

fn our_entry(event: &str, endpoint: &HookEndpoint) -> Value {
    let timeout = if event == PERMISSION_EVENT { PERMISSION_TIMEOUT_SECONDS } else { QUICK_TIMEOUT_SECONDS };
    json!({ "hooks": [{ "type": "command", "command": curl_command(event, endpoint, timeout), "timeout": timeout }] })
}

fn curl_command(event: &str, endpoint: &HookEndpoint, max_seconds: u32) -> String {
    format!(
        "curl.exe -s --max-time {max_seconds} -H \"{TOKEN_HEADER}: {token}\" -H \"Content-Type: application/json\" --data-binary \"@-\" {ENDPOINT_URL_PREFIX}{port}{ENDPOINT_PATH_MARKER}{event}",
        token = endpoint.token,
        port = endpoint.port,
    )
}

fn is_our_hook(hook: &Value) -> bool {
    hook.get("command")
        .and_then(Value::as_str)
        .is_some_and(|command| command.contains(ENDPOINT_URL_PREFIX) && command.contains(ENDPOINT_PATH_MARKER))
}

fn contains_our_hook(entries: &Value) -> bool {
    let Some(entries) = entries.as_array() else {
        return false;
    };
    entries
        .iter()
        .filter_map(|entry| entry.get("hooks").and_then(Value::as_array))
        .flatten()
        .any(is_our_hook)
}

fn remove_our_entries(entries: &mut Value) -> bool {
    let Some(entries) = entries.as_array_mut() else {
        return false;
    };
    let before = entries.len();
    let mut changed = false;
    entries.retain_mut(|entry| {
        let Some(hooks) = entry.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        let hooks_before = hooks.len();
        hooks.retain(|hook| !is_our_hook(hook));
        changed |= hooks.len() != hooks_before;
        !hooks.is_empty() || hooks.len() == hooks_before
    });
    changed || entries.len() != before
}

fn is_empty_array(value: &Value) -> bool {
    value.as_array().is_some_and(Vec::is_empty)
}

fn remove_key(settings: &mut Value, key: &str) {
    if let Some(object) = settings.as_object_mut() {
        object.remove(key);
    }
}

fn hooks_object(settings: &mut Value) -> AppResult<&mut Map<String, Value>> {
    settings
        .as_object_mut()
        .ok_or_else(|| unexpected_shape("settings.json"))?
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| unexpected_shape("hooks"))
}

pub(super) fn unexpected_shape(what: &str) -> AppError {
    AppError::Source(format!("{what} in Claude settings has an unexpected shape; nothing was changed"))
}

pub(super) fn read_settings(path: &Path) -> AppResult<Value> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let text = fs::read_to_string(path).map_err(|error| io_failure("read", path, &error))?;
    if text.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&text)
        .map_err(|error| AppError::Source(format!("{} is not valid JSON ({error}); nothing was changed", path.display())))
}

pub(super) fn write_with_backup(path: &Path, settings: &Value) -> AppResult<()> {
    if path.exists() {
        fs::copy(path, backup_path(path)).map_err(|error| io_failure("back up", path, &error))?;
    } else if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_failure("create folder for", path, &error))?;
    }
    let mut text = serde_json::to_string_pretty(settings)?;
    text.push('\n');
    fs::write(path, text).map_err(|error| io_failure("write", path, &error))
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension(BACKUP_EXTENSION)
}

fn io_failure(action: &str, path: &Path, error: &std::io::Error) -> AppError {
    AppError::Source(format!("could not {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint() -> HookEndpoint {
        HookEndpoint { port: 47653, token: "secret-token".into() }
    }

    fn user_settings() -> Value {
        json!({
            "model": "opus",
            "hooks": {
                "PreToolUse": [{ "matcher": "Bash", "hooks": [{ "type": "command", "command": "my-linter" }] }],
                "Elicitation": [{ "hooks": [{ "type": "command", "command": "other" }] }]
            }
        })
    }

    fn temp_settings_path(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("chocolatebar-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder.join("settings.json")
    }

    #[test]
    fn merge_adds_all_events_and_keeps_user_entries() {
        let mut settings = user_settings();
        merge_hooks(&mut settings, &endpoint()).unwrap();
        assert_eq!(installed_events(&settings).len(), HOOK_EVENTS.len());
        assert_eq!(settings["model"], "opus");
        assert_eq!(settings["hooks"]["Elicitation"], user_settings()["hooks"]["Elicitation"]);
        let pre_tool_use = settings["hooks"]["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_use.len(), 2);
        assert_eq!(pre_tool_use[0], user_settings()["hooks"]["PreToolUse"][0]);
    }

    #[test]
    fn merge_twice_does_not_duplicate_our_entries() {
        let mut settings = user_settings();
        merge_hooks(&mut settings, &endpoint()).unwrap();
        merge_hooks(&mut settings, &HookEndpoint { port: 50000, token: "new".into() }).unwrap();
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert!(settings["hooks"]["Stop"][0]["hooks"][0]["command"].as_str().unwrap().contains(":50000/"));
    }

    #[test]
    fn upgrading_an_older_install_adds_session_end_without_duplicates() {
        let mut settings = user_settings();
        merge_hooks(&mut settings, &endpoint()).unwrap();
        settings["hooks"].as_object_mut().unwrap().remove("SessionEnd");
        assert_eq!(installed_events(&settings).len(), HOOK_EVENTS.len() - 1);
        merge_hooks(&mut settings, &endpoint()).unwrap();
        assert_eq!(installed_events(&settings).len(), HOOK_EVENTS.len());
        assert_eq!(settings["hooks"]["SessionEnd"].as_array().unwrap().len(), 1);
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(settings["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn strip_restores_the_original_settings() {
        let mut settings = user_settings();
        merge_hooks(&mut settings, &endpoint()).unwrap();
        strip_our_hooks(&mut settings);
        assert_eq!(settings, user_settings());
    }

    #[test]
    fn strip_removes_hooks_key_when_only_ours_existed() {
        let mut settings = json!({ "model": "opus" });
        merge_hooks(&mut settings, &endpoint()).unwrap();
        strip_our_hooks(&mut settings);
        assert_eq!(settings, json!({ "model": "opus" }));
    }

    #[test]
    fn strip_leaves_user_empty_arrays_alone() {
        let mut settings = json!({ "hooks": { "Stop": [] } });
        strip_our_hooks(&mut settings);
        assert_eq!(settings, json!({ "hooks": { "Stop": [] } }));
    }

    #[test]
    fn permission_hook_waits_longer_than_quick_hooks() {
        let mut settings = json!({});
        merge_hooks(&mut settings, &endpoint()).unwrap();
        assert_eq!(settings["hooks"]["PermissionRequest"][0]["hooks"][0]["timeout"], PERMISSION_TIMEOUT_SECONDS);
        assert_eq!(settings["hooks"]["Stop"][0]["hooks"][0]["timeout"], QUICK_TIMEOUT_SECONDS);
    }

    #[test]
    fn command_carries_token_header_and_endpoint() {
        let command = curl_command("Stop", &endpoint(), 10);
        assert!(command.contains("x-chocolatebar-token: secret-token"));
        assert!(command.ends_with("http://127.0.0.1:47653/hook/Stop"));
    }

    #[test]
    fn install_and_uninstall_roundtrip_on_a_temp_file_with_backup() {
        let path = temp_settings_path("roundtrip");
        fs::write(&path, serde_json::to_string(&user_settings()).unwrap()).unwrap();
        assert!(install(&path, &endpoint()).unwrap().installed);
        assert!(backup_path(&path).exists());
        assert!(!uninstall(&path).unwrap().installed);
        let restored: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(restored, user_settings());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn invalid_json_is_never_overwritten() {
        let path = temp_settings_path("invalid");
        fs::write(&path, "{ not json").unwrap();
        assert!(install(&path, &endpoint()).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn install_creates_missing_settings_file() {
        let path = temp_settings_path("missing");
        assert!(install(&path, &endpoint()).unwrap().installed);
        assert!(!backup_path(&path).exists());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
