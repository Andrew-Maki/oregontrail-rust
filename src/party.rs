//! The travellers themselves: health, sickness and death.

use crate::rng::Rng;

/// Everything that can lay a pioneer low.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ailment {
    Cholera,
    Typhoid,
    Dysentery,
    Measles,
    Fever,
    Exhaustion,
    BrokenArm,
    BrokenLeg,
    BrokenRibs,
    SnakeBite,
    Concussion,
    Frostbite,
}

impl Ailment {
    pub fn name(self) -> &'static str {
        match self {
            Ailment::Cholera => "cholera",
            Ailment::Typhoid => "typhoid fever",
            Ailment::Dysentery => "dysentery",
            Ailment::Measles => "the measles",
            Ailment::Fever => "a fever",
            Ailment::Exhaustion => "exhaustion",
            Ailment::BrokenArm => "a broken arm",
            Ailment::BrokenLeg => "a broken leg",
            Ailment::BrokenRibs => "broken ribs",
            Ailment::SnakeBite => "a snakebite",
            Ailment::Concussion => "a bad concussion",
            Ailment::Frostbite => "severe frostbite",
        }
    }

    /// Health lost per day while suffering from this.
    pub fn daily_damage(self) -> i32 {
        match self {
            Ailment::Cholera => 14,
            Ailment::Typhoid => 11,
            Ailment::Dysentery => 9,
            Ailment::Measles => 8,
            Ailment::Fever => 6,
            Ailment::Exhaustion => 5,
            Ailment::SnakeBite => 10,
            Ailment::Concussion => 7,
            Ailment::Frostbite => 6,
            Ailment::BrokenLeg => 4,
            Ailment::BrokenArm => 3,
            Ailment::BrokenRibs => 4,
        }
    }

    /// Typical number of days before recovery becomes likely.
    pub fn duration(self, rng: &mut Rng) -> u32 {
        match self {
            Ailment::Cholera => rng.range(4, 9) as u32,
            Ailment::Typhoid => rng.range(6, 13) as u32,
            Ailment::Dysentery => rng.range(5, 12) as u32,
            Ailment::Measles => rng.range(6, 12) as u32,
            Ailment::Fever => rng.range(3, 8) as u32,
            Ailment::Exhaustion => rng.range(2, 5) as u32,
            Ailment::SnakeBite => rng.range(3, 7) as u32,
            Ailment::Concussion => rng.range(3, 8) as u32,
            Ailment::Frostbite => rng.range(4, 10) as u32,
            Ailment::BrokenArm => rng.range(14, 26) as u32,
            Ailment::BrokenLeg => rng.range(20, 36) as u32,
            Ailment::BrokenRibs => rng.range(12, 24) as u32,
        }
    }

    /// An injury rather than a disease -- it slows the wagon down but is
    /// much less likely to kill.
    pub fn is_injury(self) -> bool {
        matches!(
            self,
            Ailment::BrokenArm
                | Ailment::BrokenLeg
                | Ailment::BrokenRibs
                | Ailment::Concussion
                | Ailment::Frostbite
        )
    }

    /// Diseases that can be caught from bad water / bad camps.
    pub const DISEASES: [Ailment; 5] = [
        Ailment::Cholera,
        Ailment::Typhoid,
        Ailment::Dysentery,
        Ailment::Measles,
        Ailment::Fever,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HealthRating {
    Good,
    Fair,
    Poor,
    VeryPoor,
}

impl HealthRating {
    pub fn name(self) -> &'static str {
        match self {
            HealthRating::Good => "good",
            HealthRating::Fair => "fair",
            HealthRating::Poor => "poor",
            HealthRating::VeryPoor => "very poor",
        }
    }

    pub fn from_health(h: i32) -> HealthRating {
        if h >= 78 {
            HealthRating::Good
        } else if h >= 55 {
            HealthRating::Fair
        } else if h >= 30 {
            HealthRating::Poor
        } else {
            HealthRating::VeryPoor
        }
    }

    /// Points awarded per surviving member at journey's end.
    pub fn score_points(self) -> i64 {
        match self {
            HealthRating::Good => 500,
            HealthRating::Fair => 400,
            HealthRating::Poor => 300,
            HealthRating::VeryPoor => 200,
        }
    }

    /// Multiplier applied to the distance travelled each day.
    pub fn travel_factor(self) -> f64 {
        match self {
            HealthRating::Good => 1.0,
            HealthRating::Fair => 0.9,
            HealthRating::Poor => 0.75,
            HealthRating::VeryPoor => 0.55,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Person {
    pub name: String,
    /// 0 = dead, 100 = perfect health.
    pub health: i32,
    pub alive: bool,
    pub ailment: Option<Ailment>,
    pub ailment_days: u32,
    pub cause_of_death: Option<String>,
    pub is_leader: bool,
}

impl Person {
    pub fn new(name: &str, is_leader: bool) -> Self {
        Person {
            name: name.to_string(),
            health: 100,
            alive: true,
            ailment: None,
            ailment_days: 0,
            cause_of_death: None,
            is_leader,
        }
    }

    pub fn rating(&self) -> HealthRating {
        HealthRating::from_health(self.health)
    }

    pub fn adjust(&mut self, delta: i32) {
        if !self.alive {
            return;
        }
        self.health = (self.health + delta).min(100);
    }

    pub fn status_line(&self) -> String {
        if !self.alive {
            let cause = self
                .cause_of_death
                .clone()
                .unwrap_or_else(|| "unknown causes".to_string());
            return format!("{:<16} dead ({})", self.name, cause);
        }
        match self.ailment {
            Some(a) => format!(
                "{:<16} {:<10} -- has {}",
                self.name,
                self.rating().name(),
                a.name()
            ),
            None => format!("{:<16} {}", self.name, self.rating().name()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Party {
    pub members: Vec<Person>,
}

impl Party {
    pub fn new(names: &[String]) -> Self {
        let members = names
            .iter()
            .enumerate()
            .map(|(i, n)| Person::new(n, i == 0))
            .collect();
        Party { members }
    }

    pub fn leader_name(&self) -> String {
        self.members
            .iter()
            .find(|p| p.is_leader)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "the wagon captain".to_string())
    }

    pub fn alive_count(&self) -> usize {
        self.members.iter().filter(|p| p.alive).count()
    }

    pub fn living_indices(&self) -> Vec<usize> {
        self.members
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive)
            .map(|(i, _)| i)
            .collect()
    }

    /// Living members who are not already sick or hurt.
    pub fn healthy_indices(&self) -> Vec<usize> {
        self.members
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive && p.ailment.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn sick_indices(&self) -> Vec<usize> {
        self.members
            .iter()
            .enumerate()
            .filter(|(_, p)| p.alive && p.ailment.is_some())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn random_living(&self, rng: &mut Rng) -> Option<usize> {
        let living = self.living_indices();
        if living.is_empty() {
            None
        } else {
            Some(living[rng.below(living.len() as u32) as usize])
        }
    }

    /// Average health across the living, used for the party's overall rating.
    pub fn average_health(&self) -> i32 {
        let living: Vec<i32> = self
            .members
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.health)
            .collect();
        if living.is_empty() {
            return 0;
        }
        living.iter().sum::<i32>() / living.len() as i32
    }

    pub fn rating(&self) -> HealthRating {
        HealthRating::from_health(self.average_health())
    }

    pub fn adjust_all(&mut self, delta: i32) {
        for p in self.members.iter_mut() {
            p.adjust(delta);
        }
    }

    pub fn everyone_dead(&self) -> bool {
        self.alive_count() == 0
    }
}
