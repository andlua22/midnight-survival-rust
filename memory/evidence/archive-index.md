# Preserved reconstruction history

Updated: 2026-11-09. These paths are historical evidence/backups, not runtime
fallback inputs. No extracted assets were deleted.

- Superseded terrain candidate:
  `../../.local/failed-imports/terrain-import-1794252188261/`.
  Previously `assets/imported_map/.terrain-import-1794252188261/`.
  Its manifest SHA-256 before and after the directory rename is
  `38fd613194d13949549d04dd102cbe9586619072127d5ca82f63601f684be527`.
- Large UI manifest from before the font repair:
  `../../.local/ui-import-history/ui-import-before-font-fix.json`.
  Previously `memory/evidence/ui-import-before-font-fix.json`.
  SHA-256 before and after the rename:
  `cdacd8f185f8ab6270dd87ae75c14481f9ea03be887ff0db075b5e8bf34fb355`.
- Previous UI audit/projection document:
  `../../.local/ui-import-history/ui-reconstruction-before-source-import.md`.
  It contains historical +Z-north wording; current canonical north is -Z.
- UI package preceding staged publication:
  `../../.local/ui-import-backups/1794256913710624691/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v1/`.
- Terrain predecessors remain under `../../.local/map-import-backups/`.
  Code before overlay removal remains under
  `../../.local/code-backups/canonical-map-2026-11-09/`.

The active roots are `assets/imported_map/terrain/` and the APK-keyed source
package under `assets/imported_ui/`. See `ui-verification.json` and
`ui-reproducibility.json` for the current UI digest and verified scope.
