# OrTrail

A text-mode clone of the classic **Oregon Trail**, written in Rust with **zero
dependencies**.

```
                             _.-""""-._
                          .-'  _    _  '-.
                        .'    (_)  (_)    '.
                      _/_______________________\_
                     |   ___________________    |
            __       |  |                   |   |       __
          .'  '.     |__|___________________|___|     .'  '.
         /  ()  \____|___|___________________|___|___/  ()  \
          '.__.'                                       '.__.'
```

It is 1848. You have five people, one wagon, and about 1,900 miles between
Independence, Missouri and Oregon's Willamette Valley.

## Running it

```sh
cargo run --release
```

No crates are pulled in — the game ships its own `xorshift64*` PRNG — so it
builds offline on any machine with a Rust toolchain.

## How to play

**Pick an occupation.** This sets your starting money *and* your score
multiplier, so the poorest start is worth the most points:

| Occupation | Starting cash | Multiplier | Perk |
|---|---|---|---|
| Banker from Boston | $1,600 | ×1 | Can buy out of most trouble |
| Carpenter from Ohio | $800 | ×2 | Usually repairs a broken part without a spare |
| Farmer from Illinois | $400 | ×3 | Best at keeping a team alive |

**Pick a departure month.** April or May are correct. March means no grass for
the oxen; July means snow in the Blue Mountains.

**Outfit the wagon** at Matt's General Store — oxen ($40/yoke), food
($0.20/lb), clothing ($10/set), ammunition ($2 per 20 rounds) and spare
wheels, axles and tongues ($10 each). The wagon holds about 2,400 lbs; past
that the oxen suffer and things break.

**Then manage four dials for four months:**

- **Pace** — steady / strenuous / grueling. Faster covers more ground and
  makes people sick.
- **Rations** — filling (3 lb/person/day, health slowly rises) / meager
  (2 lb, holds steady) / bare bones (1 lb, health falls). Five people on
  filling rations eat 15 lbs a day for ~120 days, which is more than one
  wagon can carry — so you will need to hunt, trade, or tighten the belt.
- **Resting** — the only medicine in the game.
- **River crossings** — ford, caulk and float, pay a ferry, or hire a local
  guide for clothing. **Always ask for information first.** A loaded wagon
  floats at about three feet; fording deeper than that is how parties die.

## What's in it

- **17 landmarks** on the historical trail with real mileages, from the
  Kansas River to The Dalles, including the two famous forks (Fort Bridger vs.
  the Green River; Fort Walla Walla vs. The Dalles) and the final choice
  between floating the Columbia or paying the Barlow Toll Road.
- **Per-person health and disease** — cholera, typhoid, dysentery, measles,
  fever, exhaustion, frostbite, snakebite, broken limbs and concussions, each
  with its own daily toll and recovery time. If *you* die, the journey ends.
- **~26 random events** — wagon breakdowns, thieves, prairie fires, bad water,
  overgrazed pasture, wrong trails, hail, blizzards, bandits, abandoned wagons,
  wild fruit, and traders who want to swap.
- **Weather** driven by month and terrain, affecting travel speed, health,
  and river depth.
- **A hunting minigame** — spot the game, then type the word before it bolts.
- **Trading, forts with escalating markups, and slaughtering an ox** when the
  food runs out.
- **Scoring** on the original points table, with a persistent Oregon Top Ten
  in `ortrail_top_ten.txt`.
- **Two events that are not in anybody's diary.** They turn up about once in a
  hundred incidents, so most parties reach Oregon never knowing. You'll know
  them when you see them.

## Layout

| File | Contents |
|---|---|
| `main.rs` | Title screen, main menu, the in-game trail guide |
| `game.rs` | Game state, setup, the day-by-day travel loop, camp menus |
| `trail.rs` | Landmarks, mileages, terrain, river specs, forks |
| `party.rs` | People, health ratings, ailments |
| `inventory.rs` | Wagon contents, prices, load weight |
| `store.rs` | Matt's General Store and the trading posts |
| `events.rs` | The random event table |
| `river.rs` | River crossings and the Columbia float |
| `hunt.rs` | The hunting minigame |
| `weather.rs` | Temperature and conditions by month and terrain |
| `score.rs` | Scoring and the Top Ten file |
| `date.rs` | Calendar arithmetic |
| `rng.rs` | Self-contained PRNG |
| `ui.rs` | Console input, word wrapping, banners, tombstones |

## A note on difficulty

It is meant to be hard. Roughly one real emigrant in ten died on the way,
most of them of disease rather than anything dramatic. The two things that
kill most parties here are the same two that killed them then: fording a
river you were told not to ford, and running out of food because you never
changed your rations.
