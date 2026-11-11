# Player state/input Rust validation

`validation.json` records the final source/code identities, exact commands,
62-name/ID source comparison and 28 retained native/source hash checks.
Every source declaration matched the Rust enum, including original spellings.
The comparison reads the original declarations and Rust macro inventory
independently of the enum round-trip unit tests.

`game-data-tests.log` retains 36 passing tests, nine of them new player-state
tests. `game-data-clippy.log` retains the successful strict all-target package
Clippy command. `workspace-check.log` retains the successful all-target
workspace compatibility check. No full workspace test rerun is claimed for
this source-only step. Scoped rustfmt and diff whitespace checks also pass.

[Native Stand evidence](../player-state-native/README.md) and
[direction/orientation evidence](../player-movement-native/README.md) contain
the hashes/ranges used for implementation. Native zero-input fallback,
promoted float/double threshold, forward sectors, signed-zero orientation,
raw axes, unknown IDs and finite/overflow admission have explicit test coverage.

The earlier inventory milestone's logs/captures are preserved separately.
These new checks validate source data/math and compilation; active movement,
state transitions, source libm rounding/patch state and runtime parity remain
unverified. [Implementation and next steps](../../implementation-player-state-2026-11-11.md)
record the continuation.
