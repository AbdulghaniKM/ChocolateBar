use crossbeam_channel::Sender;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use windows::Win32::Media::Audio::{eCapture, eCommunications, eConsole, ERole};
use windows_core::Result;

use crate::sources::osd::endpoint::default_endpoint_volume;

const CAPTURE_ROLES: [ERole; 2] = [eConsole, eCommunications];

pub fn is_muted() -> Option<bool> {
    let endpoint = default_endpoint_volume(eCapture, eConsole).ok()?;
    unsafe { endpoint.GetMute() }.ok().map(|muted| muted.as_bool())
}

pub fn set_muted(muted: bool) -> Result<()> {
    let mut outcome = Ok(());
    for role in CAPTURE_ROLES {
        let result = default_endpoint_volume(eCapture, role)
            .and_then(|endpoint| unsafe { endpoint.SetMute(muted, std::ptr::null()) });
        if result.is_err() {
            outcome = result;
        }
    }
    outcome
}

pub fn toggle() -> Result<bool> {
    let muted = !is_muted().unwrap_or(false);
    set_muted(muted)?;
    Ok(muted)
}

pub struct MicMuteHotkey {
    app: AppHandle,
    pressed: Sender<()>,
    bound: Option<String>,
}

impl MicMuteHotkey {
    pub fn new(app: AppHandle, pressed: Sender<()>) -> Self {
        Self { app, pressed, bound: None }
    }

    pub fn bind(&mut self, accelerator: &str) {
        if self.bound.as_deref() == Some(accelerator) {
            return;
        }
        self.unbind();
        if accelerator.trim().is_empty() {
            return;
        }
        let pressed = self.pressed.clone();
        let registered = self.app.global_shortcut().on_shortcut(accelerator, move |_, _, event| {
            if event.state == ShortcutState::Pressed {
                let _ = pressed.send(());
            }
        });
        match registered {
            Ok(()) => self.bound = Some(accelerator.to_string()),
            Err(error) => log::warn!("mic mute hotkey {accelerator} not registered: {error}"),
        }
    }

    fn unbind(&mut self) {
        if let Some(previous) = self.bound.take() {
            if let Err(error) = self.app.global_shortcut().unregister(previous.as_str()) {
                log::debug!("mic mute hotkey {previous} not unregistered: {error}");
            }
        }
    }
}

impl Drop for MicMuteHotkey {
    fn drop(&mut self) {
        self.unbind();
    }
}
