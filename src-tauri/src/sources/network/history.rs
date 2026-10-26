use std::collections::VecDeque;

use super::throughput::Throughput;

const HISTORY_LENGTH: usize = 30;

#[derive(Default)]
pub struct SpeedHistory {
    download: VecDeque<u64>,
    upload: VecDeque<u64>,
}

impl SpeedHistory {
    pub fn push(&mut self, throughput: Throughput) {
        push_bounded(&mut self.download, throughput.download_bytes_per_second);
        push_bounded(&mut self.upload, throughput.upload_bytes_per_second);
    }

    pub fn download_samples(&self) -> Vec<u64> {
        self.download.iter().copied().collect()
    }

    pub fn upload_samples(&self) -> Vec<u64> {
        self.upload.iter().copied().collect()
    }
}

fn push_bounded(samples: &mut VecDeque<u64>, bytes_per_second: f64) {
    if samples.len() == HISTORY_LENGTH {
        samples.pop_front();
    }
    samples.push_back(bytes_per_second.max(0.0) as u64);
}
