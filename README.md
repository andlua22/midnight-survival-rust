# Midnight Survival

A Rust/Bevy reconstruction of the retained Unity APK, developed and tested on
Linux first. The current client explores the imported island with native Bevy UI,
Rapier collision, a day/night sun and ray-marched volumetric clouds. Original
gameplay rules and complete visual parity are still being reconstructed.

The current implementation and remaining milestones are recorded in
[the reconstruction plan](PLAN/source_reconstruction.md). Earlier prototype
work remains documented in [phase 1](PLAN/phase1_validation.md) and
[phase 2](PLAN/phase2_validation.md).

## Run on Linux

The lockfile pins Bevy **0.20.1** and bevy_rapier3d **0.37.0**. The workspace
requires Rust 1.97 or newer; this machine has Rust 1.99.1. Use the existing
Vulkan driver and X11/Wayland development libraries. Android packaging is a
later milestone.

```sh
bash tools/storage_audit.sh --check
cargo run --offline --locked -p client_runner -- --config-dir .local/client
```

The default map is `imported-island`. It requires the verified terrain,
expanded UI and gameplay packages described in
[the import notes](memory/source-imports-2026-11-11.md) and
[gameplay data notes](memory/gameplay-data-2026-11-11.md). Invalid island data
fails loading; it does not silently select a different map.

For the existing playable mechanics test fixture:

```sh
cargo run --offline --locked -p client_runner -- --map polygon --config-dir .local/client
```

`--map load-arena` selects the multiplayer load-test fixture. These fixtures
retain prototype survival, crafting, building and combat rules. Their values
are not evidence of the original game's rules.

| Input | Action |
|---|---|
| WASD / mouse | Move / look |
| Space / Shift | Jump / sprint |
| Tab | Open inventory during play; move focus inside menus (Shift+Tab moves backward) |
| I | Toggle inventory and the recovered item/blueprint catalog |
| M / minimap click | Map |
| Wheel over minimap | Minimap zoom |
| Escape | Menu or close a modal screen |
| F2 | HUD editor |
| 1–6 / wheel while playing | Prototype hotbar selection on fixtures |

Menus release the cursor and block movement input. The HUD editor retains
presets, position, scale, opacity, visibility, anchors and save/cancel behavior.
Cloud quality can be changed between 12, 24 and 48 ray steps in the menu.
`--help` lists launch and benchmark options.

## Source packages

All extraction uses Rust and the existing Unity asset crates. No Python or
Unity installation is required. Generated packages are Git-ignored; retain the
source APK, exports, hashes and import reports.

- Terrain: `assets/imported_map/terrain/`; 400 source mesh instances and 383
  enabled non-trigger source colliders, shared by local simulation and server.
  The omitted 512×512 Z-01 scene is now included at its source coordinates;
  [its predecessor is preserved](memory/terrain-z01-2026-11-11.md).
- Expanded UI: nine verified source prefabs, rendered through Bevy UI nodes,
  materials and text. The original three-panel package remains available.
- Gameplay: 16,861 items, 315 blueprints, 38 tree definitions, 67 use-item
  settings, two equipment-slot rows and one special recipe row. Original IDs,
  flags, requirements and binary tables are retained.
- Prefabs: verified supermarket hierarchy, meshes, source mip textures and
  component evidence; Bevy diagnostic rendering with source LOD thresholds;
  shared Box/Mesh collision queries. World placement, original glass/cubemap
  shading and gameplay callbacks still need source verification.

The source catalog can be browsed in bounded pages. A source inventory snapshot
and its supported count, quick-slot, capacity, durability and held-instance
changes now use the same atomic state implementation in the authority and
client. Bevy UI displays that state when supplied; a fresh island session has
no source snapshot or invented starter capacity/items. Original transport,
item-use effects, full slot swaps, expiry and crafting callbacks remain open.
Verified player definitions now include all 62 source state IDs, native input
orientation/Stand Run-Rush rules, exact RootBone standing casts and the ordered
crouch callback. Shared clearance queries retain prefab collision layers and
report unresolved layer/trigger/overlap inputs. Connecting this callback to
gameplay requires the source rig, terrain layers and FSM arbitration; active
movement/camera still use provisional values. See
[clearance/crouch validation](memory/implementation-player-clearance-2026-11-12.md),
[player state validation](memory/implementation-player-state-2026-11-11.md) and
[player/inventory validation](memory/implementation-player-inventory-2026-11-11.md).

The Rust inspector accepts explicit inventory bodies against a verified catalog:

```sh
cargo run --offline --locked -p midnight_game_data --bin inspect_inventory -- --help
```

Supply `--catalog`, then `--packet TYPE_ID REALTIME_SECONDS BODY_PATH` in
receive order, starting with `SItems` (59771964). Time means monotonic seconds
since runtime startup. `--report NEW_PATH` creates a report only after all
bodies decode and apply successfully, and refuses an existing path. Inspection
does not establish captured-body provenance or original session authentication.

## Multiplayer

Build and run an island server using the same validated packages as the client:

```sh
cargo build --offline --locked -p server --bins
umask 077
target/debug/server invite --data-dir server-data > playtest.invite
target/debug/server serve --listen 0.0.0.0:5000 --data-dir server-data --map imported-island
```

Connect from another terminal:

```sh
cargo run --offline --locked -p client_runner -- --connect 127.0.0.1:5000 --join-code-file playtest.invite --config-dir .local/client
```

The welcome message identifies both terrain and gameplay content. Clients
reject mismatched packages. Protocol version 2 is incompatible with earlier
prototype binaries. `--map polygon` or `--map load-arena` must match the
server's explicit fixture selection. External networking and capacity
acceptance procedures are in the historical phase 2 report.

## Validation and disk usage

Run builds one at a time. The workspace and extractor dev profiles disable
incremental output and debug information to keep link sizes bounded. The
[storage audit](memory/storage-2026-11-11.md) records exact rebuildable cache
candidates and protected data. The audit script only reports storage; it never
deletes files or changes system configuration.

The latest clearance/crouch validation passed 199 workspace tests (10 ignored),
the separately selected retained-supermarket collision test, all-target
workspace check and strict Clippy for the changed data/shared-library scope.
[The evidence ledger](memory/evidence/player-clearance-rust/README.md) records
the exact scope and remaining runtime dependencies. About 127 GiB was free at
the latest read-only disk check; no cleanup was needed.

```sh
cargo check --offline --locked --workspace --all-targets
cargo test --offline --locked --workspace
cargo clippy --offline --locked --workspace --all-targets
cargo test --offline --locked -p shared canonical_world_rebuilds -- --ignored
cargo test --offline --locked -p shared retained_supermarket_builds -- --ignored
cargo test --offline --locked -p midnight_map_data verifies_retained_supermarket -- --ignored
cargo test --offline --locked -p client_core native_bevy_ui_asset -- --ignored
cargo run --offline --locked -p server -- --ticks 120
```

The extractor has an independent manifest and lockfile:

```sh
cargo test --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml
cargo clippy --offline --locked --manifest-path tools/asset_unpacker/Cargo.toml --all-targets -- -D warnings
```

Native capture examples are `environment_probe`, `prefab_probe` and `minimap_probe`. Their
shader/capture assertions verify the Bevy implementation; comparisons with a
matched running Unity reference remain a separate acceptance step.

```sh
cargo run --offline --locked -p client_core --example prefab_probe -- .local/captures/prefab
cargo run --offline --locked -p client_core --example environment_probe -- .local/captures/environment
```

The prefab probe explicitly reports unsupported glass parts. Its stage is a
diagnostic preview; exact world coordinates come from separately authenticated
placement payloads. The environment probe includes a paired visible/hidden
Z-01 view. [Placement discovery](memory/world-placements-2026-11-11.md) records
the inspected APK/export results, protected-cache access limits and the strict
Rust reader contract. The production desktop preset disables the contact-shadow
effect that caused wall speckling and retains directional shadow maps; see
[MapObject validation](memory/implementation-mapobjects-2026-11-11.md).
