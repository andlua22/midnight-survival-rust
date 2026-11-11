# Player and inventory boundary — 2026-11-11

This milestone implements the recovered source inventory body/state boundary
in Rust and Bevy 0.20.1, and retains verified player definitions. It does not
activate an original controller or supply authentic captured inventory traffic.
The APK/catalog identities remain the same as the preceding map milestone.

## Implemented behavior

`crates/game_data/src/inventory_wire.rs` decodes ten native base-method message
IDs with their default per-field XOR codes. The reader retains signed values,
separate bag/skin/quick/equipment collections and opaque extra bytes. It rejects
truncation, trailing bytes, invalid lengths/booleans and duplicate quick keys.
Policy bounds are 16 MiB per body, 4096 entries per collection and 1 MiB per
item extra payload; these are not original backpack limits.

`SourceBagState` stages and validates complete changes before committing:

- Snapshot replaces the four collections, preserves supplied capacity, sorts
  bag items by `(ItemCfg.order, itemId, instanceId)` and resets held identity.
- Display capacity is `max(stored capacity, bag count)`. Shrinking capacity
  does not discard items or allocate a capacity-sized UI array.
- Count updates are absolute. Existing items preserve config/instance identity
  and normalized timeout while updating count, binding, durability and extras.
- A quick change moves a known bag item to the requested key. A known quick
  item keeps its existing key. Lookups prefer bag over quick.
- Duration updates search bag, quick, skin, then equipment. Count alerts do
  not change inventory. PutToHand stores the signed instance ID without moving
  collections; its weapon/visual effects remain unported.
- Deadlines use explicitly supplied `ceil(realtimeSinceStartup)` seconds,
  following the distinct recovered snapshot/new-item rules.

Positive stored counts, config-known IDs, per-collection unique instance IDs,
snapshot-first admission and allocation/overflow checks are reconstruction
policies. Incoming state does not enforce static ItemCfg stack limits because
native community overrides can raise them. The earlier source-ID insertion and
completed-craft helper remains a separate locally authorized transaction API.

`crates/shared/src/source_inventory.rs` exposes the same implementation to
Authority and the Bevy client. Catalog loading and player admission/departure
configure its lifetime. The default fixture world is disabled; registered
source players begin without a snapshot. The trusted Authority adapter requires
an admitted player. Online welcome/enter/leave/disconnect rebuild or remove
the appropriate client state. No original body is added to the fixture QUIC
protocol or accepted as an unauthenticated public network command.

Bevy UI reads the local player's source state after `SourceInventorySet::Apply`.
Revision tracking refreshes it after successful changes; malformed bodies do
not invalidate the previous display. The bounded inventory preview uses the
retained source font. Original inventory grid/icons and action callbacks
remain further UI work.

`crates/game_data/src/controller.rs` retains source-local Stand/Crouch/Jump
capsules, raw heel adjustments, generic camera/controller/status constructor
defaults and finite ordinary Stand/Crouch request formulas. Stand height,
center Y and radius are 2.1/1.04/0.42; Crouch is 1.5/0.73/0.42. Jump preserves
0.5/1.2/0.42 and explicitly rejects unresolved height-below-diameter conversion.
Generic camera defaults are not activated. Ordinary default Stand request 5
is distinct from StatusMgr Land/FlySpeed 6.3 and SwimSpeed 3.

## Inspection and validation

`inspect_inventory` loads the hash-verified real catalog and applies up to 32
explicit bodies in nondecreasing receive-time order, beginning with SItems.
It hashes the consumed bytes and emits bounded state/extra-payload previews.
`--report` refuses existing paths and creates a report only after every body
decodes and applies. Direct write/sync is not a crash-atomic publication scheme.

| Check | Result |
|---|---|
| Workspace tests during this milestone | 176 passed; ten ignored |
| New coverage | 7 decoder, 7 state, 5 controller, 3 shared-world/plugin, 1 Authority lifecycle and 1 same-frame UI test |
| Workspace all-target check | Passed |
| Strict Clippy, game_data all targets | Passed with `--no-deps -- -D warnings` |
| Strict Clippy, client lib/tests and shared lib | Passed with `--no-deps -- -D warnings` |
| Workspace all-target Clippy | Passed with five inherited warnings; strict all-target workspace lint is not clean |
| Scoped rustfmt and `git diff --check` | Passed; rustfmt uses `skip_children=true` to preserve inherited formatting |
| Real-catalog synthetic CLI sequence | Snapshot, count update, bag-to-quick move, shrink, durability, alert and hand identity passed |
| CLI rejection cases | Existing report preserved; truncated later body, reversed time, unsupported clear and missing initial snapshot rejected |
| Linux client build and 30-second native startup | Passed; RTX 4060 Laptop/Vulkan, 400 meshes loaded, expected timeout exit 124 |

The five inherited Clippy warnings are three collapsible conditionals in
`map_data/src/package.rs`, an unnecessary test cast in `shared/src/physics.rs`
and a collapsible conditional in `client_core/examples/terrain_probe.rs`.
The startup retains the earlier window-decoration and missing Terrain_jichang
tangent warnings; it contains no panic, GPU pipeline error or Tab warning.

Evidence and hashes are under [source-inventory](evidence/source-inventory/README.md).
The last available runner was preserved at
`.local/runtime-backups/pre-source-inventory/client_runner`; earlier
pre-contact-shadow/pre-Tab copies remain intact. Source exports, packages and
evidence were preserved. Disk audit showed 128 GiB free; no cleanup was needed.
No Python, Android build, source service or protected-cache retry was used.

## Remaining source work

1. Add exact StateID values and native Stand Run/Rush classification as
   source-only data/math. Preserve orientation thresholds, input gates and
   flags without treating classification as physical sprint speed.
2. Recover effective player transforms/capsule scaling, complete movement
   direction/mode logic, crouch/stand clearance, jump/landing and animation
   displacement before wiring source movement into shared simulation/physics.
3. Complete SetQuickUseItem swap/coffer and ClearAllBag weapon/extra-data
   dependencies; identify the SyncBagItem consumer. Their bodies decode,
   but current application rejects them atomically.
4. Recover expiry, use/harvest/repair/crafting callbacks and runtime overrides;
   supply authenticated source snapshot/deltas through a verified admission
   adapter, then bind original UI actions and compare matched source scenarios.

No authentic source snapshot, original frame/session authentication or IFix
runtime state is established by the synthetic tests. Startup proves Bevy
liveness/loading; it does not prove original-runtime movement, inventory
interaction or 1:1 visual parity. World placements remain dependent on actual
SceneInfo payloads as documented in the preceding milestone.
