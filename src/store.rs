//! Matt's General Store in Independence, and the trading posts along the way.

use crate::inventory::{
    Inventory, Part, AMMO_BOX_PRICE, BULLETS_PER_BOX, CLOTHING_PRICE, FOOD_PRICE, OXEN_PER_YOKE,
    OXEN_YOKE_PRICE, PART_PRICE, WAGON_CAPACITY_LBS,
};
use crate::ui;

/// Apply a fort's markup and round to the nearest nickel.
fn priced(base: i64, markup: f64) -> i64 {
    let raw = (base as f64 * markup).round() as i64;
    ((raw + 2) / 5) * 5
}

struct Prices {
    yoke: i64,
    food: i64,
    clothing: i64,
    ammo: i64,
    part: i64,
}

impl Prices {
    fn new(markup: f64) -> Self {
        Prices {
            yoke: priced(OXEN_YOKE_PRICE, markup),
            food: priced(FOOD_PRICE, markup),
            clothing: priced(CLOTHING_PRICE, markup),
            ammo: priced(AMMO_BOX_PRICE, markup),
            part: priced(PART_PRICE, markup),
        }
    }
}

fn show_prices(p: &Prices, inv: &Inventory) {
    ui::rule();
    println!(
        "   1. Oxen                 {:>10} per yoke (2 oxen)   have {}",
        ui::money(p.yoke),
        inv.oxen
    );
    println!(
        "   2. Food                 {:>10} per pound           have {} lbs",
        ui::money(p.food),
        ui::group(i64::from(inv.food))
    );
    println!(
        "   3. Clothing             {:>10} per set             have {}",
        ui::money(p.clothing),
        inv.clothing
    );
    println!(
        "   4. Ammunition           {:>10} per box of {}      have {}",
        ui::money(p.ammo),
        BULLETS_PER_BOX,
        inv.bullets
    );
    println!(
        "   5. Spare wagon wheel    {:>10} each                have {}",
        ui::money(p.part),
        inv.wheels
    );
    println!(
        "   6. Spare wagon axle     {:>10} each                have {}",
        ui::money(p.part),
        inv.axles
    );
    println!(
        "   7. Spare wagon tongue   {:>10} each                have {}",
        ui::money(p.part),
        inv.tongues
    );
    ui::rule();
    println!(
        "   Money left: {:<16}  Load: {} / {} lbs",
        ui::money(inv.cash),
        ui::group(i64::from(inv.weight())),
        ui::group(i64::from(WAGON_CAPACITY_LBS))
    );
}

/// Buy `unit_label` at `unit_price` cents each; returns true if anything was bought.
fn buy(inv: &mut Inventory, unit_price: i64, unit_label: &str, max_units: i64) -> Option<u32> {
    if unit_price <= 0 {
        return None;
    }
    let affordable = inv.cash / unit_price;
    if affordable <= 0 {
        println!("\nYou cannot afford any {}.", unit_label);
        return None;
    }
    let cap = affordable.min(max_units);
    println!(
        "\nYou can afford {} {} at {} each.",
        ui::group(cap),
        unit_label,
        ui::money(unit_price)
    );
    let n = ui::prompt_int(
        &format!(
            "How many {} would you like to buy? (0 to cancel) ",
            unit_label
        ),
        0,
        cap,
        Some(0),
    );
    if n == 0 {
        return None;
    }
    let cost = n * unit_price;
    inv.cash -= cost;
    println!("That will be {}. Thank you kindly.", ui::money(cost));
    Some(n as u32)
}

/// The shared shopping loop. `require_oxen` forces at least one yoke before
/// the player is allowed to leave (used at Independence).
fn shop(
    inv: &mut Inventory,
    party_size: usize,
    markup: f64,
    require_oxen: bool,
    leave_label: &str,
) {
    let p = Prices::new(markup);
    loop {
        ui::blank();
        show_prices(&p, inv);
        ui::blank();
        println!("   8. {}", leave_label);
        ui::blank();
        let choice = ui::prompt_int("Which number? ", 1, 8, None);
        match choice {
            1 => {
                if let Some(yokes) = buy(inv, p.yoke, "yoke of oxen", 12) {
                    inv.oxen += yokes * OXEN_PER_YOKE;
                }
            }
            2 => {
                if let Some(lbs) = buy(inv, p.food, "pounds of food", 4000) {
                    inv.food += lbs;
                }
            }
            3 => {
                if let Some(sets) = buy(inv, p.clothing, "sets of clothing", 40) {
                    inv.clothing += sets;
                }
            }
            4 => {
                if let Some(boxes) = buy(inv, p.ammo, "boxes of ammunition", 99) {
                    inv.bullets += boxes * BULLETS_PER_BOX;
                }
            }
            5..=7 => {
                let part = match choice {
                    5 => Part::Wheel,
                    6 => Part::Axle,
                    _ => Part::Tongue,
                };
                let label = format!("spare {}s", part.name());
                if let Some(n) = buy(inv, p.part, &label, 9) {
                    *inv.spares_mut(part) += n;
                }
            }
            _ => {
                if require_oxen && inv.oxen < OXEN_PER_YOKE {
                    println!(
                        "\n\"Hold on there. You'll not get ten miles without at least\n\
                          one yoke of oxen to pull that wagon.\""
                    );
                    continue;
                }
                if inv.is_overloaded() {
                    println!(
                        "\n\"That wagon is loaded to {} pounds. She'll break an axle\n\
                          before the Platte if you don't lighten her.\"",
                        ui::group(i64::from(inv.weight()))
                    );
                    if !ui::yes_no("Leave anyway?") {
                        continue;
                    }
                }
                if require_oxen && inv.food < 150 {
                    println!(
                        "\n\"Only {} pounds of food? That won't feed {} people to Kansas.\"",
                        inv.food, party_size
                    );
                    if !ui::yes_no("Leave anyway?") {
                        continue;
                    }
                }
                return;
            }
        }
    }
}

/// Outfitting in Independence, Missouri.
pub fn outfit_at_independence(inv: &mut Inventory, party_size: usize) {
    ui::banner("Matt's General Store");
    ui::wrap(
        "\"Welcome to my store. My name is Matt. I can fit you out with \
         everything you'll need for the trip to Oregon. Only the best goods, \
         and cheaper than anyone in Independence.",
    );
    ui::blank();
    ui::wrap(
        "\"You'll want a good team of oxen -- at least one yoke, and three or \
         four yoke if you can manage it. Plenty of food; folks always \
         underestimate how much. Clothing for the cold, and ammunition for \
         hunting. And spare parts, because wagons break.",
    );
    ui::blank();
    ui::wrap("\"Now then. What'll it be?\"");
    ui::pause();

    shop(inv, party_size, 1.0, true, "Leave the store and head west");

    ui::blank();
    ui::wrap("\"Good luck to you. You'll need it.\"");
    ui::pause();
}

/// Shopping at a fort or trading post further along the trail.
pub fn shop_at_fort(inv: &mut Inventory, party_size: usize, fort_name: &str, markup: f64) {
    ui::banner(&format!("Trading post at {}", fort_name));
    ui::wrap(
        "Everything here has been hauled a long way by wagon, and the prices \
         show it. Still, it may be better than going without.",
    );
    shop(inv, party_size, markup, false, "Leave the trading post");
}
