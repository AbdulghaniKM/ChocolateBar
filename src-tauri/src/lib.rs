pub mod core;
pub mod sources;
pub mod window;

use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::core::error::AppResult;
use crate::core::settings::SettingsStore;
use crate::core::source::Shutdown;
use crate::core::{autostart, commands, hub, tray, updater};
use crate::window::click_through::HitRegions;

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::settings_window::open_in_background(app);
        }))
        .plugin(tauri_plugin_log::Builder::new().level(log::LevelFilter::Info).build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| Ok(setup_app(app.handle())?))
        .invoke_handler(tauri::generate_handler![
            commands::list_activities,
            commands::activity_action,
            commands::source_command,
            commands::cycle_activity,
            commands::set_hit_regions,
            commands::set_island_size,
            commands::set_expanded,
            commands::get_settings,
            commands::update_settings,
            commands::open_settings,
        ])
        .build(tauri::generate_context!())
        .expect("failed to build ChocolateBar");
    app.run(stop_sources_on_exit);
}

fn setup_app(app: &AppHandle) -> AppResult<()> {
    let store = SettingsStore::load(app)?;
    if let Err(error) = autostart::sync_with_settings(app, &store.current()) {
        log::warn!("autostart sync failed: {error}");
    }
    app.manage(Shutdown::default());
    app.manage(HitRegions::default());
    app.manage(store);
    app.manage(hub::spawn(app.clone()));
    tray::setup(app)?;
    window::setup(app)?;
    sources::spawn_all(app);
    updater::spawn(app);
    Ok(())
}

fn stop_sources_on_exit(app: &AppHandle, event: RunEvent) {
    if matches!(event, RunEvent::Exit) {
        app.state::<Shutdown>().trigger();
    }
}
