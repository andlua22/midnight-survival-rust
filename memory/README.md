# Reconstruction handoff

This directory records evidence, decisions, commands, and unfinished work for
agents continuing the reconstruction of `export/lite4.0.apk` in Rust/Bevy.

Read [apk-reconstruction.md](apk-reconstruction.md) first. Focused audits belong
in `map-evidence.md` and `ui-reconstruction.md`; update their status after changes.

The user writes Ukrainian and wants English responses. Implement new tools in
Rust and reuse existing extraction utilities and C# material. Do not run or add
Python unless a concrete missing capability makes it unavoidable.

Preserve evidence and user changes. A visual approximation is not proof that a
map, UI, or gameplay subsystem matches the APK. Record verified facts separately
from inferred behavior and unresolved questions.
