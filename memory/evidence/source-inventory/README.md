# Source inventory validation artifacts

The static source/native audit is in
[inventory-rules-source-review.md](../inventory-rules-source-review.md) and
[player-rules-source-review.md](../player-rules-source-review.md).
[Implementation and next steps](../../implementation-player-inventory-2026-11-11.md)
describe the shared state, UI and remaining source-dependent work.

`synthetic-sequence.json` records seven explicitly constructed message bodies
against the verified 16,861-item source catalog. Instances 101/102/103 and
their counts/capacities are test inputs, not original captured traffic or a
starter inventory. The final state retains bag instance 102, quick instances
101/103, visible capacity 1 despite stored capacity 0, durability 8 on 101
and held instance 101. The alert reports -2 without changing its count 4.

`cli-validation.json` records assertions, exact rejection commands and
diagnostics. Existing-report refusal preserves the original SHA-256
`1dd189bf9e1699b2344953edd254b85a22a5dd0b18634b54183724fdaacd06e6`.
Malformed later input, reversed receive time and unsupported clear publish no
report; a non-snapshot first body also fails. Body hashes cover consumed bytes.

Fixture generators and bodies are retained under
`.local/validation-source-inventory/`. `create_fixtures.rs` created the original
snapshot/change/quick/size/duration fixtures. Its alert and hand masks were
incorrect and those original files were preserved unused.
`create_control_fixtures.rs` creates the reviewed alert masks 36/57/123 and
hand mask 127 under `alert-reviewed.bin` and `hand-reviewed.bin`, plus the
empty clear body. Both generators use Rust and refuse existing files.

`workspace-check.log` and `workspace-clippy.log` retain the final all-target
commands. Clippy exits successfully with five inherited warnings; full
workspace `-D warnings` is not claimed. Scoped strict checks pass for
game_data all targets, client lib/tests and shared lib.

`client-build.log` records the final offline/locked client build.
`startup.log` records a bounded 30-second Winit/Wayland run on Linux with
NVIDIA RTX 4060 Laptop Vulkan driver 615.71.09. The process loaded 400 source
terrain/road meshes and remained alive until expected timeout exit 124.
Core dumps were disabled locally and settings stayed in the project directory.
The inherited decoration/tangent warnings remain; no panic, shader/pipeline
error or Tab-navigation warning appeared. No supplied inventory snapshot was
used in this native startup; the deterministic UI test covers the source
snapshot/resize/error display path separately.

Final executable hashes:

- `client_runner`: `e593bee8e1bac9edd06ac89b02a47f4af64a68c4a487910e00085dabf0734f5b`
- `inspect_inventory`: `34e203379860e822c0305a023b9051cef532c7d8460e177d392498f4f02f6f06`

These checks establish reconstruction behavior and startup, not authentic
source transport/session provenance, IFix runtime overrides, complete weapon
effects, active source movement or matched original-game parity.
