# Final Linux startup validation — 2026-11-11

The rebuilt game passed a bounded 30-second real Winit/Wayland startup run on
the NVIDIA GeForce RTX 4060 Laptop GPU using Vulkan driver 615.71.09. It
loaded all 400 validated APK terrain/road meshes and remained alive until
the expected timeout exit 124. Core dumps were disabled for that process.
Configuration was kept under `.local/validation-contact-shadow-fixed/`.

[startup-tab-fixed.log](startup-tab-fixed.log) records the final executable,
SHA-256 `bf1c6e5bfdfdb578592d17fe69c34e345a590c1240a9a320639d6921846d88d2`.
It contains no panic, GPU pipeline error or tab-navigation warning. The
inherited window-decoration warning and missing source tangents for
`Terrain_jichang` remain visible.

[startup.log](startup.log) preserves the earlier shadow-corrected run before
the input correction. Its tab-navigation warnings led to the concrete Tab
routing fix. The separate headless regression actively sends real keyboard
messages through desktop input and verifies inventory opening, Tab/Shift+Tab
navigation, loading without controls, developer controls and I/Escape.
Absence of a warning in the final startup log alone is not used to prove input
routing correctness.

The pre-contact-shadow and pre-Tab executables remain in
`.local/runtime-backups/pre-contact-shadow-fix/` and
`.local/runtime-backups/pre-tab-routing-fix/`. No source or user data was
deleted. This validates startup/liveness; matched original-game interaction,
performance and 1:1 visual acceptance remain separate work.
