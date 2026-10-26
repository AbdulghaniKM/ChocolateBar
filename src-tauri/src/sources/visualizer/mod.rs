mod capture;
mod ring;
mod spectrum;

use std::time::{Duration, Instant};

use self::capture::LoopbackCapture;
use self::spectrum::{SpectrumAnalyzer, BAND_COUNT, FFT_SIZE};
use super::media::playback_flag;
use crate::core::source::{ActivitySource, SourceContext, SourceEvent};

const FRAME_INTERVAL: Duration = Duration::from_millis(33);
const REOPEN_DELAY: Duration = Duration::from_secs(3);
const SILENT_FRAME: [f32; BAND_COUNT] = [0.0; BAND_COUNT];

#[derive(Default)]
pub struct VisualizerSource;

impl ActivitySource for VisualizerSource {
    fn name(&self) -> &'static str {
        "visualizer"
    }

    fn start(self: Box<Self>, mut ctx: SourceContext) {
        let mut pipeline = Pipeline::default();
        let mut enabled = ctx.current_settings().media.visualizer;
        while ctx.is_running() {
            match ctx.next_event(FRAME_INTERVAL) {
                SourceEvent::Shutdown => break,
                SourceEvent::Command(command) => command.respond_err("command not supported"),
                SourceEvent::Action(_) | SourceEvent::Idle => {}
            }
            if let Some(settings) = ctx.take_changed_settings() {
                enabled = settings.media.visualizer;
            }
            pipeline.tick(&ctx, enabled && playback_flag::is_playing());
        }
        pipeline.stop(&ctx);
    }
}

struct Active {
    capture: LoopbackCapture,
    analyzer: SpectrumAnalyzer,
}

#[derive(Default)]
struct Pipeline {
    active: Option<Active>,
    retry_at: Option<Instant>,
}

impl Pipeline {
    fn tick(&mut self, ctx: &SourceContext, wanted: bool) {
        if !wanted {
            self.stop(ctx);
            return;
        }
        if self.active.as_ref().is_some_and(|active| active.capture.has_failed()) {
            self.active = None;
        }
        if self.active.is_none() {
            self.try_start();
        }
        if let Some(active) = &mut self.active {
            ctx.emit(&next_frame(active));
        }
    }

    fn stop(&mut self, ctx: &SourceContext) {
        self.retry_at = None;
        if self.active.take().is_some() {
            ctx.emit(&SILENT_FRAME);
        }
    }

    fn try_start(&mut self) {
        if self.retry_at.is_some_and(|moment| Instant::now() < moment) {
            return;
        }
        match LoopbackCapture::open() {
            Ok(capture) => {
                let analyzer = SpectrumAnalyzer::new(capture.sample_rate());
                self.active = Some(Active { capture, analyzer });
                self.retry_at = None;
            }
            Err(error) => {
                log::warn!("visualizer capture unavailable: {error}");
                self.retry_at = Some(Instant::now() + REOPEN_DELAY);
            }
        }
    }
}

fn next_frame(active: &mut Active) -> Vec<f32> {
    let levels = match active.capture.take_fresh_window(FFT_SIZE) {
        Some(window) => active.analyzer.analyze(&window),
        None => active.analyzer.decay(),
    };
    levels.iter().map(|level| (level * 100.0).round() / 100.0).collect()
}
