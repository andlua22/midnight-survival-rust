# Omitted map-1 scenes: independent evidence review — 2026-11-11

The historical map-1 comparison found 243 listed names and 241 imported
scene bundles. The two omitted normalized names were `z-01` and `zhujiemian`.
The retained Rust inspection reports now establish their different source
roles. This review reads existing evidence only; it does not extract assets,
run Android, build the project or alter the published terrain package.

## Z-01 is terrain geometry

- Exact bundle: `assets/scene/z-01.ab`, 947,656 bytes; source bundle SHA-256
  `eb4802a4dfb1aa2996400cca9666da3160fcacc4ffa1d08d1cd5ff46fb07d8c4`.
- Container: `Assets/Scenes/maps_final/zhanju_01/Z-01.unity`.
- GameObject `BuildPlayer-Z-01:13`, name `Z-01`, active, layer 18.
  Root Transform `:26` has no parent, identity rotation and scale, and
  translation `(-3639, -5.599999904632568, 757)` in Unity coordinates.
- MeshFilter `:32` and MeshCollider `:33` reference the same external mesh,
  `BuildPlayer-Z-01.sharedAssets:7`, named `Z_01_ID_639504_`.
  The mesh has 7,844 vertices and 44,670 triangle-list indices
  (14,890 triangles), with local bounds covering 512 × 512 units.
- Renderer `:27` is enabled, references sharedAssets material `:4`,
  casts no shadows and receives shadows. The linked material is
  `Z-01_Material`; its source shader is `_URP/TerrainMeshBase/4 Textures`.
- MeshCollider `:33` is enabled, non-trigger and nonconvex, with
  cookingOptions 30. Four other BoxColliders and particle effect objects
  are also present; the terrain augmentation does not claim to import them.

The independently linked GameObject/Transform/Renderer/MeshFilter/MeshCollider
chain establishes a terrain render and collision record. After the
augmentation, the canonical package contains 242 scenes, 400 render records,
288 terrain records, 112 other records and 383 enabled solid collision records.
Its Bevy bounds for Z-01 are
`[-3639, -7.101078, -1269, -3127, 26.990854, -757]`.
Publication checks and the preserved predecessor are documented in
[the Z-01 handoff](../terrain-z01-2026-11-11.md).

These facts do **not** establish that Z-01 is the neighboring island near
Military Base. Scene names and rectangular bounds do not prove shoreline
topology, runtime visibility or the user's landmark identity.

## zhujiemian_01 is a CommonScene background

The inspection names `assets/scene/zhujiemian_01.ab`, with the container
`Assets/Scenes/CommonScene/zhujiemian_01.unity`. Its scene has six
GameObjects: Global Volume, Directional Light, Main Camera, Point Light,
`zhujiemianBG_01 (1)` and `zhujiemianBG_01`. The inventory has two
MeshRenderers and two MeshFilters, and no MeshCollider, BoxCollider or
Terrain component.

The only authored mesh in `BuildPlayer-zhujiemian_01.sharedAssets` is
`:8`, named `zhujiemianBG_01`: four vertices, six triangle-list indices,
one submesh, and an almost flat local AABB. The materials are named
`zhujiemian_04`, `zhujiemian_04_shadow` and `zhujiemian_sky`.
This supports the CommonScene/menu-background role and does not justify
adding that quad as missing gameplay terrain. It does not establish every
menu script's runtime behavior.

## Retained reports and identity

Reports retain the source APK path `export/lite4.0.apk`. The augmentation
authenticates the supplied APK SHA-256
`2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df`.

| Report | SHA-256 |
| --- | --- |
| [Z-01 scene](placements-z01-inspection.json) | `4085a2e6997e31890d88b9de5cd85b425e5e304fcb62c0e5a47234a0a59d2d80` |
| [Z-01 components](placements-z01-components.json) | `c3cf2f2be9c2a2b69b8188b9fe38ac9ebedb99a147b6a679134453802d0c58a0` |
| [Z-01 geometry links](placements-z01-geometry-links.json) | `c7296f4e2c983e5a207d0642233f9d7d239251ceeb4b71659b776eb43d62bbc7` |
| [CommonScene scene](placements-main-menu-inspection.json) | `54235873e41e8b2cf928cb1cad0db190be4127a4c15983ffd9cf9a10cf09cf30` |
| [CommonScene components](placements-main-menu-components.json) | `958e8119a59696a50ee3fc3183c90b047a1708e0e2b4f02f7df56fde1a438a5f` |
| [Publication report](terrain-z01-augmentation.json) | `48f5fc8a5b961ffd30cca393e2a24f3f1d1ce7f5a47ae4e9f158d7795e091a43` |

The earlier [241-scene coverage report](mapobjects-terrain-coverage.json)
is historical evidence and remains unchanged.
