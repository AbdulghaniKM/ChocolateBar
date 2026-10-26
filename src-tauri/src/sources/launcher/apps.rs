use std::os::windows::process::CommandExt;
use std::process::Command;

use serde::{Deserialize, Serialize};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const LIST_START_APPS: &str = "[Console]::OutputEncoding=[Text.Encoding]::UTF8; \
    Get-StartApps | Select-Object Name,AppID | ConvertTo-Json -Compress";
const HIDDEN_WORDS: [&str; 4] = ["uninstall", "readme", "release notes", "help"];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub name: String,
    pub app_id: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StartAppsJson {
    Many(Vec<StartApp>),
    One(StartApp),
}

#[derive(Deserialize)]
struct StartApp {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "AppID")]
    app_id: Option<String>,
}

/// Everything in the Start menu, desktop and Store apps alike, the same list `Get-StartApps` reports.
pub fn installed() -> Result<Vec<InstalledApp>, String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", LIST_START_APPS])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|error| format!("could not list apps: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    parse(&String::from_utf8_lossy(&output.stdout))
}

fn parse(json: &str) -> Result<Vec<InstalledApp>, String> {
    let trimmed = json.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: StartAppsJson = serde_json::from_str(trimmed).map_err(|error| format!("unexpected app list: {error}"))?;
    let entries = match parsed {
        StartAppsJson::Many(entries) => entries,
        StartAppsJson::One(entry) => vec![entry],
    };
    let mut apps: Vec<InstalledApp> = entries
        .into_iter()
        .filter_map(|entry| Some(InstalledApp { name: entry.name?.trim().to_string(), app_id: entry.app_id? }))
        .filter(|app| !app.name.is_empty() && !is_noise(&app.name))
        .collect();
    apps.sort_by_key(|app| app.name.to_lowercase());
    apps.dedup_by(|a, b| a.app_id == b.app_id);
    Ok(apps)
}

fn is_noise(name: &str) -> bool {
    let lower = name.to_lowercase();
    HIDDEN_WORDS.iter().any(|word| lower.contains(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sorts_and_drops_uninstallers() {
        let json = r#"[{"Name":"zed","AppID":"z"},{"Name":"Discord","AppID":"com.squirrel.Discord.Discord"},{"Name":"Uninstall Foo","AppID":"u"}]"#;
        let apps = parse(json).unwrap();
        let names: Vec<&str> = apps.iter().map(|app| app.name.as_str()).collect();
        assert_eq!(names, vec!["Discord", "zed"]);
    }

    #[test]
    fn accepts_a_single_object_and_a_byte_order_mark() {
        let apps = parse("\u{feff}{\"Name\":\"Word\",\"AppID\":\"Microsoft.Office.WINWORD.EXE.15\"}").unwrap();
        assert_eq!(apps[0].app_id, "Microsoft.Office.WINWORD.EXE.15");
    }
}
