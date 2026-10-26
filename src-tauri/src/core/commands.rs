use std::time::Duration;

use serde_json::Value;
use tauri::{AppHandle, State};
use tokio::sync::oneshot;

use super::activity::{ActionRequest, Activity, CommandReply, SourceCommand};
use super::autostart;
use super::error::{AppError, AppResult};
use super::hub::HubHandle;
use super::settings::{Settings, SettingsStore};
use crate::window::click_through::{HitRect, HitRegions};
use crate::window::settings_window;

const SOURCE_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

#[tauri::command]
pub async fn list_activities(hub: State<'_, HubHandle>) -> AppResult<Vec<Activity>> {
    Ok(hub.list())
}

#[tauri::command]
pub fn activity_action(hub: State<'_, HubHandle>, id: String, action: String, value: Option<Value>) {
    hub.dispatch_action(ActionRequest { activity_id: id, action, value });
}

#[tauri::command]
pub async fn source_command(
    hub: State<'_, HubHandle>,
    source: String,
    command: String,
    payload: Option<Value>,
) -> AppResult<Value> {
    let (reply, receiver) = oneshot::channel();
    let payload = payload.unwrap_or(Value::Null);
    hub.dispatch_command(SourceCommand { source, command, payload, reply });
    await_reply(receiver).await
}

#[tauri::command]
pub fn cycle_activity(hub: State<'_, HubHandle>, direction: i32) {
    hub.cycle(direction);
}

#[tauri::command]
pub fn set_hit_regions(regions: State<'_, HitRegions>, rects: Vec<HitRect>) {
    regions.replace(rects);
}

#[tauri::command]
pub fn set_expanded(hub: State<'_, HubHandle>, expanded: bool) {
    hub.set_expanded(expanded);
}

#[tauri::command]
pub fn get_settings(store: State<'_, SettingsStore>) -> Settings {
    store.current()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    patch: Value,
) -> AppResult<Settings> {
    let updated = store.apply_patch(&app, patch)?;
    autostart::sync_with_settings(&app, &updated)?;
    Ok(updated)
}

#[tauri::command]
pub async fn open_settings(app: AppHandle) -> AppResult<()> {
    settings_window::open(&app)
}

async fn await_reply(receiver: oneshot::Receiver<CommandReply>) -> AppResult<Value> {
    match tokio::time::timeout(SOURCE_COMMAND_TIMEOUT, receiver).await {
        Ok(Ok(Ok(value))) => Ok(value),
        Ok(Ok(Err(message))) => Err(AppError::Source(message)),
        Ok(Err(_)) => Err(AppError::Source("source dropped the command".into())),
        Err(_) => Err(AppError::TimedOut("source command")),
    }
}
