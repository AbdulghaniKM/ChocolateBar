use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use super::error::AppResult;
use super::settings::Settings;

pub fn sync_with_settings(app: &AppHandle, settings: &Settings) -> AppResult<()> {
    let launcher = app.autolaunch();
    let currently_enabled = launcher.is_enabled()?;
    match (settings.general.autostart, currently_enabled) {
        (true, false) => launcher.enable()?,
        (false, true) => launcher.disable()?,
        _ => {}
    }
    Ok(())
}
