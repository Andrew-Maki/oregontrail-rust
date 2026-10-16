//! Random misfortunes (and the occasional kindness) along the trail.

use crate::game::{Game, Outcome, Pace, Rations};
use crate::inventory::{Inventory, Part};
use crate::party::Ailment;
use crate::rng::Rng;
use crate::trail::Terrain;
use crate::ui;
use crate::weather::{Cond, Temp};

/// Base probability that *something* happens on a given day.
fn event_chance(g: &Game) -> f64 {
    let mut p = 0.17;
    if g.pace == Pace::Strenuous {
        p += 0.03;
    }
    if g.pace == Pace::Grueling {
        p += 0.07;
    }
    if g.terrain() == Terrain::Mountains {
        p += 0.04;
    }
    if g.weather.exposure() >= 4 {
        p += 0.04;
    }
    p
}

/// Roll the daily event. Returns true if the player should be given a chance
/// to stop and react.
pub fn roll_daily_event(g: &mut Game) -> bool {
    let chance = event_chance(g);
    if !g.rng.chance(chance) {
        return false;
    }

    // Weighted table. Weights are tuned so that the common nuisances happen
    // often and the catastrophes stay rare. The last two entries are not
    // things that happened to the real emigrants.
    let roll = g.rng.below(1020);
    match roll {
        0..=79 => wagon_breakdown(g),
        80..=149 => ox_trouble(g),
        150..=209 => thieves(g),
        210..=259 => bad_water(g),
        260..=309 => little_water(g),
        310..=359 => inadequate_grass(g),
        360..=399 => wrong_trail(g),
        400..=439 => heavy_weather(g),
        440..=479 => fire(g),
        480..=509 => snakebite(g),
        510..=549 => broken_limb(g),
        550..=589 => person_lost(g),
        590..=619 => ox_wanders(g),
        620..=649 => bandits(g),
        650..=679 => impassable_trail(g),
        680..=709 => hailstorm(g),
        710..=739 => blizzard(g),
        740..=769 => rough_crossing(g),
        770..=819 => wild_fruit(g),
        820..=859 => abandoned_wagon(g),
        860..=899 => helpful_strangers(g),
        900..=939 => trader_appears(g),
        940..=969 => pass_a_grave(g),
        970..=999 => good_grazing(g),
        1000..=1009 => graboids(g),
        _ => lights_in_the_sky(g),
    }
}

fn note(text: &str) {
    ui::blank();
    ui::wrap_indent(text, 3);
}

// ------------------------------------------------------------ misfortunes ---

fn wagon_breakdown(g: &mut Game) -> bool {
    let part = *g.rng.pick(&Part::ALL);
    g.break_part(part);
    true
}

fn ox_trouble(g: &mut Game) -> bool {
    if g.inv.oxen == 0 {
        return false;
    }
    if g.rng.chance(0.45) {
        g.inv.take_oxen(1);
        note(&format!(
            "One of your oxen has died in the yoke. You have {} left.",
            g.inv.oxen
        ));
    } else {
        let days = g.rng.range(1, 3) as u32;
        note(&format!(
            "An ox has gone lame. You lay over {} waiting for it to walk sound \
             again.",
            ui::plural(days, "day", "days")
        ));
        g.pass_days(days, false);
        g.ox_fatigue = g.ox_fatigue.max(3);
    }
    true
}

fn thieves(g: &mut Game) -> bool {
    let mut losses: Vec<String> = Vec::new();
    if g.inv.food > 0 && g.rng.chance(0.7) {
        let lost = g.rng.range(20, 120) as u32;
        let lost = g.inv.take_food(lost);
        if lost > 0 {
            losses.push(format!("{} pounds of food", ui::group(i64::from(lost))));
        }
    }
    if g.inv.bullets > 0 && g.rng.chance(0.5) {
        let lost = g.rng.range(10, 60) as u32;
        let lost = g.inv.take_bullets(lost);
        if lost > 0 {
            losses.push(format!("{} bullets", lost));
        }
    }
    if g.inv.oxen > 1 && g.rng.chance(0.25) {
        g.inv.take_oxen(1);
        losses.push("an ox".to_string());
    }
    if g.inv.clothing > 0 && g.rng.chance(0.3) {
        let lost = g.inv.take_clothing(g.rng.range(1, 2) as u32);
        if lost > 0 {
            losses.push(ui::plural(lost, "set of clothing", "sets of clothing"));
        }
    }
    if losses.is_empty() {
        note("Thieves came in the night, but found nothing worth carrying off.");
    } else {
        note(&format!(
            "Thieves came in the night and made off with {}.",
            ui::list_and(&losses)
        ));
    }
    true
}

fn bad_water(g: &mut Game) -> bool {
    note(
        "The only water for miles is a stagnant slough, green at the edges. You \
         have no choice but to fill the barrel from it.",
    );
    g.party.adjust_all(-(g.rng.range(4, 10)));
    if let Some(i) = g.party.random_living(&mut g.rng) {
        if g.rng.chance(0.4) {
            let disease = *g
                .rng
                .pick(&[Ailment::Cholera, Ailment::Typhoid, Ailment::Dysentery]);
            g.strike(i, disease);
        }
    }
    true
}

fn little_water(g: &mut Game) -> bool {
    let days = g.rng.range(1, 3) as u32;
    note(&format!(
        "There is very little water on this stretch. You ration what is in the \
         barrel and push on for {} with dry throats.",
        ui::plural(days, "day", "days")
    ));
    g.party.adjust_all(-(g.rng.range(3, 8)));
    g.ox_fatigue = g.ox_fatigue.max(days + 1);
    true
}

fn inadequate_grass(g: &mut Game) -> bool {
    let days = g.rng.range(3, 7) as u32;
    note(
        "The grass here has been eaten to the roots by the trains ahead of you. \
         Your oxen go hungry and will pull poorly for some days.",
    );
    g.ox_fatigue = g.ox_fatigue.max(days);
    if g.inv.oxen > 1 && g.rng.chance(0.18) {
        g.inv.take_oxen(1);
        println!("   One of the weaker oxen does not get up again.");
    }
    true
}

fn wrong_trail(g: &mut Game) -> bool {
    let lost_miles = g.rng.range(12, 45) as u32;
    let days = g.rng.range(1, 3) as u32;
    note(&format!(
        "You have taken the wrong trail. By the time you realise it you have \
         thrown away {} miles and {}.",
        lost_miles,
        ui::plural(days, "day", "days")
    ));
    g.miles_into_leg = g.miles_into_leg.saturating_sub(lost_miles);
    g.miles_total = g.miles_total.saturating_sub(lost_miles);
    g.pass_days(days, false);
    true
}

fn heavy_weather(g: &mut Game) -> bool {
    let days = g.rng.range(1, 4) as u32;
    match g.weather.cond {
        Cond::Fog => note(&format!(
            "A fog comes down so thick you cannot see the lead team. You lose \
             {}.",
            ui::plural(days, "day", "days")
        )),
        Cond::HeavySnow | Cond::Snow => note(&format!(
            "Snow drifts across the road and the wagons cannot break through. \
             You lose {}.",
            ui::plural(days, "day", "days")
        )),
        _ => note(&format!(
            "Heavy rain has turned the road to a river of mud. You lose {}.",
            ui::plural(days, "day", "days")
        )),
    }
    if g.inv.food > 0 && g.rng.chance(0.5) {
        let spoiled = g.inv.take_food(g.rng.range(15, 70) as u32);
        if spoiled > 0 {
            println!(
                "   {} pounds of food are soaked and spoiled.",
                ui::group(i64::from(spoiled))
            );
        }
    }
    g.pass_days(days, false);
    true
}

fn fire(g: &mut Game) -> bool {
    note("A fire breaks out in the wagon!");
    let mut losses: Vec<String> = Vec::new();
    let lost_food = g.inv.take_food(g.rng.range(30, 160) as u32);
    if lost_food > 0 {
        losses.push(format!(
            "{} pounds of food",
            ui::group(i64::from(lost_food))
        ));
    }
    let lost_ammo = g.inv.take_bullets(g.rng.range(0, 80) as u32);
    if lost_ammo > 0 {
        losses.push(format!("{} bullets", lost_ammo));
    }
    let lost_clothes = g.inv.take_clothing(g.rng.range(0, 3) as u32);
    if lost_clothes > 0 {
        losses.push(ui::plural(
            lost_clothes,
            "set of clothing",
            "sets of clothing",
        ));
    }
    if losses.is_empty() {
        println!("   You beat it out with blankets before it took hold.");
    } else {
        println!("   You lose {}.", ui::list_and(&losses));
    }
    if g.rng.chance(0.3) {
        if let Some(i) = g.party.random_living(&mut g.rng) {
            let name = g.party.members[i].name.clone();
            println!("   {} is badly burned fighting the fire.", name);
            g.party.members[i].adjust(-(g.rng.range(10, 25)));
        }
    }
    true
}

fn snakebite(g: &mut Game) -> bool {
    let Some(i) = g.party.random_living(&mut g.rng) else {
        return false;
    };
    let name = g.party.members[i].name.clone();
    note(&format!(
        "A rattlesnake strikes {} on the ankle while gathering firewood.",
        name
    ));
    g.strike(i, Ailment::SnakeBite);
    g.party.members[i].adjust(-(g.rng.range(8, 20)));
    true
}

fn broken_limb(g: &mut Game) -> bool {
    let Some(i) = g.party.random_living(&mut g.rng) else {
        return false;
    };
    let name = g.party.members[i].name.clone();
    let (ailment, how) = match g.rng.below(4) {
        0 => (Ailment::BrokenArm, "is thrown from the wagon seat"),
        1 => (Ailment::BrokenLeg, "is caught under a wheel"),
        2 => (Ailment::BrokenRibs, "is kicked by an ox"),
        _ => (Ailment::Concussion, "falls while leading the team downhill"),
    };
    note(&format!("{} {}.", name, how));
    g.strike(i, ailment);
    g.party.members[i].adjust(-(g.rng.range(5, 14)));
    true
}

fn person_lost(g: &mut Game) -> bool {
    let Some(i) = g.party.random_living(&mut g.rng) else {
        return false;
    };
    let name = g.party.members[i].name.clone();
    let days = g.rng.range(1, 4) as u32;
    note(&format!(
        "{} has wandered off and cannot be found. You spend {} searching.",
        name,
        ui::plural(days, "day", "days")
    ));
    g.pass_days(days, false);
    if g.is_over() {
        return true;
    }
    if g.rng.chance(0.85) {
        println!("   {} is found, cold and frightened but alive.", name);
        g.party.members[i].adjust(-(g.rng.range(5, 15)));
    } else {
        g.kill(i, "exposure after becoming lost");
    }
    true
}

fn ox_wanders(g: &mut Game) -> bool {
    if g.inv.oxen == 0 {
        return false;
    }
    let days = g.rng.range(1, 3) as u32;
    note(&format!(
        "An ox has strayed off in the night. You hunt for it for {}.",
        ui::plural(days, "day", "days")
    ));
    g.pass_days(days, false);
    if g.rng.chance(0.6) {
        println!("   You find it grazing two miles down a coulee.");
    } else {
        g.inv.take_oxen(1);
        println!("   The ox is gone for good. You have {} left.", g.inv.oxen);
    }
    true
}

fn bandits(g: &mut Game) -> bool {
    note("Riders come out of the brush with guns drawn. They want your outfit.");
    let defended = g.inv.bullets >= 20 && g.rng.chance(0.55);
    if defended {
        let used = g.inv.take_bullets(g.rng.range(10, 30) as u32);
        println!("   You fire back and they scatter. {} bullets spent.", used);
        if g.rng.chance(0.3) {
            if let Some(i) = g.party.random_living(&mut g.rng) {
                let name = g.party.members[i].name.clone();
                println!("   {} takes a bullet in the shoulder.", name);
                g.party.members[i].adjust(-(g.rng.range(12, 28)));
            }
        }
    } else {
        let cash_lost = (g.inv.cash / 3).min(g.inv.cash);
        g.inv.cash -= cash_lost;
        let food_lost = g.inv.take_food(g.rng.range(40, 150) as u32);
        println!(
            "   They take {} and {} pounds of food, and ride off laughing.",
            ui::money(cash_lost),
            ui::group(i64::from(food_lost))
        );
        if g.rng.chance(0.2) {
            if let Some(i) = g.party.random_living(&mut g.rng) {
                g.kill(i, "a gunshot wound");
            }
        }
    }
    true
}

fn impassable_trail(g: &mut Game) -> bool {
    let days = g.rng.range(2, 6) as u32;
    note(&format!(
        "The road ahead is blocked -- a landslide has carried away the shelf \
         above the creek. You spend {} cutting a way around.",
        ui::plural(days, "day", "days")
    ));
    g.party.adjust_all(-(g.rng.range(3, 9)));
    g.pass_days(days, false);
    true
}

fn hailstorm(g: &mut Game) -> bool {
    note("A hailstorm sweeps over you, stones the size of hen's eggs.");
    let lost = g.inv.take_food(g.rng.range(20, 90) as u32);
    if lost > 0 {
        println!(
            "   {} pounds of food are ruined.",
            ui::group(i64::from(lost))
        );
    }
    let lost_ammo = g.inv.take_bullets(g.rng.range(0, 40) as u32);
    if lost_ammo > 0 {
        println!("   {} bullets are lost in the mud.", lost_ammo);
    }
    g.party.adjust_all(-(g.rng.range(2, 7)));
    let days = g.rng.range(1, 2) as u32;
    g.pass_days(days, false);
    true
}

fn blizzard(g: &mut Game) -> bool {
    if g.weather.temp != Temp::Cold && g.weather.temp != Temp::VeryCold {
        // Out of season -- substitute a thunderstorm.
        return heavy_weather(g);
    }
    let days = g.rng.range(2, 6) as u32;
    note(&format!(
        "A blizzard pins you down for {}. The wind comes through the wagon cover \
         like a knife.",
        ui::plural(days, "day", "days")
    ));
    let needed = g.party.alive_count() as u32;
    if g.inv.clothing < needed {
        println!("   You do not have enough clothing for everyone.");
        g.party.adjust_all(-(g.rng.range(12, 25)));
        if let Some(i) = g.party.random_living(&mut g.rng) {
            if g.rng.chance(0.5) {
                g.strike(i, Ailment::Frostbite);
            }
        }
    } else {
        g.party.adjust_all(-(g.rng.range(4, 11)));
    }
    g.pass_days(days, true);
    true
}

fn rough_crossing(g: &mut Game) -> bool {
    note(
        "A nameless creek has cut a steep bank. Getting the wagon down and up \
         again takes all day, and water gets into the load.",
    );
    let lost = g.inv.take_food(g.rng.range(10, 50) as u32);
    if lost > 0 {
        println!(
            "   {} pounds of food are spoiled.",
            ui::group(i64::from(lost))
        );
    }
    g.pass_days(1, false);
    true
}

// ------------------------------------------------- things best not recorded ---
//
// Two events that are not in any emigrant's diary, for the traveller
// unlucky enough to roll them. Each turns up about once in a hundred
// incidents, so most parties reach Oregon never knowing.

/// Something very large is moving under the ground.
fn graboids(g: &mut Game) -> bool {
    ui::blank();
    ui::thick_rule();
    ui::center("THE GROUND IS MOVING");
    ui::thick_rule();
    ui::wrap_indent(
        "It begins as a tremor in the wagon tongue. Then a ridge of dirt comes \
         up out of the flat ground a hundred yards off and runs toward you at \
         the speed of a galloping horse, throwing up sod as it goes. The oxen \
         scream -- not bellow, scream -- and try to break the yoke.",
        3,
    );
    ui::blank();
    ui::wrap_indent(
        "A second ridge peels off from the first. Whatever is down there hunts \
         by the drumming of feet on the earth.",
        3,
    );

    let choice = ui::menu(
        "You may:",
        &[
            "Whip up the team and run for open ground",
            "Get everyone onto the rock outcrop and stand dead still",
            "Cut the oxen loose and let them scatter",
        ],
    );

    match choice {
        // Running makes exactly the noise the things are listening for.
        1 => {
            ui::blank();
            ui::wrap_indent(
                "You lay on the whip. The ridge changes course to follow the \
                 wheels. Forty feet of grey hide comes up through the prairie \
                 alongside the wagon and something opens that is mostly mouth.",
                3,
            );
            let lost = g.inv.take_oxen(g.rng.range(1, 3) as u32);
            if lost > 0 {
                println!(
                    "\n   It takes {} out of the traces without slowing down.",
                    ui::plural(lost, "ox", "oxen")
                );
            }
            g.party.adjust_all(-(g.rng.range(8, 18)));
            if g.rng.chance(0.30) {
                if let Some(i) = g.party.random_living(&mut g.rng) {
                    if g.rng.chance(0.4) {
                        g.kill(i, "being dragged under the ground");
                    } else {
                        let name = g.party.members[i].name.clone();
                        println!("   {} is thrown clear of the wagon.", name);
                        g.strike(i, Ailment::BrokenRibs);
                    }
                }
            }
            if !g.is_over() {
                ui::blank();
                ui::wrap_indent(
                    "You outrun it at last, or it loses interest. Nobody \
                     suggests making camp until well after dark.",
                    3,
                );
            }
        }
        // Rock is the right answer. It costs you a day and your nerve.
        2 => {
            ui::blank();
            ui::wrap_indent(
                "You get the party up onto a shelf of basalt and nobody moves. \
                 For an hour the ridges circle the outcrop, close enough to \
                 shower grit on your boots. Then, hearing nothing, they go \
                 away south.",
                3,
            );
            let days = g.rng.range(1, 2) as u32;
            let lost = if g.inv.oxen > 2 && g.rng.chance(0.45) {
                g.inv.take_oxen(1)
            } else {
                0
            };
            if lost > 0 {
                println!(
                    "\n   One ox left standing in the traces is taken. The rest \
                     of the team survives."
                );
            } else {
                println!("\n   The team, by some miracle, is untouched.");
            }
            g.party.adjust_all(-(g.rng.range(3, 8)));
            println!(
                "   You lose {} getting the wagon moving again.",
                ui::plural(days, "day", "days")
            );
            g.pass_days(days, false);
        }
        // Noble, expensive, and it works.
        _ => {
            ui::blank();
            let scattered = g.inv.take_oxen(g.rng.range(2, 4) as u32);
            ui::wrap_indent(
                "You cut the yokes. The oxen bolt in every direction, hammering \
                 the ground, and the ridges go after them. The wagon sits in \
                 perfect silence while it happens.",
                3,
            );
            println!(
                "\n   {} are gone. You recover none of them.",
                ui::plural(scattered, "ox", "oxen")
            );
            g.party.adjust_all(-(g.rng.range(2, 6)));
            let days = g.rng.range(1, 3) as u32;
            g.pass_days(days, false);
        }
    }

    if !g.is_over() {
        let scribe = g
            .party
            .random_living(&mut g.rng)
            .map(|i| g.party.members[i].name.clone())
            .unwrap_or_else(|| "someone".to_string());
        ui::blank();
        ui::wrap_indent(
            &format!(
                "{} writes \"graboid\" in the journal, for want of a better \
                 word, and underlines it twice. No one east of here will \
                 believe a line of it.",
                scribe
            ),
            3,
        );
    }
    true
}

/// A silent disc of light over the camp, and the team going up into it.
fn lights_in_the_sky(g: &mut Game) -> bool {
    ui::blank();
    ui::thick_rule();
    ui::center("LIGHTS OVER THE CAMP");
    ui::thick_rule();
    ui::wrap_indent(
        "You wake because the inside of the wagon cover is lit up white. There \
         is no sound at all -- not an insect, not the creek. Above the picket \
         line hangs a disc of dull metal the width of a barn, turning slowly, \
         close enough to throw your shadow on the grass.",
        3,
    );
    ui::pause();

    if g.inv.oxen == 0 {
        ui::blank();
        ui::wrap_indent(
            "It hangs there a long while, as though looking for something it \
             cannot find. Then it tilts, and goes straight up, and is a star, \
             and then is nothing. In the morning there is a ring of scorched \
             grass around the empty picket line.",
            3,
        );
        g.party.adjust_all(-(g.rng.range(2, 6)));
        return true;
    }

    // It takes a share of the team -- but never quite all of it.
    let keep = if g.inv.oxen >= 2 { 1 } else { 0 };
    let takeable = g.inv.oxen - keep;
    let wanted = ((f64::from(g.inv.oxen) * g.rng.range_f(0.4, 0.8)).round() as u32).max(1);
    let taken = g.inv.take_oxen(wanted.min(takeable));

    ui::blank();
    ui::wrap_indent(
        "A column of light opens underneath it and the oxen stop fighting the \
         rope. They go up through it one after another, legs hanging, turning \
         gently, perfectly calm -- and that is somehow the worst part of it.",
        3,
    );
    ui::blank();
    if taken == 1 {
        println!("   One ox rises into the light and does not come back down.");
    } else {
        println!(
            "   {} oxen rise into the light and do not come back down.",
            taken
        );
    }
    if g.inv.oxen > 0 {
        println!(
            "   {} left standing in a scorched ring, shaking.",
            ui::plural(g.inv.oxen, "ox is", "oxen are")
        );
    }

    // Terror, and the odd souvenir.
    g.party.adjust_all(-(g.rng.range(5, 13)));
    g.ox_fatigue = g.ox_fatigue.max(g.rng.range(2, 5) as u32);

    if g.rng.chance(0.25) {
        if let Some(i) = g.party.random_living(&mut g.rng) {
            let name = g.party.members[i].name.clone();
            ui::blank();
            println!("   {} will not speak for two days afterward.", name);
            g.strike(i, Ailment::Fever);
        }
    } else if g.rng.chance(0.3) {
        let found = g.rng.range(25, 70) as u32;
        g.inv.food += found;
        let eater = g
            .party
            .random_living(&mut g.rng)
            .map(|i| g.party.members[i].name.clone())
            .unwrap_or_else(|| "someone".to_string());
        ui::blank();
        ui::wrap_indent(
            &format!(
                "Where the light touched the ground there is a neat heap of \
                 something like dried meat, still warm. {} eats some on a dare \
                 and is none the worse. You pack {} pounds of it.",
                eater, found
            ),
            3,
        );
    }

    ui::blank();
    ui::wrap_indent(
        "The disc tilts, and goes straight up, and is a star, and then is \
         nothing. You agree among yourselves to tell people it was horse \
         thieves.",
        3,
    );
    true
}

// --------------------------------------------------------- good fortune ---

fn wild_fruit(g: &mut Game) -> bool {
    let found = g.rng.range(15, 70) as u32;
    note(&format!(
        "You come upon a thicket heavy with wild berries and plums. Everyone \
         picks until their hands are stained, and you add {} pounds of fruit to \
         the wagon.",
        found
    ));
    g.inv.food += found;
    g.party.adjust_all(g.rng.range(1, 4));
    false
}

fn abandoned_wagon(g: &mut Game) -> bool {
    note("An abandoned wagon stands beside the road, picked over but not bare.");
    let mut gains: Vec<String> = Vec::new();
    if g.rng.chance(0.6) {
        let food = g.rng.range(10, 60) as u32;
        g.inv.food += food;
        gains.push(format!("{} pounds of food", food));
    }
    if g.rng.chance(0.4) {
        let bullets = g.rng.range(10, 50) as u32;
        g.inv.bullets += bullets;
        gains.push(format!("{} bullets", bullets));
    }
    if g.rng.chance(0.35) {
        let part = *g.rng.pick(&Part::ALL);
        *g.inv.spares_mut(part) += 1;
        gains.push(format!("a sound {}", part.name()));
    }
    if g.rng.chance(0.25) {
        let sets = g.rng.range(1, 2) as u32;
        g.inv.clothing += sets;
        gains.push(ui::plural(sets, "set of clothing", "sets of clothing"));
    }
    if gains.is_empty() {
        println!("   There is nothing left in it worth taking.");
    } else {
        println!("   You salvage {}.", ui::list_and(&gains));
    }
    false
}

fn helpful_strangers(g: &mut Game) -> bool {
    let food = g.rng.range(20, 80) as u32;
    note(&format!(
        "A band of Shoshone meet you on the road. They look over your thin \
         oxen, laugh, and trade you {} pounds of dried salmon and camas root \
         for almost nothing.",
        food
    ));
    g.inv.food += food;
    g.party.adjust_all(g.rng.range(1, 5));
    false
}

fn pass_a_grave(g: &mut Game) -> bool {
    let epitaphs: [&str; 8] = [
        "J. HOLLOWAY, DIED OF CHOLERA, JUNE 1847 -- MAY HE REST",
        "SARAH KEYES, AGED 70 YEARS -- SHE WOULD NOT TURN BACK",
        "UNKNOWN MAN, KILLED BY A WAGON WHEEL",
        "ELIZA JANE, 4 YEARS OLD. BURIED DEEP AGAINST THE WOLVES",
        "THOS. BARLOW -- DROWNED AT THIS CROSSING",
        "WE BURIED FATHER HERE. 1,100 MILES TO GO",
        "HERE LIES A MAN WHO TRUSTED A SHORTCUT",
        "MARY AND HER INFANT SON, SAME DAY, SAME GRAVE",
    ];
    let idx = g.rng.below(epitaphs.len() as u32) as usize;
    note("You pass a grave beside the road, the board still legible:");
    ui::blank();
    ui::center(epitaphs[idx]);
    g.party.adjust_all(-1);
    false
}

/// A trader rides up and offers a swap without being asked.
fn trader_appears(g: &mut Game) -> bool {
    note(
        "A trader on a spotted pony falls in beside your wagon and asks what you \
         have to swap.",
    );
    ui::blank();
    let offer = make_offer(&mut g.rng, &g.inv);
    match offer {
        Some((want_desc, give_desc, apply)) => {
            ui::wrap_indent(
                &format!(
                    "He will give you {} in exchange for {}.",
                    give_desc, want_desc
                ),
                3,
            );
            ui::blank();
            if ui::yes_no("Do you want to trade?") {
                if apply(g) {
                    println!("\nThe trade is made and he rides off.");
                } else {
                    println!("\nYou cannot make good on the deal after all.");
                }
            } else {
                println!("\n\"Then I'll wish you good day.\"");
            }
        }
        None => {
            ui::wrap_indent("He looks over your outfit and rides on.", 3);
        }
    }
    true
}

fn good_grazing(g: &mut Game) -> bool {
    note(
        "You strike a bottom where the grass stands belly-high to the oxen. \
         They graze all evening and pull like new animals.",
    );
    g.ox_fatigue = 0;
    g.party.adjust_all(g.rng.range(1, 3));
    false
}

// ------------------------------------------------------------- trading ---

/// Trade with whoever happens to be nearby.
pub fn attempt_trade(g: &mut Game) {
    ui::heading("Attempting to trade");
    if g.rng.chance(0.35) {
        ui::wrap("You find no one who wants to trade.");
        ui::pause();
        return;
    }

    let offer = make_offer(&mut g.rng, &g.inv);
    match offer {
        Some((want_desc, give_desc, apply)) => {
            ui::wrap(&format!(
                "A trader will give you {} in exchange for {}.",
                give_desc, want_desc
            ));
            ui::blank();
            if ui::yes_no("Do you want to trade?") {
                if apply(g) {
                    println!("\nThe trade is made.");
                } else {
                    println!("\nYou do not have what he wants.");
                }
            } else {
                println!("\n\"Suit yourself.\"");
            }
        }
        None => {
            ui::wrap("Nobody nearby wants anything you have.");
        }
    }
    ui::pause();
}

type TradeFn = Box<dyn Fn(&mut Game) -> bool>;

fn make_offer(rng: &mut Rng, inv: &Inventory) -> Option<(String, String, TradeFn)> {
    // Build the list of trades that are actually possible right now.
    let mut options: Vec<(String, String, TradeFn)> = Vec::new();

    if inv.food >= 100 {
        let food = rng.range(80, 150) as u32;
        let bullets = rng.range(40, 120) as u32;
        options.push((
            format!("{} pounds of food", food),
            format!("{} bullets", bullets),
            Box::new(move |g: &mut Game| {
                if g.inv.food < food {
                    return false;
                }
                g.inv.food -= food;
                g.inv.bullets += bullets;
                true
            }),
        ));
    }

    if inv.bullets >= 60 {
        let bullets = rng.range(40, 100) as u32;
        let food = rng.range(60, 140) as u32;
        options.push((
            format!("{} bullets", bullets),
            format!("{} pounds of food", food),
            Box::new(move |g: &mut Game| {
                if g.inv.bullets < bullets {
                    return false;
                }
                g.inv.bullets -= bullets;
                g.inv.food += food;
                true
            }),
        ));
    }

    if inv.clothing >= 2 {
        let sets = rng.range(1, 2) as u32;
        let food = rng.range(70, 160) as u32;
        options.push((
            ui::plural(sets, "set of clothing", "sets of clothing"),
            format!("{} pounds of food", food),
            Box::new(move |g: &mut Game| {
                if g.inv.clothing < sets {
                    return false;
                }
                g.inv.clothing -= sets;
                g.inv.food += food;
                true
            }),
        ));
    }

    if inv.cash >= 2000 {
        let cents = (rng.range(15, 45) as i64) * 100;
        let part = *rng.pick(&Part::ALL);
        options.push((
            ui::money(cents),
            format!("a spare {}", part.name()),
            Box::new(move |g: &mut Game| {
                if g.inv.cash < cents {
                    return false;
                }
                g.inv.cash -= cents;
                *g.inv.spares_mut(part) += 1;
                true
            }),
        ));
    }

    if inv.cash >= 5000 {
        let cents = (rng.range(45, 90) as i64) * 100;
        options.push((
            ui::money(cents),
            "a yoke of oxen".to_string(),
            Box::new(move |g: &mut Game| {
                if g.inv.cash < cents {
                    return false;
                }
                g.inv.cash -= cents;
                g.inv.oxen += 2;
                true
            }),
        ));
    }

    if inv.total_spares() > 0 {
        let food = rng.range(100, 220) as u32;
        let part = if inv.wheels > 0 {
            Part::Wheel
        } else if inv.axles > 0 {
            Part::Axle
        } else {
            Part::Tongue
        };
        options.push((
            format!("your spare {}", part.name()),
            format!("{} pounds of food", food),
            Box::new(move |g: &mut Game| {
                if g.inv.spares(part) == 0 {
                    return false;
                }
                *g.inv.spares_mut(part) -= 1;
                g.inv.food += food;
                true
            }),
        ));
    }

    if options.is_empty() {
        return None;
    }
    let i = rng.below(options.len() as u32) as usize;
    Some(options.swap_remove(i))
}

/// When the team is gone, see whether anybody will help.
pub fn beg_for_oxen(g: &mut Game) -> bool {
    ui::blank();
    if g.inv.cash >= 3000 && g.rng.chance(0.5) {
        let cost = (g.rng.range(30, 70) as i64) * 100;
        let cost = cost.min(g.inv.cash);
        ui::wrap(&format!(
            "A passing train has a spare ox. The owner will part with it for {}.",
            ui::money(cost)
        ));
        if ui::yes_no("Buy the ox?") {
            g.inv.cash -= cost;
            g.inv.oxen += 1;
            println!("\nYou put the new ox in the yoke.");
            return true;
        }
    } else if g.rng.chance(0.3) {
        ui::wrap(
            "A family camped nearby takes pity on you and lends you an ox to \
             get as far as the next post.",
        );
        g.inv.oxen += 1;
        return true;
    } else {
        ui::wrap("No one comes along the road today.");
    }
    false
}

// ----------------------------------------------------- talking to people ---

pub fn talk_to_people(g: &mut Game) {
    let generic: [&str; 14] = [
        "\"Don't let your oxen drink the standing water. That's what killed the Harrows.\"",
        "\"We started with eleven in our family. There's six of us now.\"",
        "\"If you can read Independence Rock by the Fourth of July, you'll make it.\"",
        "\"Lighten that wagon. The road west is paved with what folks threw away.\"",
        "\"Three yoke of oxen is enough. Four is better. Two will kill you.\"",
        "\"Never ford a river you can't see the bottom of.\"",
        "\"Rest on the Sabbath. The ones who drove through died first.\"",
        "\"A man ought not come out here without a spare axle. Mark me on that.\"",
        "\"The cholera takes you in the morning and buries you by night.\"",
        "\"Trade for food whenever you can. Money won't feed you past Fort Hall.\"",
        "\"Buffalo chips burn hot and smell no worse than coal.\"",
        "\"Keep the children out of the wagon when it's moving. Wheels don't care.\"",
        "\"We've buried four since Fort Laramie. Don't push your people so hard.\"",
        "\"There's no doctor between here and the Willamette. You are the doctor.\"",
    ];
    ui::heading("Talking to people");
    let n = g.rng.range(2, 3) as u32;
    for _ in 0..n {
        let i = g.rng.below(generic.len() as u32) as usize;
        ui::wrap(generic[i]);
        ui::blank();
    }
    if !g.graves.is_empty() && g.rng.chance(0.4) {
        ui::wrap(&format!(
            "They offer their sympathies for {}.",
            g.graves[g.graves.len() - 1].name
        ));
        ui::blank();
    }
    if g.rations == Rations::BareBones {
        ui::wrap(
            "\"You all look half starved. Feed your people or you'll be burying \
             them.\"",
        );
        ui::blank();
    }
    if g.outcome == Outcome::Playing && g.inv.oxen < 4 {
        ui::wrap("\"That's a thin team you've got. Buy more at the next post.\"");
        ui::blank();
    }
    ui::pause();
}
