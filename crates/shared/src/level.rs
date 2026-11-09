pub const SPAWN_POSITION: [f32; 3] = [0.0, 2.0, 12.0];

pub const MAP_REVISION: u32 = 1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MapId {
    #[default]
    Polygon,
    LoadArena,
}

pub fn map_geometry(map: MapId) -> Vec<LevelBox> {
    if map == MapId::Polygon {
        return test_level();
    }
    let mut boxes = vec![level_box(
        "Load arena",
        [0.0, -0.5, 0.0],
        [256.0, 0.5, 256.0],
        [0.19, 0.24, 0.23],
    )];
    for (center, size) in [
        ([0.0, 2.0, -256.0], [256.0, 2.0, 0.4]),
        ([0.0, 2.0, 256.0], [256.0, 2.0, 0.4]),
        ([-256.0, 2.0, 0.0], [0.4, 2.0, 256.0]),
        ([256.0, 2.0, 0.0], [0.4, 2.0, 256.0]),
    ] {
        boxes.push(level_box("Arena boundary", center, size, [0.3, 0.36, 0.35]));
    }
    boxes
}

/// A single source for the headless collision world and client meshes.
#[derive(Debug, Clone, Copy)]
pub struct LevelBox {
    pub name: &'static str,
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
    pub rotation_z: f32,
    pub color: [f32; 3],
}

pub fn test_level() -> Vec<LevelBox> {
    let mut boxes = vec![
        level_box(
            "Ground",
            [0.0, -0.5, 0.0],
            [30.0, 0.5, 30.0],
            [0.19, 0.24, 0.23],
        ),
        level_box(
            "Collision wall",
            [0.0, 1.5, -5.0],
            [3.5, 1.5, 0.4],
            [0.70, 0.43, 0.24],
        ),
        level_box(
            "Low ceiling",
            [12.0, 2.2, 0.0],
            [3.0, 0.2, 3.0],
            [0.38, 0.40, 0.47],
        ),
        level_box(
            "Ceiling support",
            [14.7, 1.0, 0.0],
            [0.3, 1.0, 3.0],
            [0.38, 0.40, 0.47],
        ),
    ];
    for step in 0..8 {
        let height = (step + 1) as f32 * 0.2;
        boxes.push(level_box(
            "Stairs",
            [-10.0, height / 2.0, 4.0 - step as f32 * 0.8],
            [2.0, height / 2.0, 0.4],
            [0.31, 0.49, 0.56],
        ));
    }
    for (x, degrees, color) in [
        (7.0, 20.0_f32, [0.42, 0.55, 0.35]),
        (-10.0, 55.0_f32, [0.59, 0.31, 0.28]),
    ] {
        let angle = degrees.to_radians();
        boxes.push(LevelBox {
            name: if degrees < 45.0 {
                "Walkable slope"
            } else {
                "Steep slope"
            },
            center: [x, 4.0 * angle.sin(), -13.0],
            half_extents: [4.0, 0.2, 2.0],
            rotation_z: angle,
            color,
        });
    }
    for (center, size) in [
        ([5.0, 0.5, 4.0], [0.5, 0.5, 0.5]),
        ([-5.0, 1.0, -13.0], [1.0, 1.0, 1.0]),
        ([15.0, 2.0, -20.0], [2.0, 2.0, 2.0]),
    ] {
        boxes.push(level_box("Obstacle", center, size, [0.46, 0.43, 0.33]));
    }
    boxes
}

fn level_box(
    name: &'static str,
    center: [f32; 3],
    half_extents: [f32; 3],
    color: [f32; 3],
) -> LevelBox {
    LevelBox {
        name,
        center,
        half_extents,
        rotation_z: 0.0,
        color,
    }
}

#[derive(Debug, Clone)]
pub struct ImportedMeshRecord {
    pub name: String,
    pub texture: String,
    pub center: [f32; 3],
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct ImportedPlacement {
    pub kind: u8,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}

impl ImportedPlacement {
    #[inline]
    pub fn yaw(&self) -> f32 {
        let [x, y, z, w] = self.rotation;
        (2.0 * (w * y + x * z)).atan2(1.0 - 2.0 * (y * y + z * z))
    }
}

/// Half-extent (metres) of the world area shown by `map_overview.png` (Map_map).
/// Fitted against the imported terrain coastline: the texture covers ~10.7 km,
/// centred on the island origin, including the western military-base island.
pub const MAP_OVERVIEW_HALF_EXTENT: f32 = 5360.0;

pub fn sector_label_at(pos: [f32; 3]) -> String {
    let x = pos[0];
    // Sector grid is defined in Unity space (+Z north); Bevy world uses -Z north.
    let z = -pos[2];
    if x.abs() <= 200.0 && z.abs() <= 200.0 {
        return "A-57".to_string();
    }
    let gx = ((x + 200.0) / 400.0).floor() as i32;
    let gz = ((z + 200.0) / 400.0).floor() as i32;
    let (quad, idx) = if x <= 200.0 && z >= 200.0 {
        let col = (-gx).clamp(0, 7);
        let row = (gz - 1).clamp(0, 6);
        ('A', (56 - (row * 8 + col)).clamp(1, 56))
    } else if x > 200.0 && z >= -200.0 {
        let col = (gx - 1).clamp(0, 6);
        let row = gz.clamp(0, 7);
        ('B', (50 - row * 7 + col).clamp(1, 56))
    } else if x < -200.0 && z < 200.0 {
        let col = (-1 - gx).clamp(0, 6);
        let row = (-gz).clamp(0, 7);
        ('C', (7 + row * 7 + col).clamp(1, 56))
    } else {
        let col = gx.clamp(0, 7);
        let row = (-1 - gz).clamp(0, 6);
        ('D', (1 + row * 8 + col).clamp(1, 56))
    };
    format!("{quad}-{idx:02}")
}

pub fn parse_mesh_archive(bytes: &[u8], expected_magic: &[u8; 8]) -> Vec<ImportedMeshRecord> {
    if bytes.len() < 12 || &bytes[0..8] != expected_magic {
        return Vec::new();
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut pos = 12usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if pos + 2 > bytes.len() {
            break;
        }
        let nlen = u16::from_le_bytes(bytes[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;
        if pos + nlen + 2 > bytes.len() {
            break;
        }
        let name = String::from_utf8_lossy(&bytes[pos..pos + nlen]).into_owned();
        pos += nlen;
        let tlen = u16::from_le_bytes(bytes[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;
        if pos + tlen + 20 > bytes.len() {
            break;
        }
        let texture = String::from_utf8_lossy(&bytes[pos..pos + tlen]).into_owned();
        pos += tlen;
        let cx = f32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
        let cy = f32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
        let cz = f32::from_le_bytes(bytes[pos + 8..pos + 12].try_into().unwrap());
        let vcount = u32::from_le_bytes(bytes[pos + 12..pos + 16].try_into().unwrap()) as usize;
        let icount = u32::from_le_bytes(bytes[pos + 16..pos + 20].try_into().unwrap()) as usize;
        pos += 20;
        let needed = vcount * 12 + vcount * 12 + vcount * 8 + icount * 4;
        if pos + needed > bytes.len() {
            break;
        }
        let mut positions = Vec::with_capacity(vcount);
        for _ in 0..vcount {
            let x = f32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
            let y = f32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[pos + 8..pos + 12].try_into().unwrap());
            pos += 12;
            positions.push([x, y, z]);
        }
        let mut normals = Vec::with_capacity(vcount);
        for _ in 0..vcount {
            let x = f32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
            let y = f32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[pos + 8..pos + 12].try_into().unwrap());
            pos += 12;
            normals.push([x, y, z]);
        }
        let mut uvs = Vec::with_capacity(vcount);
        for _ in 0..vcount {
            let u = f32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
            let v = f32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap());
            pos += 8;
            uvs.push([u, v]);
        }
        let mut indices = Vec::with_capacity(icount);
        for _ in 0..icount {
            let idx = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap());
            pos += 4;
            indices.push(idx);
        }
        out.push(ImportedMeshRecord {
            name,
            texture,
            center: [cx, cy, cz],
            positions,
            normals,
            uvs,
            indices,
        });
    }
    out
}

pub fn parse_placements_archive(bytes: &[u8]) -> Vec<ImportedPlacement> {
    if bytes.len() < 12 {
        return Vec::new();
    }
    let is_v2 = &bytes[0..8] == b"MSPLC002";
    let is_v1 = &bytes[0..8] == b"MSPLC001";
    if !is_v1 && !is_v2 {
        return Vec::new();
    }
    let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut pos = 12usize;
    let mut out = Vec::with_capacity(count);
    if is_v2 {
        for _ in 0..count {
            if pos + 33 > bytes.len() {
                break;
            }
            let kind = bytes[pos];
            let x = f32::from_le_bytes(bytes[pos + 1..pos + 5].try_into().unwrap());
            let y = f32::from_le_bytes(bytes[pos + 5..pos + 9].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[pos + 9..pos + 13].try_into().unwrap());
            let qx = f32::from_le_bytes(bytes[pos + 13..pos + 17].try_into().unwrap());
            let qy = f32::from_le_bytes(bytes[pos + 17..pos + 21].try_into().unwrap());
            let qz = f32::from_le_bytes(bytes[pos + 21..pos + 25].try_into().unwrap());
            let qw = f32::from_le_bytes(bytes[pos + 25..pos + 29].try_into().unwrap());
            let scale = f32::from_le_bytes(bytes[pos + 29..pos + 33].try_into().unwrap());
            pos += 33;
            out.push(ImportedPlacement {
                kind,
                position: [x, y, z],
                rotation: [qx, qy, qz, qw],
                scale,
            });
        }
    } else {
        for _ in 0..count {
            if pos + 21 > bytes.len() {
                break;
            }
            let kind = bytes[pos];
            let x = f32::from_le_bytes(bytes[pos + 1..pos + 5].try_into().unwrap());
            let y = f32::from_le_bytes(bytes[pos + 5..pos + 9].try_into().unwrap());
            let z = f32::from_le_bytes(bytes[pos + 9..pos + 13].try_into().unwrap());
            let yaw = f32::from_le_bytes(bytes[pos + 13..pos + 17].try_into().unwrap());
            let scale = f32::from_le_bytes(bytes[pos + 17..pos + 21].try_into().unwrap());
            pos += 21;
            let half = yaw * 0.5;
            out.push(ImportedPlacement {
                kind,
                position: [x, y, z],
                rotation: [0.0, half.sin(), 0.0, half.cos()],
                scale,
            });
        }
    }
    out
}

/// Stored terrain records (`island_terrain.bin`) are in Unity's left-handed
/// world frame (+X east, +Z north). Bevy is right-handed with -Z north, so Z is
/// mirrored here once for both rendering and physics. Mirroring flips triangle
/// winding, so indices are swapped to keep faces consistent with the normals.
fn unity_terrain_to_world(mut rec: ImportedMeshRecord) -> ImportedMeshRecord {
    rec.center[2] = -rec.center[2];
    for p in &mut rec.positions {
        p[2] = -p[2];
    }
    for n in &mut rec.normals {
        n[2] = -n[2];
    }
    for tri in rec.indices.as_chunks_mut::<3>().0 {
        tri.swap(1, 2);
    }
    rec
}

/// Converts a scene placement from Unity's left-handed frame into Bevy's.
/// Stored rotations are already X-mirrored (matching the X-mirrored model
/// meshes); combined with the world Z mirror this becomes a 180° turn about Y.
fn unity_placement_to_world(mut plc: ImportedPlacement) -> ImportedPlacement {
    plc.position[2] = -plc.position[2];
    let [x, y, z, w] = plc.rotation;
    // Ry(PI) * q, with Ry(PI) = (0, 1, 0, 0).
    plc.rotation = [z, w, -x, -y];
    plc
}

pub fn load_imported_terrain_records() -> Vec<ImportedMeshRecord> {
    for candidate in [
        "assets/imported_map/island_terrain.bin",
        "../../assets/imported_map/island_terrain.bin",
    ] {
        if let Ok(bytes) = std::fs::read(candidate) {
            return parse_mesh_archive(&bytes, b"MSMAP001")
                .into_iter()
                .map(unity_terrain_to_world)
                .collect();
        }
    }
    Vec::new()
}

pub fn load_imported_model_records() -> Vec<ImportedMeshRecord> {
    for candidate in [
        "assets/imported_map/models.bin",
        "../../assets/imported_map/models.bin",
    ] {
        if let Ok(bytes) = std::fs::read(candidate) {
            return parse_mesh_archive(&bytes, b"MSMDL001");
        }
    }
    Vec::new()
}

pub fn load_imported_placements() -> Vec<ImportedPlacement> {
    for candidate in [
        "assets/imported_map/placements.bin",
        "../../assets/imported_map/placements.bin",
    ] {
        if let Ok(bytes) = std::fs::read(candidate) {
            return parse_placements_archive(&bytes)
                .into_iter()
                .map(unity_placement_to_world)
                .collect();
        }
    }
    Vec::new()
}

pub fn load_imported_map_grid() -> Option<Vec<u8>> {
    for candidate in [
        "assets/imported_map/map_grid.bin",
        "../../assets/imported_map/map_grid.bin",
    ] {
        if let Ok(bytes) = std::fs::read(candidate)
            && bytes.len() == 128 * 128 * 3
        {
            return Some(bytes);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TreeConfig {
    pub id: u32,
    pub name: String,
    pub health: u32,
    pub drop_item_id: u32,
    pub drop_count_per_hit: u32,
    pub sound_stages: [i32; 5],
    pub icon: String,
    pub map_icon: String,
    pub marker_type: u32,
}

pub fn load_imported_tree_configs() -> Vec<TreeConfig> {
    for candidate in [
        "assets/imported_map/tree_configs.json",
        "../../assets/imported_map/tree_configs.json",
    ] {
        if let Ok(data) = std::fs::read_to_string(candidate)
            && let Ok(records) = serde_json::from_str::<Vec<TreeConfig>>(&data)
        {
            return records;
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_labels_match_xapk_quadrants() {
        assert_eq!(sector_label_at([0.0, 0.0, 0.0]), "A-57");
        // Bevy -Z is Unity north, so Unity SW corner (A-01) is +Z here.
        assert_eq!(sector_label_at([-2800.0, 0.0, -2800.0]), "A-01");
        assert_eq!(sector_label_at([0.0, 0.0, 400.0]), "D-01");
        assert_eq!(sector_label_at([400.0, 0.0, 0.0]), "B-50");
        assert_eq!(sector_label_at([-400.0, 0.0, 0.0]), "C-07");
    }

    #[test]
    fn imported_map_archives_parse_cleanly_if_present() {
        let terrain = load_imported_terrain_records();
        if !terrain.is_empty() {
            assert!(terrain.len() >= 225);
            assert!(terrain.iter().any(|r| r.name == "A-57"));
            assert!(terrain.iter().any(|r| r.name == "A-01"));
            assert!(terrain.iter().any(|r| r.name == "D-56"));
        }
        let models = load_imported_model_records();
        if !models.is_empty() {
            assert!(models.iter().any(|m| m.name == "tree_temperate"));
            assert!(models.iter().any(|m| m.name == "vehicle_buggy"));
            assert!(models.iter().any(|m| m.name == "grass_clump"));
            assert!(models.iter().any(|m| m.name == "rock_small"));
            assert!(models.iter().any(|m| m.name == "rock_boulder"));
            assert!(models.iter().any(|m| m.name == "building_gas_station"));
            assert!(models.iter().any(|m| m.name == "building_supermarket"));
            assert!(models.iter().any(|m| m.name == "building_pylon"));
            assert!(models.iter().any(|m| m.name == "prop_fence"));
        }
        let placements = load_imported_placements();
        if !placements.is_empty() {
            assert!(placements.iter().any(|p| p.kind == 3)); // grass_clump
            assert!(placements.iter().any(|p| p.kind == 0)); // tree_temperate
            assert!(placements.iter().any(|p| p.kind == 1)); // tree_jungle
            assert!(placements.iter().any(|p| p.kind == 2)); // rock_boulder
            assert!(placements.iter().any(|p| p.kind == 5)); // building_gas_station
            assert!(placements.iter().any(|p| p.kind == 11)); // building_pylon
            assert!(placements.iter().any(|p| p.kind == 13)); // prop_fence
        }
        let tree_cfgs = load_imported_tree_configs();
        if !tree_cfgs.is_empty() {
            assert_eq!(tree_cfgs.len(), 22);
            assert_eq!(tree_cfgs[0].name, "榆树");
            assert_eq!(tree_cfgs[1].name, "白桦树");
        }
    }
}
