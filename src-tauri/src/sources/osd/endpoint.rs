use crossbeam_channel::Sender;
use windows::core::implement;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::Win32::Media::Audio::{
    eConsole, eRender, EDataFlow, ERole, IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows_core::Result;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VolumeLevel {
    pub scalar: f32,
    pub muted: bool,
}

impl VolumeLevel {
    pub fn percent(self) -> u8 {
        (self.scalar.clamp(0.0, 1.0) * 100.0).round() as u8
    }
}

pub enum AudioEvent {
    VolumeChanged(VolumeLevel),
    DefaultDeviceChanged,
}

#[implement(IAudioEndpointVolumeCallback)]
struct VolumeCallback {
    events: Sender<AudioEvent>,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, notification: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<()> {
        if let Some(data) = unsafe { notification.as_ref() } {
            let level = VolumeLevel { scalar: data.fMasterVolume, muted: data.bMuted.as_bool() };
            let _ = self.events.send(AudioEvent::VolumeChanged(level));
        }
        Ok(())
    }
}

pub fn default_endpoint_volume(flow: EDataFlow, role: ERole) -> Result<IAudioEndpointVolume> {
    let enumerator: IMMDeviceEnumerator = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
    let device = unsafe { enumerator.GetDefaultAudioEndpoint(flow, role)? };
    unsafe { device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) }
}

pub struct VolumeEndpoint {
    endpoint: IAudioEndpointVolume,
    callback: IAudioEndpointVolumeCallback,
}

impl VolumeEndpoint {
    pub fn attach(events: Sender<AudioEvent>) -> Result<Self> {
        let endpoint = default_endpoint_volume(eRender, eConsole)?;
        let callback: IAudioEndpointVolumeCallback = VolumeCallback { events }.into();
        unsafe { endpoint.RegisterControlChangeNotify(&callback)? };
        Ok(Self { endpoint, callback })
    }

    pub fn level(&self) -> Result<VolumeLevel> {
        let scalar = unsafe { self.endpoint.GetMasterVolumeLevelScalar()? };
        let muted = unsafe { self.endpoint.GetMute()? }.as_bool();
        Ok(VolumeLevel { scalar, muted })
    }

    pub fn set_scalar(&self, scalar: f32) -> Result<()> {
        unsafe { self.endpoint.SetMasterVolumeLevelScalar(scalar.clamp(0.0, 1.0), std::ptr::null()) }
    }

    pub fn set_muted(&self, muted: bool) -> Result<()> {
        unsafe { self.endpoint.SetMute(muted, std::ptr::null()) }
    }
}

impl Drop for VolumeEndpoint {
    fn drop(&mut self) {
        let _ = unsafe { self.endpoint.UnregisterControlChangeNotify(&self.callback) };
    }
}
