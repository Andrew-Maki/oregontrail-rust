//! The Oregon Trail -- a text-mode clone of the 1848 wagon-train classic.

mod date;
mod events;
mod game;
mod hunt;
mod inventory;
mod party;
mod river;
mod rng;
mod score;
mod store;
mod trail;
mod ui;
mod weather;

use game::Game;

fn main() {
    title_screen();

    loop {
        let choice = ui::menu(
            "The Oregon Trail",
            &[
                "Travel the trail",
                "Learn about the trail",
                "See the Oregon Top Ten",
                "End",
            ],
        );
        match choice {
            1 => {
                if let Some(mut g) = Game::new_from_setup() {
                    g.run();
                }
            }
            2 => learn_about_the_trail(),
            3 => score::show_top_ten(),
            _ => {
                ui::blank();
                ui::center("Good luck, and watch the water.");
                ui::blank();
                return;
            }
        }
    }
}

fn title_screen() {
    ui::blank();
    ui::thick_rule();
    ui::blank();
    ui::center("T H E   O R E G O N   T R A I L");
    ui::blank();
    ui::center("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    ui::blank();
    for line in WAGON_ART.lines() {
        println!("{}", line);
    }
    ui::blank();
    ui::center("Independence, Missouri  ->  Oregon's Willamette Valley");
    ui::center("1,900 miles. Five people. One wagon.");
    ui::blank();
    ui::thick_rule();
    ui::pause();
}

const WAGON_ART: &str = r#"
                             _.-""""-._
                          .-'  _    _  '-.
                        .'    (_)  (_)    '.
                      _/_______________________\_
                     |   ___________________    |
                     |  |                   |   |
            __       |  |                   |   |       __
          .'  '.     |__|___________________|___|     .'  '.
         /  ()  \____|___|___________________|___|___/  ()  \
        |   __   |                                   |   __   |
         \ (__) /                                     \ (__) /
          '.__.'                                       '.__.'
"#;

fn learn_about_the_trail() {
    loop {
        let choice = ui::menu(
            "Learn about the trail",
            &[
                "What the journey is",
                "Choosing an occupation",
                "What to buy at the store",
                "Pace and rations",
                "Sickness and injury",
                "Crossing rivers",
                "How points are scored",
                "Back to the main menu",
            ],
        );
        match choice {
            1 => {
                ui::heading("The journey");
                ui::wrap(
                    "In 1848 some twelve thousand people are on the road to \
                     Oregon. You will start from Independence, Missouri, in the \
                     spring and try to reach the Willamette Valley before the \
                     snow closes the Cascade passes -- about 1,900 miles.",
                );
                ui::blank();
                ui::wrap(
                    "You travel by landmark: the Kansas River, Fort Kearney, \
                     Chimney Rock, Fort Laramie, Independence Rock, South Pass, \
                     the Green River, Fort Hall, the Snake River, Fort Boise, \
                     the Blue Mountains and The Dalles. At each one you may \
                     rest, hunt, trade, or buy supplies if there is anyone to \
                     sell them.",
                );
                ui::blank();
                ui::wrap(
                    "Roughly one emigrant in ten died on the way. Most of them \
                     of disease, not of anything dramatic.",
                );
            }
            2 => {
                ui::heading("Choosing an occupation");
                ui::wrap(
                    "A BANKER begins with $1,600 and scores single points. He \
                     can buy his way out of most trouble, but cannot mend a \
                     wagon to save his life.",
                );
                ui::blank();
                ui::wrap(
                    "A CARPENTER begins with $800 and scores double points. He \
                     can usually repair a broken wheel, axle or tongue even \
                     without a spare.",
                );
                ui::blank();
                ui::wrap(
                    "A FARMER begins with $400 and scores triple points. It is \
                     the hardest start and the best score.",
                );
            }
            3 => {
                ui::heading("Outfitting the wagon");
                ui::wrap(
                    "OXEN pull the wagon. Two oxen make a yoke. Four yoke (eight \
                     oxen) is a full team and will make about twenty miles a \
                     day; fewer, and you crawl. Oxen die, so buy spares.",
                );
                ui::blank();
                ui::wrap(
                    "FOOD is the thing people underestimate. Five people on \
                     filling rations eat fifteen pounds a day, and the trip \
                     takes four or five months. A thousand pounds is a sensible \
                     start; hunting can make up the rest.",
                );
                ui::blank();
                ui::wrap(
                    "CLOTHING matters in the mountains and at the end of the \
                     season. One set per person is the minimum; two is better.",
                );
                ui::blank();
                ui::wrap(
                    "AMMUNITION comes in boxes of twenty. You cannot hunt \
                     without it, and hunting is how most parties avoid \
                     starving.",
                );
                ui::blank();
                ui::wrap(
                    "SPARE PARTS -- a wheel, an axle and a tongue -- cost ten \
                     dollars each and will save you a week apiece.",
                );
                ui::blank();
                ui::wrap(
                    "The wagon holds about 2,400 pounds. Overload it and the \
                     oxen suffer, you travel slowly, and things break.",
                );
            }
            4 => {
                ui::heading("Pace and rations");
                ui::wrap(
                    "A STEADY pace covers normal ground and keeps people well. \
                     A STRENUOUS pace is a quarter faster and wears the party \
                     down. A GRUELING pace is half again as fast and will make \
                     your people sick, then kill them.",
                );
                ui::blank();
                ui::wrap(
                    "FILLING rations are three pounds per person per day and the \
                     party slowly gains health. MEAGER rations are two pounds \
                     and hold steady. BARE BONES rations are one pound and the \
                     party weakens every day.",
                );
                ui::blank();
                ui::wrap(
                    "The usual mistake is to run grueling and bare bones at the \
                     same time. That is how whole families ended up under \
                     boards beside the Platte.",
                );
            }
            5 => {
                ui::heading("Sickness and injury");
                ui::wrap(
                    "Cholera, typhoid, dysentery, measles and fever all travelled \
                     the trail. Bad water brings them on; so does exhaustion \
                     and poor feeding. A sick traveller loses health every day \
                     until they recover or die.",
                );
                ui::blank();
                ui::wrap(
                    "Resting is the only medicine you have. Stopping for a week \
                     costs you a week -- and may save a life. Broken limbs heal \
                     slowly and slow the whole wagon down.",
                );
                ui::blank();
                ui::wrap("If YOU die, the journey is over. The others turn back.");
            }
            6 => {
                ui::heading("Crossing rivers");
                ui::wrap(
                    "You may FORD the river -- drive straight through. This is \
                     safe in shallow water and suicidal past three feet, because \
                     a loaded wagon floats and then goes where the current puts \
                     it.",
                );
                ui::blank();
                ui::wrap(
                    "You may CAULK THE WAGON and float it across. This works in \
                     deep water but is a gamble in a swift current, and useless \
                     where the river is too shallow to float.",
                );
                ui::blank();
                ui::wrap(
                    "You may take a FERRY if there is one, which is safe but \
                     costs money and several days of waiting. Or you may HIRE A \
                     GUIDE, paying in clothing, which is the safest crossing of \
                     all if you can afford the shirts.",
                );
                ui::blank();
                ui::wrap(
                    "You may also WAIT, and hope the water drops. Always ask for \
                     information first. The river tells you the depth; believe \
                     it.",
                );
            }
            7 => {
                ui::heading("Scoring");
                ui::wrap("If you reach Oregon you are awarded points for:");
                ui::blank();
                println!("   Each surviving person, by health:");
                println!("      good health .................. 500 points");
                println!("      fair health .................. 400 points");
                println!("      poor health .................. 300 points");
                println!("      very poor health ............. 200 points");
                println!("   The wagon ....................... 50 points");
                println!("   Each ox ......................... 4 points");
                println!("   Each spare part ................. 2 points");
                println!("   Each set of clothing ............ 2 points");
                println!("   Each 50 bullets ................. 1 point");
                println!("   Each 25 pounds of food .......... 1 point");
                println!("   Each $5 in cash ................. 1 point");
                println!("   Arriving before October ......... 100 points");
                ui::blank();
                ui::wrap(
                    "The total is then multiplied by one for a banker, two for a \
                     carpenter and three for a farmer. If you fail to reach \
                     Oregon you get a small amount of credit for the distance \
                     you covered.",
                );
            }
            _ => return,
        }
        ui::pause();
    }
}
