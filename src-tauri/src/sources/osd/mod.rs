mod brightness;
mod device_change;
pub mod endpoint;
mod view;
mod volume_keys;

use std::time::Duration;

use crossbeam_channel::{select, unbounded, Receiver, Sender};

use self::brightness::{spawn_brightness_watcher, BrightnessControl};
use self::device_change::DefaultDeviceWatch;
use self::endpoint::{AudioEvent, VolumeEndpoint};
use self::volume_keys::{key_receiver, VolumeKey, VolumeKeyHook};
use crate::core::activity::ActionRequest;
use crate::core::settings::Settings;
use crate::core::source::{ActivitySource, SourceContext};

const IDLE_TICK: Duration = Duration::from_secs(5);
const VOLUME_STEP: f32 = 0.02;

#[derive(Default)]
pub struct OsdSource;

impl ActivitySource for OsdSource {
    fn name(&self) -> &'static str {
        "osd"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let (audio_events, audio_inbox) = unbounded();
        let (brightness_events, brightness_inbox) = unbounded();
        let key_inbox = key_receiver();
        let mut osd = Osd::new(ctx.clone(), audio_events, brightness_events);
        osd.apply_settings(&ctx.current_settings());
        while ctx.is_running() {
            if !osd.pump(&ctx, &audio_inbox, &brightness_inbox, &key_inbox) {
                break;
            }
            if let Some(settings) = ctx.take_changed_settings() {
                osd.apply_settings(&settings);
            }
        }
    }
}

struct Osd {
    ctx: SourceContext,
    audio_events: Sender<AudioEvent>,
    brightness_events: Sender<u8>,
    volume: Option<VolumeEndpoint>,
    _device_watch: Option<DefaultDeviceWatch>,
    brightness: Option<BrightnessControl>,
    brightness_watching: bool,
    key_hook: VolumeKeyHook,
    replace_volume: bool,
    show_brightness: bool,
}

impl Osd {
    fn new(ctx: SourceContext, audio_events: Sender<AudioEvent>, brightness_events: Sender<u8>) -> Self {
        let device_watch = DefaultDeviceWatch::start(audio_events.clone())
            .inspect_err(|error| log::warn!("default audio device watch failed: {error}"))
            .ok();
        Self {
            ctx,
            audio_events,
            brightness_events,
            volume: None,
            _device_watch: device_watch,
            brightness: None,
            brightness_watching: false,
            key_hook: VolumeKeyHook::install(),
            replace_volume: false,
            show_brightness: false,
        }
    }

    fn pump(
        &mut self,
        ctx: &SourceContext,
        audio_inbox: &Receiver<AudioEvent>,
        brightness_inbox: &Receiver<u8>,
        key_inbox: &Receiver<VolumeKey>,
    ) -> bool {
        select! {
            recv(ctx.actions) -> request => match request {
                Ok(request) => self.handle_action(&request),
                Err(_) => return false,
            },
            recv(ctx.commands) -> command => match command {
                Ok(command) => command.respond_err("command not supported"),
                Err(_) => return false,
            },
            recv(audio_inbox) -> event => if let Ok(event) = event { self.handle_audio(event) },
            recv(brightness_inbox) -> percent => if let Ok(percent) = percent { self.handle_brightness(percent) },
            recv(key_inbox) -> key => if let Ok(key) = key { self.handle_key(key) },
            default(IDLE_TICK) => self.ensure_volume_endpoint(),
        }
        true
    }

    fn apply_settings(&mut self, settings: &Settings) {
        self.replace_volume = settings.osd.replace_volume;
        self.show_brightness = settings.osd.brightness;
        self.ensure_volume_endpoint();
        self.sync_key_swallowing();
        self.ensure_brightness_watcher();
    }

    fn ensure_volume_endpoint(&mut self) {
        if self.volume.is_none() {
            self.attach_volume_endpoint();
        }
    }

    fn attach_volume_endpoint(&mut self) {
        self.volume = None;
        self.volume = VolumeEndpoint::attach(self.audio_events.clone())
            .inspect_err(|error| log::debug!("volume endpoint unavailable: {error}"))
            .ok();
        self.sync_key_swallowing();
    }

    fn sync_key_swallowing(&self) {
        self.key_hook.set_swallowing(self.replace_volume && self.volume.is_some());
    }

    fn ensure_brightness_watcher(&mut self) {
        if self.show_brightness && !self.brightness_watching {
            self.brightness_watching = true;
            spawn_brightness_watcher(self.brightness_events.clone());
        }
    }

    fn handle_audio(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::VolumeChanged(level) => self.ctx.upsert(view::volume_activity(&self.ctx, level)),
            AudioEvent::DefaultDeviceChanged => self.attach_volume_endpoint(),
        }
    }

    fn handle_brightness(&mut self, percent: u8) {
        if self.show_brightness {
            self.ctx.upsert(view::brightness_activity(&self.ctx, percent));
        }
    }

    fn handle_key(&mut self, key: VolumeKey) {
        let Some(endpoint) = &self.volume else { return };
        let Ok(level) = endpoint.level() else { return };
        let result = match key {
            VolumeKey::Up => step_volume(endpoint, level, VOLUME_STEP),
            VolumeKey::Down => step_volume(endpoint, level, -VOLUME_STEP),
            VolumeKey::Mute => endpoint.set_muted(!level.muted),
        };
        match result.and_then(|()| endpoint.level()) {
            Ok(updated) => self.ctx.upsert(view::volume_activity(&self.ctx, updated)),
            Err(error) => log::warn!("volume key handling failed: {error}"),
        }
    }

    fn handle_action(&mut self, request: &ActionRequest) {
        let outcome = match request.action.as_str() {
            view::VOLUME_ACTION => self.set_volume_percent(request),
            view::MUTE_ACTION => self.set_mute(request),
            view::BRIGHTNESS_ACTION => self.set_brightness_percent(request),
            other => Err(format!("unknown action {other}")),
        };
        if let Err(message) = outcome {
            log::warn!("osd action {} failed: {message}", request.action);
        }
    }

    fn set_volume_percent(&self, request: &ActionRequest) -> Result<(), String> {
        let percent = requested_number(request).ok_or("missing value")?;
        let endpoint = self.volume.as_ref().ok_or("no volume endpoint")?;
        endpoint.set_scalar((percent / 100.0) as f32).map_err(|error| error.to_string())
    }

    fn set_mute(&self, request: &ActionRequest) -> Result<(), String> {
        let endpoint = self.volume.as_ref().ok_or("no volume endpoint")?;
        let muted = match request.value.as_ref().and_then(serde_json::Value::as_bool) {
            Some(muted) => muted,
            None => !endpoint.level().map_err(|error| error.to_string())?.muted,
        };
        endpoint.set_muted(muted).map_err(|error| error.to_string())
    }

    fn set_brightness_percent(&mut self, request: &ActionRequest) -> Result<(), String> {
        let percent = requested_number(request).ok_or("missing value")?;
        if self.brightness.is_none() {
            self.brightness = Some(BrightnessControl::connect().map_err(|error| error.to_string())?);
        }
        let control = self.brightness.as_ref().ok_or("no brightness control")?;
        control.set_percent(percent.round().clamp(0.0, 100.0) as u8).map_err(|error| error.to_string())
    }
}

fn step_volume(endpoint: &VolumeEndpoint, level: endpoint::VolumeLevel, delta: f32) -> windows_core::Result<()> {
    let stepped = ((level.scalar + delta).clamp(0.0, 1.0) * 100.0).round() / 100.0;
    if level.muted {
        endpoint.set_muted(false)?;
    }
    endpoint.set_scalar(stepped)
}

fn requested_number(request: &ActionRequest) -> Option<f64> {
    request.value.as_ref().and_then(serde_json::Value::as_f64)
}
