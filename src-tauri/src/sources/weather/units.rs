#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Units {
    Metric,
    Imperial,
}

impl Units {
    pub fn parse(name: &str) -> Self {
        if name.eq_ignore_ascii_case("imperial") {
            Self::Imperial
        } else {
            Self::Metric
        }
    }

    pub fn temperature(self, celsius: f64) -> i32 {
        match self {
            Self::Metric => celsius.round() as i32,
            Self::Imperial => (celsius * 9.0 / 5.0 + 32.0).round() as i32,
        }
    }

    pub fn temperature_unit(self) -> &'static str {
        match self {
            Self::Metric => "\u{00B0}C",
            Self::Imperial => "\u{00B0}F",
        }
    }

    pub fn speed(self, kmh: f64) -> i32 {
        match self {
            Self::Metric => kmh.round() as i32,
            Self::Imperial => (kmh * 0.621_371).round() as i32,
        }
    }

    pub fn speed_unit(self) -> &'static str {
        match self {
            Self::Metric => "km/h",
            Self::Imperial => "mph",
        }
    }
}
