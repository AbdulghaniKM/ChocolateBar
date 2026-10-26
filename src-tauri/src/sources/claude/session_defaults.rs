use std::path::Path;

use serde::Serialize;
use serde_json::{json, Map, Value};

use super::hooks_install::{read_settings, unexpected_shape, write_with_backup};
use crate::core::error::{AppError, AppResult};

pub const PERMISSION_MODES: [&str; 4] = ["default", "acceptEdits", "plan", "auto"];
pub const MODEL_CHOICES: [&str; 7] = ["default", "opus", "sonnet", "haiku", "opusplan", "opus[1m]", "sonnet[1m]"];
const MODEL_RUNTIME_DEFAULT: &str = "default";

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionDefaults {
    pub permission_mode: Option<String>,
    pub model: Option<String>,
    pub permission_modes: Vec<&'static str>,
    pub models: Vec<&'static str>,
    pub settings_path: String,
}

pub fn read_defaults(path: &Path) -> SessionDefaults {
    let settings = read_settings(path).unwrap_or(Value::Null);
    SessionDefaults {
        permission_mode: text_at(&settings, "/permissions/defaultMode"),
        model: text_at(&settings, "/model"),
        permission_modes: PERMISSION_MODES.to_vec(),
        models: MODEL_CHOICES.to_vec(),
        settings_path: path.display().to_string(),
    }
}

pub fn set_default_permission_mode(path: &Path, mode: &str) -> AppResult<SessionDefaults> {
    require_choice("permission mode", mode, &PERMISSION_MODES)?;
    let mut settings = read_settings(path)?;
    merge_permission_mode(&mut settings, mode)?;
    write_with_backup(path, &settings)?;
    Ok(read_defaults(path))
}

pub fn set_default_model(path: &Path, model: &str) -> AppResult<SessionDefaults> {
    require_choice("model", model, &MODEL_CHOICES)?;
    let mut settings = read_settings(path)?;
    merge_model(&mut settings, model)?;
    write_with_backup(path, &settings)?;
    Ok(read_defaults(path))
}

pub fn merge_permission_mode(settings: &mut Value, mode: &str) -> AppResult<()> {
    let permissions = settings_object(settings)?
        .entry("permissions")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| unexpected_shape("permissions"))?;
    permissions.insert("defaultMode".into(), json!(mode));
    Ok(())
}

pub fn merge_model(settings: &mut Value, model: &str) -> AppResult<()> {
    let object = settings_object(settings)?;
    if model == MODEL_RUNTIME_DEFAULT {
        object.remove("model");
    } else {
        object.insert("model".into(), json!(model));
    }
    Ok(())
}

fn settings_object(settings: &mut Value) -> AppResult<&mut Map<String, Value>> {
    settings.as_object_mut().ok_or_else(|| unexpected_shape("settings.json"))
}

fn require_choice(what: &str, value: &str, allowed: &[&str]) -> AppResult<()> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(AppError::Source(format!("{value} is not a supported {what}")))
}

fn text_at(settings: &Value, pointer: &str) -> Option<String> {
    settings.pointer(pointer)?.as_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn temp_settings_path(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("chocolatebar-defaults-{name}-{}", std::process::id()));
        fs::create_dir_all(&folder).unwrap();
        folder.join("settings.json")
    }

    fn user_settings() -> Value {
        json!({
            "env": { "A": "1" },
            "permissions": { "allow": ["Bash(npm run lint)"], "defaultMode": "plan" },
            "model": "haiku",
            "hooks": { "Stop": [] }
        })
    }

    #[test]
    fn permission_mode_merge_keeps_every_other_key() {
        let mut settings = user_settings();
        merge_permission_mode(&mut settings, "acceptEdits").unwrap();
        let mut expected = user_settings();
        expected["permissions"]["defaultMode"] = json!("acceptEdits");
        assert_eq!(settings, expected);
    }

    #[test]
    fn permission_mode_merge_creates_the_permissions_object() {
        let mut settings = json!({ "model": "opus" });
        merge_permission_mode(&mut settings, "auto").unwrap();
        assert_eq!(settings, json!({ "model": "opus", "permissions": { "defaultMode": "auto" } }));
    }

    #[test]
    fn model_merge_sets_only_the_model_key() {
        let mut settings = user_settings();
        merge_model(&mut settings, "opus[1m]").unwrap();
        let mut expected = user_settings();
        expected["model"] = json!("opus[1m]");
        assert_eq!(settings, expected);
    }

    #[test]
    fn choosing_the_runtime_default_model_removes_the_override() {
        let mut settings = user_settings();
        merge_model(&mut settings, "default").unwrap();
        let mut expected = user_settings();
        expected.as_object_mut().unwrap().remove("model");
        assert_eq!(settings, expected);
    }

    #[test]
    fn unexpected_shapes_are_rejected_untouched() {
        let mut permissions_not_object = json!({ "permissions": [] });
        assert!(merge_permission_mode(&mut permissions_not_object, "plan").is_err());
        assert_eq!(permissions_not_object, json!({ "permissions": [] }));
        assert!(merge_model(&mut json!([]), "opus").is_err());
    }

    #[test]
    fn writing_both_defaults_to_a_temp_file_backs_up_and_preserves_the_rest() {
        let path = temp_settings_path("both");
        fs::write(&path, serde_json::to_string(&user_settings()).unwrap()).unwrap();
        set_default_permission_mode(&path, "auto").unwrap();
        let defaults = set_default_model(&path, "sonnet").unwrap();
        assert_eq!(defaults.permission_mode.as_deref(), Some("auto"));
        assert_eq!(defaults.model.as_deref(), Some("sonnet"));
        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut expected = user_settings();
        expected["permissions"]["defaultMode"] = json!("auto");
        expected["model"] = json!("sonnet");
        assert_eq!(written, expected);
        assert!(path.with_extension("json.chocolatebar.bak").exists());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn unsupported_values_never_touch_the_file() {
        let path = temp_settings_path("invalid-value");
        fs::write(&path, "{\"model\":\"haiku\"}").unwrap();
        assert!(set_default_permission_mode(&path, "bypassPermissions").is_err());
        assert!(set_default_model(&path, "gpt-9").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"model\":\"haiku\"}");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn invalid_json_is_never_overwritten() {
        let path = temp_settings_path("invalid-json");
        fs::write(&path, "{ not json").unwrap();
        assert!(set_default_model(&path, "opus").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ not json");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn reading_a_missing_file_reports_no_defaults() {
        let path = temp_settings_path("absent");
        let defaults = read_defaults(&path);
        assert_eq!((defaults.permission_mode, defaults.model), (None, None));
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
