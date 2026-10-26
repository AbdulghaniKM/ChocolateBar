use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, InputCallbackInfo, SampleFormat, SizedSample, Stream, StreamConfig};

use super::ring::MonoRing;
use crate::core::error::{AppError, AppResult};

type SharedRing = Arc<Mutex<MonoRing>>;

pub struct LoopbackCapture {
    _stream: Stream,
    ring: SharedRing,
    failed: Arc<AtomicBool>,
    sample_rate: u32,
    last_seen_total: u64,
}

impl LoopbackCapture {
    pub fn open() -> AppResult<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AppError::Missing("default output device"))?;
        let supported = device.default_output_config().map_err(source_error)?;
        let format = supported.sample_format();
        let sample_rate = supported.sample_rate();
        let config: StreamConfig = supported.into();
        let ring = SharedRing::default();
        let failed = Arc::new(AtomicBool::new(false));
        let stream = build_stream(&device, &config, format, &ring, &failed)?;
        stream.play().map_err(source_error)?;
        Ok(Self { _stream: stream, ring, failed, sample_rate, last_seen_total: 0 })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn has_failed(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }

    pub fn take_fresh_window(&mut self, length: usize) -> Option<Vec<f32>> {
        let ring = self.ring.lock().ok()?;
        let total = ring.total_pushed();
        if total == self.last_seen_total {
            return None;
        }
        self.last_seen_total = total;
        Some(ring.latest(length))
    }
}

fn build_stream(
    device: &Device,
    config: &StreamConfig,
    format: SampleFormat,
    ring: &SharedRing,
    failed: &Arc<AtomicBool>,
) -> AppResult<Stream> {
    match format {
        SampleFormat::F32 => build_typed_stream::<f32>(device, config, ring, failed),
        SampleFormat::I16 => build_typed_stream::<i16>(device, config, ring, failed),
        SampleFormat::I32 => build_typed_stream::<i32>(device, config, ring, failed),
        other => Err(AppError::Source(format!("unsupported loopback sample format {other}"))),
    }
}

fn build_typed_stream<T>(
    device: &Device,
    config: &StreamConfig,
    ring: &SharedRing,
    failed: &Arc<AtomicBool>,
) -> AppResult<Stream>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let channels = config.channels as usize;
    let sink = Arc::clone(ring);
    let failure_flag = Arc::clone(failed);
    let on_data = move |data: &[T], _: &InputCallbackInfo| push_converted(&sink, data, channels);
    let on_error = move |error: cpal::Error| {
        log::warn!("visualizer loopback stream error: {error}");
        failure_flag.store(true, Ordering::Relaxed);
    };
    device.build_input_stream::<T, _, _>(*config, on_data, on_error, None).map_err(source_error)
}

fn push_converted<T>(ring: &SharedRing, data: &[T], channels: usize)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let floats: Vec<f32> = data.iter().map(|sample| sample.to_sample::<f32>()).collect();
    if let Ok(mut ring) = ring.lock() {
        ring.push_downmixed(&floats, channels);
    }
}

fn source_error(error: cpal::Error) -> AppError {
    AppError::Source(error.to_string())
}
