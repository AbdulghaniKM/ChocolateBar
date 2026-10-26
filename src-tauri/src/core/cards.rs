use super::activity::ActivityKind;

pub const CLOCK: &str = "clock";
pub const DEFAULT_ORDER: [&str; 17] = [
    "network", "media", "claude", "timer", "weather", "battery", "bluetooth", "buds", "download", "game",
    "sports", "privacy", "osd", "keyboard", "notification", "ocr", "shelf",
];

/// The user's choice of what the island shows when nothing is happening, and the order cards are flipped through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPrefs {
    pub default_card: String,
    pub order: Vec<String>,
}

impl Default for CardPrefs {
    fn default() -> Self {
        Self { default_card: CLOCK.into(), order: DEFAULT_ORDER.iter().map(|kind| (*kind).to_string()).collect() }
    }
}

impl CardPrefs {
    pub fn shows_clock_when_idle(&self) -> bool {
        self.default_card == CLOCK
    }

    pub fn is_default(&self, kind: ActivityKind) -> bool {
        self.default_card == kind_name(kind)
    }

    /// Earlier in the user's order scores higher; kinds left out of the list come last.
    pub fn order_score(&self, kind: ActivityKind) -> usize {
        let name = kind_name(kind);
        self.order.iter().position(|entry| *entry == name).map_or(0, |index| self.order.len() - index)
    }
}

pub fn kind_name(kind: ActivityKind) -> String {
    serde_json::to_value(kind).ok().and_then(|value| value.as_str().map(String::from)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earlier_kinds_score_higher_and_missing_ones_score_zero() {
        let prefs = CardPrefs { default_card: CLOCK.into(), order: vec!["network".into(), "media".into()] };
        assert!(prefs.order_score(ActivityKind::Network) > prefs.order_score(ActivityKind::Media));
        assert_eq!(prefs.order_score(ActivityKind::Weather), 0);
    }

    #[test]
    fn default_card_matches_by_kind_name() {
        let prefs = CardPrefs { default_card: "network".into(), order: Vec::new() };
        assert!(prefs.is_default(ActivityKind::Network));
        assert!(!prefs.shows_clock_when_idle());
        assert!(CardPrefs::default().shows_clock_when_idle());
    }
}
