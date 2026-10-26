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
    styles::apply(hwnd);
    placement::place(hwnd);
    subclass::install(hwnd)?;
    window.set_ignore_cursor_events(true)?;
    replace_on_scale_change(&window);
    start_watchers(app, window);
    styles::show_without_activation(hwnd);
    Ok(())
}

pub fn apply_visibility(app: &AppHandle, visible: bool) {
    let result = app.run_on_main_thread(move || {
        let hwnd = native::island();
        if visible {
            styles::show_without_activation(hwnd);
        } else {
            styles::hide(hwnd);
        }
    });
    if let Err(error) = result {
        log::warn!("visibility change failed: {error}");
    }
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
    click_through::spawn(window, regions, shutdown.clone());
    zorder::spawn();
    fullscreen::spawn(hub, settings, shutdown);
}
