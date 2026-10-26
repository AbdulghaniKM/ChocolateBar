use std::time::Duration;

const SECONDS_PER_HOUR: u64 = 3600;

pub fn clock_text(seconds: u64) -> String {
    let (hours, minutes, seconds) =
        (seconds / SECONDS_PER_HOUR, seconds % SECONDS_PER_HOUR / 60, seconds % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

pub fn whole_seconds_rounded_up(duration: Duration) -> u64 {
    duration.as_millis().div_ceil(1000) as u64
}
