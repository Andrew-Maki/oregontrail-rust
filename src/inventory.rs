//! Wagon contents, prices and the player's purse.

use crate::ui;

// Prices at Matt's General Store in Independence, in cents.
pub const OXEN_YOKE_PRICE: i64 = 4000; // a yoke is two oxen
pub const FOOD_PRICE: i64 = 20; // per pound
pub const CLOTHING_PRICE: i64 = 1000; // per set
pub const AMMO_BOX_PRICE: i64 = 200; // per box of 20 bullets
pub const PART_PRICE: i64 = 1000; // wheel, axle or tongue

pub const BULLETS_PER_BOX: u32 = 20;
pub const OXEN_PER_YOKE: u32 = 2;

/// Beyond this the wagon is overloaded and the oxen struggle. A real
/// emigrant wagon carried about a ton and a quarter.
pub const WAGON_CAPACITY_LBS: u32 = 2400;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Wheel,
    Axle,
    Tongue,
}

impl Part {
    pub fn name(self) -> &'static str {
        match self {
            Part::Wheel => "wagon wheel",
            Part::Axle => "wagon axle",
            Part::Tongue => "wagon tongue",
        }
    }

    pub fn weight(self) -> u32 {
        match self {
            Part::Wheel => 55,
            Part::Axle => 60,
            Part::Tongue => 45,
        }
    }

    pub const ALL: [Part; 3] = [Part::Wheel, Part::Axle, Part::Tongue];
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub cash: i64,
    pub oxen: u32,
    pub food: u32,
    pub clothing: u32,
    pub bullets: u32,
    pub wheels: u32,
    pub axles: u32,
    pub tongues: u32,
}

impl Inventory {
    pub fn new(cash: i64) -> Self {
        Inventory {
            cash,
            oxen: 0,
            food: 0,
            clothing: 0,
            bullets: 0,
            wheels: 0,
            axles: 0,
            tongues: 0,
        }
    }

    pub fn spares(&self, part: Part) -> u32 {
        match part {
            Part::Wheel => self.wheels,
            Part::Axle => self.axles,
            Part::Tongue => self.tongues,
        }
    }

    pub fn spares_mut(&mut self, part: Part) -> &mut u32 {
        match part {
            Part::Wheel => &mut self.wheels,
            Part::Axle => &mut self.axles,
            Part::Tongue => &mut self.tongues,
        }
    }

    pub fn total_spares(&self) -> u32 {
        self.wheels + self.axles + self.tongues
    }

    /// Load carried in the wagon bed, in pounds.
    pub fn weight(&self) -> u32 {
        self.food
            + self.clothing * 10
            + self.bullets / 10
            + self.wheels * Part::Wheel.weight()
            + self.axles * Part::Axle.weight()
            + self.tongues * Part::Tongue.weight()
            + 200 // the wagon's own fixtures, water barrel, tools
    }

    pub fn is_overloaded(&self) -> bool {
        self.weight() > WAGON_CAPACITY_LBS
    }

    /// 1.0 when within capacity, dropping off as the wagon gets heavier.
    pub fn load_factor(&self) -> f64 {
        let w = self.weight();
        if w <= WAGON_CAPACITY_LBS {
            1.0
        } else {
            let over = (w - WAGON_CAPACITY_LBS) as f64;
            (1.0 - over / 2500.0).max(0.55)
        }
    }

    /// Remove up to `lbs` of food; returns how much was actually lost.
    pub fn take_food(&mut self, lbs: u32) -> u32 {
        let taken = lbs.min(self.food);
        self.food -= taken;
        taken
    }

    pub fn take_bullets(&mut self, n: u32) -> u32 {
        let taken = n.min(self.bullets);
        self.bullets -= taken;
        taken
    }

    pub fn take_clothing(&mut self, n: u32) -> u32 {
        let taken = n.min(self.clothing);
        self.clothing -= taken;
        taken
    }

    pub fn take_oxen(&mut self, n: u32) -> u32 {
        let taken = n.min(self.oxen);
        self.oxen -= taken;
        taken
    }

    pub fn can_afford(&self, cents: i64) -> bool {
        self.cash >= cents
    }

    pub fn print(&self, party_size: usize) {
        ui::heading("Your wagon contains");
        println!("   {:<22}{}", "Oxen", self.oxen);
        println!(
            "   {:<22}{} pounds",
            "Food",
            ui::group(i64::from(self.food))
        );
        println!("   {:<22}{} sets", "Clothing", self.clothing);
        println!(
            "   {:<22}{} bullets",
            "Ammunition",
            ui::group(i64::from(self.bullets))
        );
        println!("   {:<22}{}", "Spare wagon wheels", self.wheels);
        println!("   {:<22}{}", "Spare wagon axles", self.axles);
        println!("   {:<22}{}", "Spare wagon tongues", self.tongues);
        println!("   {:<22}{}", "Money left", ui::money(self.cash));
        ui::blank();
        println!(
            "   Load: {} of {} lbs{}",
            ui::group(i64::from(self.weight())),
            ui::group(i64::from(WAGON_CAPACITY_LBS)),
            if self.is_overloaded() {
                "   *** OVERLOADED ***"
            } else {
                ""
            }
        );
        if party_size > 0 && self.clothing < party_size as u32 {
            println!(
                "   Warning: only {} for {} travellers -- cold weather will hurt.",
                ui::plural(self.clothing, "set of clothing", "sets of clothing"),
                party_size
            );
        }
    }
}
