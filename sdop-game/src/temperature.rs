pub enum TemperatureLevel {
    VeryHot,
    Hot,
    Pleasant,
    Cold,
    VeryCold,
}

impl TemperatureLevel {
    pub const fn is_hot(&self) -> bool {
        matches!(self, TemperatureLevel::VeryHot) || matches!(self, TemperatureLevel::Hot)
    }

    pub const fn is_cold(&self) -> bool {
        matches!(self, TemperatureLevel::Cold) || matches!(self, TemperatureLevel::VeryCold)
    }
}

impl From<f32> for TemperatureLevel {
    fn from(value: f32) -> Self {
        if value > 35. {
            Self::VeryHot
        } else if value > 27. {
            Self::Hot
        } else if value > 10. {
            Self::Pleasant
        } else if value > 0. {
            Self::Cold
        } else {
            Self::VeryCold
        }
    }
}

impl TemperatureLevel {}
