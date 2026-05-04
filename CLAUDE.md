# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project: Tower of the Sorcerer / 魔塔 — Rust port

A turn-based, grid-based puzzle/RPG. The hero ascends a tower floor by floor, fighting monsters, collecting keys to open doors, and gathering items that boost stats. Combat is deterministic: `damage = max(0, attacker.attack - defender.defense)`; the hero strikes first each round and the two sides alternate until one dies.

Wikipedia (Chinese): https://zh.wikipedia.org/zh-cn/魔塔

The repository is intended to host implementations in multiple languages/engines, with **Rust as the first target**. Each implementation lives in its own top-level directory so they don't couple.

## Repository layout

- `rust/` — the Rust implementation (the active first version). See **Build / run** and **Rust crate layout** below.
- `Screenshot 2026-05-03 at 11.09.23 PM.png` — floor-1 UI reference (left HUD: HP/Atk/Def/Gold; right: equipped weapon/shield + name+icons; center: 11×11 map).

The repo's earlier C++ commits (`TowerOfTheSorcerer/Character.cpp`, `Hero.cpp`, etc.) are **abandoned**. Commit `5ebdfd8 remove all` deliberately wiped them — do not resurrect or reference.

## Game rules (essential)

- Hero stats: HP, Attack, Defense, Gold (cap 9990 in the original), plus keys in yellow / blue / red.
- Combat: hero strikes first; each side does `max(0, atk - def)` per round; if neither can damage the other the fight is impossible. Some monsters drain, regen, or hit twice.
- Doors: yellow / blue / red consume matching keys; cyan doors are event gates (typically "kill all monsters on this floor").
- Items: red gem +atk, blue gem +def, potions +HP, weapons/shields permanently set higher equipped stats, specials trigger scripted effects.
- Movement: arrow keys or WASD (one tile per press; turn-based).

## Build / run

The Rust crate lives in `rust/`. All cargo commands assume that working directory.

- `cargo run` — build and launch the game window (Bevy 2D, Metal-backed on macOS).
- `cargo build` — compile only.
- `cargo test` — run unit tests (combat formula, JSON loaders).

First Bevy compile takes several minutes; the dev profile is configured to fully optimize dependencies (`opt-level = 3` for deps, `1` for our crate) so incremental rebuilds of our own code stay fast.

## Rust crate layout

The crate is **`lib + bin`** so the library is testable and the binary is a thin wrapper.

```
rust/
├── Cargo.toml              dev profile tuned for fast incremental rebuilds
├── assets/data/
│   ├── map.json            tile placements per floor
│   └── properties.json     resource catalog (env / item / actor / enemy)
└── src/
    ├── lib.rs              re-exports the modules
    ├── main.rs             builds the App, adds DefaultPlugins + game/input/render plugins
    ├── app_state.rs        AppState States enum (Playing / FloorCleared / GameOver)
    ├── data/               serde structs + loaders, no Bevy types
    │   ├── map_info.rs     MapResourceInfo, load_map, filter_floor
    │   └── resource_info.rs ResourceInfo, ResourceKind, load_properties
    ├── game/               pure game logic + GamePlugin
    │   ├── stats.rs        PlayerStats (Component), EnemyStats, floor1_enemy_stats
    │   ├── combat.rs       resolve_combat with the canonical max(0, atk-def) formula
    │   ├── floor.rs        Floor (Resource), Tile, TilePos (Component), DoorColor, ItemKind
    │   ├── action.rs       step_player(floor, stats, pos, dir) -> ActionResult
    │   └── mod.rs          GamePlugin, GameSetup SystemSet, Player marker, StatusMessage resource
    ├── input.rs            InputPlugin, MoveIntent message, keyboard system
    └── render/             Bevy 2D rendering layer (the only Bevy-coupled code besides input + plugins)
        ├── tiles.rs        camera, tile sprites (placeholder colors), player sprite, sync systems
        ├── hud.rs          UI Node with stats label + status label
        └── mod.rs          RenderPlugin (Startup + Update + OnEnter(state) handlers)
```

### Architectural conventions

- **Three plugins compose the App**: `GamePlugin` (state + logic), `InputPlugin` (keyboard → `MoveIntent`), `RenderPlugin` (sprites + HUD + state transitions). `main.rs` only wires them in.
- **Player is an entity** with a `Player` marker, `PlayerStats`, and `TilePos` components. Game systems query for it via `Single<(&mut TilePos, &mut PlayerStats), With<Player>>`. There is exactly one Player entity at a time.
- **The floor map is a `Resource`** (`HashMap<TilePos, Tile>`), not a swarm of tile entities — gives O(1) tile lookup for action dispatch. Tile entities exist only at the render layer (one `Sprite` per tile, despawned when the corresponding map entry is removed).
- **`AppState` (a Bevy `States` enum)** drives the game-flow state machine. The turn handler runs only `.run_if(in_state(AppState::Playing))`; `OnEnter(FloorCleared)` and `OnEnter(GameOver)` set the status text.
- **`StatusMessage`** is a separate `Resource` for the UI banner — UI concerns don't leak into the game state.
- **Pure-Rust action layer**: `game::action::step_player` takes plain `&mut` references and returns an `ActionResult` enum; the Bevy system in `game/mod.rs` is the only place that wires it to ECS. Combat tests call it without spinning up an `App`.
- **Bevy 0.18 events**: this version renamed events to messages — `Event` → `Message`, `EventReader/Writer` → `MessageReader/Writer`, `add_event` → `add_message`. Use the new names.
- **Asset loading**: floor data is `include_str!`-ed into the binary at compile time. Replace with `AssetServer` once we want hot-reload.

### Floor-1 enemy stats

`game::stats::floor1_enemy_stats` is a hand-written table for the six monster IDs that appear on floor 1. `properties.json` only carries names/info/icons — numerical stats are not in it. Future floors will need their stat tables added similarly (or a single data file grown to include stats).

### Coordinate system

`TilePos { col, row }` with origin at the bottom-left, `0..GRID_SIZE` in both axes. The source map data uses origin-centered floats (e.g. -5.0..=5.0); `TilePos::from_origin_centered` does the conversion at load.

## Working principles

**Always follow current Rust idioms and Bevy idioms, and apply standard turn-based-game design patterns** when adding or modifying code in `rust/`. Don't copy patterns from older Bevy versions or game engines without checking they still match the current Bevy major version. When in doubt between competing approaches, ask before committing — the user wants the codebase to track best practice as the project grows.

Concrete defaults this project commits to:

- **ECS-native where it earns its keep.** Entities + components for things that have many instances or change over time (Player, sprites). Resources for static or singleton state (Floor map, lookup tables, current status). Don't make tile entities just to be "ECS-pure" — random-access data is what HashMaps are for.
- **Plugins as the unit of organization.** Each top-level module exposes a `Plugin`; `main.rs` only composes plugins. Cross-plugin ordering goes through `SystemSet`s, not direct `.after(fn_name)` (the one current exception is the input → turn handler ordering, which is intentionally explicit because it's a single edge).
- **`States` for game flow**, never ad-hoc `bool` flags on resources. Gate Update systems with `.run_if(in_state(...))`; do transition work in `OnEnter(...)` / `OnExit(...)`.
- **Pure-Rust core.** `game::action` / `game::combat` take plain `&mut` and return enums; only the system in `game/mod.rs` translates between Bevy and the core. Tests run without a Bevy `App`.
- **Data-driven.** Numeric tuning (enemy stats, item amounts, drop tables) lives in `assets/data/*.json`, loaded once via serde. Don't bake numbers into Rust unless they're truly invariants of the rules (e.g. the combat formula).
- **Bevy 0.18 events are messages.** Use `Message`, `MessageReader`, `MessageWriter`, `add_message`. Don't search-and-replace from older tutorials.
- **No backwards-compatibility shims.** This is a fresh codebase; if something has a better shape, change it everywhere.

## TODO / future work

- **Replace placeholder colored tiles with real artwork.** Tiles and the player are currently solid-color squares (`render::tiles::tile_color`). Pick a sprite source, add to `rust/assets/sprites/`, switch `Sprite { color, .. }` to `Sprite { image: asset_server.load("..."), .. }`.
- **Polish the HUD to match the screenshot reference.** Current HUD is a top-anchored row; the reference has left/right side panels (Tower / HP / ATK / DEF / GOLD on the left; weapon / shield / character portrait on the right).
- **Multi-floor progression.** `FLOOR_TO_PLAY` is hard-coded to 1; stairs end the run. Adding floor 2+ needs an `OnEnter(AppState::FloorCleared)` handler that swaps the `Floor` resource for the next floor's tile set and repositions the Player.
- **Implement the unhandled item kinds.** `ItemKind::Unimplemented` covers items whose effect we haven't wired up (e.g. the teleport scepter). When floor coverage demands them, expand `ItemKind` and `apply_item`.
- **Asset hot-reload.** Floor and property JSON are baked in via `include_str!`. Migrate to `AssetServer` once edit-and-see-the-change becomes worthwhile (no rebuild needed; Bevy can watch the file and re-trigger the load system).
- **Cyan / magic doors.** Floor 1 only uses yellow doors, but `properties.json` defines magic doors that open after defeating specific enemies. Needs a per-floor "guard list → door entity" wiring once we get to floors that have them.
