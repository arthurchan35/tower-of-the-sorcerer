# Copilot review instructions — Tower of the Sorcerer / 魔塔

These instructions guide GitHub Copilot when reviewing pull requests in this repository. Read top to bottom; sections are ordered from "must know" to "nice to have".

## 1. Project context

This repo is a port of **魔塔 (Magic Tower / Tower of the Sorcerer)** — a turn-based, grid-based RPG where the hero ascends a tower one floor at a time, bumping into adjacent tiles to fight monsters, pick up items, open colored doors, and reach stairs.

**Core mechanics** (these are invariants — divergence is a bug):
- Combat is deterministic: `damage = max(0, attacker.attack - defender.defense)`. Hero strikes first; sides alternate until one HP ≤ 0.
- A fight where the hero's attack ≤ enemy defense is unwinnable (refused, not infinite-looped).
- Doors come in yellow / blue / red and consume a matching key; cyan / "magic" doors are event gates ("kill all guards on this floor").
- Items either modify stats permanently (gems, weapons, shields), restore HP (potions), grant keys, or trigger scripted effects.

## 2. Repository structure

The repo is **multi-implementation**. Each top-level directory is a self-contained port; they don't share code:

- `rust/` — the active first implementation (Rust + Bevy 0.18). All Rust review guidance below applies here.
- Future siblings (e.g. `godot/`, `python/`, `unity/`) may appear later. **Do not suggest sharing code or types across implementations.** Game *data* (JSON in `assets/data/`) may be reused; source code is not.
- The game rules in §1 are invariants across every implementation. Flag any divergence.

## 3. Top review priorities (in order)

### 3.1 Best practice for the active stack

The standing rule for this project is: **always follow current Rust idioms, current Bevy idioms, and standard turn-based-RPG game-design patterns**. Don't approve patterns copied from older Bevy versions or unrelated engines without checking they match the active major version (currently Bevy 0.18). When two patterns are reasonable, prefer the more idiomatic one and call out the trade-off in the review.

For Rust + Bevy specifically:
- **ECS-native where it earns its keep.** Entities + components for things with many instances or that change over time (Player, sprites, enemies). `Resource` for static or singleton state (the floor map, lookup tables, status banner).
- **Plugins are the unit of organization.** Each top-level module exposes a `Plugin`; `main.rs` only composes plugins. Cross-plugin ordering goes through `SystemSet`s.
- **`States` for game flow**, never ad-hoc `bool` flags. `.run_if(in_state(...))` for gating, `OnEnter`/`OnExit` for transitions.
- **Pure-Rust core.** Combat, action dispatch, and other game logic take plain `&mut` references and return enums. Bevy types appear only at the boundary system. Tests run without spinning up an `App`.
- **Data-driven numbers.** Stats, drop tables, item amounts live in `rust/assets/data/*.json`. Hand-coded stat tables in Rust source are a regression — flag them.
- **Bevy 0.18 events were renamed to messages.** Use `Message` / `MessageReader` / `MessageWriter` / `add_message`. The older `Event` / `EventReader` / `add_event` names still exist for observer-style events but are not the right tool for buffered system-to-system signaling here.

### 3.2 Correctness and ECS hazards to watch for

- **Negative query filters** (`Without<X>`) silently matching entities the author didn't expect. Suggest a positive marker component instead. The codebase has been bitten by this — there's a `TileSprite` marker now precisely for this reason.
- **`Single<>` skips its system silently** if zero or multiple entities match. For systems that must always run, prefer `Query<>` with explicit `Result` handling via `single_mut()`. Reserve `Single<>` for cases where skipping is genuinely correct.
- **`Mut<T>::deref_mut()` always sets the changed flag**, even if the value isn't actually mutated. Over-triggering of `Changed<T>` filters often indicates an unnecessary mutable borrow — flag and suggest `Mut::set_if_neq` or read-only access.
- **Borrow-checker workarounds via `into_inner()`** are fine but verify they're necessary, not papering over a design issue.
- **Required components** (Bevy 0.15+ pattern): inserting `Sprite` auto-adds `Transform` with default. If the author passes a tuple `(Sprite, Transform)`, the explicit `Transform` wins — don't flag that as redundant.

### 3.3 Cross-implementation coherence

- The combat formula, grid size (currently 11×11 per floor), door/key colors, and stat semantics are invariants. Any code change that would alter these should be called out as a rules change, not a refactor.
- JSON schema in `assets/data/` is shared spirit across implementations. Field names should be stable.

### 3.4 Tests

- Combat / action / data-loader changes should ship with unit tests in the same module.
- Tests must run without a Bevy `App` (the pure-Rust core makes this possible — flag tests that require constructing an `App` for logic that doesn't need one).

## 4. What NOT to flag

These are recurring false-positive sources — please skip:

- **Placeholder colored-square tile rendering** in `rust/src/render/tiles.rs::tile_color`. There is an explicit TODO in `CLAUDE.md` for real artwork; repeated "suggest using sprite images" comments are noise.
- **`include_str!` for asset loading.** This is the v0.1 strategy. Suggest `AssetServer` only when the PR is explicitly about asset hot-reload.
- **The dev profile setting `opt-level = 1` on the crate and `opt-level = 3` on dependencies.** This is intentional — Bevy is unusably slow without optimized deps in dev.
- **Comment density.** This codebase is sparse-comment by convention; only the non-obvious *why* is documented. Don't request docstrings on self-describing code.

## 5. Style and PR hygiene

- The project follows `cargo fmt` defaults; no extra style rules.
- Names: prefer concise, domain-appropriate (`Floor`, `TilePos`, `step_player`) over Bevy-flavored (`World`, `WorldCoord`, `move_player_system`).
- Comments are sparse — explain *why*, not *what*. Self-describing code is preferred.
- PR descriptions should be concise; reviewers don't need a generated essay.
