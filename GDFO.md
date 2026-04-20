# Game Development Framework Overview

> Comprehensive design overview for agentic game development built on tanren
> orchestration and the Bevy engine. This document is intended as a recovery
> artifact — it should give any reader (future Trevor, a future agent, a
> collaborator joining later) the full motivation, philosophy, architecture,
> and roadmap of the framework without requiring access to the conversations
> that produced it.

---

## 0. How to read this document

This is a long document organized so you can start anywhere:

- **Sections 1–3** explain the *why* — the problem being solved, the
  organizing insight, the engine choice. Read these if you're new to the
  project.
- **Sections 4–6** explain the *principles* — the design philosophy and
  non-negotiable invariants that everything else follows from. Read these
  if you need to evaluate a proposed change and want to know what's
  load-bearing.
- **Sections 7–17** explain the *architecture* — crate topology, boundaries,
  subsystems. Read these when implementing or reviewing a spec that
  touches simulation, presentation, networking, persistence, or modding.
- **Sections 18–22** explain the *quality model* — testing, CI, performance,
  scaling. Read these when designing tests or when CI time becomes painful.
- **Sections 23–27** explain the *tanren integration* — how specs, tasks,
  and gates map onto game work. Read these to understand how day-to-day
  development actually operates.
- **Sections 28–30** explain the *rust-bevy profile* — the standards bundle
  tanren installs into the game repo.
- **Sections 31–34** describe the *first project* — a pick-3 roguelike used
  to prove the framework.
- **Sections 35–41** enumerate *open questions* resolved and deferred
  decisions remaining.
- **Sections 42–46** lay out the *project strategy* — open source, release
  path, template extraction, dev environment, game design pitch.
- **Section 47** lays out *learning maximization* — reflection discipline,
  upstream posture, community engagement.
- **Sections 48–50** lay out the *roadmap* for drafting the detailed specs
  that this overview points to, plus the decision log and success
  criteria.

Appendices A and B provide a glossary and references.

Cross-references within this document use section numbers (e.g. §12).
References to tanren repo paths assume the lane-0.5 branch or its
descendants.

---

# Part I — The Problem and the Insight

## 1. Context and motivation

Trevor's primary engineering practice has converged on **agentic software
development with rigorous orchestration**. The tanren project
(`trevorWieland/tanren`) is an opinionated control plane for that practice:
specs enter a typed lifecycle, get decomposed into tasks, tasks are
dispatched to agent runtimes (Claude Code, Codex, OpenCode), each task is
gated by `just ci` + `audit-task` + `adhere-task` before transitioning to
complete, and the aggregate spec is demoed and audited before merge. This
loop produces high-quality code at high velocity with minimal human babysitting.

The goal of this framework is to extend that loop to game development.

Traditional game engines are structurally hostile to this workflow:

1. **Editor-centric organization.** Unity, Unreal, Godot, Fyrox all place
   the editor at the center. Scenes, prefabs, nodes, and assets are
   manipulated through a GUI. The canonical project representation mixes
   binary scene files, script files, and editor-managed metadata. Agents
   can edit scripts but struggle with scene files and cannot introspect the
   editor's state.
2. **Implicit runtime state.** Visual tooling encourages patterns where
   game state is scattered across scene hierarchy, component inspectors,
   and script globals. Agents cannot reason about state that isn't in text.
3. **Closed-source and proprietary components.** Unity and Unreal are
   closed-source; their behavior is a black box. This makes agent-driven
   development at a rigor level matching tanren functionally impossible.
4. **Tight coupling of simulation and rendering.** Most engines assume
   simulation runs every frame in lockstep with rendering. This prevents
   headless simulation, which breaks machine-verifiable testing.

Bevy is the one mainstream engine that does not have these problems:

1. Code-first with no mandatory editor.
2. Everything is Rust structs and functions; state is explicit.
3. Fully open source, MIT/Apache dual-licensed.
4. ECS architecture with `FixedUpdate` schedule separates simulation from
   rendering by design.

This makes Bevy the correct foundation. The framework on top of it adapts
tanren's orchestration patterns to game work and adds the game-specific
standards that make agentic development tractable at scale.

## 2. The organizing insight

The central intellectual move of this framework, distilled to one sentence:

> **If the game exposes a typed command/observation interface at its
> simulation boundary, then AI playability, machine testability, human
> auditability, and iterative development all collapse into the same
> property.**

An AI agent that can play the game (issue commands, read observations,
decide next action) is also an agent that can test the game (replay a
committed command sequence and assert observations), audit the game
(walk a demo script and verify behavior), and iterate on the game
(propose changes, verify their effect through the same interface).

The architecture must therefore be built around that command/observation
interface as a first-class boundary.

This is the direct game-dev analog to tanren's own organizing insight: by
moving from ad-hoc markdown checkbox parsing to a typed tool surface
(MCP commands), tanren made every agent action auditable, every state
transition explicit, and every workflow reproducible. Same move, different
domain.

## 3. Engine choice: Bevy

Bevy 0.18 (January 2026) is the baseline engine. The specific reasons:

1. **Rust as the implementation language** gives compile-time strictness
   that matches tanren's existing rust-cargo profile. The lints that make
   tanren safe (`unsafe_code = forbid`, `unwrap_used = deny`, `panic = deny`,
   file and function size limits) port directly.
2. **ECS as the architecture** means game state is explicit typed data
   accessed through typed queries. Agents can reason about state without
   visual inspection.
3. **Plugins as the composition unit** maps naturally to tanren's thin
   vertical specs — each spec can own a plugin.
4. **`FixedUpdate` schedule** separates simulation from rendering by design,
   enabling headless testing and deterministic replay.
5. **Standalone `bevy_ecs` crate** can be used independently, allowing the
   domain layer to depend on ECS without pulling in the full engine.
6. **No mandatory editor.** Bevy has an optional editor in development
   (`bevy_editor`) but the primary interface is code.
7. **Active, well-funded development** by the Bevy Foundation; Carter
   Anderson is full-time on the project.

Bevy's rough edges are accepted deliberately:

1. **Pre-1.0 with ~quarterly breaking changes.** Each migration is a
   tanren spec. The migration guides are excellent. The cadence is actually
   healthy for an agent-driven workflow — it forces continuous maintenance
   rather than letting rot accumulate.
2. **Asset pipeline is less mature than Unity/Godot.** We mitigate by
   committing to a content VFS architecture (§11) that makes the asset
   pipeline our concern rather than the engine's.
3. **No mature visual editor.** For us this is a feature, not a bug.
4. **Smaller community than Unity/Godot.** Growing fast; the
   `bevy/assets` ecosystem has mature options for most subsystems
   (physics: `avian`, `bevy_rapier`; audio: `bevy_kira_audio`; dialogue:
   `YarnSpinner-Rust`; linting: `bevy_lint`).

Alternatives considered and rejected:

1. **Godot with godot-rust** — scene files (`.tscn`) are a visual-first
   organizational unit that doesn't serialize cleanly for agent editing.
   Good engine, wrong culture.
2. **Fyrox 1.0** — just hit stable in March 2026. Editor-centric despite
   being Rust. Would re-introduce the editor problem.
3. **Unity / Unreal** — closed source, ruled out.
4. **Bootstrap from wgpu + bevy_ecs + rapier + kira** — possible, but would
   spend engineering effort on engine-shaped problems rather than
   game-shaped ones. Re-evaluate if we hit a hard limit in Bevy.
5. **Macroquad / ggez** — too minimal for ambitious scope.

---

# Part II — Design Philosophy

## 4. Core principles

These are the principles that govern all subsequent design. They are
ordered so that earlier principles constrain later ones.

1. **Typed interfaces at observable boundaries.** Every boundary that an
   external observer (an agent, a network peer, a test harness) crosses
   must be expressed as typed data structures, not as free-text or
   implicit state. The command/observation surface is the canonical
   example.

2. **Determinism is a workspace-level invariant.** Given the same
   initial state and the same command sequence, the simulation produces
   byte-identical trajectories across runs, machines, and speeds.
   Determinism is not a feature; it is a property the CI enforces.

3. **Commands are the only mutation surface.** Every change to simulation
   state happens because a typed command was applied. No exceptions — not
   for UI, not for debugging, not for admin overrides. Debug commands are
   still commands.

4. **Presentation is strictly read-only over simulation.** Rendering
   systems, UI, and audio observe simulation state but cannot mutate it.
   Enforced by crate boundary.

5. **Content is data, not code.** Gameplay-affecting values — item stats,
   enemy definitions, dungeon layouts, dialogue trees, balance numbers —
   live in RON/JSON/`.yarn` files, not in Rust constants. The only code
   that references content is the loader, and all other references are
   by content ID.

6. **The base game is a privileged mod.** The game's own content loads
   through the same VFS and manifest machinery that third-party mods
   use. There is no "built-in" path that bypasses the mod loader.

7. **Test time scales with changes, not content size.** Adding a dungeon
   cannot double the CI time. CI runtime must scale with the number of
   subsystems a change touches, not with the total volume of game content.

8. **Every observable outcome is versioned.** Save files carry version
   tags; replays are pinned to content hashes; command schemas carry
   version numbers; observation schemas carry version numbers. Breaking
   changes require explicit version bumps and typed migrations.

## 5. Non-negotiable invariants

These are hard rules. Violations fail code review. They are the game-dev
analog to tanren's seven-rule Dependency DAG.

1. **Presentation never mutates simulation state.** Enforced by crate
   boundary: `game-presentation` cannot declare `Query<&mut T>` where `T`
   is a simulation-owned component. Lint or architectural test in CI.

2. **Commands are the only simulation mutation.** No system outside the
   command-application system writes to simulation-owned components.
   Enforced by convention + review; in principle lintable.

3. **Deterministic simulation.** No `std::time::Instant` in simulation
   crates. No `rand::thread_rng()`. No `HashMap` iteration order
   dependence (use `BTreeMap` or sorted iteration where order matters).
   No `Rc`/`Arc` in simulation state (they can't be snapshotted reliably).
   Enforced by workspace lints + dedicated test that runs the same seed
   twice and compares state hashes.

4. **Time is simulation-ticks, not wall-clock.** Gameplay timers count
   ticks (`cooldown_ticks: u32`), not durations. Wall-clock only enters
   through the input layer (when the player pressed a key) and the render
   layer (frame pacing). Enforced by convention + dedicated test that
   runs the same episode at 60 TPS and at unbounded TPS and asserts
   identical end state.

5. **Observations are snapshots, not references.** Serializable values,
   not borrows into live world state. Agents must never hold a reference
   that could change under them.

6. **Input adapters produce commands, never mutate directly.** Keyboard,
   gamepad, touch, network packet — all produce `GameCommand` values that
   flow through the same queue. Enforced by crate structure.

7. **Content is data.** No gameplay-affecting magic numbers in Rust
   outside `game-domain` type definitions. Enforced by grep in CI against
   simulation crate source trees. Balance numbers, drop rates, ability
   stats live in RON/JSON.

8. **Command and observation schemas are versioned.** Any breaking change
   to `GameCommand` or `Observation` enums increments a schema version.
   Old replays and save files either migrate forward or are explicitly
   invalidated. Enforced by snapshot test against committed schema hash.

9. **No full-engine dependency in simulation crates.** `game-simulation`
   depends on `bevy_ecs`, `bevy_app`, `bevy_time`, `bevy_math` — the
   individual sub-crates, not the `bevy` umbrella. This keeps the server
   and MCP binaries small and fast to build. Enforced by `cargo check`
   against a minimal feature set in CI.

---

# Part III — Architecture

## 6. Crate topology

The workspace is a flat Cargo workspace following tanren's `rust-cargo`
profile conventions (§28). Library crates in `crates/`, binary crates
in `bin/`.

```
crates/
  game-domain          # Pure types: components, events, errors
  game-command         # GameCommand enum, validation, application
  game-observation     # Observation types, per-client filtering
  game-content         # Content VFS, loaders, manifest
  game-simulation      # FixedUpdate systems, headless
  game-replay          # Episode recording, replay, snapshots
  game-presentation    # Rendering, UI, audio
  game-input           # Device → command adapters
  game-net             # Network protocol (deferred implementation)
  game-mcp             # MCP transport (same shape as game-net)
  game-mod-api         # Mod loading, hook registration
  game-testing         # Test harness, fixtures, assertions
  game-platform        # Platform-specific shims (desktop/web/mobile)

bin/
  game                 # Full client: presentation + input + simulation
  game-server          # Dedicated server: simulation + net
  game-mcp             # Agent interface: simulation + mcp transport
  game-headless        # Batch runner: replay, fuzz, balance
  game-balance         # Monte Carlo balance reports

xtask/                 # Rust-native task runner for complex CI
```

The crate layering rules (analog of tanren's Dependency DAG):

1. **Domain rule**: `game-domain` depends only on standard crates, `serde`,
   `thiserror`, `bevy_ecs` (standalone). No other workspace crate.
2. **Simulation rule**: `game-simulation` depends on `game-domain`,
   `game-command`, `game-observation`, `game-content`, and the
   minimal Bevy sub-crates. Never on `game-presentation`, `game-input`,
   or the full `bevy` umbrella.
3. **Presentation rule**: `game-presentation` depends on `game-simulation`
   (for read access to simulation state) and full Bevy. Never writes to
   simulation-owned components.
4. **Command rule**: `game-command` depends on `game-domain`. Command
   application logic lives here and is pure; its dependency on Bevy is
   limited to `bevy_ecs` (systems as functions over queries).
5. **Replay rule**: `game-replay` can depend on `game-simulation` and
   serialization utilities. It must not depend on presentation.
6. **Content rule**: `game-content` owns content loading and validation.
   It does not execute game logic; it produces typed content values that
   `game-simulation` consumes.
7. **Mod rule**: `game-mod-api` provides hook registration and VFS
   layering. Mods themselves are content packages + optional script
   modules (later).
8. **Binary rule**: each binary is thin. It composes plugins. It owns no
   game logic.

A diagram of the dependency graph:

```
               game-domain
              /    |    |    \
             /     |    |     \
   game-command  game-content  game-observation
            \    /       \    /
             \  /         \  /
         game-simulation ───┐
          /  |  |    \      │
         /   |  |     \     │
 game-replay │  │   game-net│
             │  │     │     │
             │  │     │   game-mcp
             │  │     │     │
    game-presentation game-mod-api
         │
    game-input
```

This graph is enforceable. `cargo-deny` and/or `cargo-modules` can be
configured to fail builds if crates grow forbidden dependencies.

## 7. The domain layer

`game-domain` is the smallest crate and the one with the strictest rules.

**Contents:**

1. Component types (`#[derive(Component)]` on pure data).
2. Event types (`#[derive(Event)]` on typed events that simulation emits).
3. Error types (`thiserror`-derived, used across the workspace).
4. ID newtypes (wrapping `uuid::Uuid` for entity references, content
   references, session tokens, etc.).
5. Content-as-type definitions (item schemas, enemy schemas, ability
   schemas) — the *shape* of content, not content values themselves.

**What does not belong:**

1. Systems, queries, or any Bevy `App` registration.
2. Content *values* (those live in `game-content`).
3. Any I/O, any `tokio`, any async.
4. Any presentation concern (no colors, no icons, no text beyond IDs).

**Dependencies allowed:**

- `bevy_ecs` (for the `Component` and `Event` derives)
- `serde`, `serde_json`, `ron`
- `thiserror`
- `uuid`
- No other workspace crate

**Rationale:** This crate is the lingua franca of the project. Every
other crate imports from it. Keeping it minimal keeps compile times
fast, keeps the surface area auditable, and ensures the domain model
is not polluted by presentation or infrastructure concerns.

## 8. Command and observation surface

The command/observation interface is the architectural core. Everything
else is in service to it.

### 8.1 Commands

`GameCommand` is an enum defined in `game-command`. Every way to mutate
simulation state is a variant.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GameCommand {
    // Movement
    Move { direction: Vec2, sprinting: bool },

    // Combat
    UseAbility { ability_id: AbilityId, target: CommandTarget },
    Attack { target: EntityId },

    // Interaction
    Interact { target: EntityId },
    PickUp { item_entity: EntityId },
    UseItem { item_id: ItemId },

    // Dialogue
    AdvanceDialogue,
    ChooseDialogueOption { option_index: u8 },

    // Meta
    Pause,
    Resume,
    SaveGame { slot: SaveSlot },
    LoadGame { slot: SaveSlot },

    // Debug (only available when debug capability is granted)
    DebugSpawn { entity_kind: EntityKind, location: Vec2 },
    DebugSetHealth { entity: EntityId, hp: i32 },
}
```

**Properties:**

1. Every command is serializable.
2. Every command has a typed target (no string IDs, no magic numbers).
3. Debug commands are gated by a capability system; in production builds
   they refuse to execute.
4. The enum is `#[non_exhaustive]` to allow additions without breaking
   consumers at compile-time; breaking changes increment schema version.
5. Commands are validated before application. Validation is pure.

**Command application** is a dedicated system in `game-simulation` that
consumes a `CommandBuffer` resource, validates each command against
current state, and applies valid commands as component mutations.

**Command sources** — anything that produces commands must be a dedicated
adapter system in a dedicated crate:

1. `game-input` — keyboard, gamepad, touch → `GameCommand`
2. `game-mcp` — MCP tool call → `GameCommand`
3. `game-net` — network packet → `GameCommand` (deferred)
4. `game-replay` — recorded command → `GameCommand` (during replay)
5. AI policy systems (in `game-simulation` or in mods) — heuristic or
   learned policy → `GameCommand` (for bot opponents, scripted NPCs)

All of these deposit commands into the same `CommandBuffer`. The
simulation does not know which source produced which command.

### 8.2 Observations

`Observation` is the complement. It's what a command-issuing agent can
see of the world.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u32,
    pub tick: SimulationTick,
    pub player: PlayerObservation,
    pub visible_entities: Vec<EntityObservation>,
    pub inventory: InventoryObservation,
    pub dialogue_state: Option<DialogueObservation>,
    pub flags: FlagsObservation,
    pub recent_events: Vec<EventObservation>,
}
```

**Properties:**

1. Entirely self-contained — no references into live world state.
2. Per-client filtered. A multiplayer server computes one observation
   per connected client, with visibility rules applied. Single-player
   computes one observation per tick with full visibility.
3. Serializable to JSON for MCP transport and to a compact binary format
   for network transport.
4. Versioned. Schema version is part of the observation; consumers can
   check compatibility before parsing.

**Observation extraction** is a dedicated system that reads simulation
state (via `Query`) and produces `Observation` values for each consumer.
It is read-only.

### 8.3 The boundary

Commands in, observations out. This is the complete contract between
the simulation and any external consumer:

```
┌─────────────────────────┐      ┌─────────────────────────────────┐
│  External consumer      │      │  Simulation                     │
│  (client, agent, test)  │      │                                 │
│                         │      │                                 │
│                         │ GameCommand                            │
│                         │─────────────────────────────▶         │
│                         │      │                                 │
│                         │      │  Tick advances                  │
│                         │      │  Commands applied               │
│                         │      │  Events emitted                 │
│                         │      │                                 │
│                         │ Observation                            │
│                         │◀─────────────────────────────          │
│                         │      │                                 │
└─────────────────────────┘      └─────────────────────────────────┘
```

Nothing crosses this boundary except typed commands and typed
observations. No shared memory, no direct component access, no
reaching in.

## 9. The simulation layer

`game-simulation` contains the systems that advance game state. All
simulation systems run in `FixedUpdate`.

**Responsibilities:**

1. Spawn and despawn entities based on rules (enemy waves, item drops,
   projectile lifetimes).
2. Apply commands via the command-application system.
3. Enforce game rules (collision, damage calculation, AI decisions).
4. Emit events for significant state changes.
5. Compute observations after each tick.

**What it does not own:**

1. Rendering — `game-presentation` reads state but does not own the
   tick loop.
2. Input interpretation — `game-input` translates devices to commands
   but does not reach into simulation state.
3. Network synchronization — `game-net` sends and receives serialized
   commands and observations but does not own simulation timing.

**FixedUpdate discipline:**

1. Default tick rate: 60 TPS. Configurable at app startup via
   `Time<Fixed>`.
2. Systems that affect simulation state run in `FixedUpdate`.
3. Systems that read simulation state for presentation run in `Update`.
4. Interpolation between ticks happens in presentation, not simulation.
5. Catch-up behavior: if wall-clock time advances faster than simulation
   (e.g., after a pause), `FixedUpdate` runs multiple times per frame to
   catch up. This is transparent to systems as long as they don't use
   wall-clock directly.

**Determinism enforcement:**

1. Seeded RNG as a resource: `SimulationRng` holds a `ChaCha8Rng` or
   similar seeded at run start. All random draws go through it.
2. No `rand::thread_rng()` or `SmallRng::from_entropy()` in simulation
   crates. Enforced by lint or grep.
3. No `HashMap` iteration without sorting. Use `BTreeMap` where order
   matters for determinism.
4. No system runs in parallel with another system that has any
   observable data dependency. Explicit `.before()`/`.after()` ordering
   on every system that writes to state.

## 10. The presentation layer

`game-presentation` is strictly a consumer of simulation state.

**Responsibilities:**

1. Render the world to the screen.
2. Play audio in response to simulation events.
3. Display UI (HUD, menus, dialogue windows).
4. Interpolate between simulation ticks for smooth visuals at render
   FPS higher than simulation TPS.

**What it does not do:**

1. Mutate simulation state.
2. Own any gameplay logic.
3. Produce commands (that's `game-input`'s job).

**Headless-parity invariant:**

A headless run (no presentation) and a presented run (full rendering)
must produce byte-identical simulation trajectories when given the
same seed, commands, and content. If they diverge, presentation has
leaked into simulation. This is a test in CI.

**FPS decoupling:**

1. Presentation runs in `Update` at whatever rate the main loop
   achieves.
2. For each rendered frame, presentation reads the two most recent
   simulation snapshots and interpolates visual state.
3. Target render FPS: 144+ as the design commitment. The game should
   feel smooth on high-refresh monitors.
4. VSync is the player's choice; we don't force it.

## 11. Content VFS and mod support

Content — sprites, audio, RON data files, `.yarn` dialogue trees,
localization strings — lives in a **layered virtual filesystem**. The
game, third-party mods, and user-authored content all use the same
loading path.

### 11.1 The layered VFS

```
VFS layers (priority: higher overrides lower)
  ┌─────────────────────────────┐
  │  N: player-local overrides  │  (savefile-adjacent customizations)
  ├─────────────────────────────┤
  │  N-1: third-party mods      │  (ordered by user config)
  │  ...                        │
  ├─────────────────────────────┤
  │  1: DLC / official add-ons  │  (if any)
  ├─────────────────────────────┤
  │  0: base game               │  (ships with the binary)
  └─────────────────────────────┘
```

Every content lookup walks the layers top-down and returns the first
match. The base game is a mod with priority 0; it has no special
bypass.

### 11.2 Mod manifest

Each mod ships a `mod.ron` manifest:

```ron
(
    id: "shopkeeper-expansion",
    name: "Shopkeeper Expansion",
    author: "Example Author",
    version: "1.2.0",
    game_compat: "^0.5",
    mod_api_compat: "^1.0",
    content_roots: ["content/"],
    dialogue_roots: ["dialogue/"],
    asset_manifest: "assets.manifest.ron",
    dependencies: [
        (id: "base-items", version: "^0.5"),
    ],
    content_hash: "sha256:...",
)
```

**Properties:**

1. Every mod declares its semver compatibility with both the game version
   and the mod API version.
2. Every mod declares its content hash; mismatch fails loading.
3. Dependencies on other mods are explicit.
4. The base game's `mod.ron` is generated at build time from the build
   metadata.

### 11.3 Mod API

Mods extend the game through a typed hook surface, not arbitrary code
execution.

**Initial API (data mods only):**

1. Add new content (items, enemies, abilities, dungeons, dialogue trees).
2. Override existing content by ID (fully replace or patch specific
   fields).
3. Register localization overlays.
4. Register new asset paths.

**Deferred API (code mods):**

1. Register typed event hooks (`on_entity_spawn`, `on_damage_resolved`,
   `on_dialogue_node_entered`).
2. Contribute scripted policies (NPC behavior, AI opponents).
3. Add new `GameCommand` variants (with schema version coordination).

**Code mod sandboxing** (deferred): WASM via `wasmtime` is the probable
choice. The sandbox gives deterministic execution, resource limits, and
no host access beyond a declared capability set. This is a significant
spec later, not for the first project.

### 11.4 Asset handling

A key practical concern for remote-VM development (tanren uses ephemeral
cloud VMs for agent workspaces).

**Principles:**

1. Content text files (RON, `.yarn`, localization) are in git directly.
2. Binary assets (sprites, audio, shaders) are either in git-lfs or in
   an external object store (S3/R2) keyed by content hash.
3. The asset manifest lives in git and specifies where each asset lives.
4. `just fetch-assets` populates the VM's asset cache from the manifest,
   verifying hashes.
5. CI enforces that no code references an asset not declared in a
   manifest.

**Asset manifest entry:**

```ron
(
    path: "sprites/player/idle.png",
    hash: "sha256:...",
    size: 4096,
    source: RemoteStore(bucket: "game-assets-v0", key: "sprites/player/idle.png"),
)
```

**Rationale:** This lets agents on fresh VMs pull only the assets they
need for a given task (an agent working on combat doesn't need dialogue
audio), and guarantees replay reproducibility because asset hashes are
pinned into replay metadata.

## 12. Time and simulation rate

Three independent dials.

1. **Simulation TPS** (`SimulationRate` resource): default 60, set per
   project. Bounded in normal play. Unbounded in headless CI.
2. **Wall-clock scale** (`SimulationScale` resource): default 1.0. Set
   higher for fast-forward in debug or time-skipped content. Only
   meaningful when TPS is bounded.
3. **Render FPS**: VSync-bound in normal play, uncapped for benchmarks,
   disabled in headless.

### 12.1 Tick-based vs time-based logic

**Tick-based** (count ticks directly):

- Cooldowns (`cooldown_ticks: u32`)
- Turn counters
- Discrete effect durations
- Any logic where "N times per second" needs to be exact

**Time-based** (multiply by `time.delta_seconds()`):

- Continuous movement (velocity × delta)
- Continuous animation (if any runs in simulation, which is rare)
- Continuous effects with non-integer scaling

**Rule:** use tick-based for any discrete event, time-based only when the
underlying physics are genuinely continuous. Mixing is the category of
TPS-dependent bugs.

### 12.2 Determinism-equivalence test

In CI, a sample episode runs at each of 30, 60, 120, 240 TPS and at
unbounded TPS. The end-state hash must match across all runs (modulo
documented floating-point tolerance). If a change introduces TPS-dependent
behavior, this test catches it immediately.

### 12.3 CI speed

In headless/CI mode, `SimulationRate` is set to unbounded. `FixedUpdate`
runs back-to-back at CPU speed. Any wall-clock `sleep` or `wait` in
simulation code is a bug. This is why the "no `std::time` in simulation"
rule is load-bearing — otherwise CI can't fast-forward.

Budget math (representative):

1. Episode = 5 simulated minutes = 18,000 ticks @ 60 TPS.
2. Balance tier = 1,000 episodes = 18M ticks.
3. Full-gate replay corpus = ~10M ticks.
4. Budget: ~10 min wall-clock for simulation in a full gate.
5. Required: 50,000 TPS on a single core, or ~12,000 TPS per core on a
   4-core runner, or ~3,000 TPS per core on a 16-core runner.

For a roguelike, this is achievable. For dense physics or heavy AI, the
numbers tighten and may demand parallel episode execution. Either way,
the simulation tick time budget is explicit: **<0.3ms per tick on
CI-class hardware** for the first project, revisable.

## 13. Save files, replay, and snapshots

One serialization format, three uses.

### 13.1 The common abstraction

A `SerializableWorld` captures the full simulation state at a single
tick. It is a `serde`-compatible struct containing all component data
for every entity, all resources, and metadata (tick number, seed,
content hash, schema version).

This one type backs:

1. **Save files**: serialize current state, persist to disk, restore on load.
2. **Replay checkpoints**: serialize at key moments during recording; on
   replay, restore to a checkpoint and proceed from there.
3. **Demo setup**: a demo script may start from a serialized initial
   state rather than building state from commands.
4. **Test fixtures**: unit tests load a serialized fixture state rather
   than constructing it programmatically.
5. **Network state sync**: new clients receive a serialized state to
   initialize from; subsequent sync is command-driven.

### 13.2 Replay format

```
Episode {
    seed: u64,
    content_hash: Sha256,
    schema_version: u32,
    initial_state: Option<SerializableWorld>,  // or None for "from scratch"
    commands: Vec<(SimulationTick, GameCommand)>,
    checkpoints: Vec<(SimulationTick, SerializableWorld)>,  // optional, for fast seeking
    terminal: TerminationReason,
    final_state_hash: Sha256,  // for integrity verification
}
```

An episode can be replayed deterministically given the same content.

### 13.3 Save file versioning

Breaking changes to `SerializableWorld` require a migration. Migrations
are typed transforms:

```rust
fn migrate_v3_to_v4(old: WorldV3) -> Result<WorldV4, MigrationError> {
    // explicit transform, no magic
}
```

Loading a save runs all applicable migrations in sequence. Save files
older than the minimum supported version are rejected with a clear
error message, not silently corrupted.

### 13.4 Golden checkpoints for test scaling

For a long game, test scaling demands that no test plays the full game
from scratch. Instead, the CI maintains a set of **golden checkpoints** —
serialized world states at key progression points.

For the first project:

1. `checkpoint_start_of_run.world`
2. `checkpoint_after_first_room.world`
3. `checkpoint_at_reward_room.world`
4. `checkpoint_after_polite_shopkeeper.world`
5. `checkpoint_after_hostile_shopkeeper.world`
6. `checkpoint_at_boss_room.world`

Tests that exercise the boss room load `checkpoint_at_boss_room.world`
and proceed. They do not play the first six rooms first.

Checkpoints are content — they are invalidated when upstream content
changes. The invalidation is automatic: each checkpoint stores the
content hash it was generated against. CI detects mismatches and
regenerates invalidated checkpoints on a schedule.

For a hypothetical 40-hour game, probably 50-100 checkpoints, each
representing ~30 minutes of content. Tests run only the segment after
the checkpoint, not the path to it.

## 14. Server runtime and network topology

### 14.1 The server binary

`bin/game-server` is a dedicated server: it runs simulation, accepts
commands from connected clients, and sends observations.

**Crate dependencies:**

1. `game-domain`, `game-command`, `game-observation`, `game-simulation`,
   `game-content`, `game-replay` — needed.
2. `game-net` — needed (transport, session management, authentication).
3. `game-presentation`, `game-input`, `game-ui` — **must not compile
   into the server**.

**Enforcement:** `cargo check -p game-server --no-default-features` in
CI ensures no presentation dependency leaks in.

**Observation:** `bin/game-server` and `bin/game-mcp` are the same binary
with different transports. An MCP agent is structurally a networked
player. Building `game-mcp` first (first project) gets us 80% of
`game-server` (later project).

### 14.2 Topology options

The framework supports all four topologies by preserving the invariants
that keep them possible. The choice of topology is a per-game decision
deferred to when multiplayer is actually a spec.

1. **Single-player** (trivially): client and server are the same process;
   the "network boundary" is a method call.
2. **Central-authoritative**: server runs simulation, clients render.
   Easy anti-cheat. Standard for most modern online games.
3. **Deterministic lockstep P2P**: every peer runs simulation; commands
   gossip. Requires byte-level determinism. Low bandwidth, high latency.
   Classic RTS / turn-based.
4. **Rollback netcode P2P**: each peer runs local prediction, rolls back
   on late inputs. Low perceived latency. Complex. `ggrs` is the Rust
   reference implementation.
5. **Hybrid authoritative + prediction**: server-authoritative, clients
   predict for responsiveness, server reconciles. Standard modern
   competitive multiplayer (Overwatch, Valorant pattern).

**Invariants that preserve optionality:**

1. Deterministic simulation (§12.2) — required for all P2P modes.
2. Commands as only mutation (§5, §8) — required for all modes.
3. Serializable state (§13) — required for state sync on connect.
4. Per-client observation filtering (§8.2) — required for anti-cheat
   and fog-of-war.
5. Stable versioned command/observation schemas (§5) — required for
   any network transport.
6. No `Rc`/`Arc` in simulation state (§4, §5) — required for byte-level
   determinism.

**Anti-cheat**: commands validated server-side before application,
observations filtered server-side before transmission, RNG seeded
server-side. No anti-cheat design in the first project; design
patterns that allow it later.

## 15. RL bridge

Full RL training is a deferred project. The architecture exposes a
Gymnasium-compatible interface from day one so that training can slot
in without simulation-layer changes.

### 15.1 Interface shape

```python
# game_rl_bridge (Python, via PyO3)

env = game_rl.make("shugyou-v0", seed=42, render_mode=None)
observation, info = env.reset(seed=42)
for _ in range(1000):
    action = policy.choose(observation)
    observation, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        observation, info = env.reset()
```

The Python package wraps a Rust `game-mcp`-like server, exposing it
as a `gymnasium.Env`. The same Rust code serves both LLM agents over
MCP and RL policies over the Gymnasium interface.

### 15.2 Vector environments

Training throughput requires parallel episodes. The server binary
supports a vector mode:

```
bin/game-server --vector 32 --transport gymnasium-ipc
```

This runs 32 independent simulation instances, round-robin stepping
them on a single process. Python wrapper presents them as a single
`VectorEnv` to the training algorithm.

### 15.3 Observation and action spaces

Gymnasium 2026 supports richer spaces: Graph, Text, Sequence, disjoint
unions. The roguelike's natural observation is heterogeneous:

```python
observation_space = Dict({
    "player": Box(low=..., high=..., shape=(10,)),  # position, hp, stats
    "inventory": Sequence(Discrete(n_items)),
    "visible_map": Graph(node_space=..., edge_space=...),  # room graph
    "dialogue": Optional(Text(max_length=500)),
    "flags": MultiBinary(n_flags),
})

action_space = Dict({
    "movement": Discrete(9),  # 8 directions + none
    "primary": Discrete(n_abilities + 1),  # abilities + attack
    "dialogue_choice": Discrete(n_dialogue_options + 1),  # +1 for none
})
```

### 15.4 Offline RL and Minari

Every recorded episode (from any command source) is a valid training
sample. A Minari-compatible export from the replay corpus allows
offline RL and imitation learning without dedicated training runs.

### 15.5 Multi-agent via PettingZoo

If multiplayer lands, PettingZoo is the right API to target. Same
Rust server binary, different wrapper.

### 15.6 What this is not (yet)

1. A training stack. Training is Python, uses existing tools
   (SB3, CleanRL, RLlib).
2. A commitment to Rust-native ML. `burn` and `candle` exist; we use
   them only if there's a compelling reason.
3. A reason to build training infrastructure now. The interface makes
   training possible later; nothing currently requires training.

---

# Part IV — Testing and CI

## 16. The scaling invariant

> **CI time must scale with the size of a change, not with the size of
> the game.**

A PR that touches one crate should run tests for that crate and its
reverse dependencies — not the entire test suite. A PR that adds a
new dungeon should run tests for that dungeon's content and the
shared systems it uses — not tests for the other dungeons.

Without this invariant, a 40-hour game has unworkable CI. This is
probably the single most important architectural commitment in the
framework.

## 17. Tiered gates

CI is stratified into tiers with explicit time budgets. Different
tiers run at different points in the workflow.

1. **Fast-gate** (per-task, <90s): `cargo fmt --check`, `cargo clippy
   -D warnings`, `cargo check`, unit tests on crates touched by the
   change. Runs on every agent commit during `do-task`.

2. **Standard-gate** (per-task, <3min): fast-gate + scoped integration
   tests (crates touched and their direct reverse dependencies). Runs
   as the `TASK_VERIFICATION_HOOK` for `do-task`.

3. **Spec-gate** (per-spec, <10min): standard-gate + subsystem-scoped
   demos + small balance samples + documentation build. Runs as the
   `SPEC_VERIFICATION_HOOK` when tasks complete and before `audit-spec`.

4. **Nightly gate** (<1hr, scheduled): full demo corpus + full balance
   sweep + cross-subsystem tests + TPS sweep + determinism-equivalence
   tests. Catches issues that per-spec scoping missed.

5. **Pre-release gate** (<4hr, on release candidate branch): nightly +
   full checkpoint replay suite + localization smoke + all-platform
   builds + binary size budgets.

6. **Release gate** (hours, before tagged release): pre-release +
   extended fuzz + load tests + release artifact validation + mod
   compatibility smoke.

Per-PR developer experience: fast-gate on commit (< 90s), standard-gate
on PR open (< 3min). Agents see fast feedback; expensive checks wait.

## 18. Subsystem tagging

The primary mechanism for scoped testing.

### 18.1 Spec frontmatter

Specs declare the subsystems they touch:

```yaml
---
spec_id: S-0412
title: Reduce dash cooldown
subsystems:
  - simulation.combat
  - simulation.abilities
  - content.abilities
  - presentation.hud  # for cooldown indicator
---
```

### 18.2 Test tagging

Tests declare the subsystems they exercise:

```rust
#[test]
#[subsystem(simulation::combat)]
#[subsystem(simulation::abilities)]
fn dash_cooldown_prevents_double_use() { ... }
```

The `subsystem` attribute is a custom proc macro (or a simple
convention parsed by `xtask`) that registers the test under the given
subsystem tags.

### 18.3 Test selection

`just ci-for-spec S-0412` resolves the spec's subsystems, unions them
with subsystems that declare dependencies on those (via a subsystem
dependency graph, maintained alongside the crate dependency graph),
and runs only matching tests.

Subsystem taxonomy is hierarchical:

```
simulation.*
    simulation.combat
    simulation.abilities
    simulation.movement
    simulation.ai
    simulation.dialogue
content.*
    content.abilities
    content.items
    content.dungeons
    content.dungeons.dungeon_1
    content.dialogue
presentation.*
    presentation.rendering
    presentation.hud
    presentation.audio
infrastructure.*
    infrastructure.save
    infrastructure.replay
    infrastructure.networking
    infrastructure.mod_loading
```

A wildcard match (`simulation.*`) subscribes to any child. This lets
tests of "core simulation" tag once and catch changes in any
simulation subsystem.

### 18.4 Upstream to tanren

Subsystem tagging is not game-specific. A backend spec touches
`api.billing`, a database migration touches `store.schema`. This
pattern is worth contributing back to tanren as a general-purpose
feature of the rust-cargo profile or of tanren's orchestrator.

## 19. Demo scripts — three-tier layered

A `DemoScript` is a structured artifact produced by `shape-spec` and
consumed by `run-demo`. It exercises the feature end-to-end.

### 19.1 The three layers

1. **Invariant assertions** (rigid, near-immutable): properties that
   must hold for the feature to be correct. "After dash is used,
   cooldown resource equals dash_cooldown_duration." Machine-verifiable.
   Edits require re-shaping the spec (treated as scope change).
   Typical count: 3-5 per spec.

2. **Behavioral traces** (medium rigid): command sequences with
   expected observation deltas. "Issue Dash → expect player position
   to advance by dash_range within 10 ticks." Tolerates minor timing
   variation. Machine-verifiable with slack. Edits require explicit
   task-level justification, audited at `audit-task`. Typical count:
   2-4 per spec.

3. **Narrative demos** (soft, LLM-graded): prose description of what
   a human observer should see. "Player dashes forward, leaves a
   brief afterimage, cannot dash again for half a second." Graded by
   the `run-demo` agent comparing its own observation stream to the
   narrative. Not strictly machine-verifiable. Edits free — intentionally
   non-binding. Typical count: 1 per spec.

### 19.2 Execution

`run-demo` walks the script:

1. Load initial state (from seed or serialized fixture).
2. For each step:
   - If a command: issue it via the command interface.
   - If an invariant assertion: evaluate against current state; fail hard
     on mismatch.
   - If a behavioral trace: issue commands, sample observations, compare
     to expected with tolerance; fail on out-of-tolerance divergence.
   - If a narrative checkpoint: ask the run-demo agent (an LLM) to
     compare the observation stream to the narrative; accept confident
     agreement, flag disagreement for review.
   - If a timing marker: advance N ticks.
3. Report: pass / fail / warn, with per-step evidence.

### 19.3 Fuzz corpus as backstop

The random-policy fuzz corpus runs on every spec-gate. It doesn't
test features directly — it asserts domain invariants hold across
thousands of random-input episodes (health never negative, no entity
leaks, content references resolve). A spec that passes its own demo
but breaks invariants in the fuzz corpus fails the gate.

This catches the category of demo-gaming where an agent tunes
behavior to pass the specific demo while breaking general invariants.

## 20. Replay regression and golden trajectories

A **golden trajectory** is a committed episode whose end-state hash
is pinned. CI replays all committed golden trajectories on every
spec-gate and asserts hash equality.

Breaking a golden trajectory fails CI. This catches any change that
alters simulation behavior, even if all other tests pass.

Intentional trajectory changes (e.g., a balance patch that changes
combat outcomes) require regenerating the affected trajectories as
part of the spec. The regeneration is an explicit artifact, visible
in review.

For the first project, probably 20-50 golden trajectories covering
main flows. For a long game, hundreds, organized by content area.

## 21. Performance budgets

### 21.1 Simulation tick time

Target: **<0.3ms per tick on CI-class hardware** for the first project.
Enforced by a benchmark test. Regressions fail the build.

### 21.2 Test tier timing

Per §17. Fast-gate <90s, standard <3min, spec <10min, nightly <1hr,
pre-release <4hr. Tier timing is measured; if a tier consistently
exceeds budget, either split into sub-tiers or optimize.

### 21.3 Compile time

Clean workspace build <5min on CI-class hardware (revisable).
Incremental build <30s after a single-crate change. Small crates
help; large crates hurt. This is why strict crate boundaries are
also a performance concern.

### 21.4 Binary size

Per-platform release binary size budgets (wasm especially — <20MB
compressed for web builds). Enforced on pre-release gate.

### 21.5 Frame budget

Rendering target: 144+ FPS on consumer hardware for the first
project. Measured via in-engine frame time tracking; reports in
release-gate artifacts.

---

# Part V — Tanren Integration

## 22. How specs shape game work

Tanren's `shape-spec` is interactive — Trevor plus an agent define
the feature together. For game work, `shape-spec` produces a richer
artifact set than for backend work:

1. A written spec (same as any tanren spec).
2. A subsystem declaration (§18.1).
3. A `DemoScript` with three-tier layering (§19).
4. A test plan sketch listing expected test additions.
5. A performance budget delta (if any) — does this spec change
   simulation cost?
6. Schema version implications — does this spec add a `GameCommand`
   variant, a new component type, a new content kind?

The demo script is the deliverable that most distinguishes game specs
from backend specs. Without a concrete demo, "feature complete" is
not verifiable.

## 23. The per-task gate loop

Unchanged from tanren's backend loop in shape, specialized in
`just check` and `just ci` contents for game work:

1. **`do-task`** — agent implements the task's scope. Calls
   `start_task` at session start, records signposts for non-obvious
   decisions, calls `complete_task` with evidence refs at end.
2. **Task verification hook** (`just check`) — fast-gate from §17.
   Runs during the implementation session.
3. **`audit-task`** — rubric scoring against 10 quality pillars
   (tanren's rubric, customized per repo). For game work, pillars
   extend with determinism adherence, frame-budget impact, content
   integrity.
4. **`adhere-task`** — checks against repo-specific standards
   (standards files shipped from `profiles/rust-bevy`). For game
   work, this checks things like "new UI strings use Fluent keys,"
   "new assets are in the manifest," "new content references resolve."
5. All three guards must pass for the task to transition to Complete
   (monotonic — Complete is terminal).

## 24. Spec-level checks

After all tasks complete:

1. **`audit-spec`** — reviews the full branch diff for cross-task
   issues. Checks that the combined changes are coherent, not just
   individually correct.
2. **Spec verification hook** (`just ci`) — full spec-gate from §17.
3. **`run-demo`** — walks the `DemoScript` step by step. If every
   step passes, emits a demo success artifact. If any step fails,
   emits a signpost and escalates.
4. **`walk-spec`** (interactive) — Trevor reviews the demo output,
   asks questions, approves or requests changes. Concludes with
   PR readiness.

## 25. Parallelism strategies

Three modes tanren supports, applied to game work:

1. **Fully parallel** (disjoint specs on separate branches): the
   default. Two specs touching disjoint subsystems run concurrently
   with no coordination. Merge conflicts are rare.
2. **Stacked diffs**: spec B depends on spec A. Spec B can start work
   when A reaches review. If A needs edits, B rebases. **Constraint
   specific to game work**: spec B can stack on spec A only if A does
   not break `GameCommand` or `Observation` schemas. Breaking
   schema changes force strict sequential merging.
3. **Intra-spec task parallelism**: multiple agents on different tasks
   within a single spec. Least useful for games (game features tend
   to be tightly coupled through the simulation stack), but available.

The practical velocity multiplier is from #1 and #2. #3 is reserved
for specs where tasks are genuinely independent (e.g., "add three new
enemy types" where each enemy is a separate task).

## 26. Merge-conflict resolution as agentic command

Tanren supports agent-driven merge resolution via `merge-parent` and
`merge-parallel`. Both commands receive spec context — the intent of
the conflicting specs, not just the diff — and plan merge strategies
that preserve intent.

**Game-specific context for these commands:**

1. **Per-spec changelogs**: when a spec completes, it emits a structured
   changelog listing types added, types extended, new content IDs,
   new commands, new components. `merge-parent` reads the upstream
   spec's changelog.
2. **Content UUID stability**: content additions use stable UUIDs
   and append-only order. Merges rarely conflict textually even when
   both specs add content.
3. **Schema version coordination**: breaking schema changes trigger
   explicit version bumps that `merge-*` commands surface as
   high-attention items.

## 27. Early access and feedback loop (future use case)

When the game reaches Steam Early Access, the tanren workflow extends:

1. **`triage-feedback`** (new command): processes raw player feedback
   into structured candidates. Classifies (bug/balance/feature/
   content/UX), aggregates by frequency, proposes spec candidates.
2. **Weekly release cadence**: each week's patch is composed of
   specs completed in that window. Release notes auto-generated from
   spec changelogs.
3. **Save migration per release**: every release that changes
   persistent state ships a migration. Tested against saved player
   states captured in CI.
4. **Feature flags as gradual rollout**: flip a flag for a percentage
   of the player base, monitor telemetry, expand if clean.
5. **Public roadmap**: spec state exposed as a public view. Players
   see upcoming work, can vote, respond to shape-spec drafts.

None of these are required for the first project. The architecture
is designed to not preclude them.

---

# Part VI — The rust-bevy Profile

## 28. Profile inheritance

The `rust-bevy` profile inherits from tanren's existing `rust-cargo`
profile and adds game-specific standards.

### 28.1 Inherited from rust-cargo (unchanged)

1. Edition 2024, stable channel, `rust-toolchain.toml`.
2. Workspace-level lints: `unsafe_code = forbid`, `unwrap_used = deny`,
   `panic = deny`, `todo = deny`, `dbg_macro = deny`, `print_stdout =
   deny`, `print_stderr = deny`, `unimplemented = deny`,
   `allow_attributes = deny`.
3. Error handling: `thiserror` in libraries, `anyhow` only in binaries.
4. File and function limits: 500 lines per .rs file, 100 lines per
   function.
5. Dependency management: pinned in `[workspace.dependencies]`,
   permissive licenses, `cargo-deny` enforcement.
6. Three-tier test structure: unit (`#[cfg(test)]` colocated), integration
   (`tests/`), doc tests.
7. Tooling: `cargo nextest`, `insta`, `proptest`, `cargo-deny`,
   `cargo-machete`, `taplo`, `lefthook`.
8. CI gate: `just ci` as single entrypoint composing fmt, lint, check,
   check-lines, check-suppression, check-deps, deny, test, machete, doc.
9. Conventional commits with scope.
10. `tracing` for all logging (no `println!`).
11. Secrets wrapped in `secrecy::Secret<T>`.

### 28.2 Overridden from rust-cargo

1. `architecture/crate-layering.md` → `architecture/crate-layering.md`
   (different shape: domain/command/observation/simulation/presentation
   rather than domain/store/policy/orchestrator).
2. `testing/mock-boundaries.md` → `testing/mock-boundaries.md`
   (wiremock is irrelevant; the mock boundary is simulation vs.
   presentation).
3. `testing/test-timing-rules.md` → `testing/test-timing-rules.md`
   (tiered with balance and replay as explicit additional tiers,
   <5s integration budget preserved for unit+integration).

### 28.3 Newly added

1. `architecture/ecs-layering.md` — domain/command/observation/
   simulation/replay/presentation/input topology.
2. `architecture/headless-crate-purity.md` — server and MCP binaries
   must not pull in presentation dependencies.
3. `architecture/command-observation-surface.md` — typed command and
   observation invariants.
4. `architecture/plugin-shape.md` — one plugin per simulation concern,
   strict registration, no cross-plugin global state.
5. `architecture/time-and-simulation-rate.md` — TPS/FPS/CI-speed
   decoupling, tick-based vs time-based logic.
6. `architecture/determinism.md` — seeded RNG, iteration order,
   parallel system ordering, no wall-clock in simulation.
7. `architecture/content-vfs.md` — layered VFS, manifests, priority.
8. `architecture/mod-api-surface.md` — mod loading, hook taxonomy,
   base-game-is-a-mod invariant, semver compat.
9. `architecture/asset-handling.md` — manifest, storage policy,
   remote-VM provisioning, content-integrity CI.
10. `architecture/save-file-versioning.md` — versioned state, typed
    migrations, minimum supported version.
11. `architecture/simulation-client-boundary.md` — server binary,
    client/server separation even in single-player.
12. `rust/fallible-systems.md` — Bevy's `Result<(), BevyError>` systems,
    patterns for avoiding `.unwrap()` in query handling.
13. `rust/plugin-registration.md` — `TryAddPlugin` patterns, no duplicate-
    register panics.
14. `rust/bevy-subcrate-imports.md` — import from `bevy_ecs`, `bevy_app`
    etc. rather than `bevy::*` in simulation crates.
15. `testing/headless-parity.md` — headless and presented must produce
    identical simulation trajectories.
16. `testing/replay-regression.md` — golden trajectory testing.
17. `testing/balance-regression.md` — Monte Carlo sampling, balance
    envelope tests.
18. `testing/deterministic-simulation.md` — TPS sweep, seed replay.
19. `testing/fuzz-invariants.md` — random-policy fuzzing for domain
    invariants.
20. `testing/ci-scaling-invariant.md` — subsystem tagging, test
    selection by intersection.
21. `testing/golden-checkpoints.md` — checkpoint generation,
    invalidation, content hashing.
22. `performance/frame-budget.md` — per-tick budget, per-frame budget,
    enforcement.
23. `performance/compile-time-budget.md` — clean build and incremental
    build targets.
24. `performance/binary-size-budget.md` — per-platform release size.
25. `performance/simulation-throughput-budget.md` — CI math, TPS
    targets.
26. `localization/fluent-discipline.md` — no string literals in UI code,
    all user-facing strings are Fluent keys.
27. `content/dialogue-authoring.md` — Yarn Spinner discipline, typed
    commands in dialogue.
28. `content/feature-flags.md` — flag declarations, flag state as
    persistent resource, flag-gated content.

The precise count will shift as we draft. This is the inventory.

## 29. Standards organization within the profile

```
profiles/rust-bevy/
├── README.md                    # profile overview, inheritance
├── _inherits_from: rust-cargo   # declared; enforced at install time
├── architecture/
│   ├── ecs-layering.md
│   ├── crate-layering.md         # OVERRIDE of rust-cargo
│   ├── headless-crate-purity.md
│   ├── command-observation-surface.md
│   ├── plugin-shape.md
│   ├── time-and-simulation-rate.md
│   ├── determinism.md
│   ├── content-vfs.md
│   ├── mod-api-surface.md
│   ├── asset-handling.md
│   ├── save-file-versioning.md
│   └── simulation-client-boundary.md
├── rust/
│   ├── fallible-systems.md
│   ├── plugin-registration.md
│   └── bevy-subcrate-imports.md
├── testing/
│   ├── headless-parity.md
│   ├── replay-regression.md
│   ├── balance-regression.md
│   ├── deterministic-simulation.md
│   ├── fuzz-invariants.md
│   ├── ci-scaling-invariant.md
│   ├── golden-checkpoints.md
│   ├── mock-boundaries.md        # OVERRIDE
│   └── test-timing-rules.md      # OVERRIDE
├── performance/
│   ├── frame-budget.md
│   ├── compile-time-budget.md
│   ├── binary-size-budget.md
│   └── simulation-throughput-budget.md
├── localization/
│   └── fluent-discipline.md
└── content/
    ├── dialogue-authoring.md
    └── feature-flags.md
```

## 30. Upstream to tanren

Several standards are not Bevy-specific and belong upstream in
`rust-cargo` once generalized:

1. Subsystem tagging for test selection (useful for any project).
2. Tiered CI gates (useful for any project with expensive tests).
3. Golden checkpoint patterns (useful for any project with long-running
   tests or expensive setup).
4. Performance budgets (useful for any performance-sensitive project).

These are candidate contributions to tanren itself as the framework
matures.

---

# Part VII — The First Project

## 31. Scope

A pick-3 ability roguelike with branching narrative. Working name:
**shugyou** (修行, "training/discipline") — to be confirmed or changed.

### 31.1 Mechanics

1. Single procedurally-assembled dungeon, 5-7 rooms.
2. At run start: player selects 3 abilities from 5 random offers drawn
   from a pool of ~15.
3. Mid-run reward room with 2-3 additional offers.
4. Each ability has stats in content (cooldown, damage, range) — not
   hardcoded.
5. ~4-5 enemy types with distinct behaviors.
6. ~8-12 items (abilities + consumables + situational).
7. Shopkeeper NPC in the mid-run reward room with branching dialogue.
   Polite path → discount. Hostile path → unlocks a secret taunt item,
   but makes the final boss swap to shopkeeper-as-boss.
8. Two boss variants: generic and shopkeeper.
9. Single dungeon, single run. No meta-progression across runs (scope
   ceiling for the first project).

### 31.2 Features exercised

Every axis of the framework:

1. **Simulation**: real-time with fixed tick, entity movement, combat,
   interactions, AI behaviors.
2. **Commands**: movement, abilities, interactions, dialogue choices.
3. **Observations**: partial (fog-of-war for unexplored rooms), rich
   (player state + visible map + inventory + dialogue + flags).
4. **Assets**: pixel art sprites, tilemaps, audio tracks, SFX.
5. **Localization**: English + Japanese (two languages to validate the
   system).
6. **Dialogue**: Yarn-based with branching, flag-setting, and
   localization-keyed text.
7. **Feature flags**: shopkeeper hostility, boss variant, secret unlock.
8. **Save/load**: save after each room, resume across sessions.
9. **Content**: every gameplay value in RON/Yarn, none in Rust constants.
10. **Platforms**: desktop (Linux/macOS/Windows) + web (wasm).
11. **Demo scripts**: multiple — "player reaches boss and wins (generic),"
    "player reaches boss and wins (shopkeeper variant)," "player loses
    to first room," "player finds secret."
12. **Balance regression**: random policy should win occasionally (not
    never, not always), aggressive policy should fail to ranged enemies,
    optimal scripted policy should win reliably.
13. **Mod support**: at minimum one extra ability and one extra enemy
    shipped as a separate mod to prove the mod loader works.

### 31.3 Realistic timeline

6–12 months of agent-driven work at roughly one spec per week, 10-20
specs total. Faster with parallelism, slower if we hit framework
mismatches that require revision.

### 31.4 Exit criteria — framework is proven when

1. The game ships on Steam (or itch.io as an interim proof).
2. A non-trivial community mod exists (even if authored by us as a
   proof).
3. An RL policy can be trained against it that plays above random
   baseline.
4. Someone else uses `rust-bevy` on another project (internal or
   external).
5. At least three standards documents have been upstreamed from
   `rust-bevy` back into `rust-cargo`.

## 32. Why not start smaller

Pong exercises simulation, commands, observations, replay — but misses
localization, dialogue, mod support, feature flags, save/load, content
VFS, and asset handling. The first project needs to hit all architectural
axes or the framework isn't actually proven.

A minimal roguelike with one dungeon and dialogue is the smallest scope
that exercises every dimension. Smaller means the framework carries
unproven machinery into whatever project comes second. Larger means
we invest heavily before verifying the framework works.

## 33. Scaffolding phase — pre-tanren

The very first work is *not* a tanren-driven spec. It's manual setup
with Claude Code, producing enough infrastructure that the first
tanren spec can be purely additive.

**Scaffolding deliverables:**

1. Full workspace structure created, all crates scaffolded with empty
   plugins.
2. `just ci` passes on an empty game (compiles, no tests fail, all
   lints clean).
3. `rust-bevy` profile installed (pre-draft or partial is acceptable;
   the profile can evolve in parallel).
4. Base-game-as-mod invariant established (even with empty mod).
5. Content VFS loads an empty manifest.
6. Headless runner executable, stepping a no-op simulation.
7. MCP server executable, exposing empty catalog.
8. Minimal rendering — a black window with "game loaded" text.
9. CI pipeline committed to GitHub Actions.
10. First `DemoScript` running successfully — "app starts, runs 60
    ticks, exits cleanly."
11. `tanren install` run, rendering commands to agent targets.

After scaffolding, the first tanren-driven spec is small and additive:
"player entity can be spawned and moved." 2-3 tasks, narrow subsystems.

**Why manual:** scaffolding establishes 20+ conventions simultaneously,
each touching the others. An agent running 15 tanren specs in sequence
would constantly hit "this convention isn't established yet." Manual
Claude Code pair-programming is the right tool. Estimated 2-4 focused
sessions.

## 34. First real specs after scaffolding

Rough order, as a sketch (not a plan — actual ordering happens at
planning time):

1. Player entity + movement commands.
2. Camera following player.
3. Tilemap rendering of a hardcoded room.
4. Simple enemy that stands still.
5. Player attack + damage.
6. Enemy AI that chases player.
7. Health bar UI.
8. Player death + game-over state.
9. Dungeon generation (hardcoded room layouts initially).
10. Room transitions.
11. First ability (dash) with content definition.
12. Ability selection at run start.
13. First NPC (shopkeeper) with static dialogue.
14. Branching dialogue choices.
15. Dialogue-driven feature flags.
16. Boss variant logic.
17. Localization (second language added).
18. Save / load.
19. Balance regression harness.
20. First mod — extra enemy type shipped separately.

Each is a thin vertical slice. Each produces a `DemoScript`. Each
takes a week or two of agent work.

---

# Part VIII — Open Questions and Deferred Decisions

These are decisions explicitly not made in this document. They're
listed so that future work knows where the edges are.

## 35. Naming

1. **Framework name**: the standards bundle is named `rust-bevy` (tanren
   profile). **Decided (2026-04-20)**: the framework is a tanren profile,
   not a separate framework; the long-term goal is that tanren itself
   covers game development natively. No separate codename.
2. **First project name**: **shugyou** (修行). **Decided (2026-04-20)**.

## 36. Specific library choices

Per Trevor's standing rule, no dep gets added without explicit approval.
Decisions made 2026-04-20; dependencies still require approval at the
spec level before being added.

1. **Physics**: **deferred**. `avian` (modern, Bevy-native) is the likely
   choice when needed. Not needed for the first project.
2. **Audio**: **`bevy_kira_audio` approved in principle** (Kira integration)
   over built-in Bevy audio.
3. **Dialogue**: **`YarnSpinner-Rust` approved in principle** — strongly
   recommended, avoids writing a dialogue tree system from scratch.
4. **UI**: native Bevy UI for in-game UI. `bevy_egui` for debug/tools
   builds only (excluded from release via feature flag).
5. **Localization**: **`bevy_fluent` approved in principle** for Fluent
   integration.
6. **Linting**: **`bevy_lint` approved in principle** — catches
   Bevy-specific antipatterns beyond `clippy`.
7. **Asset pipeline**: Bevy's built-in + custom manifest.
   **`bevy_asset_loader` approved in principle** for organized asset
   registration as needed.
8. **Persistence**: format for save files is `ron` via the
   `SerializableWorld` type. No separate database.
9. **Networking**: **likely `ggrs` for action-oriented projects** requiring
   rollback, `lightyear` or `bevy_replicon` for central-authoritative
   multiplayer. Per-project decision. Deferred for shugyou
   (single-player only).
10. **RL bridge**: **`pyo3` approved in principle** for Python FFI. Python
    tooling side must enforce strict standards (`uv`, `ruff`, typed
    interfaces). Deferred — not needed for first project.

"Approved in principle" means: the choice is endorsed at the framework
level; actual addition to any specific project's `Cargo.toml` still
requires explicit per-spec approval per the standing dependency rule.

## 37. Mod API scripting

**Decided (2026-04-20)**: data-only mods for the first project, WASM
(probably `wasmtime`-based, possibly via `wasvy`) for eventual code mods.

Rationale:

1. **Data mods cover most of shugyou's realistic mod surface.** Adding
   abilities, enemies, items, dialogue branches — all data, no new logic.
   The content VFS + manifest machinery handles these cleanly.
2. **WASM wins on determinism and sandboxing**, both non-negotiable for
   our framework. Lua sandboxing is famously best-effort; WASM with no
   `wasi:filesystem` import is capability-denied by construction.
3. **The barrier-to-entry problem is solvable with tooling**. When code
   mods ship, modders get a `shugyou-mod-template` repo, a Claude Code
   skill for authoring mods with AI assistance, and clear tutorials. The
   modder doesn't need to understand WIT or the component model manually.
4. **Mod-making with AI assistance becomes a distinct affordance**:
   downstream mod creators can use Claude Code (or any agent) to generate
   mod code from natural-language prompts, using purpose-built skills that
   encode the mod API. This is separate from (and lighter than) the full
   tanren process for first-party game development — it's modder-AI,
   not dev-AI.

**Deferred decisions within this space:**

1. Exact WASM runtime: `wasmtime` (mature, heavy) vs. `wasmi` (lighter,
   interpreted) vs. `wasvy`-managed (Bevy-integrated, handles WIT plumbing).
2. Whether to expose scripting to first-party game content (probably not
   — first-party stays Rust).
3. Sandbox capability policy: what imports mods can request, what limits
   are enforced.

## 38. Network topology

1. No topology decision for the first project (single-player only).
   **Decided (2026-04-20)**: shugyou is single-player; multiplayer is not
   a goal for the first project.
2. The invariants that preserve all four topology options (§14.2) are
   commitments to be upheld from day one, enforced by the profile.
3. When multiplayer is a spec, topology choice depends on game feel:
   turn-based → lockstep, real-time competitive → rollback or hybrid
   authoritative.

## 39. Asset sourcing

**Decided (2026-04-20)**:

1. Open-source asset packs in a single consistent style, or pure
   placeholders. No style mixing.
2. Asset additions require audit-trail controls (license recorded, source
   documented, hash pinned) enforced in CI. Every asset in the manifest
   has a `license` and `source` field; CI rejects assets missing these.
3. Primary sourcing target for shugyou: **Kenney assets (CC0)** for
   visual consistency and license simplicity. Music from Incompetech
   (CC-BY) or FreePD (CC0). SFX from Kenney or freesound.org (filtered
   to CC0/CC-BY).
4. Commit to pixel art at 32×32 base resolution with a fixed ~16-32 color
   palette. Revisit at the first visual shape-spec.
5. AI-generated assets deferred indefinitely for first-party content.
   Interesting long-term but the current quality bar for agent-generated
   art/music is lower than for agent-generated code.
6. License audit in CI: every asset's license and source is in the
   manifest; attribution document auto-generated from manifest (for
   Steam/itch.io compliance).

**No asset is added to the repo without going through an audited
`asset add` workflow** — likely a `just asset-add <path> --source=<url>
--license=<id>` recipe that updates the manifest and rejects drift.

## 40. Telemetry and crash reporting

**Decided (2026-04-20)**: deferred as specific specs, **but
architectural accommodation required from day one** — both for
eventual player-side telemetry AND for server-side observability
(dedicated game servers, hosted environments).

**Architectural commitments made now:**

1. All simulation events are structured typed variants. A telemetry
   sink reading the event stream is additive, not a rewrite.
2. The `game-server` binary exposes `metrics` crate integration points
   from scaffolding, even if no actual metrics emit. Empty hooks are
   free; retrofitting them later is expensive.
3. Structured logs via `tracing` with correlation IDs (session, match,
   player) are standard across all binaries.
4. Command/observation streams support opt-in sampling for analysis.
5. Crash reporter hooks at the top of each binary (thin `panic` hook +
   optional replay attachment).

**Privacy discipline up-front:** opt-in, no PII, aggregate-only where
possible, clear data residency policies when hosted. Design for
jurisdictions with strict privacy rules (GDPR, UK DPA, etc.) from the
start rather than retrofitting.

**New profile standard planned:**
`profiles/rust-bevy/observability/server-telemetry-hooks.md` — requires
metrics integration points, structured log conventions, command/
observation sampling mechanism, crash reporter hooks. Implementation
deferred, scaffolding required.

Server-side observability matters because:

1. Hosted multiplayer servers need real production telemetry.
2. Running balance analyses on real player data (when opted in)
   augments scripted-bot balance sweeps.
3. Performance regressions in production are only visible with metrics
   collection.
4. A future early-access launch demands live crash reporting.

## 41. Visual style commitment

**Decided (2026-04-20)**:

1. **Pixel art at 32×32 base sprite resolution** with nearest-neighbor
   scaling and a fixed ~16-32 color palette. Rationale: Kenney's
   roguelike/tactical/RPG packs provide broad coverage in consistent
   style, CC0-licensed, scales cleanly across resolutions.
2. 3D ruled out for shugyou (multiplies asset cost and rendering
   complexity).
3. Commitment revisited at the first visual shape-spec — the scope-spec
   phase will include concrete style references and palette choices.
4. Asset consistency enforced by the audit workflow from §39.

---

# Part IX — Project Strategy

## 42. Open source posture

**Decided (2026-04-20)**: shugyou is **open source from day one**, public
repository, dual-licensed MIT/Apache-2.0 for code and CC-BY-SA 4.0 for
original game content.

Rationale:

1. **Framework validation requires external eyes.** One of the exit
   criteria is that someone else uses `rust-bevy` on another project.
   That requires the first project to be inspectable.
2. **Consistency with tanren.** Tanren is open source; a closed
   flagship example would create a weird asymmetry.
3. **Credibility signal.** An open-source Bevy game with
   tanren-orchestrated agent development publicly visible is a stronger
   signal in the Rust, Bevy, and AI-dev communities than any marketing.
4. **Contribution flow.** Reusable template, mod examples, framework
   improvements can only flow in if the repo is open.
5. **No meaningful competitive disadvantage.** The moat for a
   framework-proof roguelike isn't the code; it's the execution.

**Repository-level commitments:**

1. The shugyou README explicitly states its dual purpose: "a playable
   game AND a validation of the rust-bevy framework." Links to tanren,
   the framework overview, and the live spec dashboard if built.
2. `CONTRIBUTING.md` welcomes contributions with clear scope guidance.
3. `ASSETS.md` documents every asset source, license, and attribution.
4. Public issue tracker, public spec lifecycle visible.
5. Tagged releases produce downloadable artifacts from GitHub Actions.

## 43. Release path

**Decided (2026-04-20)**: phased release — GitHub releases → itch.io
(free) → Steam Early Access → Steam full release.

Rationale:

1. **Steam forces real-world pressure on the framework.** Build artifact
   pipelines, platform compat testing, save-file versioning, player
   feedback ingestion, crash reporting — these only become real problems
   when shipping to a storefront.
2. **Early Access validates the §27 workflow** — player feedback to spec
   to weekly patch. The framework claims to support that cycle; actually
   running it is the validation.
3. **itch.io is the low-friction precursor.** Permissive platform,
   existing roguelike community, fast iteration.

**Phased path:**

| Phase | When | What |
|-------|------|------|
| 1 — GitHub releases | Early dev | Tagged builds from CI, invited testers only |
| 2 — itch.io (free) | First playable 10-15 min loop | Public availability, initial feedback collection |
| 3 — Steam Early Access | Full pick-3 loop + 2 endings + 2 mods | Weekly patch cadence, §27 workflow active |
| 4 — Steam full release | Framework exit criteria validated | Stable release, continued patches |

Total horizon: 12-18 months across all phases. Pricing: free through
Phase 3, optional pricing at Phase 4 ($5-10 or free).

## 44. Template extraction and shared plugins

**Decided (2026-04-20)**: the reusable parts of shugyou are extracted
continuously, following a **module → crate → repo → crates.io** lifecycle.

### 44.1 The extraction lifecycle

A piece of code starts life inside shugyou, migrates out as it stabilizes:

1. **Module in shugyou** — first implementation, game-specific.
2. **Standalone crate within shugyou workspace** — when the module is
   used in more than one place and has clear boundaries. Still in the
   shugyou repo.
3. **Isolated repository** — when the crate has survived at least two
   uses (shugyou plus one other user, even if that other user is a mod
   or a second project prototype). Moved to its own GitHub repo under
   the same namespace.
4. **Published to crates.io** — when the crate API has stabilized,
   semver is meaningful, documentation is complete, and the crate has
   external users.

Rule governing step 3: **contribute patterns that have survived at least
two uses in our own work**. One use is idiosyncratic; two uses is
general.

### 44.2 The template repository

A separate repo (`shugyou-template`, `rust-bevy-template`, or similar
— name TBD) holds the scaffolding of shugyou stripped of game-specific
content. Updated continuously as shugyou evolves.

Contents (minimum):

1. Full Cargo workspace layout per §6.
2. `justfile` with all the tiered CI recipes.
3. `rust-bevy` profile pre-installed as standards.
4. `tanren.yml` configured for game work.
5. Empty plugins wired up — player, combat, dialogue, content loader,
   mod loader — ready to be filled.
6. Minimum viable demo script ("app starts, runs 60 ticks, exits cleanly").
7. Empty base mod.
8. GitHub Actions workflow.
9. `README.md` with "how to start a new game from this template."

When someone (including us, for a future project) wants to start a new
Bevy game following the framework, they clone the template and
customize.

### 44.3 Candidate plugin crates

Early candidates for extraction — not commitments, just the likely
sequence:

1. **`bevy-framework-demos`** — the `DemoScript` type, run-demo executor,
   three-tier assertion mechanics.
2. **`bevy-framework-replay`** — `SerializableWorld`, episode recording,
   replay playback, golden trajectory testing harness.
3. **`bevy-framework-vfs`** — layered content VFS, manifest parsing,
   priority resolution, hash pinning.
4. **`bevy-framework-mods`** — mod loader, manifest validator, semver
   compat, base-game-is-a-mod invariant enforcement.
5. **`bevy-framework-mcp`** — MCP transport wrapper for game
   command/observation interfaces. Shapes close to tanren's `rmcp`
   usage.
6. **`bevy-framework-determinism`** — seeded RNG resource, TPS-sweep
   harness, determinism-equivalence test utilities.
7. **`bevy-framework-subsystem-tags`** — subsystem tagging for specs
   and tests, test selection by intersection. Also a candidate for
   tanren upstream.
8. **`bevy-framework-fluent`** — Fluent integration beyond what
   `bevy_fluent` provides, if we find gaps.

Each becomes a crate when it has two users. Each becomes a repo when
it has crate-scale scope and external interest. Each publishes to
crates.io when stable.

### 44.4 Governance of extracted crates

1. Each extracted repo has its own `CONTRIBUTING.md`, its own
   versioning, its own release cadence.
2. Breaking changes in a plugin crate are a tanren spec in the plugin
   repo, not in shugyou.
3. Shugyou always depends on some specific version of each plugin —
   updates to plugins happen as tanren specs in shugyou.
4. License consistent with the framework: MIT/Apache-2.0.

## 45. Development environment

**Decided (2026-04-20)**: primary development on Windows desktop
(high-resource), verification on WSL and M5 laptop, CI on GitHub
Actions.

**Primary surface: Desktop Windows (native).**

1. Bevy rendering on DirectX 12 is mature and well-tested.
2. NVIDIA driver support is first-class; the RTX 4090 is dramatic
   overkill for 2D pixel art but means rendering is never a bottleneck.
3. Windows is where playtesting, shader iteration, and GPU debugging
   all work cleanly.
4. Configure `git config core.autocrlf input` and enable long-path
   support in group policy before scaffolding to avoid common gotchas.
5. Tanren agents operate directly or provision remote VMs; either works.

**Verification surface: WSL.**

1. Runs `just ci` in a Linux environment matching GitHub Actions.
2. Used periodically to verify Linux compatibility.
3. Not the primary surface — WSL GPU passthrough for a real-time
   renderer is functional but not ideal for iteration.
4. Good fit for agents targeting Linux-based runtimes or for
   long-running headless tests.

**Release verification surface: M5 Mac.**

1. Used occasionally to verify macOS builds work.
2. Excellent Rust compile times (Apple Silicon cores are fast).
3. Not daily driver; reserved for pre-release validation.

**Remote CI surface: GitHub Actions.**

1. Full cross-platform matrix (Linux + Windows + macOS + wasm) on every PR.
2. Local dev doesn't run the full matrix; only the surface relevant
   to current work.

**Remote work VMs:** tanren's existing Hetzner/GCP provisioning for
agent workspaces. Local doesn't bottleneck on agent compute.

## 46. Game design pitch and creative direction

**Decided (2026-04-20)**: shugyou must be **a game someone genuinely
wants to make and play**, not a minimum-viable framework demo.

Rationale:

1. A public open-source game that ships on Steam needs to be
   non-trivial. "Proof of framework" alone is insufficient for the
   audience we're inviting to engage.
2. Trevor's sustained effort is only realistic on a game he
   personally wants to see exist. Framework-proof-by-itself is not
   that game.
3. The framework's credibility depends on the game being *at least*
   competent as a game, ideally distinctive.

### 46.1 What needs to exist before scaffolding

Before agent-driven development starts, the following are Trevor's
responsibility to produce (with writing/critique collaboration
available):

1. **Shugyou pitch document** — 2-4 pages. What the game is, what
   makes it distinctive, what the "twist on the mechanics" is, why
   anyone would play it, what the tone and feel are.
2. **Core mechanic pitch** — beyond "pick-3 roguelike." What's the
   actual spine of the game moment-to-moment? What does the
   shopkeeper twist gesture toward thematically — agency, morality,
   the cost of unkindness?
3. **Character roster draft** — the shopkeeper, the generic boss, the
   protagonist. Names, motivations, visual sketches or references.
4. **Scenario sketches** — what happens in each of the 5-7 rooms?
   What's the environmental storytelling? What's the emotional arc
   of one run?
5. **Tone reference** — mood board, reference games, music references,
   visual references. Not asset choices yet; just the feel.
6. **Why someone would care** — one paragraph. If this is answered
   honestly ("I want to make something in the tradition of X with a
   Y twist because Z") the rest becomes easier.

### 46.2 Framework-agnostic design work

The above is deliberately separate from framework concerns. It's
game design, not engineering. It must be led by Trevor because:

1. Design authenticity only comes from the person whose game it is.
2. Framework concerns are orthogonal to "is this game worth making."
3. An agent can help critique, refine, and draft, but cannot have an
   opinion about what Trevor wants to build.

### 46.3 Integration point

The pitch document informs the scaffolding runbook (some elements
like visual style, asset-pack selection, audio direction depend on
the pitch). But the scaffolding runbook can be drafted in parallel
with the pitch — architectural decisions don't wait on creative
decisions for most of the scaffolding work.

### 46.4 Ongoing creative capture

As game design evolves, it's captured as tanren product documentation
(per the `product/` templates in `tanren/templates/product/`).
Equivalent to `mission.md`, `roadmap.md`, `tech-stack.md` but
oriented at game content: `game-pitch.md`, `characters.md`,
`scenarios.md`, `tone-and-style.md`.

**This is itself a gap in tanren** — the product templates today are
oriented at backend/SaaS projects. Game-design product templates are
a candidate upstream contribution to tanren, the same way the
`rust-bevy` profile is.

---

# Part X — Learning Maximization

The framework is proven not by shipping shugyou alone but by what gets
learned, extracted, and contributed during the process. This section
captures the mechanisms for making that learning actually happen.

## 47. Mechanisms for extracting maximum learning

### 47.1 Every spec declares its framework-property validation

**Decided (2026-04-20)**: shape-spec for every shugyou spec includes a
"framework property validated" field in frontmatter. Shape-spec is
blocked from dispatch if this field is empty.

Example frontmatter additions:

```yaml
spec_id: S-0003
title: Dash ability with cooldown
subsystems:
  - simulation.abilities
  - simulation.combat
  - content.abilities
  - presentation.hud
framework_validates:
  - "Tick-based cooldown discipline per §12.1"
  - "Content-driven ability parameters per §5.5"
  - "Three-tier demo script authoring per §19"
```

This forces every piece of work to be dual-purpose: ships the feature
AND exercises a framework property. Without this, shugyou risks being
"a game that incidentally uses the framework" rather than "a framework
validation that produces a game."

### 47.2 PR body template mandates a reflection section

**Decided (2026-04-20)**: every PR body includes a standard
`## Reflection` section. Content is free-form but required.

Template section:

```markdown
## Reflection

<!-- What did this spec teach us about the framework? -->
<!-- Did any profile standard need revision? -->
<!-- Did we discover a Bevy idiom or pattern worth documenting? -->
<!-- Did anything in the framework get in the way unnecessarily? -->
<!-- Did anything in the framework save noticeable time/effort? -->
<!-- Any candidate for upstream (tanren, plugin, Bevy)? -->

(remove any questions that don't apply; one or two concrete answers
is enough; empty section is not acceptable)
```

Enforcement: PR template ships in the template repo. Audit-spec checks
that the reflection section has substantive content; empty reflections
block merge.

### 47.3 Periodic reflection summaries

At specific inflection points, the accumulated PR reflections get
summarized into `LEARNINGS.md`:

1. After scaffolding completes (pre-first-spec).
2. After first 5 specs merged.
3. After first 10 specs merged.
4. After first mod ships.
5. After itch.io release.
6. After Steam Early Access launch.
7. After each quarterly review thereafter.

The reflection summary is itself a tanren spec (minimal, focused on
documentation). It produces a `LEARNINGS.md` entry listing framework
revisions, Bevy patterns discovered, candidate upstream contributions,
and proposed profile changes.

This is light-weight because the raw material (PR reflection sections)
already exists. Summarizing is aggregation, not original work.

### 47.4 Continuous upstream posture

Three levels of upstream contribution, tracked publicly:

1. **To tanren itself (highest priority).**
   - Subsystem tagging for test selection (general utility).
   - Tiered CI gates (general utility).
   - Golden checkpoint patterns (general utility).
   - Performance budget standards (general utility).
   - Game-design product templates (`game-pitch.md`, `characters.md`,
     etc.).
   - Merge-parent/merge-parallel refinements discovered during game
     work.
   - Specific `rust-bevy` profile rules that generalize.

2. **To Bevy ecosystem.**
   - `bevy_lint` rules specific to determinism or ECS purity.
   - Extensions to `bevy_fluent` if we hit gaps.
   - Contributions to `YarnSpinner-Rust` for game-integration features.
   - `bevy_asset_loader` improvements if we find them.
   - Starter templates combining tanren with Bevy CI.

3. **To Bevy itself (highest effort, highest leverage).**
   - Only things that belong in the engine core.
   - Candidates: deterministic FixedUpdate guarantees, headless-mode
     support, `SerializableWorld` primitives, WASI component model
     integration.
   - Conservative posture — contribute only after internal success.

**Contribution target for the first 12 months post-scaffolding:**

1. ≥3 contributions to tanren.
2. ≥2 contributions to the Bevy ecosystem.
3. ≥1 attempt at a Bevy core contribution (may be rejected; that's
   fine, the attempt is what matters).

Tracked in a public `UPSTREAM.md` in shugyou's repo with PR links,
status, and outcome.

### 47.5 Community posture during development

1. **Public tanren spec dashboard** — a live view of shugyou's active
   specs, state, and upcoming work. Implementing it for shugyou
   validates this tanren feature.
2. **Devlog cadence** — roughly weekly public writeups on framework
   discoveries. Not marketing; technical. Audience: other
   framework-curious Rust/Bevy/AI-dev folks.
3. **Conference submission** — after first playable milestone, submit
   a talk to a Rust conference, a game dev conference, or the Bevy
   community meetup. "Agentic game development with tanren and Bevy"
   is a unique-enough topic to likely land somewhere.
4. **Invite contribution** — explicit welcome in README and
   `CONTRIBUTING.md`. Issues tagged for external contributors.
   Decision-making transparency through the public spec lifecycle.

### 47.6 Design shugyou to stress-test the framework

Rather than building shugyou and hoping framework validation happens
incidentally, build it so that each spec is chosen partly for the
framework property it exercises:

1. First dialogue spec exercises Yarn integration + Fluent integration
   + content VFS + feature flags — deliberately all in one spec.
2. First mod spec exercises the full mod loader: manifest, hash
   verification, dependency resolution, base-game-is-a-mod invariant.
3. First replay regression spec exercises determinism-equivalence test,
   golden trajectory commitment, fuzz-invariant backstop.

This is the most important meta-discipline. It's the difference between
"a game that happens to use the framework" and "a framework validation
exercise that also produces a game."

---

# Part XI — Roadmap and Criteria

## 48. Artifact drafting order

The detailed specs this overview points to need to be drafted. Proposed
order, revisable:

1. **`rust-bevy` profile skeleton** — targeted as tanren **lane 0.6**
   (starts after lane 0.5 merges). Umbrella for all standards; forces
   enumeration of what exists; unblocks everything downstream.
2. **Scaffolding runbook** — unblocks actually starting shugyou. Can be
   drafted in parallel with the profile skeleton since it surfaces
   gaps in the profile.
3. **Command/observation/simulation-client-boundary spec** — architectural
   core; everything else assumes this.
4. **Time and simulation-rate spec** — TPS/FPS/CI-speed decoupling,
   determinism mechanics.
5. **Content VFS + mod API + asset manifest spec** — the content triad;
   shares manifest machinery.
6. **Demo layering spec** — three-tier demos; run-demo integration.
7. **CI scaling invariant spec** — subsystem tagging; tiered gates;
   test selection by intersection.
8. **First-project scope doc** — shugyou feature matrix; exit criteria;
   integrated with the pitch document from §46.
9. **Performance budget spec** — tier budgets; frame budget; benchmark-
   as-gate discipline.
10. **Save-file versioning + replay spec** — serialization; migrations;
    golden checkpoints.
11. **Server runtime spec** — headless crate purity; topology
    optionality; server-side observability.
12. **RL bridge spec** — Gymnasium-compatible surface; vector envs;
    Minari recording. Deferred until shugyou is playable.

## 49. Decision log

Decisions are recorded here with date, summary, and §-reference for
rationale. New decisions go to a separate `DECISION_LOG.md` once the
framework repo exists.

### 2026-04-19 — Initial design session

1. Bevy chosen as engine. Alternatives (Godot, Fyrox, Unity,
   bootstrap-from-libs) considered and rejected. See §3.
2. Rust-bevy as a new tanren profile rather than an extension of
   rust-cargo. See §28.
3. 60 TPS default simulation rate, 144+ FPS design target for
   rendering. Configurable per project. See §12.
4. Server-authoritative soft-from-day-one architecture
   (simulation/client boundary exists even in single-player). Actual
   networking deferred. See §14.
5. Base game is a mod; no special-case content loading. See §11.1.
6. Three-tier demo scripts (invariant / behavioral / narrative).
   See §19.
7. Subsystem tagging as the primary CI scaling mechanism. See §18.
8. RL bridge is Gymnasium-compatible, Python via PyO3, deferred
   training. See §15.
9. First project is a pick-3 roguelike with shopkeeper-as-boss
   branching. Pong too small. See §31.

### 2026-04-20 — Strategic and refinement session

10. Framework is a tanren profile, not a separate framework. Long-term
    goal: tanren covers game dev natively. See §35.
11. First project name: shugyou (修行). See §35.
12. Library choices approved in principle: bevy_kira_audio,
    YarnSpinner-Rust, bevy_fluent, bevy_lint, bevy_asset_loader,
    pyo3. Per-spec approval still required for addition. See §36.
13. Mod scripting: data mods first, WASM later (probably
    wasmtime/wasvy), with AI-assisted mod authoring via Claude Code
    skills for downstream mod creators. See §37.
14. Asset sourcing: Kenney CC0 primary, audited additions only,
    pixel art 32×32, fixed palette. See §39, §41.
15. Server-side telemetry: deferred implementation, but architectural
    hooks (metrics crate integration points, tracing correlation,
    command/observation sampling, crash reporter) required in
    scaffolding. See §40.
16. Shugyou open-sourced from day one, MIT/Apache-2.0 for code,
    CC-BY-SA 4.0 for content. See §42.
17. Release path: GitHub → itch.io free → Steam Early Access → Steam
    full. See §43.
18. Template extraction and shared plugins follow module → crate →
    repo → crates.io lifecycle. Plugins get isolated repos when they
    have two users. See §44.
19. Primary dev environment: Windows desktop native. WSL for Linux
    verification. M5 Mac for macOS verification. See §45.
20. Shugyou requires a full game-design pitch from Trevor before
    scaffolding can fully proceed. Game design is Trevor's lead; tanren
    templates for game-design product documentation is a gap to fill.
    See §46.
21. PR body mandates a `## Reflection` section; periodic
    reflection-summary specs produce `LEARNINGS.md` entries. See §47.
22. Upstream contribution targets: ≥3 to tanren, ≥2 to Bevy ecosystem,
    ≥1 attempt at Bevy core in first 12 months post-scaffolding.
    See §47.4.
23. Every spec declares a `framework_validates` frontmatter field;
    shape-spec blocks dispatch if empty. See §47.1.

## 50. Success criteria

The framework is proven when all five conditions hold:

1. **Shugyou ships.** Steam full release is the shipping target;
   itch.io free release is the fallback minimum.
2. **A non-trivial community mod exists.** Authored externally, not
   just by us as a proof. Expanded framework-proof criterion: we also
   ship at least one first-party mod to demonstrate the mod loader.
3. **An RL policy trains above random baseline.** Gymnasium-compatible
   environment wrapper works; training is tractable.
4. **A second project uses `rust-bevy`.** Could be a prototype, a
   hackathon entry, or a full project. Template repo validates the
   extracted scaffolding.
5. **≥3 standards upstreamed from `rust-bevy` into `rust-cargo`.**
   Generalizable patterns make it back to tanren's core.

If three of five hold at the end of the first project, the framework is
substantially validated. If only the first, the framework is a harness
around a single game. If all five, the framework is a real contribution
to the agentic-dev landscape.

Additional qualitative criteria (not binary, but tracked):

1. Framework revision cadence is healthy — learnings feed back into the
   profile continuously, not only at the end.
2. CI time stays within tier budgets as the game grows.
3. External adoption signals: GitHub stars, forks, issues from
   non-contributors, mentions in Rust/Bevy community discourse.
4. Reflection discipline is sustained — PR reflection sections have
   real content, not perfunctory filler.
5. Game design is genuine — shugyou is something Trevor wants to play,
   not only a framework proof.

---

# Appendix A — Glossary

- **Adapter**: a module translating between an external interface
  (device, network, MCP) and the game's typed command/observation
  surface.
- **Ability**: a player-activated action with cooldown and effect,
  selected at run start.
- **Audit** / **audit-task** / **audit-spec**: tanren commands that
  score work against rubric pillars.
- **Base game**: the shipped game itself, loaded as priority-0 mod in
  the VFS.
- **Bevy**: the Rust game engine used. ECS-based, code-first.
- **Checkpoint**: a serialized simulation state at a known progression
  point, used to avoid replaying long prefixes in tests.
- **Command**: a typed mutation applied to simulation state.
- **Content**: gameplay-affecting data (items, enemies, abilities,
  dialogue) in RON/Yarn files, not in Rust.
- **Content hash**: SHA-256 of the manifest; pins replay and save files
  to specific content versions.
- **Demo script**: a structured artifact produced by `shape-spec` and
  executed by `run-demo` to verify a feature works end-to-end.
- **Determinism-equivalence test**: CI check that same seed + same
  commands produces byte-identical state at different TPS values.
- **DomainEvent** / **GameEvent**: typed events emitted by simulation,
  observable by consumers.
- **ECS**: Entity Component System, Bevy's core architecture.
- **Episode**: a complete playthrough instance; `(seed, commands,
  terminal)`.
- **Fast-gate / standard-gate / spec-gate / nightly / pre-release /
  release**: the six CI tiers (§17).
- **FixedUpdate**: Bevy schedule for tick-rate simulation systems.
- **Flag**: a persistent boolean in game state, settable by dialogue
  or simulation, readable by both.
- **Fluent**: Mozilla's localization library.
- **Fuzz corpus**: random-policy episodes used to catch invariant
  violations regardless of feature-specific demos.
- **Golden trajectory**: a committed episode with pinned final state
  hash.
- **Manifest**: a structured declaration of a mod's contents and
  compat.
- **Mod**: a loadable bundle of content (and eventually code); the
  base game is one.
- **Monotonic task state**: tanren rule that task status progresses
  only forward; Complete is terminal.
- **Observation**: a typed, self-contained snapshot of what an
  external consumer can see.
- **Policy**: an agent that chooses commands given observations.
- **Replay**: a recorded episode that can be re-executed
  deterministically.
- **Run-demo**: tanren command that walks a DemoScript and reports
  pass/fail.
- **Schema version**: integer version on `GameCommand` and
  `Observation` enums; breaking changes bump it.
- **Shape-spec**: tanren command that decomposes an issue into a
  concrete spec with demo.
- **Signpost**: evidence of a non-obvious decision or issue, recorded
  during `do-task`.
- **Simulation tick**: one iteration of `FixedUpdate`.
- **Stacked diff**: a spec that starts work depending on another
  spec's pending merge.
- **Standards**: profile-shipped markdown docs describing rules the
  project must follow.
- **Subsystem tag**: metadata linking a spec or test to a subsystem;
  used for scoped test selection.
- **Tanren**: the orchestration control plane this framework extends.
- **Task**: a unit of work within a spec.
- **TPS / FPS**: ticks-per-second (simulation) and frames-per-second
  (rendering).
- **VFS**: virtual filesystem, the layered content store.
- **Walk-spec**: tanren command where Trevor reviews a completed spec
  and prepares it for merge.
- **Yarn Spinner**: the dialogue scripting language / runtime used.

# Appendix B — References

## Tanren documentation referenced

(Paths relative to tanren repo, lane-0.5 branch or descendants.)

- `README.md` — project overview
- `CLAUDE.md` — conventions for the Rust rewrite
- `AGENTS.md` — conventions for the legacy Python codebase
- `docs/rewrite/HLD.md` — high-level architecture
- `docs/rewrite/MOTIVATIONS.md` — rationale for the rewrite
- `docs/rewrite/ROADMAP.md` — phased delivery plan
- `docs/rewrite/DESIGN_PRINCIPLES.md` — 10+ decision rules
- `docs/rewrite/RUST_STACK.md` — stack choices
- `docs/rewrite/CRATE_GUIDE.md` — workspace topology
- `docs/rewrite/METHODOLOGY_BOUNDARY.md` — operational ownership
- `docs/rewrite/tasks/LANE-0.5-*` — current lane documentation
- `profiles/rust-cargo/**` — rust-cargo profile (parent of rust-bevy)
- `commands/spec/*.md` — shared command templates
- `commands/project/*.md` — project-management commands
- `docs/architecture/*.md` — architecture specs

## Bevy resources

- Bevy 0.18 release notes (January 2026): https://bevy.org/news/
- Bevy asset catalog: https://bevy.org/assets/
- `bevy_lint`: community Bevy-specific lints
- `YarnSpinner-Rust`: dialogue integration
- `bevy_github_ci_template`: CI patterns to crib
- `bevy_ai_editor`: reference for LLM/ECS exposure patterns

## RL and game-dev ecosystem

- Gymnasium: `gymnasium.farama.org`
- PettingZoo (multi-agent)
- Minari (offline RL datasets)
- CleanRL (single-file reference implementations)
- HUD (production agent training, YC W25)
- Stable Baselines 3
- RLlib (distributed training on Ray)
- `ggrs` (rollback netcode in Rust)

## Open-source asset sources

- Kenney assets (CC0)
- OpenGameArt.org
- Freesound.org
- Incompetech (Kevin MacLeod, CC-BY)
- FreePD

---

*End of document.*
*Original version 2026-04-19 — initial comprehensive consolidation of the
design.*
*Revised 2026-04-20 — open questions closed, strategic decisions added
(open source, release path, template extraction, dev environment,
game pitch, reflection discipline, upstream targets). Parts IX and X
added for project strategy and learning maximization; former Part IX
(Roadmap) renumbered to Part XI.*
*Further revisions should be recorded in `DECISION_LOG.md` (once the
framework repo exists) with cross-references to updated sections here.*