//! Scoring and the Oregon Top Ten.

use crate::date::Date;
use crate::game::{Game, Outcome, PARTY_SIZE};
use crate::party::HealthRating;
use crate::ui;

use std::fs;
use std::path::PathBuf;

const SCORE_FILE: &str = "ortrail_top_ten.txt";

pub struct ScoreLine {
    pub label: String,
    pub detail: String,
    pub points: i64,
}

pub struct Breakdown {
    pub lines: Vec<ScoreLine>,
    pub subtotal: i64,
    pub multiplier: i64,
    pub total: i64,
}

pub fn compute(g: &Game) -> Breakdown {
    let mut lines: Vec<ScoreLine> = Vec::new();

    // Only an arriving party scores for people and goods.
    let arrived = g.outcome == Outcome::Arrived;

    if arrived {
        // People, grouped by health.
        for rating in [
            HealthRating::Good,
            HealthRating::Fair,
            HealthRating::Poor,
            HealthRating::VeryPoor,
        ] {
            let n = g
                .party
                .members
                .iter()
                .filter(|p| p.alive && p.rating() == rating)
                .count() as i64;
            if n > 0 {
                lines.push(ScoreLine {
                    label: format!("People in {} health", rating.name()),
                    detail: format!("{} x {}", n, rating.score_points()),
                    points: n * rating.score_points(),
                });
            }
        }

        lines.push(ScoreLine {
            label: "Wagon".to_string(),
            detail: "1 x 50".to_string(),
            points: 50,
        });

        let oxen = i64::from(g.inv.oxen);
        if oxen > 0 {
            lines.push(ScoreLine {
                label: "Oxen".to_string(),
                detail: format!("{} x 4", oxen),
                points: oxen * 4,
            });
        }

        let spares = i64::from(g.inv.total_spares());
        if spares > 0 {
            lines.push(ScoreLine {
                label: "Spare parts".to_string(),
                detail: format!("{} x 2", spares),
                points: spares * 2,
            });
        }

        let clothing = i64::from(g.inv.clothing);
        if clothing > 0 {
            lines.push(ScoreLine {
                label: "Sets of clothing".to_string(),
                detail: format!("{} x 2", clothing),
                points: clothing * 2,
            });
        }

        let bullets = i64::from(g.inv.bullets);
        if bullets >= 50 {
            lines.push(ScoreLine {
                label: "Ammunition".to_string(),
                detail: format!("{} bullets, 1 per 50", bullets),
                points: bullets / 50,
            });
        }

        let food = i64::from(g.inv.food);
        if food >= 25 {
            lines.push(ScoreLine {
                label: "Food".to_string(),
                detail: format!("{} lbs, 1 per 25", ui::group(food)),
                points: food / 25,
            });
        }

        let cash = g.inv.cash;
        if cash >= 500 {
            lines.push(ScoreLine {
                label: "Cash".to_string(),
                detail: format!("{}, 1 per $5", ui::money(cash)),
                points: cash / 500,
            });
        }

        // A small bonus for arriving before the snow closes the passes.
        if g.date.month <= 9 {
            lines.push(ScoreLine {
                label: "Arrived before October".to_string(),
                detail: "bonus".to_string(),
                points: 100,
            });
        }
    } else {
        // Partial credit for distance covered.
        let miles = i64::from(g.miles_total);
        lines.push(ScoreLine {
            label: "Distance travelled".to_string(),
            detail: format!("{} miles, 1 per 20", ui::group(miles)),
            points: miles / 20,
        });
        let survivors = g.party.alive_count() as i64;
        if survivors > 0 {
            lines.push(ScoreLine {
                label: "Survivors".to_string(),
                detail: format!("{} x 25", survivors),
                points: survivors * 25,
            });
        }
    }

    let subtotal: i64 = lines.iter().map(|l| l.points).sum();
    let multiplier = g.occupation.score_multiplier();
    Breakdown {
        lines,
        subtotal,
        multiplier,
        total: subtotal * multiplier,
    }
}

pub fn show_final(g: &Game) {
    let b = compute(g);

    ui::banner("How you did");
    for l in &b.lines {
        println!("   {:<30} {:<24} {:>6}", l.label, l.detail, l.points);
    }
    ui::rule();
    println!("   {:<55} {:>6}", "Subtotal", b.subtotal);
    println!(
        "   {:<55} {:>6}",
        format!("Occupation multiplier ({})", g.occupation.short_name()),
        format!("x{}", b.multiplier)
    );
    ui::thick_rule();
    println!("   {:<55} {:>6}", "TOTAL POINTS", b.total);
    ui::thick_rule();

    ui::blank();
    println!(
        "   Journey: {} miles in {} days, finishing on {}.",
        ui::group(i64::from(g.miles_total)),
        g.days_on_trail,
        g.date
    );
    println!("   Survivors: {} of {}.", g.party.alive_count(), PARTY_SIZE);
    if !g.graves.is_empty() {
        ui::blank();
        println!("   You buried:");
        for grave in &g.graves {
            println!(
                "      {:<16} {:<24} {}",
                grave.name, grave.cause, grave.date
            );
        }
    }
    ui::pause();

    if b.total > 0 {
        record(
            &g.party.leader_name(),
            g.occupation.short_name(),
            b.total,
            g.date,
        );
    }
    show_top_ten();
}

// --------------------------------------------------------- persistence ---

fn score_path() -> PathBuf {
    PathBuf::from(SCORE_FILE)
}

struct Entry {
    name: String,
    occupation: String,
    points: i64,
    date: String,
}

fn load() -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();
    if let Ok(text) = fs::read_to_string(score_path()) {
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 4 {
                if let Ok(points) = parts[2].trim().parse::<i64>() {
                    entries.push(Entry {
                        name: parts[0].to_string(),
                        occupation: parts[1].to_string(),
                        points,
                        date: parts[3].to_string(),
                    });
                }
            }
        }
    }
    entries.sort_by_key(|a| std::cmp::Reverse(a.points));
    entries.truncate(10);
    entries
}

fn save(entries: &[Entry]) {
    let mut out = String::new();
    for e in entries {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            e.name, e.occupation, e.points, e.date
        ));
    }
    let _ = fs::write(score_path(), out);
}

pub fn record(name: &str, occupation: &str, points: i64, date: Date) {
    let mut entries = load();
    entries.push(Entry {
        name: name.to_string(),
        occupation: occupation.to_string(),
        points,
        date: date.to_string(),
    });
    entries.sort_by_key(|a| std::cmp::Reverse(a.points));
    entries.truncate(10);
    save(&entries);
}

pub fn show_top_ten() {
    let entries = load();
    ui::banner("The Oregon Top Ten");
    if entries.is_empty() {
        ui::wrap("No one has made the journey yet.");
    } else {
        println!(
            "   {:<4} {:<16} {:<12} {:<18} {:>7}",
            "", "Name", "Rating", "Arrived", "Points"
        );
        ui::rule();
        for (i, e) in entries.iter().enumerate() {
            println!(
                "   {:<4} {:<16} {:<12} {:<18} {:>7}",
                format!("{}.", i + 1),
                e.name,
                e.occupation,
                e.date,
                ui::group(e.points)
            );
        }
        ui::rule();
        println!("   Scores are kept in {}", SCORE_FILE);
    }
    ui::pause();
}
