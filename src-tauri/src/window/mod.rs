pub mod click_through;
pub mod fullscreen;
pub mod native;
pub mod no_activate;
pub mod placement;
pub mod settings_window;
pub mod styles;
pub mod subclass;
pub mod zorder;

use tauri::{AppHandle, Manager, WebviewWindow, WindowEvent};

use crate::core::error::{AppError, AppResult};
use crate::core::hub::HubHandle;
use crate::core::settings::SettingsStore;
use crate::core::source::Shutdown;
use click_through::HitRegions;

const MAIN_LABEL: &str = "main";

pub fn setup(app: &AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window(MAIN_LABEL)
        .ok_or(AppError::Missing("main window"))?;
    let hwnd = native::capture(&window)?;
    show_as_overlay(&window)?;
    placement::set_position(hwnd, app.state::<SettingsStore>().current().general.position);
    placement::place(hwnd);
    subclass::install(hwnd)?;
    replace_on_scale_change(&window);
    start_watchers(app, window);
    Ok(())
}

fn show_as_overlay(window: &WebviewWindow) -> AppResult<()> {
    window.show()?;
    restyle_island();
    Ok(())
}

fn restyle_island() {
    let hwnd = native::island();
    styles::apply(hwnd);
    styles::set_click_through(hwnd, true);
}

pub fn apply_visibility(app: &AppHandle, visible: bool) {
    let Some(window) = app.get_webview_window(MAIN_LABEL) else {
        return;
    };
    let result = app.run_on_main_thread(move || {
        if let Err(error) = change_visibility(&window, visible) {
            log::warn!("visibility change failed: {error}");
        }
    });
    if let Err(error) = result {
        log::warn!("visibility change failed: {error}");
    }
}

fn change_visibility(window: &WebviewWindow, visible: bool) -> AppResult<()> {
    if visible {
        return show_as_overlay(window);
    }
    window.hide()?;
    Ok(())
}

fn replace_on_scale_change(window: &WebviewWindow) {
    window.on_window_event(|event| {
        if matches!(event, WindowEvent::ScaleFactorChanged { .. }) {
            placement::place(native::island());
        }
    });
}

fn start_watchers(app: &AppHandle, window: WebviewWindow) {
    let shutdown = app.state::<Shutdown>().flag();
    let settings = app.state::<SettingsStore>().subscribe();
    let hub = app.state::<HubHandle>().inner().clone();
    let regions = app.state::<HitRegions>().inner().clone();
    follow_position_setting(app.state::<SettingsStore>().subscribe());
    click_through::spawn(window, regions, shutdown.clone());
    zorder::spawn();
    fullscreen::spawn(hub, settings, shutdown);
}

fn follow_position_setting(mut settings: tokio::sync::watch::Receiver<crate::core::settings::Settings>) {
    tauri::async_runtime::spawn(async move {
        while settings.changed().await.is_ok() {
            let position = settings.borrow_and_update().general.position;
            placement::set_position(native::island(), position);
        }
    });
}
