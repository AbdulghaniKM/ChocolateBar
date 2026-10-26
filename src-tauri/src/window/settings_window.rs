use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::core::error::AppResult;

const SETTINGS_LABEL: &str = "settings";
const SETTINGS_URL: &str = "index.html#settings";

pub fn open(app: &AppHandle) -> AppResult<()> {
    if let Some(existing) = app.get_webview_window(SETTINGS_LABEL) {
        existing.show()?;
        existing.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App(SETTINGS_URL.into()))
        .title("ChocolateBar Settings")
        .inner_size(900.0, 640.0)
        .min_inner_size(640.0, 480.0)
        .decorations(true)
        .resizable(true)
        .center()
        .build()?;
    Ok(())
}

pub fn open_in_background(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = open(&app) {
            log::error!("opening settings failed: {error}");
        }
    });
}
