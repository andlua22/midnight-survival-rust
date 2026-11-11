# Initial source-prefab GPU captures — rejected shadow appearance

These Linux/Vulkan captures verify the supermarket's 303 composed node poses,
source mip uploads, explicit unsupported glass report and near/far source LOD
selection. They use Bevy 0.20.1 on the RTX 4060 Laptop GPU.

**Their shadow appearance is rejected.** Manual review found black speckling
on walls even though the cast-shadow numerical check passed. The initial
camera used default screen-space `ContactShadows`. This directory preserves
that result as diagnosis evidence and must not be used as the accepted
desktop appearance baseline.

The contact-off comparison in `../prefab-shadow-review/` removes the wall
dots while retaining directional cast shadows. The thinner/finer/close/repeat
cases in `../prefab-contact-presets/` showed that reducing ray thickness did
not give a consistently clean result. The shared production desktop policy
therefore disables contact shadows and retains directional shadow maps.

See [the current implementation validation](../../implementation-mapobjects-2026-11-11.md)
for accepted captures and final checks. These are standalone Bevy preview
stages. No original world positions or source-runtime pixel parity are claimed.
