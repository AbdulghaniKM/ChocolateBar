use std::sync::Mutex;
use std::time::Duration;

use tauri::menu::MenuItem;
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_updater::{Update, UpdaterExt};

use super::error::AppResult;

const STARTUP_DELAY: Duration = Duration::from_secs(15);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
const CHECK_LABEL: &str = "Check for updates";

pub struct UpdaterState {
    item: MenuItem<Wry>,
    pending: Mutex<Option<Update>>,
}

pub fn menu_item(app: &AppHandle, id: &str) -> AppResult<MenuItem<Wry>> {
    let item = MenuItem::with_id(app, id, CHECK_LABEL, true, None::<&str>)?;
    app.manage(UpdaterState { item: item.clone(), pending: Mutex::new(None) });
    Ok(item)
}

/// Installs any update found right after launch, then re-checks periodically and
/// only offers later updates from the tray so a running session is never interrupted.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(STARTUP_DELAY).await;
        match check(&app).await {
            Ok(Some(update)) => install(&app, update).await,
            Ok(None) => {}
            Err(error) => log::warn!("update check failed: {error}"),
        }
        loop {
            tokio::time::sleep(CHECK_INTERVAL).await;
            match check(&app).await {
                Ok(Some(update)) => offer(&app, update),
                Ok(None) => {}
                Err(error) => log::warn!("update check failed: {error}"),
            }
        }
    });
}

pub fn on_menu_click(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let pending = app.state::<UpdaterState>().pending.lock().unwrap().take();
        let update = match pending {
            Some(update) => Some(update),
            None => {
                set_label(&app, "Checking for updates…", false);
                match check(&app).await {
                    Ok(update) => update,
                    Err(error) => {
                        log::warn!("update check failed: {error}");
                        set_label(&app, "Update check failed, retry", true);
                        return;
                    }
                }
            }
        };
        match update {
            Some(update) => install(&app, update).await,
            None => set_label(&app, &format!("Up to date (v{})", app.package_info().version), true),
        }
    });
}

async fn check(app: &AppHandle) -> AppResult<Option<Update>> {
    Ok(app.updater()?.check().await?)
}

fn offer(app: &AppHandle, update: Update) {
    set_label(app, &format!("Install update v{} and restart", update.version), true);
    *app.state::<UpdaterState>().pending.lock().unwrap() = Some(update);
}

async fn install(app: &AppHandle, update: Update) {
    log::info!("installing update v{}", update.version);
    set_label(app, &format!("Downloading v{}…", update.version), false);
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(()) => app.restart(),
        Err(error) => {
            log::warn!("update install failed: {error}");
            set_label(app, "Update failed, retry", true);
        }
    }
}

fn set_label(app: &AppHandle, text: &str, enabled: bool) {
    let item = &app.state::<UpdaterState>().item;
    let _ = item.set_text(text);
    let _ = item.set_enabled(enabled);
}
