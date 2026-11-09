# APK reconstruction

Started: 2026-11-09.

## User objective

Reconstruct the game in `export/lite4.0.apk`, including its map and UI, as faithfully
as the source evidence permits. The existing map is misaligned and the code and
asset pipelines are confusing. Establish one canonical map and a maintainable
Rust implementation. Start with the map and preserve a handoff for later agents.

Constraints: English communication; Rust for new extraction/verification tools;
reuse installed Unity extraction libraries and existing C# exports. No routine
Python execution. Do not overwrite unrelated uncommitted work.

## Verified starting state

- Source APK: `export/lite4.0.apk`, approximately 1.1 GiB. SHA-256 is being recorded.
- Root `Cargo.toml` targets Bevy `0.20.1`, `bevy_egui` `0.43.1`, and
  `bevy_rapier3d` `0.37.0`. Some README version claims are stale.
- Existing Rust extractor: `tools/asset_unpacker`, using `unity-asset-binary`,
  `zip`, `rayon`, `image`, and `clap`. Its independent Cargo.lock exists.
- Runtime map assets are under `assets/imported_map`: terrain/model/placement
  binaries, splatmaps, terrain textures, and several material/position manifests.
- Existing UI extraction is under `assets/ui`: three layouts, sprites, and metadata.
- Several legacy Python extraction/baking scripts remain in `tools/`; they are
  being read for provenance, not executed.
- The worktree already contained modified Cargo/runtime/Python files and untracked
  extracted assets and the Rust extractor before this task. Those are baseline
  user work, not changes created by this task.

## Work checklist

- [in progress] Trace APK scene transforms, mesh coordinates, UVs, and materials.
- [in progress] Audit existing geometry/texture outputs and their provenance.
- [pending] Establish a reproducible Rust import and verification command.
- [pending] Select one canonical map package consumed by runtime and physics.
- [in progress] Audit UI layout and sprite fidelity against source evidence.
- [pending] Validate changed import/loading behavior with meaningful checks.
- [pending] Capture and compare visual results when the available environment permits.

## Evidence discipline

REA is installed via its setup command. The current host does not expose its MCP
tools; do not repeat setup to resolve missing live tools. Use the installed public
CLI if an APK/code question needs it, and ordinary Rust/repository tools for source
analysis. Preserve complete REA Evidence when used; shell/extractor observations
are not REA Evidence IDs.

Full gameplay, server behavior, and exact UI fidelity remain unverified. This
document must not be read as a claim that the complete game has already been ported.
