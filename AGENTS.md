# OTR — Oregon Trail (Rust)

Persistent project memory. Short key: **OTR**.

- **Path:** `C:\tools\OrTrail`
- **Crate:** `ortrail` (binary), edition 2021
- **Remote:** https://github.com/Andrew-Maki/oregontrail-rust — branch `main`
- **Status:** v1.0 complete and pushed. ~5,500 lines, 14 modules.

A text-mode clone of the 1848 Oregon Trail: 17 historical landmarks, per-person
disease model, weather by month/terrain, river crossings, hunting minigame,
trading, original scoring table with a persistent top-ten file.

---

## OPEN ITEM — dual presentation modes (to discuss later)

**Decision pending. Do not implement unprompted.**

Add a startup option letting the player choose how to play:

1. **Text mode** — what exists today: prose, numbered menus, word-wrapped output.
2. **ASCII-graphic mode** — a drawn/visual presentation still inside the
   terminal: trail map, wagon, river scenes, hunting screen, status panels.

Andrew wants to **discuss the design before any code is written.** Things to
settle in that conversation:

- Scope of "ASCII graphic": static ASCII art per screen, box-drawn TUI panels
  with live gauges, or animated frames?
- Dependency stance: a TUI crate (`ratatui` + `crossterm`) would be the first
  external dependency this project has taken. See constraints below.
- How the mode is chosen: main-menu item, CLI flag, or config file.
- Whether both modes stay at feature parity long-term, or ASCII becomes primary.

### Blocker to be aware of

**Presentation is currently interleaved with simulation logic.** `println!` and
`ui::` calls (`ui::menu`, `ui::yes_no`, `ui::wrap`) are made directly from
inside `game.rs`, `events.rs`, `river.rs`, and `hunt.rs`. The code is *not*
cleanly layered, so dual modes require real refactoring first, not a patch:

- Extract an output abstraction (an event/message log the renderer consumes)
  instead of direct `println!`.
- Extract a "request a decision" abstraction instead of calling `ui::menu`
  inline.
- Then `ui.rs` becomes one of two backends behind that interface.

Estimate this as a presentation-layer rewrite, not a feature add.

---

## Established conventions — preserve these

- **Zero external dependencies.** Deliberate. Ships its own `xorshift64*` PRNG
  in `rng.rs` so it builds offline. Any proposal to add a crate needs an
  explicit decision (see open item above). Verified: `cargo build --offline`
  succeeds from a clean `target/`.
- **`.cargo/config.toml` sets `+crt-static` for Windows MSVC targets** so the
  release exe is fully self-contained (521 KB, no `vcruntime140.dll`, no VC++
  Redistributable needed). This is what makes "just send someone the .exe"
  viable, and the README promises it — don't remove it. Without it the binary
  is 424 KB but imports `vcruntime140.dll`.
- **Clean bill of health required:** `cargo fmt` and `cargo clippy` must both be
  silent before committing.
- **Prose voice is dry, period-appropriate, understated.** Matter-of-fact 1840s
  register, not jokey. Keep new text in that voice.
- **Historical accuracy for mechanics:** real landmark mileages, ~2,400 lb wagon
  capacity, original scoring table, occupation multipliers (banker ×1,
  carpenter ×2, farmer ×3).
- **Difficulty is intentionally harsh.** The two main killers — fording rivers
  too deep and never adjusting rations — are by design. Don't "fix" them.
- **Easter eggs:** two deliberately rare events (~1% each) sit at the end of the
  event table in `events.rs` — `graboids` (Tremors) and `lights_in_the_sky`
  (UFO abducting oxen). Tonally deadpan on purpose; keep them rare and dry.

## Environment quirks (cost real time before — don't rediscover)

- **crates.io fails with an SSL revocation error** (`CRYPT_E_NO_REVOCATION_CHECK`).
  Workaround: `$env:CARGO_HTTP_CHECK_REVOKE="false"`. Relevant if we ever add a
  dependency.
- **A GUI/windowed build cannot be verified from the agent shell.** Launching
  the exe via `Start-Process` yields `MainWindowHandle=0` — no console attaches.
  Interactive play must happen in the user's own terminal. This is a core reason
  the project went text-mode.
- **Testing method that works:** pipe scripted stdin into the binary and assert
  on stdout (`Start-Process -RedirectStandardInput`). The game treats EOF as
  quit, so scripted runs terminate cleanly. This caught the starvation balance
  bug, a misaligned tombstone, and subject-verb agreement errors.
- **PowerShell gotchas:** no `<<<` heredoc (use `git commit -F <file>`);
  `Set-Content -Encoding UTF8` writes a **BOM** — use
  `[IO.File]::WriteAllText($p, $s, (New-Object System.Text.UTF8Encoding $false))`.
- **`gh` CLI is not installed.** Use plain `git`.
- Set `$env:GIT_TERMINAL_PROMPT="0"` on remote git ops so auth failures fail
  fast instead of hanging the shell.

## Git identity for this repo

This repo sets a **local-only** git identity; the global config is
intentionally left untouched:

```
user.name  = Andrew-Maki
user.email = Andrew-Maki@users.noreply.github.com
```

The machine's global identity is a corporate work address. Keep it out of this
public repo — always commit here with the local override above. Verify before
committing with:

```
git log -1 --pretty='%an <%ae>'
```

## Module map

| File | Contents |
|---|---|
| `main.rs` | Title screen, main menu, in-game trail guide |
| `game.rs` | State, setup, day-by-day travel loop, camp menus |
| `trail.rs` | Landmarks, mileages, terrain, river specs, forks |
| `party.rs` | People, health ratings, ailments |
| `inventory.rs` | Wagon contents, prices, load weight |
| `store.rs` | Matt's General Store and trading posts |
| `events.rs` | Random event table (easter eggs at the tail) |
| `river.rs` | River crossings and the Columbia float |
| `hunt.rs` | Hunting minigame |
| `weather.rs` | Temperature and conditions by month/terrain |
| `score.rs` | Scoring and the top-ten file |
| `date.rs` | Calendar arithmetic |
| `rng.rs` | Self-contained PRNG |
| `ui.rs` | Console I/O, word wrap, banners, tombstones |
