//! Hunting. A day of your life, a fistful of bullets, and a typing test.

use crate::game::Game;
use crate::trail::Terrain;
use crate::ui;
use std::time::Instant;

struct Quarry {
    name: &'static str,
    article: &'static str,
    min_lbs: u32,
    max_lbs: u32,
    /// Relative likelihood of running into this animal.
    weight: u32,
    /// How hard it is to actually hit.
    difficulty: f64,
}

#[rustfmt::skip]
const PLAINS_GAME: [Quarry; 7] = [
    Quarry { name: "buffalo",  article: "a",  min_lbs: 350, max_lbs: 600, weight: 14, difficulty: 0.35 },
    Quarry { name: "deer",     article: "a",  min_lbs: 40,  max_lbs: 80,  weight: 22, difficulty: 0.45 },
    Quarry { name: "antelope", article: "an", min_lbs: 35,  max_lbs: 60,  weight: 16, difficulty: 0.60 },
    Quarry { name: "jackrabbit", article: "a", min_lbs: 2, max_lbs: 5,   weight: 20, difficulty: 0.55 },
    Quarry { name: "prairie hen", article: "a", min_lbs: 2, max_lbs: 4,  weight: 14, difficulty: 0.50 },
    Quarry { name: "squirrel", article: "a",  min_lbs: 1,   max_lbs: 2,   weight: 10, difficulty: 0.55 },
    Quarry { name: "duck",     article: "a",  min_lbs: 2,   max_lbs: 4,   weight: 4,  difficulty: 0.52 },
];

#[rustfmt::skip]
const MOUNTAIN_GAME: [Quarry; 6] = [
    Quarry { name: "elk",      article: "an", min_lbs: 200, max_lbs: 400, weight: 16, difficulty: 0.45 },
    Quarry { name: "black bear", article: "a", min_lbs: 150, max_lbs: 280, weight: 8, difficulty: 0.55 },
    Quarry { name: "mule deer", article: "a", min_lbs: 60, max_lbs: 110, weight: 24, difficulty: 0.48 },
    Quarry { name: "bighorn sheep", article: "a", min_lbs: 90, max_lbs: 160, weight: 12, difficulty: 0.62 },
    Quarry { name: "grouse",   article: "a",  min_lbs: 1,   max_lbs: 3,   weight: 22, difficulty: 0.50 },
    Quarry { name: "rabbit",   article: "a",  min_lbs: 2,   max_lbs: 4,   weight: 18, difficulty: 0.55 },
];

#[rustfmt::skip]
const DESERT_GAME: [Quarry; 5] = [
    Quarry { name: "pronghorn", article: "a", min_lbs: 35, max_lbs: 65,  weight: 20, difficulty: 0.62 },
    Quarry { name: "mule deer", article: "a", min_lbs: 60, max_lbs: 100, weight: 14, difficulty: 0.52 },
    Quarry { name: "sage hen", article: "a",  min_lbs: 2,  max_lbs: 5,   weight: 26, difficulty: 0.50 },
    Quarry { name: "jackrabbit", article: "a", min_lbs: 2, max_lbs: 5,   weight: 28, difficulty: 0.52 },
    Quarry { name: "coyote",   article: "a",  min_lbs: 0,  max_lbs: 0,   weight: 12, difficulty: 0.58 },
];

/// The most meat five people can drag back to the wagon.
const CARRY_LIMIT: u32 = 100;

/// Minimum ammunition needed to bother going out.
const MIN_BULLETS: u32 = 10;

const SHOUTS: [&str; 5] = ["BANG", "POW", "WHAM", "BLAM", "CRACK"];

fn table_for(terrain: Terrain) -> &'static [Quarry] {
    match terrain {
        Terrain::Mountains => &MOUNTAIN_GAME,
        Terrain::Desert => &DESERT_GAME,
        _ => &PLAINS_GAME,
    }
}

pub fn go_hunting(g: &mut Game) {
    ui::heading("Hunting");

    if g.inv.bullets < MIN_BULLETS {
        ui::wrap(&format!(
            "You have only {} bullets. That is not enough to go hunting.",
            g.inv.bullets
        ));
        ui::pause();
        return;
    }

    let terrain = g.terrain();
    ui::wrap(&format!(
        "You take the rifle and walk out into the {}. You have {} bullets.",
        terrain.name(),
        g.inv.bullets
    ));

    // Is there anything out here at all?
    let mut game_chance = match terrain {
        Terrain::Plains => 0.88,
        Terrain::Highlands => 0.80,
        Terrain::Mountains => 0.74,
        Terrain::Desert => 0.60,
    };
    if g.ox_fatigue > 0 {
        // Overgrazed, overhunted country.
        game_chance -= 0.15;
    }

    if !g.rng.chance(game_chance) {
        ui::blank();
        ui::wrap("You hunt all day and see nothing but your own tracks.");
        g.pass_days(1, false);
        ui::pause();
        return;
    }

    // Choose the quarry.
    let table = table_for(terrain);
    let total: u32 = table.iter().map(|q| q.weight).sum();
    let mut roll = g.rng.below(total);
    let mut chosen = 0usize;
    for (i, q) in table.iter().enumerate() {
        if roll < q.weight {
            chosen = i;
            break;
        }
        roll -= q.weight;
    }
    let quarry = &table[chosen];

    ui::blank();
    ui::wrap(&format!(
        "You spot {} {}. Get ready...",
        quarry.article, quarry.name
    ));

    // The typing test.
    let shout = *g.rng.pick(&SHOUTS);
    ui::blank();
    print!("Press ENTER, then type the word as fast as you can. ");
    ui::flush();
    let _ = ui::read_line();
    ui::blank();
    println!("   ***  {}  ***", shout);
    ui::blank();
    let start = Instant::now();
    let typed = ui::prompt("> ");
    let elapsed = start.elapsed().as_secs_f64();

    let correct = typed.trim().eq_ignore_ascii_case(shout);

    // Reaction quality: 1.0 is excellent, 0.0 is hopeless.
    let quality = if !correct {
        0.0
    } else if elapsed <= 1.2 {
        1.0
    } else if elapsed >= 6.0 {
        0.05
    } else {
        1.0 - (elapsed - 1.2) / 4.8
    };

    let bullets_used = g.rng.range(4, 16) as u32;
    let bullets_used = g.inv.take_bullets(bullets_used);

    ui::blank();
    if !correct {
        println!("You fumble the shot. ({} bullets spent)", bullets_used);
        ui::wrap("The animal is gone before you get the rifle to your shoulder.");
        g.pass_days(1, false);
        ui::pause();
        return;
    }

    println!(
        "You fire in {:.1} seconds, spending {} bullets.",
        elapsed, bullets_used
    );

    let hit_chance = (quarry.difficulty + quality * 0.5).min(0.95);
    if !g.rng.chance(hit_chance) {
        ui::blank();
        ui::wrap(&format!(
            "You wound the {} but it gets away into the brush.",
            quarry.name
        ));
        g.pass_days(1, false);
        ui::pause();
        return;
    }

    if quarry.max_lbs == 0 {
        ui::blank();
        ui::wrap(&format!(
            "You kill the {}. There is nothing on it worth eating.",
            quarry.name
        ));
        g.pass_days(1, false);
        ui::pause();
        return;
    }

    let meat = g.rng.range(quarry.min_lbs as i32, quarry.max_lbs as i32) as u32;
    let carried = meat.min(CARRY_LIMIT);

    ui::blank();
    println!(
        "You shot {} {} -- about {} pounds of meat.",
        quarry.article,
        quarry.name,
        ui::group(i64::from(meat))
    );
    if carried < meat {
        println!(
            "Unfortunately you were able to carry only {} pounds back to the wagon.",
            carried
        );
    }
    g.inv.food += carried;
    g.party.adjust_all(g.rng.range(0, 3));
    println!(
        "You now have {} pounds of food.",
        ui::group(i64::from(g.inv.food))
    );

    g.pass_days(1, false);
    ui::pause();
}
