//! River crossings -- the part everybody remembers.

use crate::game::{Game, Outcome};
use crate::trail::RiverSpec;
use crate::ui;

/// Describe how fast the water is running.
fn swiftness(depth: f64, width: u32) -> (&'static str, f64) {
    let v = depth * 1.2 + f64::from(width) / 700.0;
    if v < 2.5 {
        ("slow", 0.6)
    } else if v < 4.0 {
        ("moderate", 1.0)
    } else if v < 6.0 {
        ("swift", 1.5)
    } else {
        ("very swift", 2.1)
    }
}

/// Landmark names end in "Crossing", which reads badly in a sentence.
/// "Kansas River Crossing" -> "Kansas River".
fn prose_name(name: &str) -> &str {
    name.strip_suffix(" Crossing").unwrap_or(name)
}

fn describe(g: &Game, name: &str, depth: f64, spec: &RiverSpec) {
    let (swift, _) = swiftness(depth, spec.width_ft);
    ui::heading(name);
    println!(
        "   Width of river:     {} feet",
        ui::group(i64::from(spec.width_ft))
    );
    println!("   Depth of river:     {:.1} feet", depth);
    println!("   Current:            {}", swift);
    println!("   Weather:            {}", g.weather.describe());
    println!(
        "   Your wagon:         {} lbs, {} oxen",
        ui::group(i64::from(g.inv.weight())),
        g.inv.oxen
    );
}

/// Current depth of the river, given recent weather.
fn current_depth(g: &mut Game, spec: &RiverSpec) -> f64 {
    let seasonal = match g.date.month {
        4..=6 => 1.5,
        3 | 7 => 0.6,
        _ => 0.0,
    };
    let noise = g.rng.range_f(-0.6, 0.9);
    (spec.base_depth + seasonal + g.weather.cond.river_rise() + noise).max(0.4)
}

pub fn cross(g: &mut Game, name: &'static str, spec: RiverSpec) {
    let mut depth = current_depth(g, &spec);
    let mut waited = 0u32;

    loop {
        if g.is_over() {
            return;
        }

        describe(g, name, depth, &spec);

        let ferry_label = format!("Take a ferry across (costs {})", ui::money(spec.ferry_cost));
        let guide_label = format!(
            "Hire a local guide (costs {})",
            ui::plural(spec.guide_clothing, "set of clothing", "sets of clothing")
        );

        let mut options: Vec<String> = vec![
            "Attempt to ford the river".to_string(),
            "Caulk the wagon and float it across".to_string(),
        ];
        if spec.has_ferry {
            options.push(ferry_label);
        }
        if spec.has_guide {
            options.push(guide_label);
        }
        options.push("Wait a day to see if conditions improve".to_string());
        options.push("Get more information about the river".to_string());

        let refs: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
        let choice = ui::menu("You may:", &refs);
        let picked = refs[choice - 1].to_string();

        if picked.starts_with("Attempt to ford") {
            if ford(g, name, depth, &spec) {
                return;
            }
        } else if picked.starts_with("Caulk") {
            if caulk(g, name, depth, &spec) {
                return;
            }
        } else if picked.starts_with("Take a ferry") {
            if ferry(g, name, &spec) {
                return;
            }
        } else if picked.starts_with("Hire a local guide") {
            if guide(g, name, &spec) {
                return;
            }
        } else if picked.starts_with("Wait a day") {
            waited += 1;
            g.pass_days(1, true);
            if g.is_over() {
                return;
            }
            let old = depth;
            depth = (current_depth(g, &spec) * 0.6 + depth * 0.4).max(0.4);
            ui::blank();
            if depth < old - 0.2 {
                println!("The water has dropped a little overnight.");
            } else if depth > old + 0.2 {
                println!("The water has risen overnight.");
            } else {
                println!("The river looks much the same as yesterday.");
            }
            if waited >= 4 {
                ui::wrap("You cannot sit on this bank forever. Winter is coming on.");
            }
        } else {
            information(g, name, depth, &spec);
        }
    }
}

fn information(g: &Game, name: &str, depth: f64, spec: &RiverSpec) {
    let (swift, _) = swiftness(depth, spec.width_ft);
    ui::heading(&format!("About the {}", prose_name(name)));
    ui::wrap(&format!(
        "The river here is {} feet across and about {:.1} feet deep, with a {} \
         current.",
        ui::group(i64::from(spec.width_ft)),
        depth,
        swift
    ));
    ui::blank();
    if depth < 2.5 {
        ui::wrap("\"Shallow as that, you can drive straight through. Mind the holes.\"");
    } else if depth < 4.0 {
        ui::wrap(
            "\"I wouldn't ford that. Water over three feet will float a loaded \
             wagon and then it goes where the river wants.\"",
        );
    } else {
        ui::wrap(
            "\"Ford that and you'll drown your team. Caulk her and float, or pay \
             the ferryman.\"",
        );
    }
    ui::blank();
    if spec.has_ferry {
        ui::wrap(&format!(
            "\"The ferry is {} the wagon. Safe enough, but there's a line of \
             wagons waiting.\"",
            ui::money(spec.ferry_cost)
        ));
    } else {
        ui::wrap("\"There's no ferry on this river. Never has been.\"");
    }
    if spec.has_guide {
        ui::blank();
        ui::wrap(&format!(
            "\"The people who live here will take your outfit over for {}. \
             They've done it a thousand times.\"",
            ui::plural(spec.guide_clothing, "set of clothing", "sets of clothing")
        ));
    }
    let _ = g;
    ui::pause();
}

/// Returns true when the crossing is finished (one way or another).
fn ford(g: &mut Game, name: &str, depth: f64, spec: &RiverSpec) -> bool {
    let (_, swift_mult) = swiftness(depth, spec.width_ft);
    ui::blank();
    println!("You start across, the oxen chest-deep in the brown water...");

    // Wagons float at roughly three feet; past that it is a lottery.
    let mut risk = if depth <= 2.0 {
        0.03
    } else if depth <= 3.0 {
        0.18
    } else if depth <= 4.0 {
        0.48
    } else {
        0.75
    };
    risk *= 0.6 + swift_mult * 0.4;
    if g.inv.is_overloaded() {
        risk += 0.08;
    }
    if g.inv.oxen < 4 {
        risk += 0.07;
    }
    risk = risk.min(0.92);

    g.pass_days(1, false);
    if g.is_over() {
        return true;
    }

    if g.rng.chance(risk) {
        disaster(g, name, if depth > 4.0 { 1.3 } else { 1.0 });
    } else {
        ui::blank();
        println!("You ford the {} without mishap.", prose_name(name));
        ui::pause();
    }
    true
}

fn caulk(g: &mut Game, name: &str, depth: f64, spec: &RiverSpec) -> bool {
    let (_, swift_mult) = swiftness(depth, spec.width_ft);
    ui::blank();
    println!("You caulk the seams with tar, unhitch the team, and float the wagon...");

    let mut risk = if depth < 2.0 {
        // Too shallow -- you scrape and stick.
        0.22
    } else if depth <= 6.0 {
        0.16
    } else {
        0.30
    };
    risk *= 0.5 + swift_mult * 0.5;
    if g.inv.is_overloaded() {
        risk += 0.12;
    }
    risk = risk.min(0.85);

    let days = 1 + g.rng.below(2);
    g.pass_days(days, false);
    if g.is_over() {
        return true;
    }

    if g.rng.chance(risk) {
        disaster(g, name, 0.9);
    } else {
        ui::blank();
        println!(
            "The wagon floats across like a scow. You are over the {}.",
            prose_name(name)
        );
        ui::pause();
    }
    true
}

fn ferry(g: &mut Game, name: &str, spec: &RiverSpec) -> bool {
    if !g.inv.can_afford(spec.ferry_cost) {
        ui::blank();
        println!(
            "The ferryman wants {} and you have {}. He waves you away.",
            ui::money(spec.ferry_cost),
            ui::money(g.inv.cash)
        );
        ui::pause();
        return false;
    }
    let wait = g.rng.range(1, 5) as u32;
    ui::blank();
    println!(
        "You pay {} and take your place in line. There are wagons ahead of you; \
         the wait is {}.",
        ui::money(spec.ferry_cost),
        ui::plural(wait, "day", "days")
    );
    g.inv.cash -= spec.ferry_cost;
    g.pass_days(wait + 1, true);
    if g.is_over() {
        return true;
    }

    if g.rng.chance(0.05) {
        ui::blank();
        println!("The ferry swings in the current and takes water!");
        disaster(g, name, 0.6);
    } else {
        ui::blank();
        println!(
            "The ferry sets you safely on the far bank of the {}.",
            prose_name(name)
        );
        ui::pause();
    }
    true
}

fn guide(g: &mut Game, name: &str, spec: &RiverSpec) -> bool {
    if g.inv.clothing < spec.guide_clothing {
        ui::blank();
        println!(
            "They want {} and you have only {}. They shake their heads.",
            ui::plural(spec.guide_clothing, "set of clothing", "sets of clothing"),
            g.inv.clothing
        );
        ui::pause();
        return false;
    }
    g.inv.take_clothing(spec.guide_clothing);
    ui::blank();
    println!(
        "You hand over {} and follow them upstream to a gravel bar nobody else \
         seems to know about.",
        ui::plural(spec.guide_clothing, "set of clothing", "sets of clothing")
    );
    g.pass_days(2, false);
    if g.is_over() {
        return true;
    }

    if g.rng.chance(0.08) {
        disaster(g, name, 0.5);
    } else {
        ui::blank();
        println!(
            "The guides take you over the {} without the least trouble.",
            prose_name(name)
        );
        ui::pause();
    }
    true
}

/// Something has gone wrong in the water.
fn disaster(g: &mut Game, name: &str, severity: f64) {
    ui::blank();
    ui::thick_rule();
    ui::center("THE WAGON HAS TIPPED OVER IN THE RIVER");
    ui::thick_rule();
    ui::blank();

    let mut losses: Vec<String> = Vec::new();

    let food_pct = g.rng.range_f(0.2, 0.75) * severity;
    let lost_food = ((f64::from(g.inv.food) * food_pct) as u32).min(g.inv.food);
    g.inv.take_food(lost_food);
    if lost_food > 0 {
        losses.push(format!(
            "{} pounds of food",
            ui::group(i64::from(lost_food))
        ));
    }

    if g.rng.chance(0.6 * severity) {
        let lost = g.inv.take_bullets((f64::from(g.inv.bullets) * 0.5) as u32);
        if lost > 0 {
            losses.push(format!("{} bullets", lost));
        }
    }

    if g.rng.chance(0.5 * severity) {
        let lost = g.inv.take_clothing(g.rng.range(1, 3) as u32);
        if lost > 0 {
            losses.push(ui::plural(lost, "set of clothing", "sets of clothing"));
        }
    }

    if g.rng.chance(0.45 * severity) && g.inv.oxen > 0 {
        let lost = g.inv.take_oxen(g.rng.range(1, 2) as u32);
        losses.push(ui::plural(lost, "ox", "oxen"));
    }

    if g.rng.chance(0.3 * severity) {
        for part in crate::inventory::Part::ALL.iter() {
            if g.inv.spares(*part) > 0 && g.rng.chance(0.5) {
                *g.inv.spares_mut(*part) -= 1;
                losses.push(format!("a spare {}", part.name()));
            }
        }
    }

    if losses.is_empty() {
        println!("By great luck, nothing is lost but your dignity.");
    } else {
        ui::wrap(&format!(
            "The {} takes {}.",
            prose_name(name),
            ui::list_and(&losses)
        ));
    }

    g.party.adjust_all(-(g.rng.range(5, 15)));

    // Somebody may drown.
    if g.rng.chance(0.28 * severity) {
        if let Some(i) = g.party.random_living(&mut g.rng) {
            g.kill(i, "drowning");
            if g.is_over() {
                return;
            }
        }
    }

    let days = g.rng.range(1, 4) as u32;
    ui::blank();
    println!(
        "You spend {} drying out and repacking what is left.",
        ui::plural(days, "day", "days")
    );
    g.pass_days(days, true);
    if !g.is_over() {
        ui::pause();
    }
}

// --------------------------------------------------- floating the Columbia ---

/// The final gamble: raft the wagon down the Columbia to Oregon City.
pub fn float_columbia(g: &mut Game) {
    ui::banner("The Columbia River");
    ui::wrap(
        "You break the wagon down, lash it to a raft of green logs, and push \
         out into the Columbia. The current takes you at once.",
    );
    ui::pause();

    let mut hazards = 4;
    let mut miles_run = 0u32;

    while hazards > 0 {
        hazards -= 1;
        g.pass_days(1, false);
        if g.is_over() {
            return;
        }
        miles_run += 25;
        g.miles_total += 25;

        let kind = g.rng.below(5);
        ui::blank();
        match kind {
            0 => {
                ui::wrap(
                    "The river narrows between black walls of rock and the raft \
                     begins to buck.",
                );
                if g.rng.chance(0.35) {
                    rapids_damage(g, 1.0);
                } else {
                    println!("You shoot through and come out the other side soaked but whole.");
                }
            }
            1 => {
                ui::wrap("A submerged snag catches the corner of the raft.");
                if g.rng.chance(0.3) {
                    rapids_damage(g, 0.8);
                } else {
                    println!("You pry loose with a pole and swing clear.");
                }
            }
            2 => {
                ui::wrap(
                    "Rain comes down in sheets. You cannot see the bank, let \
                     alone the channel.",
                );
                if g.rng.chance(0.25) {
                    rapids_damage(g, 0.7);
                } else {
                    println!("You run blind for an hour and strike nothing.");
                }
            }
            3 => {
                ui::wrap(
                    "The Cascades of the Columbia -- a mile of white water. There \
                     is no going back now.",
                );
                if g.rng.chance(0.45) {
                    rapids_damage(g, 1.3);
                } else {
                    println!("The raft holds together. Somebody is laughing and crying at once.");
                }
            }
            _ => {
                ui::wrap(
                    "The river widens and slows. Indians in canoes come out to \
                     trade smoked salmon.",
                );
                let food = g.rng.range(10, 40) as u32;
                g.inv.food += food;
                println!("You take on {} pounds of food.", food);
            }
        }
        if g.is_over() {
            return;
        }
        ui::pause();
    }

    let _ = miles_run;
    g.outcome = Outcome::Arrived;
}

fn rapids_damage(g: &mut Game, severity: f64) {
    ui::blank();
    println!("The raft strikes hard and water pours over the load!");
    let pct = g.rng.range_f(0.15, 0.6) * severity;
    let lost = ((f64::from(g.inv.food) * pct) as u32).min(g.inv.food);
    g.inv.take_food(lost);
    if lost > 0 {
        println!(
            "   {} pounds of food go into the river.",
            ui::group(i64::from(lost))
        );
    }
    if g.rng.chance(0.5) {
        let b = g.inv.take_bullets((f64::from(g.inv.bullets) * 0.4) as u32);
        if b > 0 {
            println!("   {} bullets are lost.", b);
        }
    }
    if g.rng.chance(0.4) {
        let c = g.inv.take_clothing(g.rng.range(1, 3) as u32);
        if c > 0 {
            println!(
                "   {} washed away.",
                ui::plural(c, "set of clothing is", "sets of clothing are")
            );
        }
    }
    g.party.adjust_all(-(g.rng.range(6, 16)));
    if g.rng.chance(0.3 * severity) {
        if let Some(i) = g.party.random_living(&mut g.rng) {
            g.kill(i, "drowning in the Columbia");
        }
    }
}
