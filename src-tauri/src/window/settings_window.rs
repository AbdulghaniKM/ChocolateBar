use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, Theme, WebviewUrl, WebviewWindowBuilder};

use crate::core::error::AppResult;

const SETTINGS_LABEL: &str = "settings";
const SETTINGS_URL: &str = "index.html#settings";
const TAB_EVENT: &str = "settings://tab";
const BACKGROUND: Color = Color(0x14, 0x0e, 0x0b, 0xff);

pub fn open(app: &AppHandle, tab: Option<&str>) -> AppResult<()> {
    if let Some(existing) = app.get_webview_window(SETTINGS_LABEL) {
        existing.show()?;
        existing.unminimize()?;
        existing.set_focus()?;
        if let Some(tab) = tab {
            existing.emit(TAB_EVENT, tab)?;
        }
        return Ok(());
    }
    let url = match tab {
        Some(tab) => format!("{SETTINGS_URL}/{tab}"),
        None => SETTINGS_URL.to_string(),
    };
    WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::App(url.into()))
        .title("ChocolateBar Settings")
        .inner_size(960.0, 680.0)
        .min_inner_size(720.0, 520.0)
        .theme(Some(Theme::Dark))
        .background_color(BACKGROUND)
        .decorations(true)
        .resizable(true)
        .center()
        .build()?;
    Ok(())
}

pub fn open_in_background(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = open(&app, None) {
            log::error!("opening settings failed: {error}");
        }
    });
}
