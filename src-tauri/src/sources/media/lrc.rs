use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
}

pub fn parse_lrc(source: &str) -> Vec<LyricLine> {
    let mut lines: Vec<LyricLine> = source.lines().flat_map(parse_lrc_line).collect();
    lines.sort_by_key(|line| line.time_ms);
    lines
}

fn parse_lrc_line(line: &str) -> Vec<LyricLine> {
    let (timestamps, text) = split_timestamps(line);
    timestamps
        .into_iter()
        .map(|time_ms| LyricLine { time_ms, text: text.to_string() })
        .collect()
}

fn split_timestamps(line: &str) -> (Vec<u64>, &str) {
    let mut rest = line.trim_start();
    let mut timestamps = Vec::new();
    while let Some(after_bracket) = rest.strip_prefix('[') {
        let Some((tag, remainder)) = after_bracket.split_once(']') else { break };
        let Some(time_ms) = parse_timestamp(tag) else { break };
        timestamps.push(time_ms);
        rest = remainder.trim_start();
    }
    (timestamps, rest.trim())
}

fn parse_timestamp(tag: &str) -> Option<u64> {
    let (minutes, seconds) = tag.split_once(':')?;
    let minutes: u64 = minutes.trim().parse().ok()?;
    let seconds: f64 = seconds.trim().parse().ok()?;
    (seconds.is_finite() && seconds >= 0.0).then(|| minutes * 60_000 + (seconds * 1000.0).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_timestamps_and_skips_metadata_tags() {
        let lines = parse_lrc("[ar:Someone]\n[00:12.50][01:00.00] Hello\n[00:05.00]Intro");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], LyricLine { time_ms: 5_000, text: "Intro".into() });
        assert_eq!(lines[1], LyricLine { time_ms: 12_500, text: "Hello".into() });
        assert_eq!(lines[2].time_ms, 60_000);
    }

    #[test]
    fn keeps_empty_lines_as_instrumental_gaps() {
        let lines = parse_lrc("[00:01.00]\n[00:02.00]Words");
        assert_eq!(lines[0].text, "");
        assert_eq!(lines[1].text, "Words");
    }
}
