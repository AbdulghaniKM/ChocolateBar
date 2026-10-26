pub fn is_storm(code: u32) -> bool {
    (95..=99).contains(&code)
}

pub fn is_rain(code: u32) -> bool {
    (61..=67).contains(&code) || (80..=82).contains(&code)
}

pub fn icon_for(code: u32, is_day: bool) -> &'static str {
    match code {
        0 if is_day => "\u{2600}\u{FE0F}",
        0 => "\u{1F319}",
        1 if is_day => "\u{1F324}\u{FE0F}",
        1 => "\u{1F319}",
        2 => "\u{26C5}",
        3 => "\u{2601}\u{FE0F}",
        45 | 48 => "\u{1F32B}\u{FE0F}",
        51..=57 => "\u{1F326}\u{FE0F}",
        61..=67 | 80..=82 => "\u{1F327}\u{FE0F}",
        71..=77 | 85 | 86 => "\u{1F328}\u{FE0F}",
        95..=99 => "\u{26C8}\u{FE0F}",
        _ => "\u{1F321}\u{FE0F}",
    }
}

pub fn description_for(code: u32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51..=57 => "Drizzle",
        61..=65 | 80..=82 => "Rain",
        66 | 67 => "Freezing rain",
        71..=77 | 85 | 86 => "Snow",
        95..=99 => "Thunderstorm",
        _ => "Unsettled",
    }
}
