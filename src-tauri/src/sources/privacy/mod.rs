mod consent_store;
mod mic_mute;
mod registry;
mod view;

use std::time::Duration;

use crossbeam_channel::{select, unbounded};

use self::consent_store::{apps_in_use, Capability};
use self::mic_mute::MicMuteHotkey;
use crate::core::activity::ActionRequest;
use crate::core::source::{ActivitySource, SourceContext};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Default)]
pub struct PrivacySource;

impl ActivitySource for PrivacySource {
    fn name(&self) -> &'static str {
        "privacy"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let (hotkey_pressed, hotkey_inbox) = unbounded();
        let mut hotkey = MicMuteHotkey::new(ctx.app.clone(), hotkey_pressed);
        hotkey.bind(&ctx.current_settings().hotkeys.mic_mute);
        let mut tracker = UsageTracker::default();
        while ctx.is_running() {
            select! {
                recv(ctx.actions) -> request => match request {
                    Ok(request) => handle_action(&request),
                    Err(_) => break,
                },
                recv(ctx.commands) -> command => match command {
                    Ok(command) => command.respond_err("command not supported"),
                    Err(_) => break,
                },
                recv(hotkey_inbox) -> _ => toggle_mute(),
                default(POLL_INTERVAL) => {}
            }
            if let Some(settings) = ctx.take_changed_settings() {
                hotkey.bind(&settings.hotkeys.mic_mute);
            }
            tracker.refresh(&ctx);
        }
        tracker.withdraw_all(&ctx);
    }
}

#[derive(Default)]
struct UsageTracker {
    mic: Option<(Vec<String>, bool)>,
    camera: Option<Vec<String>>,
    muted: Option<bool>,
}

impl UsageTracker {
    fn refresh(&mut self, ctx: &SourceContext) {
        let muted = mic_mute::is_muted();
        self.announce_mute_change(ctx, muted);
        self.refresh_mic(ctx, apps_in_use(Capability::Microphone), muted.unwrap_or(false));
        self.refresh_camera(ctx, apps_in_use(Capability::Camera));
    }

    fn withdraw_all(&mut self, ctx: &SourceContext) {
        ctx.remove(view::MIC_KEY);
        ctx.remove(view::CAMERA_KEY);
        ctx.remove(view::MUTE_KEY);
    }

    fn announce_mute_change(&mut self, ctx: &SourceContext, muted: Option<bool>) {
        if let (Some(previous), Some(current)) = (self.muted, muted) {
            if previous != current {
                ctx.upsert(view::mute_notice_activity(ctx, current));
            }
        }
        self.muted = muted;
    }

    fn refresh_mic(&mut self, ctx: &SourceContext, apps: Vec<String>, muted: bool) {
        let state = (!apps.is_empty()).then_some((apps, muted));
        if state == self.mic {
            return;
        }
        match &state {
            Some((apps, muted)) => ctx.upsert(view::mic_activity(ctx, apps, *muted)),
            None => ctx.remove(view::MIC_KEY),
        }
        self.mic = state;
    }

    fn refresh_camera(&mut self, ctx: &SourceContext, apps: Vec<String>) {
        let state = (!apps.is_empty()).then_some(apps);
        if state == self.camera {
            return;
        }
        match &state {
            Some(apps) => ctx.upsert(view::camera_activity(ctx, apps)),
            None => ctx.remove(view::CAMERA_KEY),
        }
        self.camera = state;
    }
}

fn handle_action(request: &ActionRequest) {
    if request.action != view::MUTE_ACTION {
        log::debug!("privacy ignored action {}", request.action);
        return;
    }
    match request.value.as_ref().and_then(serde_json::Value::as_bool) {
        Some(muted) => apply_mute(muted),
        None => toggle_mute(),
    }
}

fn apply_mute(muted: bool) {
    if let Err(error) = mic_mute::set_muted(muted) {
        log::warn!("microphone mute failed: {error}");
    }
}

fn toggle_mute() {
    if let Err(error) = mic_mute::toggle() {
        log::warn!("microphone mute toggle failed: {error}");
    }
}
