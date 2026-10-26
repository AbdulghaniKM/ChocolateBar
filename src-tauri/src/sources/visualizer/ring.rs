use std::collections::VecDeque;

const RING_CAPACITY: usize = 8192;

#[derive(Default)]
pub struct MonoRing {
    samples: VecDeque<f32>,
    total_pushed: u64,
}

impl MonoRing {
    pub fn push_downmixed(&mut self, interleaved: &[f32], channels: usize) {
        let channels = channels.max(1);
        for frame in interleaved.chunks_exact(channels) {
            self.push(frame.iter().sum::<f32>() / channels as f32);
        }
    }

    pub fn total_pushed(&self) -> u64 {
        self.total_pushed
    }

    pub fn latest(&self, count: usize) -> Vec<f32> {
        let missing = count.saturating_sub(self.samples.len());
        let available = count - missing;
        let skip = self.samples.len() - available;
        let mut window = vec![0.0; missing];
        window.extend(self.samples.iter().skip(skip));
        window
    }

    fn push(&mut self, sample: f32) {
        if self.samples.len() == RING_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
        self.total_pushed += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downmixes_stereo_frames_to_their_average() {
        let mut ring = MonoRing::default();
        ring.push_downmixed(&[1.0, 0.0, 0.5, 0.5], 2);
        assert_eq!(ring.latest(2), vec![0.5, 0.5]);
    }

    #[test]
    fn pads_the_front_with_silence_when_short() {
        let mut ring = MonoRing::default();
        ring.push_downmixed(&[0.25], 1);
        assert_eq!(ring.latest(3), vec![0.0, 0.0, 0.25]);
        assert_eq!(ring.total_pushed(), 1);
    }
}
