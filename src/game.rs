//! The game state machine: setup, the day-by-day travel loop, camp menus
//! and the end of the journey.

use crate::date::Date;
use crate::events;
use crate::hunt;
use crate::inventory::{Inventory, Part};
use crate::party::{Ailment, Party};
use crate::river;
use crate::rng::Rng;
use crate::score;
use crate::store;
use crate::trail::{self, Fork, Kind, Landmark, Terrain};
use crate::ui;
use crate::weather::{self, Weather};

pub const PARTY_SIZE: usize = 5;

// --------------------------------------------------------------- choices ---

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Occupation {
    Banker,
    Carpenter,
    Farmer,
}

impl Occupation {
    pub fn name(self) -> &'static str {
        match self {
            Occupation::Banker => "banker from Boston",
            Occupation::Carpenter => "carpenter from Ohio",
            Occupation::Farmer => "farmer from Illinois",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Occupation::Banker => "banker",
            Occupation::Carpenter => "carpenter",
            Occupation::Farmer => "farmer",
        }
    }

    pub fn starting_cash(self) -> i64 {
        match self {
            Occupation::Banker => 160_000,
            Occupation::Carpenter => 80_000,
            Occupation::Farmer => 40_000,
        }
    }

    pub fn score_multiplier(self) -> i64 {
        match self {
            Occupation::Banker => 1,
            Occupation::Carpenter => 2,
            Occupation::Farmer => 3,
        }
    }

    /// Chance of repairing a broken wagon part without a spare.
    pub fn repair_skill(self) -> f64 {
        match self {
            Occupation::Banker => 0.25,
            Occupation::Carpenter => 0.75,
            Occupation::Farmer => 0.45,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pace {
    Steady,
    Strenuous,
    Grueling,
}

impl Pace {
    pub fn name(self) -> &'static str {
        match self {
            Pace::Steady => "steady",
            Pace::Strenuous => "strenuous",
            Pace::Grueling => "grueling",
        }
    }

    pub fn factor(self) -> f64 {
        match self {
            Pace::Steady => 1.0,
            Pace::Strenuous => 1.25,
            Pace::Grueling => 1.5,
        }
    }

    pub fn health_cost(self) -> i32 {
        match self {
            Pace::Steady => 0,
            Pace::Strenuous => 2,
            Pace::Grueling => 5,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rations {
    Filling,
    Meager,
    BareBones,
}

impl Rations {
    pub fn name(self) -> &'static str {
        match self {
            Rations::Filling => "filling",
            Rations::Meager => "meager",
            Rations::BareBones => "bare bones",
        }
    }

    /// Pounds of food per person per day.
    pub fn lbs_per_person(self) -> u32 {
        match self {
            Rations::Filling => 3,
            Rations::Meager => 2,
            Rations::BareBones => 1,
        }
    }

    pub fn health_effect(self) -> i32 {
        match self {
            Rations::Filling => 2,
            Rations::Meager => 0,
            Rations::BareBones => -3,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Playing,
    Arrived,
    Died,
    TurnedBack,
    Quit,
}

#[derive(Clone, Debug)]
pub struct Grave {
    pub name: String,
    pub cause: String,
    pub date: Date,
    pub miles: u32,
}

// ------------------------------------------------------------------ game ---

pub struct Game {
    pub rng: Rng,
    pub date: Date,
    pub occupation: Occupation,
    pub party: Party,
    pub inv: Inventory,
    pub pace: Pace,
    pub rations: Rations,
    pub weather: Weather,

    pub route: Vec<Landmark>,
    /// Index in `route` of the landmark we are travelling toward.
    pub leg: usize,
    pub miles_total: u32,
    pub miles_into_leg: u32,
    pub days_on_trail: u32,

    /// Days remaining of poor grazing, which slows the team.
    pub ox_fatigue: u32,
    /// Consecutive days the party has gone hungry.
    pub starving_days: u32,
    /// Days remaining of being stuck without oxen.
    pub stranded_days: u32,
    pub graves: Vec<Grave>,
    pub outcome: Outcome,
    pub death_cause: Option<String>,
    pub route_extended_at: Vec<usize>,
}

impl Game {
    // ------------------------------------------------------------ setup ---

    pub fn new_from_setup() -> Option<Game> {
        let mut rng = Rng::from_entropy();

        ui::banner("The Oregon Trail");
        ui::wrap(
            "It is 1848. Thousands of emigrants are gathering in Independence, \
             Missouri, to make the two-thousand-mile journey to Oregon's \
             Willamette Valley.",
        );
        ui::blank();
        ui::wrap("Before you leave, you must decide who you are.");

        let occupation = Self::choose_occupation()?;
        let month = Self::choose_month(occupation)?;
        let names = Self::choose_names();

        let party = Party::new(&names);
        let inv = Inventory::new(occupation.starting_cash());
        let date = Date::new(1848, month, 1);
        let route = trail::opening_route();
        let w = weather::generate(month, Terrain::Plains, &mut rng);

        Some(Game {
            rng,
            date,
            occupation,
            party,
            inv,
            pace: Pace::Steady,
            rations: Rations::Filling,
            weather: w,
            route,
            leg: 1,
            miles_total: 0,
            miles_into_leg: 0,
            days_on_trail: 0,
            ox_fatigue: 0,
            starving_days: 0,
            stranded_days: 0,
            graves: Vec::new(),
            outcome: Outcome::Playing,
            death_cause: None,
            route_extended_at: Vec::new(),
        })
    }

    fn choose_occupation() -> Option<Occupation> {
        loop {
            let choice = ui::menu(
                "Many kinds of people made the trip to Oregon. You may be:",
                &[
                    "a banker from Boston",
                    "a carpenter from Ohio",
                    "a farmer from Illinois",
                    "Find out the differences between these choices",
                    "Go back to the main menu",
                ],
            );
            match choice {
                1 => return Some(Occupation::Banker),
                2 => return Some(Occupation::Carpenter),
                3 => return Some(Occupation::Farmer),
                4 => {
                    ui::heading("Choosing an occupation");
                    ui::wrap(
                        "Deciding your occupation is important because it \
                         determines how much money you begin with, and how many \
                         points you receive at the end of the game.",
                    );
                    ui::blank();
                    ui::wrap(
                        "A BANKER starts with $1,600 -- plenty to buy whatever \
                         he likes -- but earns the fewest points. He is also \
                         hopeless at mending a wagon.",
                    );
                    ui::blank();
                    ui::wrap(
                        "A CARPENTER starts with $800 and earns double points. \
                         He can repair a broken wagon part even without a spare.",
                    );
                    ui::blank();
                    ui::wrap(
                        "A FARMER starts with only $400 and earns triple points. \
                         He knows how to keep oxen alive, which counts for a \
                         great deal.",
                    );
                    ui::pause();
                }
                _ => return None,
            }
        }
    }

    fn choose_month(occupation: Occupation) -> Option<u32> {
        ui::heading(&format!(
            "You are a {}. You have {} in cash.",
            occupation.name(),
            ui::money(occupation.starting_cash())
        ));
        ui::wrap(
            "It is time to leave Independence. The month you choose matters a \
             great deal: leave too early and there is no grass for the oxen; \
             leave too late and you will meet snow in the mountains.",
        );
        loop {
            let choice = ui::menu(
                "When do you want to set out?",
                &[
                    "March",
                    "April",
                    "May",
                    "June",
                    "July",
                    "Ask for advice",
                    "Go back to the main menu",
                ],
            );
            match choice {
                1 => return Some(3),
                2 => return Some(4),
                3 => return Some(5),
                4 => return Some(6),
                5 => return Some(7),
                6 => {
                    ui::heading("Advice from an old hand");
                    ui::wrap(
                        "\"March is too early. The grass is still brown and your \
                         oxen will starve on the prairie.\"",
                    );
                    ui::blank();
                    ui::wrap(
                        "\"April and May are the proper months. You'll have grass \
                         for the team, high water at the river crossings, and \
                         time enough to beat the snow.\"",
                    );
                    ui::blank();
                    ui::wrap(
                        "\"June will serve, if you push along. July is a gamble -- \
                         the rivers are low and easy, but the Blue Mountains will \
                         be white before you reach them.\"",
                    );
                    ui::pause();
                }
                _ => return None,
            }
        }
    }

    fn choose_names() -> Vec<String> {
        ui::heading("Your party");
        ui::wrap(
            "You are travelling with four other people. Your own name goes \
             first -- if you die, the journey is over.",
        );
        ui::blank();
        let mut names: Vec<String> = Vec::with_capacity(PARTY_SIZE);
        names.push(ui::prompt_name("What is your name? "));
        for i in 2..=PARTY_SIZE {
            let n = ui::prompt_name(&format!("Name of traveller #{}: ", i));
            names.push(n);
        }
        ui::blank();
        println!("Your party:");
        for n in &names {
            println!("   {}", n);
        }
        ui::blank();
        if !ui::yes_no("Are these names correct?") {
            return Self::choose_names();
        }
        names
    }

    // ------------------------------------------------------- quick facts ---

    pub fn terrain(&self) -> Terrain {
        self.route
            .get(self.leg)
            .map(|l| l.terrain)
            .unwrap_or(Terrain::Plains)
    }

    pub fn next_landmark_name(&self) -> &'static str {
        self.route.get(self.leg).map(|l| l.name).unwrap_or("Oregon")
    }

    pub fn last_landmark_name(&self) -> &'static str {
        if self.leg == 0 {
            return self.route[0].name;
        }
        self.route[self.leg - 1].name
    }

    pub fn miles_to_next(&self) -> u32 {
        self.route
            .get(self.leg)
            .map(|l| l.miles_from_prev.saturating_sub(self.miles_into_leg))
            .unwrap_or(0)
    }

    pub fn daily_food_need(&self) -> u32 {
        self.rations.lbs_per_person() * self.party.alive_count() as u32
    }

    pub fn is_over(&self) -> bool {
        self.outcome != Outcome::Playing
    }

    // ------------------------------------------------------- main driver ---

    pub fn run(&mut self) {
        ui::banner("Independence, Missouri");
        ui::wrap(self.route[0].blurb);
        ui::pause();

        store::outfit_at_independence(&mut self.inv, PARTY_SIZE);

        ui::banner(&format!("Setting out -- {}", self.date));
        ui::wrap(&format!(
            "On {} your wagon rolls west out of Independence. The next landmark \
             is {}, {} miles away.",
            self.date.long_form(),
            self.next_landmark_name(),
            ui::group(i64::from(self.miles_to_next()))
        ));
        ui::pause();

        while !self.is_over() {
            self.camp_menu(true);
            if self.is_over() {
                break;
            }
            self.travel_to_next_landmark();
        }

        self.finish();
    }

    // ------------------------------------------------------------ travel ---

    fn travel_to_next_landmark(&mut self) {
        let mut days_since_checkpoint = 0u32;

        ui::blank();
        println!(
            "Travelling toward {} ({} miles).",
            self.next_landmark_name(),
            ui::group(i64::from(self.miles_to_next()))
        );
        ui::rule();

        loop {
            if self.is_over() {
                return;
            }

            // No oxen means no movement.
            if self.inv.oxen == 0 {
                self.handle_stranded();
                if self.is_over() || self.inv.oxen == 0 {
                    return;
                }
            }

            let interrupted = self.advance_one_day();
            days_since_checkpoint += 1;

            if self.is_over() {
                return;
            }

            // Arrived?
            if self.miles_into_leg >= self.route[self.leg].miles_from_prev {
                self.arrive_at_landmark();
                return;
            }

            if interrupted {
                ui::pause();
                self.camp_menu(false);
                days_since_checkpoint = 0;
            } else if days_since_checkpoint >= 10 {
                ui::blank();
                if ui::yes_no("Stop to look over the wagon?") {
                    self.camp_menu(false);
                }
                days_since_checkpoint = 0;
                ui::rule();
            }
        }
    }

    /// One day on the trail. Returns true if something happened that should
    /// make the player stop and think.
    fn advance_one_day(&mut self) -> bool {
        self.date.next_day();
        self.days_on_trail += 1;
        self.weather = weather::generate(self.date.month, self.terrain(), &mut self.rng);

        // Distance covered.
        let miles = self.compute_daily_miles();
        self.miles_into_leg += miles;
        self.miles_total += miles;
        if self.ox_fatigue > 0 {
            self.ox_fatigue -= 1;
        }

        let starving = self.consume_food();
        self.apply_daily_health(false);
        self.apply_ailments(false);
        let newly_sick = self.check_new_illness();
        self.reap();

        if self.is_over() {
            return true;
        }

        // Daily one-line report.
        println!(
            "{:<18} {:>4} mi today  {:>5} mi total  food {:>5} lbs  health {}",
            format!("{} {}", self.date.month_name(), self.date.day),
            miles,
            ui::group(i64::from(self.miles_total)),
            ui::group(i64::from(self.inv.food)),
            self.party.rating().name()
        );

        // Don't nag every single day of a long famine.
        let starve_alert =
            starving && (self.starving_days == 1 || self.starving_days.is_multiple_of(3));
        let mut interrupted = starve_alert || newly_sick;

        if starving {
            println!(
                "   !! There is no food left. Your party is starving ({}).",
                ui::plural(self.starving_days, "day", "days")
            );
            if starve_alert && self.inv.oxen >= 2 {
                println!("      You could slaughter an ox for meat.");
            }
        }

        // Random trail event.
        if events::roll_daily_event(self) {
            interrupted = true;
        }
        self.reap();
        if self.is_over() {
            return true;
        }

        if self.party.average_health() < 25 {
            println!("   !! Your party is in very poor health. Consider resting.");
            interrupted = true;
        }

        interrupted
    }

    fn compute_daily_miles(&mut self) -> u32 {
        let ox_ratio = f64::from(self.inv.oxen) / 8.0;
        let ox_factor = ox_ratio.clamp(0.30, 1.20);
        let fatigue = if self.ox_fatigue > 0 { 0.75 } else { 1.0 };

        let mut miles = 20.0
            * self.pace.factor()
            * ox_factor
            * fatigue
            * self.party.rating().travel_factor()
            * self.terrain().travel_factor()
            * self.weather.travel_factor()
            * self.inv.load_factor();

        // Injured travellers slow everybody down.
        let injured = self
            .party
            .members
            .iter()
            .filter(|p| p.alive && p.ailment.map(|a| a.is_injury()).unwrap_or(false))
            .count();
        if injured > 0 {
            miles *= 1.0 - 0.12 * injured as f64;
        }

        miles += self.rng.range_f(-2.0, 2.0);
        miles.max(1.0).round() as u32
    }

    /// Eat. Returns true if the party went hungry.
    fn consume_food(&mut self) -> bool {
        let need = self.daily_food_need();
        if need == 0 {
            return false;
        }
        if self.inv.food >= need {
            self.inv.food -= need;
            self.starving_days = 0;
            false
        } else {
            let short = need - self.inv.food;
            self.inv.food = 0;
            self.starving_days += 1;
            // Scale by how badly each person is short, not by party size.
            let alive = self.party.alive_count().max(1) as u32;
            let per_person = (short / alive) as i32;
            let penalty = 3 + 2 * per_person.min(4);
            self.party.adjust_all(-penalty);
            true
        }
    }

    fn apply_daily_health(&mut self, resting: bool) {
        let mut delta = self.rations.health_effect();

        if resting {
            delta += 6;
        } else {
            delta -= self.pace.health_cost();
        }

        let mut exposure = self.weather.exposure();
        if self.weather.temp.needs_clothing() {
            let needed = self.party.alive_count() as u32;
            if self.inv.clothing < needed {
                exposure += 2 + (needed - self.inv.clothing) as i32;
            }
        }
        delta -= exposure;

        if self.terrain() == Terrain::Mountains && !resting {
            delta -= 1;
        }
        if self.inv.is_overloaded() && !resting {
            delta -= 1;
        }

        self.party.adjust_all(delta);
    }

    fn apply_ailments(&mut self, resting: bool) {
        let mut recovered: Vec<String> = Vec::new();
        for p in self.party.members.iter_mut() {
            if !p.alive {
                continue;
            }
            let Some(ailment) = p.ailment else { continue };
            let mut dmg = ailment.daily_damage();
            if resting {
                dmg = (dmg + 1) / 2;
            }
            p.health -= dmg;
            if p.ailment_days > 0 {
                p.ailment_days -= if resting { 2 } else { 1 };
            }
            if p.ailment_days == 0 && p.health > 0 {
                p.ailment = None;
                p.health = (p.health + 14).min(100);
                recovered.push(p.name.clone());
            }
        }
        for name in recovered {
            println!("   {} has recovered.", name);
        }
    }

    /// Roll for new sickness. Returns true if somebody fell ill.
    fn check_new_illness(&mut self) -> bool {
        let candidates = self.party.healthy_indices();
        let mut any = false;
        for i in candidates {
            let health = self.party.members[i].health;
            let mut p = 0.004;
            if health < 60 {
                p += 0.010;
            }
            if health < 35 {
                p += 0.018;
            }
            if self.rations == Rations::BareBones {
                p += 0.009;
            }
            if self.pace == Pace::Grueling {
                p += 0.007;
            }
            if self.weather.exposure() >= 4 {
                p += 0.006;
            }
            if self.rng.chance(p) {
                let ailment = if health < 45 && self.rng.chance(0.4) {
                    Ailment::Exhaustion
                } else {
                    *self.rng.pick(&Ailment::DISEASES)
                };
                self.strike(i, ailment);
                any = true;
            }
        }
        any
    }

    // ------------------------------------------------- health / mortality ---

    /// Inflict an ailment on a party member.
    pub fn strike(&mut self, idx: usize, ailment: Ailment) {
        let days = ailment.duration(&mut self.rng);
        let p = &mut self.party.members[idx];
        if !p.alive {
            return;
        }
        // A second illness on top of the first just makes it worse.
        if p.ailment.is_some() {
            p.ailment_days += days / 2;
            p.health -= 5;
            println!("   {} is getting worse.", p.name);
            return;
        }
        p.ailment = Some(ailment);
        p.ailment_days = days;
        println!("   {} has {}.", p.name, ailment.name());
    }

    /// Kill a party member outright.
    pub fn kill(&mut self, idx: usize, cause: &str) {
        if !self.party.members[idx].alive {
            return;
        }
        {
            let p = &mut self.party.members[idx];
            p.alive = false;
            p.health = 0;
            p.ailment = None;
            p.cause_of_death = Some(cause.to_string());
        }
        let name = self.party.members[idx].name.clone();
        let is_leader = self.party.members[idx].is_leader;

        self.graves.push(Grave {
            name: name.clone(),
            cause: cause.to_string(),
            date: self.date,
            miles: self.miles_total,
        });

        ui::tombstone(&[
            name.clone(),
            "died of".to_string(),
            cause.to_string(),
            format!(
                "{} {}, {}",
                self.date.month_name(),
                self.date.day,
                self.date.year
            ),
        ]);

        if is_leader {
            self.death_cause = Some(cause.to_string());
            self.outcome = Outcome::Died;
            ui::wrap(&format!(
                "{} was the leader of this party. With {} gone, the others turn \
                 back toward the settlements.",
                name, name
            ));
            ui::pause();
        } else if self.party.everyone_dead() {
            self.death_cause = Some(cause.to_string());
            self.outcome = Outcome::Died;
        } else {
            ui::pause();
        }
    }

    /// Convert zero health into actual deaths.
    fn reap(&mut self) {
        loop {
            let dying = self
                .party
                .members
                .iter()
                .position(|p| p.alive && p.health <= 0);
            match dying {
                Some(i) => {
                    let cause = self.party.members[i]
                        .ailment
                        .map(|a| a.name().to_string())
                        .unwrap_or_else(|| {
                            if self.inv.food == 0 {
                                "starvation".to_string()
                            } else {
                                "exhaustion".to_string()
                            }
                        });
                    self.kill(i, &cause);
                    if self.is_over() {
                        return;
                    }
                }
                None => return,
            }
        }
    }

    /// Advance the calendar without travelling: delays, repairs, searching.
    pub fn pass_days(&mut self, days: u32, resting: bool) {
        for _ in 0..days {
            if self.is_over() {
                return;
            }
            self.date.next_day();
            self.days_on_trail += 1;
            self.weather = weather::generate(self.date.month, self.terrain(), &mut self.rng);
            self.consume_food();
            self.apply_daily_health(resting);
            self.apply_ailments(resting);
            self.reap();
        }
    }

    // ------------------------------------------------------- no more oxen ---

    fn handle_stranded(&mut self) {
        ui::blank();
        ui::wrap(
            "You have no oxen left. The wagon cannot be moved until you get \
             another animal into the yoke.",
        );
        self.stranded_days += 1;
        if events::beg_for_oxen(self) {
            self.stranded_days = 0;
            return;
        }
        self.pass_days(3, true);
        if self.stranded_days >= 5 && self.inv.oxen == 0 {
            ui::blank();
            ui::wrap(
                "Day after day you wait beside a wagon that will not move. At \
                 last you abandon it and start walking back east.",
            );
            self.outcome = Outcome::TurnedBack;
        }
    }

    // --------------------------------------------------------- landmarks ---

    fn arrive_at_landmark(&mut self) {
        let idx = self.leg;
        let name = self.route[idx].name;
        let kind = self.route[idx].kind;
        let fork = self.route[idx].fork;
        let blurb = self.route[idx].blurb;

        self.miles_into_leg = 0;

        ui::banner(name);
        println!(
            "   {}          {} miles travelled          Weather: {}",
            self.date,
            ui::group(i64::from(self.miles_total)),
            self.weather.describe()
        );
        ui::blank();
        ui::wrap(blurb);
        ui::pause();

        match kind {
            Kind::River(spec) => {
                river::cross(self, name, spec);
                if self.is_over() {
                    return;
                }
            }
            Kind::Fort { markup } => {
                if ui::yes_no(&format!("Do you want to look around {}?", name)) {
                    store::shop_at_fort(&mut self.inv, self.party.alive_count(), name, markup);
                }
            }
            Kind::Finish => {
                self.outcome = Outcome::Arrived;
                return;
            }
            _ => {}
        }

        if let Some(f) = fork {
            if !self.route_extended_at.contains(&idx) {
                self.route_extended_at.push(idx);
                self.resolve_fork(f);
                if self.is_over() {
                    return;
                }
            }
        }

        self.leg += 1;
        if self.leg >= self.route.len() {
            // Should not happen, but never leave the player in limbo.
            self.outcome = Outcome::Arrived;
        }
    }

    fn resolve_fork(&mut self, fork: Fork) {
        match fork {
            Fork::SouthPass => {
                ui::heading("The trail divides");
                ui::wrap(
                    "Beyond South Pass the road splits. One branch runs straight \
                     for the Green River ford. The other swings south to Fort \
                     Bridger -- a hundred and fifty miles further, but there is a \
                     blacksmith there, and oxen for sale.",
                );
                let choice = ui::menu(
                    "You may:",
                    &[
                        "Head for the Green River crossing",
                        "Take the southern route to Fort Bridger",
                    ],
                );
                let via_bridger = choice == 2;
                let mut tail = trail::after_south_pass(via_bridger);
                tail.extend(trail::middle_route());
                self.route.extend(tail);
            }
            Fork::BlueMountains => {
                ui::heading("The trail divides");
                ui::wrap(
                    "From the crest of the Blue Mountains two roads drop away. \
                     One goes directly to The Dalles. The other bends north to \
                     Fort Walla Walla, a little longer but with a trading post at \
                     the end of it.",
                );
                let choice = ui::menu(
                    "You may:",
                    &[
                        "Go directly to The Dalles",
                        "Travel north to Fort Walla Walla",
                    ],
                );
                let via_walla = choice == 2;
                self.route.extend(trail::after_blue_mountains(via_walla));
            }
            Fork::Dalles => {
                ui::heading("The last decision");
                ui::wrap(
                    "The wagon road ends here. You may build a raft and float \
                     your wagon down the Columbia River through the rapids, or \
                     pay the toll on the Barlow Road, which climbs around the \
                     south shoulder of Mount Hood.",
                );
                loop {
                    let choice = ui::menu(
                        "You may:",
                        &[
                            "Float your wagon down the Columbia River",
                            &format!(
                                "Take the Barlow Toll Road (costs {})",
                                ui::money(trail::BARLOW_TOLL)
                            ),
                            "Ask about the two routes",
                        ],
                    );
                    match choice {
                        1 => {
                            river::float_columbia(self);
                            return;
                        }
                        2 => {
                            if self.inv.can_afford(trail::BARLOW_TOLL) {
                                self.inv.cash -= trail::BARLOW_TOLL;
                                println!(
                                    "\nYou pay the {} toll and start up the mountain road.",
                                    ui::money(trail::BARLOW_TOLL)
                                );
                                self.route.extend(trail::after_dalles(true));
                                ui::pause();
                                return;
                            }
                            println!(
                                "\nYou have only {}. The toll keeper shakes his head.",
                                ui::money(self.inv.cash)
                            );
                        }
                        _ => {
                            ui::heading("The two routes");
                            ui::wrap(
                                "\"The river is quick but it is a graveyard. Rafts \
                                 break up in the rapids and the whole outfit goes \
                                 to the bottom. The Barlow Road is slow and steep \
                                 and costs money, but you'll likely arrive with \
                                 your wagon.\"",
                            );
                            ui::pause();
                        }
                    }
                }
            }
        }
    }

    // --------------------------------------------------------- camp menu ---

    /// The menu shown whenever the party stops. `at_landmark` is true when
    /// standing at a named place (which allows shopping and sightseeing).
    fn camp_menu(&mut self, at_landmark: bool) {
        loop {
            if self.is_over() {
                return;
            }
            self.print_status();

            let at = if at_landmark && self.leg > 0 {
                Some(self.leg - 1)
            } else {
                None
            };
            let fort = at.and_then(|i| {
                let l = &self.route[i];
                if l.is_fort() {
                    Some((l.name, l.markup()))
                } else {
                    None
                }
            });

            let mut options: Vec<String> = vec![
                "Continue on the trail".to_string(),
                "Check supplies".to_string(),
                "Look at the map".to_string(),
                "Change pace".to_string(),
                "Change food rations".to_string(),
                "Stop to rest".to_string(),
                "Attempt to trade".to_string(),
                "Hunt for food".to_string(),
                "Look at your party".to_string(),
            ];
            if at_landmark {
                options.push("Talk to people".to_string());
            }
            if self.inv.oxen >= 2 {
                options.push("Slaughter an ox for meat".to_string());
            }
            if let Some((fname, _)) = fort {
                options.push(format!("Buy supplies at {}", fname));
            }
            options.push("Quit the journey".to_string());

            let refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            let choice = ui::menu("You may:", &refs);
            let picked = refs[choice - 1].to_string();

            if picked.starts_with("Continue") {
                return;
            } else if picked.starts_with("Check supplies") {
                self.inv.print(self.party.alive_count());
                ui::pause();
            } else if picked.starts_with("Look at the map") {
                self.show_map();
                ui::pause();
            } else if picked.starts_with("Change pace") {
                self.change_pace();
            } else if picked.starts_with("Change food") {
                self.change_rations();
            } else if picked.starts_with("Stop to rest") {
                self.rest();
            } else if picked.starts_with("Attempt to trade") {
                events::attempt_trade(self);
            } else if picked.starts_with("Hunt") {
                hunt::go_hunting(self);
            } else if picked.starts_with("Look at your party") {
                self.print_party();
                ui::pause();
            } else if picked.starts_with("Talk to people") {
                events::talk_to_people(self);
            } else if picked.starts_with("Slaughter") {
                self.slaughter_ox();
            } else if picked.starts_with("Buy supplies") {
                if let Some((fname, markup)) = fort {
                    store::shop_at_fort(&mut self.inv, self.party.alive_count(), fname, markup);
                }
            } else {
                if ui::yes_no("Really give up and turn back east?") {
                    self.outcome = Outcome::Quit;
                    return;
                }
            }
        }
    }

    pub fn print_status(&self) {
        ui::blank();
        ui::thick_rule();
        println!(
            "   Date: {:<22} Weather: {}",
            self.date.to_string(),
            self.weather.describe()
        );
        println!(
            "   Health: {:<20} Pace: {}",
            self.party.rating().name(),
            self.pace.name()
        );
        println!(
            "   Food: {:<22} Rations: {}",
            format!("{} lbs", ui::group(i64::from(self.inv.food))),
            self.rations.name()
        );
        println!(
            "   Oxen: {:<22} Money: {}",
            self.inv.oxen,
            ui::money(self.inv.cash)
        );
        println!(
            "   Travelled: {:<17} Next: {} ({} mi)",
            format!("{} miles", ui::group(i64::from(self.miles_total))),
            self.next_landmark_name(),
            ui::group(i64::from(self.miles_to_next()))
        );
        let sick = self.party.sick_indices();
        if !sick.is_empty() {
            let names: Vec<String> = sick
                .iter()
                .map(|i| {
                    let p = &self.party.members[*i];
                    format!("{} ({})", p.name, p.ailment.unwrap().name())
                })
                .collect();
            println!("   Ailing: {}", ui::list_and(&names));
        }
        // Mid-September onward, the Cascade passes start to close.
        if self.date.day_of_year() >= 258 && self.outcome == Outcome::Playing {
            println!("   !! It is late in the season. Snow will close the passes.");
        }
        ui::thick_rule();
    }

    pub fn print_party(&self) {
        ui::heading("Your party");
        for p in &self.party.members {
            println!(
                "   {}{}",
                p.status_line(),
                if p.is_leader { "   (you)" } else { "" }
            );
        }
    }

    fn show_map(&self) {
        ui::heading("The Oregon Trail");
        let mut cumulative = 0u32;
        for (i, l) in self.route.iter().enumerate() {
            cumulative += l.miles_from_prev;
            let marker = if i < self.leg {
                "[x]"
            } else if i == self.leg {
                " > "
            } else {
                "   "
            };
            println!(
                "  {} {:<26} {:>5} miles   {}",
                marker,
                l.name,
                ui::group(i64::from(cumulative)),
                l.terrain.name()
            );
        }
        ui::blank();
        println!(
            "   You are {} miles past {} and {} miles short of {}.",
            ui::group(i64::from(self.miles_into_leg)),
            self.last_landmark_name(),
            ui::group(i64::from(self.miles_to_next())),
            self.next_landmark_name()
        );
        if self.route.len() <= 8 {
            ui::blank();
            ui::wrap("   (Beyond South Pass the trail is not yet decided.)");
        }
        if !self.graves.is_empty() {
            ui::blank();
            println!("   Graves left behind:");
            for g in &self.graves {
                println!(
                    "      {:<16} {:<22} mile {}",
                    g.name,
                    g.cause,
                    ui::group(i64::from(g.miles))
                );
            }
        }
    }

    fn change_pace(&mut self) {
        ui::heading(&format!("Your pace is currently {}.", self.pace.name()));
        ui::wrap(
            "A steady pace means walking a normal day and making camp in good \
             time. A strenuous pace covers more ground but wears people down. \
             A grueling pace will make you sick.",
        );
        let choice = ui::menu(
            "Set the pace to:",
            &[
                "A steady pace",
                "A strenuous pace",
                "A grueling pace",
                "Leave it as it is",
            ],
        );
        match choice {
            1 => self.pace = Pace::Steady,
            2 => self.pace = Pace::Strenuous,
            3 => self.pace = Pace::Grueling,
            _ => {}
        }
        println!("\nPace is now {}.", self.pace.name());
    }

    fn change_rations(&mut self) {
        let n = self.party.alive_count() as u32;
        ui::heading(&format!("Rations are currently {}.", self.rations.name()));
        println!(
            "   filling     -- 3 lbs per person per day ({} lbs a day)",
            3 * n
        );
        println!(
            "   meager      -- 2 lbs per person per day ({} lbs a day)",
            2 * n
        );
        println!(
            "   bare bones  -- 1 lb  per person per day ({} lbs a day)",
            n
        );
        let choice = ui::menu(
            "Set rations to:",
            &[
                "Filling -- well fed",
                "Meager -- not quite enough",
                "Bare bones -- barely any food",
                "Leave them as they are",
            ],
        );
        match choice {
            1 => self.rations = Rations::Filling,
            2 => self.rations = Rations::Meager,
            3 => self.rations = Rations::BareBones,
            _ => {}
        }
        println!("\nRations are now {}.", self.rations.name());
    }

    fn rest(&mut self) {
        let days = ui::prompt_int(
            "\nHow many days would you like to rest? (0 to 20) ",
            0,
            20,
            Some(0),
        ) as u32;
        if days == 0 {
            return;
        }
        let before = self.party.average_health();
        self.pass_days(days, true);
        if self.is_over() {
            return;
        }
        let after = self.party.average_health();
        ui::blank();
        println!(
            "You rest for {}. It is now {}.",
            ui::plural(days, "day", "days"),
            self.date
        );
        if after > before {
            println!(
                "Your party is feeling better ({}).",
                self.party.rating().name()
            );
        } else if after < before {
            println!(
                "The rest did not help much -- health is {}.",
                self.party.rating().name()
            );
        }
        // Grazing recovers while you sit still.
        self.ox_fatigue = 0;
        ui::pause();
    }

    /// Kill one of the team for meat -- a real and desperate measure.
    fn slaughter_ox(&mut self) {
        ui::heading("Slaughtering an ox");
        if self.inv.oxen < 2 {
            ui::wrap(
                "You have too few oxen as it is. Kill another and the wagon \
                 will never move again.",
            );
            ui::pause();
            return;
        }
        ui::wrap(&format!(
            "You have {} oxen. Butchering one will feed the party for a week or \
             two, but the wagon will travel more slowly for the rest of the \
             journey.",
            self.inv.oxen
        ));
        ui::blank();
        if !ui::yes_no("Slaughter an ox?") {
            return;
        }
        self.inv.take_oxen(1);
        // Most of the carcass spoils before it can be eaten or dried.
        let meat = self.rng.range(150, 260) as u32;
        self.inv.food += meat;
        self.party.adjust_all(self.rng.range(1, 4));
        ui::blank();
        println!(
            "You butcher the ox and dry what you can. {} pounds of meat go into \
             the wagon; the rest spoils.",
            meat
        );
        println!(
            "You have {} oxen left and {} pounds of food.",
            self.inv.oxen,
            ui::group(i64::from(self.inv.food))
        );
        self.pass_days(1, false);
        if !self.is_over() {
            ui::pause();
        }
    }

    // ------------------------------------------------------- repair logic ---

    /// Handle a broken wagon part. Used by the event module.
    pub fn break_part(&mut self, part: Part) {
        ui::blank();
        println!("!! A {} has broken.", part.name());
        if self.inv.spares(part) > 0 {
            *self.inv.spares_mut(part) -= 1;
            let days = self.rng.range(1, 2) as u32;
            println!(
                "   You fit your spare {} and lose {}.",
                part.name(),
                ui::plural(days, "day", "days")
            );
            self.pass_days(days, false);
            return;
        }
        println!("   You have no spare {}.", part.name());
        let skill = self.occupation.repair_skill();
        if self.rng.chance(skill) {
            let days = self.rng.range(2, 5) as u32;
            println!(
                "   Working with what you have, you contrive a repair. It costs \
                 you {}.",
                ui::plural(days, "day", "days")
            );
            self.pass_days(days, false);
        } else {
            let days = self.rng.range(5, 11) as u32;
            println!(
                "   You cannot mend it. You wait {} for another wagon train to \
                 come by and lend you one.",
                ui::plural(days, "day", "days")
            );
            self.party.adjust_all(-4);
            self.pass_days(days, false);
        }
        self.reap();
    }

    // ---------------------------------------------------------- the end ---

    fn finish(&mut self) {
        match self.outcome.clone() {
            Outcome::Arrived => {
                ui::banner("The Willamette Valley");
                ui::wrap(&format!(
                    "On {} your wagon comes down out of the hills into the \
                     Willamette Valley. You have travelled {} miles in {} days.",
                    self.date.long_form(),
                    ui::group(i64::from(self.miles_total)),
                    self.days_on_trail
                ));
                ui::blank();
                if self.party.alive_count() == PARTY_SIZE {
                    ui::wrap(
                        "Every member of your party has come through alive -- a \
                         rare thing on this road.",
                    );
                } else {
                    ui::wrap(&format!(
                        "{} of the {} who left Independence have arrived. The \
                         others lie in graves along the trail.",
                        self.party.alive_count(),
                        PARTY_SIZE
                    ));
                }
                ui::pause();
                score::show_final(self);
            }
            Outcome::Died => {
                ui::banner("The journey ends");
                let cause = self
                    .death_cause
                    .clone()
                    .unwrap_or_else(|| "misfortune".to_string());
                if self.party.everyone_dead() {
                    ui::wrap(
                        "Not one of your party lived to see Oregon. The wagon \
                         stands abandoned on the prairie with its tongue in the \
                         dust.",
                    );
                } else {
                    ui::wrap(&format!(
                        "You died of {} after {} miles, on {}.",
                        cause,
                        ui::group(i64::from(self.miles_total)),
                        self.date.long_form()
                    ));
                }
                ui::blank();
                self.print_party();
                ui::pause();
                score::show_final(self);
            }
            Outcome::TurnedBack => {
                ui::banner("Turning back");
                ui::wrap(&format!(
                    "After {} miles you give up the road to Oregon and start \
                     back toward the settlements.",
                    ui::group(i64::from(self.miles_total))
                ));
                ui::pause();
                score::show_final(self);
            }
            Outcome::Quit => {
                ui::banner("You abandon the trail");
                ui::wrap(&format!(
                    "You turn the wagon around at mile {}. Oregon will have to \
                     get along without you.",
                    ui::group(i64::from(self.miles_total))
                ));
                ui::pause();
            }
            Outcome::Playing => {}
        }
    }
}
