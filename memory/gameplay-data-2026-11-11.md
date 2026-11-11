# Source gameplay data — 2026-11-11

`midnight_game_data` reads the APK's concatenated Octets configuration records
in Rust. It uses signed little-endian integers, finite IEEE floats, strict
boolean bytes, length-prefixed UTF-8 and bounded lists. Field order follows the
recovered cfg declarations and `Unmarshal` pop sequences in `export/`.
Successful decoding is a serialization observation, not a recording of the
original game's callbacks or IFix overrides.

The verified schema-2 package is
`assets/imported_gameplay/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v2/`.
The earlier five-table schema-1 package is preserved under the same hash with
`-v1`; the runtime selects only v2.

| Table | Records | Retained source bytes |
|---|---:|---:|
| cfg.ItemCfg.oc | 16,861 | 4,309,704 |
| cfg.DrawingCfg.oc | 315 | 42,805 |
| cfg.TreeCfg.oc | 38 | 2,708 |
| cfg.UseItemCfg.oc | 67 | 1,072 |
| cfg.EquipSlotCfg.oc | 2 | 40 |
| cfg.CraftRecipeCfg.oc | 1 | 32 |

The JSON catalog is 16,389,253 bytes, SHA-256
`7c7180460167a2cc202d7280bdd3cb4eca0d00e25712698b3f936f068061b38c`.
All six original tables are retained alongside it. Table/file identities and
counts are in [gameplay-verification.json](evidence/gameplay-verification.json).
Every listed blueprint target/material ID resolves to an item in this package.

`DrawingCfg` supplies normal blueprint definitions: target item, materials,
product count, time, workbench/level, mode and unlock flags. The single
`CraftRecipeCfg` row is ID 51010, referenced by `BatteryMakerBoard`; treating
that table as all normal crafting recipes would omit the 315 blueprints.
The source equipment-slot rows are not a declaration of backpack/hotbar
capacity. Bag-size messages and runtime setup still need tracing.

## Runtime integration and transaction support

The island's `LoadedWorld` retains `Arc<GameplayCatalog>`, and the dedicated
server's `Authority` retains the same loaded catalog. Loading rejects a
terrain/gameplay APK identity mismatch, an invalid catalog/table, altered
checksums or a JSON catalog that differs from decoding retained source bytes.
The connection descriptor includes APK, terrain, gameplay manifest and catalog
identities; the client rejects unequal content. Native Bevy UI displays item
and blueprint definitions in bounded pages.

`SourceInventory` keeps original item IDs and caller-supplied capacity.
Insertions use source pileability/maximum pile counts and reject unresolved
non-positive limits. Insertion and completed-drawing transactions stage changes
so full bags, missing materials or invalid definitions cannot lose items.
The completed-drawing API consumes one definition's exact material quantities
and inserts its explicit `targetItemId`/`productNum`. Its caller must authorize
completion and supply recovered timing, unlock, mode and workbench decisions;
the API is not yet connected to an original-runtime crafting scheduler.

Prototype items, starter inventories, 6/18 slot limits and fixture recipes are
not mapped to source IDs by names. The imported island has no fabricated
starter inventory or fixture resource locations. Source harvesting, consumable
effects, durability and full item use remain separate implementation steps.

## Reproduce and validate

Existing output packages are never replaced by this importer. Use a new path
to repeat import; use `--verify` to check the current package without rewriting it.

```sh
cargo run --offline --locked -p midnight_game_data --bin import_gameplay -- \
  --apk export/lite4.0.apk \
  --out assets/imported_gameplay/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v2

cargo run --offline --locked -p midnight_game_data --bin import_gameplay -- \
  --verify assets/imported_gameplay/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v2 \
  --report memory/evidence/gameplay-verification.json

cargo test --offline --locked -p midnight_game_data
cargo clippy --offline --locked -p midnight_game_data --all-targets -- -D warnings
cargo test --offline --locked -p shared canonical_world_rebuilds -- --ignored
```

Eight data tests passed: truncated/duplicate records, invalid UTF-8/boolean/NaN,
blueprint requirements, stack/capacity bounds, atomic completed-craft rollback,
package integrity, source/JSON consistency and manifest counts/identity.
Scoped all-target Clippy with `-D warnings` passed. The real APK import consumed
all selected bytes and verified staging before publication. No Python or
Android runtime was used.
