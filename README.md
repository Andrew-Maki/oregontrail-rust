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

## Getting it running

If you already have Rust: `cargo run --release` and you're done. Otherwise,
pick whichever section below matches your machine. **No programming knowledge
is needed** — it's copy, paste, press Enter.

A note that makes this easier than most Rust projects: **it has no
dependencies.** Nothing gets downloaded from the internet when you build it.
Once you have the code on your machine, it compiles offline in about two
seconds. That also means it builds fine on locked-down work computers that
block package downloads.

---

### Easiest option — Windows, no Rust at all

The built program is a **single 521 KB file** with nothing else alongside it.
No installer, no runtime, no DLLs. If someone sends you `ortrail.exe`, just
put it anywhere and double-click it.

One thing will happen the first time: Windows shows a blue
**"Windows protected your PC"** box, because the file isn't code-signed
(signing certificates cost money). To get past it, click **More info**, then
**Run anyway**. That's normal for small independent programs.

> Tip: put the `.exe` in its own folder before running it. It creates a
> `ortrail_top_ten.txt` next to itself to remember high scores.

---

### Windows — building it yourself

**Roughly 20–40 minutes, mostly waiting on one big download.**

**1. Install the Microsoft C++ Build Tools.** Rust needs these on Windows to
assemble the final program. This is the slow part — around 2–7 GB.

- Go to https://visualstudio.microsoft.com/visual-cpp-build-tools/
- Click **Download Build Tools**, run the downloaded file
- When the installer opens, tick the box labelled
  **"Desktop development with C++"**
- Click **Install** and let it finish

**2. Install Rust.**

- Go to https://rustup.rs
- Click the **rustup-init.exe** download button (64-bit)
- Run it. A black window appears with a wall of text.
- Press **Enter** to accept the default option (`1`)
- Wait for "Rust is installed now. Great!"

**3. Open a fresh terminal.** The install only takes effect in new windows.
Press the **Windows key**, type `powershell`, press Enter.

**4. Check it worked.** Type this and press Enter:

```powershell
rustc --version
```

You should see something like `rustc 1.99.1`. If instead it says the term is
not recognised, close the window, open a new one, and try again.

**5. Get the game.** Either:

```powershell
git clone https://github.com/Andrew-Maki/oregontrail-rust.git
cd oregontrail-rust
```

Or, if you don't have Git: open
https://github.com/Andrew-Maki/oregontrail-rust in a browser, click the green
**Code** button, choose **Download ZIP**, right-click the downloaded file and
**Extract All**. Then in PowerShell, `cd` into that folder.

**6. Play.**

```powershell
cargo run --release
```

The first build takes a few seconds. After that it starts instantly.

---

### Linux — building it yourself

**Roughly 10 minutes.**

**1. Open a terminal.** On most systems: `Ctrl` + `Alt` + `T`.

**2. Install a compiler and Git.** Pick the line matching your system:

```sh
# Ubuntu, Debian, Linux Mint, Pop!_OS, Raspberry Pi OS
sudo apt update && sudo apt install build-essential git curl

# Fedora, RHEL, CentOS, Rocky
sudo dnf install gcc git curl

# Arch, Manjaro, EndeavourOS, Steam Deck
sudo pacman -S base-devel git curl

# openSUSE
sudo zypper install gcc git curl
```

It will ask for your password. Typing shows nothing on screen — that's normal.

**3. Install Rust.** Paste this whole line:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

When it asks, press **Enter** for the default install (`1`).

**4. Load Rust into your current terminal:**

```sh
source "$HOME/.cargo/env"
```

(New terminals pick this up automatically from then on.)

**5. Get the game and play:**

```sh
git clone https://github.com/Andrew-Maki/oregontrail-rust.git
cd oregontrail-rust
cargo run --release
```

---

### macOS — building it yourself

**1. Install Apple's command line tools.** In Terminal
(`Cmd` + `Space`, type "Terminal"):

```sh
xcode-select --install
```

Click **Install** on the popup and wait.

**2. Install Rust:**

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Press **Enter** for the default. Then:

```sh
source "$HOME/.cargo/env"
git clone https://github.com/Andrew-Maki/oregontrail-rust.git
cd oregontrail-rust
cargo run --release
```

Works on both Intel and Apple Silicon (M1/M2/M3/M4) Macs.

---

### If something goes wrong

| What you see | What to do |
|---|---|
| `cargo: command not found` or `'cargo' is not recognized` | Close the terminal and open a new one. On macOS/Linux run `source "$HOME/.cargo/env"`. |
| `linker 'link.exe' not found` (Windows) | The C++ Build Tools in step 1 are missing or incomplete. Reinstall and make sure **"Desktop development with C++"** is ticked. |
| `error: linker 'cc' not found` (Linux) | You skipped step 2 — install `build-essential` / `gcc`. |
| `cannot find Cargo.toml` | You're in the wrong folder. `cd` into the one containing `Cargo.toml`. |
| Windows blocks the `.exe` | Click **More info** → **Run anyway**. It's unsigned, not malicious. |
| Text looks misaligned or wraps oddly | Make the terminal window at least 80 characters wide. The game formats to 72 columns. |
| Your corporate machine blocks crates.io | Doesn't matter here — this project has zero dependencies and builds offline. |

---

## Will it run on my device?

The game is plain Rust with no dependencies and no platform-specific code — it
only needs a **terminal and a keyboard**. So it runs essentially anywhere Rust
does:

| Device | Works? | Notes |
|---|---|---|
| **Windows 10 / 11** (64-bit) | Yes | Prebuilt `.exe` or build it. Windows 7/8 are no longer supported by Rust. |
| **Windows on ARM** (Surface Pro X, etc.) | Yes | Build from source. |
| **macOS** (Intel or Apple Silicon) | Yes | macOS 11 Big Sur or newer. |
| **Linux desktop** (x86_64) | Yes | Any distribution. |
| **Raspberry Pi** 3 / 4 / 5 | Yes | Use 64-bit Raspberry Pi OS. Compiling takes a minute or two on a Pi; playing is instant. |
| **Chromebook** | Yes | Turn on **Linux (Crostini)** in Settings, then follow the Linux steps. |
| **Steam Deck** | Yes | Switch to Desktop Mode, open Konsole, follow the Arch instructions. |
| **Android phone / tablet** | Yes | Install [Termux](https://termux.dev), then `pkg install rust git`. A Bluetooth keyboard helps a lot. |
| **iPhone / iPad** | Not realistically | iOS has no terminal or compiler, and App Store rules prevent one. |
| **In a web browser** | Not yet | Would need code changes — see below. |
| **Old server / NAS / router** | Probably | If Rust targets it and it has a shell, it'll run. |

**Why not a browser?** Rust can compile to WebAssembly, but this game pauses
and waits for you to type (`stdin`), which doesn't exist in a browser. Making
it work on a web page needs the input and output separated from the game logic
— the same refactor needed for a graphical version. Possible, not free.

**Other machines:** you can also build on one computer *for* another with
Rust's cross-compilation (`rustup target add ...`), but installing Rust
directly on the target machine is simpler.

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
