# 🏰 Rust Royale — Codebase Status Report (March 28, 2026)

> Cross-referenced against the [postmortem roadmap](file:///Users/parthagrawal99/rust_royale/rust_royale_postmortem.md) and the actual source code.

---

## ✅ What Has Been DONE (Fixed Since Postmortem)

These items from the postmortem roadmap are **confirmed resolved** in the current code:

### Phase 1 (Critical Bugs) — Mostly Done ✅

| Item | Status | Evidence |
|------|--------|----------|
| **1.1** `despawn()` → `despawn_recursive()` | ✅ **FIXED** | All despawn calls in [combat.rs](file:///Users/parthagrawal99/rust_royale/engine/src/systems/combat.rs) now use `despawn_recursive()` (lines 431, 482, 500, 512, 536, 643, 684, 717) |
| **1.2** Splash damage for Valkyrie/Baby Dragon | ✅ **FIXED** | `SplashProfile` component added ([components.rs:134-139](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L134-L139)). `Projectile` now carries `splash_radius` ([components.rs:170](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L170)). `projectile_flight_system` has full AoE splash damage logic (lines 395-418 in combat.rs) using position snapshots to avoid double-borrow issues |
| **1.4** Per-unit projectile speed | ✅ **FIXED** | `AttackStats` now has `projectile_speed: i32` field ([components.rs:131](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L131)). `TroopStats` has `projectile_speed: Option<i32>` ([stats.rs:102](file:///Users/parthagrawal99/rust_royale/core/src/stats.rs#L102)). Spawning uses `troop_data.projectile_speed.unwrap_or(6000)` |
| **1.5** Goblin Barrel spawner spell | ✅ **FIXED** | `SpellType::Spawner` branch properly implemented ([spawning.rs:369-418](file:///Users/parthagrawal99/rust_royale/engine/src/systems/spawning.rs#L369-L418)). It looks up the troop by ID, creates a `DeathSpawn` component on the barrel entity, which triggers `handle_death_spawns_system` on impact |
| **1.6** Tower death logic deduplication | ✅ **FIXED** | Extracted into `apply_tower_destruction_rules()` in [match_manager.rs:7-45](file:///Users/parthagrawal99/rust_royale/engine/src/systems/match_manager.rs#L7-L45). Both `projectile_flight_system` and `spell_impact_system` call this shared function |

### Phase 2 (Engine Correctness) — Mostly Done ✅

| Item | Status | Evidence |
|------|--------|----------|
| **2.1** FixedUpdate at 60Hz | ✅ **FIXED** | `Time::<Fixed>::from_seconds(1.0 / 60.0)` inserted as resource ([main.rs:56](file:///Users/parthagrawal99/rust_royale/app/src/main.rs#L56)). All game logic systems run in `FixedUpdate` schedule (lines 76-91) |
| **2.2** System ordering (`.chain()`) | ✅ **FIXED** | All 10 game systems chained in explicit order ([main.rs:76-91](file:///Users/parthagrawal99/rust_royale/app/src/main.rs#L76-L91)): `spawn → deploy → match_clock → target → combat → projectiles → spells → movement → collision → death_spawns` |
| **2.3** VecDeque for waypoints | ✅ **FIXED** | `WaypointPath` uses `VecDeque<(i32, i32)>` ([components.rs:191](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L191)). Movement uses `path.0.front()` and `path.0.pop_front()` for O(1) operations |
| **2.4** Path caching for A* | ✅ **FIXED** | `PathCache` resource added ([components.rs:290-293](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L290-L293)). `try_calc_path()` in [movement.rs:20-46](file:///Users/parthagrawal99/rust_royale/engine/src/systems/movement.rs#L20-L46) checks cache before running A*, keyed on `(start, goal, is_flying, range_tiles)` |
| **2.5** Game state machine (AppState) | 🟡 **PARTIAL** | `AppState` enum defined ([main.rs:24-32](file:///Users/parthagrawal99/rust_royale/app/src/main.rs#L24-L32)) with `Playing` as default. `MainMenu`, `Paused`, `GameOver` exist only as comments. State is initialized but never transitions |
| **2.6** Red team deck rotation fix | ✅ **FIXED** | `PlayerDeck` now has separate `blue_selected` and `red_selected` fields ([components.rs:255-256](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L255-L256)). Spawning uses per-team selection ([spawning.rs:124-141](file:///Users/parthagrawal99/rust_royale/engine/src/systems/spawning.rs#L124-L141)) |
| **2.7** Multi-tile footprint validation | ❌ **NOT DONE** | Spawning still only validates the single deployment tile, not the footprint area |

### Other Fixes (Not in Postmortem)

| Feature | Status | Evidence |
|---------|--------|----------|
| **Drag-and-drop deployment** | ✅ **DONE** | Full drag-and-drop with hologram indicators ([input.rs:128-215](file:///Users/parthagrawal99/rust_royale/engine/src/systems/input.rs#L128-L215)). `DragState` and `DragHologram` components. Spell radius circle gizmo during drag |
| **Card UI bar** | ✅ **DONE** | Bottom card bar with 4 clickable buttons ([ui.rs:159-205](file:///Users/parthagrawal99/rust_royale/engine/src/systems/ui.rs#L159-L205)). Dynamic highlighting of selected card ([ui.rs:252-278](file:///Users/parthagrawal99/rust_royale/engine/src/systems/ui.rs#L252-L278)) |
| **Tiebreaker HP percentage fix** | ✅ **FIXED** | Now uses `health.0 / max_health.0` as HP percentage ([match_manager.rs:84-106](file:///Users/parthagrawal99/rust_royale/engine/src/systems/match_manager.rs#L84-L106)). `MaxHealth` component exists. Per-team comparison instead of global minimum |
| **TowerDeathEvent** | ✅ **DONE** | Event struct defined ([components.rs:42-47](file:///Users/parthagrawal99/rust_royale/core/src/components.rs#L42-L47)) though usage appears limited |
| **Input: visual separation** | ✅ **DONE** | Input systems run in `Update`, game logic in `FixedUpdate`, visuals back in `Update` — clean separation |
| **Camera auto-scaling** | ✅ **DONE** | `ScalingMode::AutoMin` ensures full arena visible on any screen ([input.rs:15-18](file:///Users/parthagrawal99/rust_royale/engine/src/systems/input.rs#L15-L18)) |

---

## ❌ What REMAINS To Be Done

### 🔴 Priority 1: Game-Breaking Gaps

| # | Item | Difficulty | Notes |
|---|------|------------|-------|
| 1 | **AI Opponent** | Hard | **No AI exists at all.** Both teams are human-controlled. The game is unplayable solo. This is the single biggest blocker to making this a "game." |
| 2 | **Visual Assets / Sprites** | Medium | `assets/sprites/` directory is **empty**. Everything is still colored squares. No sprite sheets, no pixel art, no animations. |
| 3 | **Sound Effects / Music** | Medium | **Zero audio.** No sound system, no audio assets, complete silence. |
| 4 | **Game State Transitions** | Easy | `AppState` enum has `Playing` only. No `MainMenu`, `Paused`, `GameOver` screens. Game jumps straight to gameplay, no way to restart. |

### 🟡 Priority 2: Significant Missing Features

| # | Item | Difficulty | Notes |
|---|------|------------|-------|
| 5 | **Multi-tile footprint validation** (Postmortem 2.7) | Easy | Golem 2×2 can still overlap into river on spawn |
| 6 | **Building Cards** (Tesla, Inferno, Bomb Tower) | Hard | `is_building: true` troop type with lifetime timer doesn't exist in engine |
| 7 | **More Spells** (Zap, Log, Poison, Rage, Tornado) | Medium | Only 3 spells exist. No DoT, buffs, pull mechanics, or moving hitboxes |
| 8 | **More Troops** (Hog Rider, Wizard, Witch, Barbarians) | Medium | Only 11 troops in stats.json. No periodic spawning (Witch), building-jump (Hog), etc. |
| 9 | **Card Upgrade/Level System** | Medium | No level multipliers, no progression. All cards are level 1 permanently |
| 10 | **Deck Builder Screen** | Medium | Deck is hardcoded in `Deck::new_shuffled()` — no way to customize the 8-card deck |
| 11 | **Menus & Game Flow** | Medium | No main menu, match result screen, pause, settings, or restart |

### 🟢 Priority 3: Polish & Nice-to-Have

| # | Item | Difficulty | Notes |
|---|------|------------|-------|
| 12 | **Troop Animations** (idle, walk, attack, death) | Medium | Not started |
| 13 | **Particle Effects** (deploy dust, death explosion, trails) | Medium | Not started |
| 14 | **Tower Health Bars** (segmented, CR-style) | Easy | Currently just text numbers above entities |
| 15 | **Animated Elixir Bar** | Easy | Text-only display, no purple fill bar |
| 16 | **Crown Counter UI** | Easy | Text-only, no star animations |
| 17 | **Match Timer Display** | Easy | Embedded in HUD text, not a centered timer |
| 18 | **Banner Announcements** ("DOUBLE ELIXIR!", "OVERTIME!") | Easy | Only console prints |
| 19 | **Replay System** | Medium | Not started |
| 20 | **Min-Range Attacks** (Mortar) | Easy | `range_min` in stats but unused |
| 21 | **Multiplayer / Netcode** | Very Hard | Not started |

### 🐛 Remaining Bugs

| # | Severity | Description |
|---|----------|-------------|
| 1 | 🟡 Medium | **Path cache never invalidated** — `PathCache` is only cleared in match_manager tiebreaker ([match_manager.rs:122](file:///Users/parthagrawal99/rust_royale/engine/src/systems/match_manager.rs#L122)). When a tower is destroyed during normal play, cached paths that routed around it are stale. Troops may keep walking around a destroyed tower's ghost footprint. |
| 2 | 🟡 Medium | **select_card_system sets both teams** — Pressing 1-4 sets both `blue_selected` AND `red_selected` to the same slot ([input.rs:106-121](file:///Users/parthagrawal99/rust_royale/engine/src/systems/input.rs#L106-L121)). This is a sandbox convenience but would be a bug in a real game. |
| 3 | 🟢 Low | **App ≡ Sandbox** — `app/src/main.rs` and `sandbox/src/main.rs` are still near-identical. No real differentiation. |
| 4 | 🟢 Low | **No error handling** — `unwrap()` calls on stats.json loading and tower data lookups |
| 5 | 🟢 Low | **No unit tests** — `tests/` directory contains only `.gitkeep` |

---

## 📊 Updated Scoring

| Category | Postmortem Score | Current Score | Change |
|----------|:---:|:---:|:---:|
| **Core Simulation** | 7/10 | **9/10** | ⬆️ +2 (splash, spawner spells, per-unit proj speed all fixed) |
| **Game Rules** | 8/10 | **9/10** | ⬆️ +1 (tiebreaker fixed, FixedUpdate, system ordering) |
| **Card Variety** | 3/10 | **3/10** | ➡️ (no new cards added) |
| **Visuals** | 1/10 | **2/10** | ⬆️ +1 (drag hologram, card bar, but still colored squares) |
| **Audio** | 0/10 | **0/10** | ➡️ |
| **UI/UX** | 2/10 | **3/10** | ⬆️ +1 (card bar, drag-and-drop) |
| **AI** | 0/10 | **0/10** | ➡️ |
| **Multiplayer** | 0/10 | **0/10** | ➡️ |
| **Engine Quality** | — | **8/10** | ⬆️ (FixedUpdate, system chain, VecDeque, path cache) |
| **Overall** | **2.5/10** | **~3.5/10** | ⬆️ Engine is now rock-solid, but the "game shell" is still missing |

---

## 🎯 Recommended Next 5 Actions

1. **🤖 Build a Basic AI** — Random card plays at intervals → makes the game playable solo  
2. **🎮 Add Game State Machine** — Wire up `MainMenu → Playing → GameOver` with restart  
3. **🎨 Replace Colored Squares with Sprites** — Even simple pixel art transforms the experience  
4. **🔊 Add Basic Audio** — Deploy SFX, attack SFX, background music  
5. **🧹 Invalidate Path Cache on Tower Death** — Clear `PathCache` in `projectile_flight_system` and `spell_impact_system` when a tower is destroyed, not just during tiebreaker

> [!TIP]
> The **engine is now excellent** — deterministic, ordered, cached, and correctly separated. All critical simulation bugs are fixed. The gap is entirely on the **experience side**: AI, visuals, audio, and menus. Focus there.
