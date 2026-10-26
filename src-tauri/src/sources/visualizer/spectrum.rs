use std::sync::Arc;

use rustfft::num_complex::Complex;
use rustfft::{Fft, FftPlanner};

pub const FFT_SIZE: usize = 1024;
pub const BAND_COUNT: usize = 12;

const LOWEST_HZ: f32 = 50.0;
const HIGHEST_HZ: f32 = 16_000.0;
const FLOOR_DB: f32 = -78.0;
const CEILING_DB: f32 = -18.0;
const TILT_DB_PER_BAND: f32 = 1.4;
const FALL_PER_FRAME: f32 = 0.12;

pub struct SpectrumAnalyzer {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    band_bins: Vec<(usize, usize)>,
    levels: [f32; BAND_COUNT],
}

impl SpectrumAnalyzer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            fft: FftPlanner::new().plan_fft_forward(FFT_SIZE),
            window: hann_window(),
            band_bins: band_bins(sample_rate as f32),
            levels: [0.0; BAND_COUNT],
        }
    }

    pub fn analyze(&mut self, samples: &[f32]) -> [f32; BAND_COUNT] {
        let magnitudes = self.magnitudes(samples);
        for (band, &(first, last)) in self.band_bins.iter().enumerate() {
            let peak = magnitudes[first..=last].iter().copied().fold(0.0, f32::max);
            self.levels[band] = fall_towards(self.levels[band], normalize_db(peak, band));
        }
        self.levels
    }

    pub fn decay(&mut self) -> [f32; BAND_COUNT] {
        for level in &mut self.levels {
            *level = fall_towards(*level, 0.0);
        }
        self.levels
    }

    fn magnitudes(&self, samples: &[f32]) -> Vec<f32> {
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .zip(&self.window)
            .map(|(sample, weight)| Complex::new(sample * weight, 0.0))
            .collect();
        buffer.resize(FFT_SIZE, Complex::new(0.0, 0.0));
        self.fft.process(&mut buffer);
        buffer[..FFT_SIZE / 2].iter().map(|bin| bin.norm() * 4.0 / FFT_SIZE as f32).collect()
    }
}

fn fall_towards(current: f32, target: f32) -> f32 {
    if target >= current {
        target
    } else {
        (current - FALL_PER_FRAME).max(target)
    }
}

fn normalize_db(magnitude: f32, band: usize) -> f32 {
    let decibels = 20.0 * magnitude.max(1e-9).log10() + band as f32 * TILT_DB_PER_BAND;
    ((decibels - FLOOR_DB) / (CEILING_DB - FLOOR_DB)).clamp(0.0, 1.0)
}

fn hann_window() -> Vec<f32> {
    let last = (FFT_SIZE - 1) as f32;
    (0..FFT_SIZE)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / last).cos())
        .collect()
}

fn band_bins(sample_rate: f32) -> Vec<(usize, usize)> {
    let highest = HIGHEST_HZ.min(sample_rate / 2.0 * 0.95);
    let ratio = (highest / LOWEST_HZ).powf(1.0 / BAND_COUNT as f32);
    let bin_of = |hz: f32| ((hz * FFT_SIZE as f32 / sample_rate) as usize).clamp(1, FFT_SIZE / 2 - 1);
    (0..BAND_COUNT)
        .map(|band| {
            let first = bin_of(LOWEST_HZ * ratio.powi(band as i32));
            let last = bin_of(LOWEST_HZ * ratio.powi(band as i32 + 1));
            (first, last.max(first))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency: f32, sample_rate: f32) -> Vec<f32> {
        (0..FFT_SIZE)
            .map(|index| (std::f32::consts::TAU * frequency * index as f32 / sample_rate).sin() * 0.8)
            .collect()
    }

    #[test]
    fn low_tone_lights_low_bands_more_than_high_bands() {
        let mut analyzer = SpectrumAnalyzer::new(48_000);
        let levels = analyzer.analyze(&sine(120.0, 48_000.0));
        assert!(levels[1] > levels[10]);
    }

    #[test]
    fn silence_decays_to_zero() {
        let mut analyzer = SpectrumAnalyzer::new(48_000);
        analyzer.analyze(&sine(440.0, 48_000.0));
        let levels = (0..20).map(|_| analyzer.decay()).last().unwrap_or_default();
        assert!(levels.iter().all(|level| *level == 0.0));
    }
}
