use tauri::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use super::error::{AppError, AppResult};
use super::hub::HubHandle;
use crate::window::settings_window;

const SETTINGS_ID: &str = "settings";
const PAUSE_ID: &str = "pause";
const QUIT_ID: &str = "quit";

pub fn setup(app: &AppHandle) -> AppResult<()> {
    let settings_item = MenuItem::with_id(app, SETTINGS_ID, "Settings", true, None::<&str>)?;
    let pause_item = CheckMenuItem::with_id(app, PAUSE_ID, "Pause island", true, false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &pause_item, &quit_item])?;
    let icon = app.default_window_icon().cloned().ok_or(AppError::Missing("window icon"))?;
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("ChocolateBar")
        .menu(&menu)
        .on_menu_event(move |app, event| handle_menu_event(app, &event, &pause_item))
        .build(app)?;
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: &MenuEvent, pause_item: &CheckMenuItem<tauri::Wry>) {
    match event.id.as_ref() {
        SETTINGS_ID => settings_window::open_in_background(app),
        PAUSE_ID => {
            let paused = pause_item.is_checked().unwrap_or(false);
            app.state::<HubHandle>().set_paused(paused);
        }
        QUIT_ID => app.exit(0),
        _ => {}
    }
}
