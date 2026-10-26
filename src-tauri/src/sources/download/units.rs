const KIBIBYTE: f64 = 1024.0;
const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

pub fn format_bytes(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit_index = 0;
    while value >= KIBIBYTE && unit_index < UNITS.len() - 1 {
        value /= KIBIBYTE;
        unit_index += 1;
    }
    if unit_index == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

pub fn format_rate(bytes_per_second: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_second.max(0.0) as u64))
}
