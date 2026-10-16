//! Weather, which on the Oregon Trail is mostly a way of killing you.

use crate::rng::Rng;
use crate::trail::Terrain;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Temp {
    VeryHot,
    Hot,
    Warm,
    Cool,
    Cold,
    VeryCold,
}

impl Temp {
    pub fn name(self) -> &'static str {
        match self {
            Temp::VeryHot => "very hot",
            Temp::Hot => "hot",
            Temp::Warm => "warm",
            Temp::Cool => "cool",
            Temp::Cold => "cold",
            Temp::VeryCold => "very cold",
        }
    }

    fn from_degrees(f: i32) -> Temp {
        if f >= 92 {
            Temp::VeryHot
        } else if f >= 78 {
            Temp::Hot
        } else if f >= 62 {
            Temp::Warm
        } else if f >= 46 {
            Temp::Cool
        } else if f >= 30 {
            Temp::Cold
        } else {
            Temp::VeryCold
        }
    }

    /// True when the party needs warm clothing to stay healthy.
    pub fn needs_clothing(self) -> bool {
        matches!(self, Temp::Cold | Temp::VeryCold)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    Clear,
    Rain,
    HeavyRain,
    Snow,
    HeavySnow,
    Fog,
    Wind,
    Hail,
}

impl Cond {
    pub fn name(self) -> &'static str {
        match self {
            Cond::Clear => "clear",
            Cond::Rain => "rainy",
            Cond::HeavyRain => "very rainy",
            Cond::Snow => "snowy",
            Cond::HeavySnow => "heavy snow",
            Cond::Fog => "foggy",
            Cond::Wind => "windy",
            Cond::Hail => "hailing",
        }
    }

    pub fn travel_factor(self) -> f64 {
        match self {
            Cond::Clear => 1.0,
            Cond::Wind => 0.95,
            Cond::Rain => 0.88,
            Cond::HeavyRain => 0.7,
            Cond::Fog => 0.72,
            Cond::Snow => 0.72,
            Cond::HeavySnow => 0.45,
            Cond::Hail => 0.65,
        }
    }

    /// Extra feet of water in the rivers.
    pub fn river_rise(self) -> f64 {
        match self {
            Cond::HeavyRain => 1.6,
            Cond::Rain => 0.7,
            Cond::HeavySnow => 0.4,
            Cond::Snow => 0.2,
            Cond::Hail => 0.5,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Weather {
    pub degrees: i32,
    pub temp: Temp,
    pub cond: Cond,
}

impl Weather {
    pub fn describe(&self) -> String {
        if self.cond == Cond::Clear {
            format!("{} ({} F)", self.temp.name(), self.degrees)
        } else {
            format!(
                "{} and {} ({} F)",
                self.temp.name(),
                self.cond.name(),
                self.degrees
            )
        }
    }

    pub fn travel_factor(&self) -> f64 {
        let t = match self.temp {
            Temp::VeryHot => 0.85,
            Temp::Hot => 0.95,
            Temp::Warm => 1.0,
            Temp::Cool => 1.0,
            Temp::Cold => 0.92,
            Temp::VeryCold => 0.78,
        };
        t * self.cond.travel_factor()
    }

    /// Health drain per day from exposure, before clothing is considered.
    pub fn exposure(&self) -> i32 {
        let mut d = match self.temp {
            Temp::VeryHot => 3,
            Temp::Hot => 1,
            Temp::Warm => 0,
            Temp::Cool => 0,
            Temp::Cold => 2,
            Temp::VeryCold => 5,
        };
        d += match self.cond {
            Cond::HeavyRain | Cond::HeavySnow => 3,
            Cond::Rain | Cond::Snow | Cond::Hail => 1,
            _ => 0,
        };
        d
    }
}

/// Average plains temperature for each month, in degrees Fahrenheit.
fn monthly_base(month: u32) -> i32 {
    match month {
        1 => 26,
        2 => 31,
        3 => 43,
        4 => 56,
        5 => 66,
        6 => 76,
        7 => 83,
        8 => 81,
        9 => 70,
        10 => 56,
        11 => 40,
        12 => 29,
        _ => 60,
    }
}

pub fn generate(month: u32, terrain: Terrain, rng: &mut Rng) -> Weather {
    let degrees = monthly_base(month) + terrain.temp_offset() + rng.range_centered(-16, 16);
    let temp = Temp::from_degrees(degrees);

    // Wet months and mountains get more weather.
    let mut wet_chance = match month {
        4..=6 => 0.34,
        3 | 7 | 10 | 11 => 0.27,
        _ => 0.22,
    };
    if terrain == Terrain::Mountains {
        wet_chance += 0.12;
    }
    if terrain == Terrain::Desert {
        wet_chance -= 0.14;
    }

    let cond = if rng.chance(wet_chance) {
        let heavy = rng.chance(0.3);
        if degrees <= 34 {
            if heavy {
                Cond::HeavySnow
            } else {
                Cond::Snow
            }
        } else if heavy {
            Cond::HeavyRain
        } else {
            Cond::Rain
        }
    } else if rng.chance(0.05) {
        Cond::Hail
    } else if rng.chance(0.08) {
        Cond::Fog
    } else if rng.chance(0.12) {
        Cond::Wind
    } else {
        Cond::Clear
    };

    Weather {
        degrees,
        temp,
        cond,
    }
}
