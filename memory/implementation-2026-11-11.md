# Desktop reconstruction validation — 2026-11-11

This pass implements the first stages in [the source reconstruction ledger](../PLAN/source_reconstruction.md).
Original-runtime parity milestones remain open. New extraction uses Rust;
Android builds were deferred as requested.

## Final checks

| Check | Observed result |
|---|---|
| Offline locked workspace all-target check | Passed |
| Offline locked workspace tests | 133 passed; eight intentionally ignored cases in the ordinary run |
| Explicit retained UI asset test | Passed, including map/player bindings and zero-area text |
| Explicit retained island asset test | Passed; identical rebuilt collision/spawn and source catalog identity |
| Standalone core_checks | 45 passed |
| Independent extractor tests | 35 passed |
| Independent extractor strict all-target Clippy | Passed |
| Gameplay data strict all-target Clippy | Passed |
| Client library strict Clippy with --no-deps | Passed |
| Workspace all-target Clippy | Completed; five inherited warnings remain in map_data/package.rs, shared/physics.rs and examples/terrain_probe.rs |
| Scoped formatting and git diff --check | Passed |
| Linux server --ticks 120 | Passed; position 0, 0.920, 12, grounded |
| Local QUIC certificate/code/version admission | Passed with real localhost UDP sockets |
| Local QUIC movement/collision/capacity/cleanup scenario | Passed with real localhost UDP sockets |
| Full Linux game-window startup | Final build remained alive for the bounded 30-second run, loaded all 399 terrain/road records and used NVIDIA Vulkan; expected timeout exit 124 |
| Environment GPU scenarios | Eight cases passed, with the lower-bias comparison preserved separately |
| Segmentation-enabled native minimap GPU scenarios | 12 cases passed at both window sizes and non-unit DPI |
| Read-only disk/build-reserve gate | Passed; approximately 130 GiB available in the final sampled project filesystem |

The ordinary workspace run does not execute the 150-connection stress test,
long loss/jitter/timeout scenario or sustained CPU timing test. The selected
local QUIC checks validate the changed transport/control path; full capacity
and Internet acceptance remain separate. Standalone core tests re-use
production modules and overlap workspace tests.

Full workspace formatting still exposes earlier formatting in untouched
shared/capsule_query.rs and shared/physics.rs; the extractor has separate
earlier formatting in untouched files. Only scoped formatting is claimed.

## Validation-driven corrections

The first full-window run found a duplicate Node in a native text bundle.
That bundle was fixed and a deferred-overlay test now exercises all modal
states and native text fields. A later run found missing Chinese segmentation
models and zero-size text warnings. The official Bevy
`complex_script_segmentation` feature is now enabled; invisible zero-area or
nonpositive-font source text is skipped. Final window/probe output contains
neither diagnostic.

GPU review found that a changed coordinate Image could leave a UiMaterial
bound to its previous GPU texture view. The associated material now refreshes
when coordinate pixels or dimensions change. A focused test covers changed
and unchanged image cases, and a regional GPU comparison requires a visible
north/east heading change. Texture-mask text now uses bounded, nonoverlapping
native clipping regions, preserving holes and nested mask intersections.

The shadow comparison found additional low-sun speckling with normal bias
0.60. Production now uses Bevy's default 1.8, with a visibly continuous
reference cast shadow. Residual slope artifacts and source lighting metadata
remain documented.

## Evidence and reproduction

- [Environment captures and limits](evidence/environment-bevy/README.md):
  403,390 sky pixels change when clouds are toggled, zero opaque foreground
  pixels change, and 5,522 ground pixels darken under the reference caster.
- [Native UI captures and limits](evidence/minimap-bevy-ui/README.md):
  heading, recentering, zoom, masks, coordinates, time and night indicators.
- [Source import notes](source-imports-2026-11-11.md): nine UI panels,
  supermarket prefab, bounded deterministic extraction and retained warnings.
- [Gameplay data](gameplay-data-2026-11-11.md): 16,861 items, 315 blueprints
  and the remaining source tables with original IDs and raw bytes.
- [Storage audit](storage-2026-11-11.md): exact cache candidates and protected
  source/rollback data. No further cleanup was performed after the handoff.

The final startup test used `timeout --signal=TERM 30s` with core dumps disabled
for that process. It exercised actual Winit/Wayland and host NVIDIA Vulkan.
It was a startup/liveness check; interactive original-game comparison remains
pending. One earlier failing launch produced a system-managed core dump; no
system directories or coredump settings were altered or cleaned.

Current launch and verification commands are in [README.md](../README.md).
Source MapObject placements/runtime spawning, the neighboring Military Base
island, full character/animation/weapon/building rules, original UI transitions
and complete lighting/shader parity remain the next ledger milestones.
