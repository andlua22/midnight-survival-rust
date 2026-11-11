pub const SPAWN_POSITION: [f32; 3] = [0.0, 2.0, 12.0];

pub const MAP_REVISION: u32 = 1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MapId {
    #[default]
    Polygon,
    LoadArena,
    ImportedIsland,
}

pub fn map_geometry(map: MapId) -> Vec<LevelBox> {
    if map == MapId::ImportedIsland {
        return Vec::new();
    }
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

pub use midnight_map_data::MeshRecord as ImportedMeshRecord;

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

/// Provisional half-extent in metres for the legacy overview image.
/// APK image identity, origin and coastline calibration still need verification.
pub const MAP_OVERVIEW_HALF_EXTENT: f32 = 5360.0;

pub const SECTOR_BOUNDS: &[(f32, f32, f32, f32, f32, f32, &str)] = &[
    (-3000.0, -2600.0, 2600.0, 3000.0, -2767.4, 2783.8, "A-01"),
    (-2600.0, -2200.0, 2600.0, 3000.0, -2414.3, 2784.5, "A-02"),
    (-2200.0, -1800.0, 2600.0, 3000.0, -1926.7, 2677.6, "A-03"),
    (-1800.0, -1400.0, 2600.0, 3000.0, -1636.1, 2684.3, "A-04"),
    (-1400.0, -1000.0, 2600.0, 3000.0, -1231.2, 2664.7, "A-05"),
    (-1000.0, -600.0, 2600.0, 3000.0, -774.8, 2697.1, "A-06"),
    (-600.0, -200.0, 2600.0, 3000.0, -400.0, 2721.2, "A-07"),
    (-200.0, 200.0, 2600.0, 3000.0, -2.1, 2740.2, "A-08"),
    (-3000.0, -2600.0, 2200.0, 2600.0, -2789.2, 2391.1, "A-09"),
    (-2600.0, -2200.0, 2200.0, 2600.0, -2341.8, 2303.6, "A-10"),
    (-2200.0, -1800.0, 2200.0, 2600.0, -1986.3, 2388.1, "A-11"),
    (-1803.1, -1400.0, 2200.0, 2600.0, -1583.4, 2402.9, "A-12"),
    (-1400.0, -1000.0, 2200.0, 2600.0, -1207.7, 2393.9, "A-13"),
    (-1000.0, -600.0, 2200.0, 2600.0, -805.1, 2391.0, "A-14"),
    (-600.0, -199.2, 2200.0, 2600.0, -385.9, 2387.5, "A-15"),
    (-200.0, 200.0, 2200.0, 2600.0, -6.4, 2402.0, "A-16"),
    (-3000.0, -2600.0, 1800.0, 2200.0, -2699.0, 1952.3, "A-17"),
    (-2600.0, -2200.0, 1800.0, 2200.0, -2388.6, 1989.3, "A-18"),
    (-2200.0, -1800.0, 1800.0, 2200.0, -1997.1, 2001.8, "A-19"),
    (-1800.0, -1400.0, 1800.0, 2200.0, -1607.9, 2000.1, "A-20"),
    (-1400.0, -1000.0, 1796.9, 2200.0, -1198.8, 2022.4, "A-21"),
    (-1000.0, -600.0, 1800.0, 2200.0, -755.2, 2025.6, "A-22"),
    (-600.0, -200.0, 1800.0, 2200.0, -357.2, 1934.6, "A-23"),
    (-200.0, 200.0, 1800.0, 2200.0, 7.1, 1899.0, "A-24"),
    (-3000.0, -2600.0, 1400.0, 1800.0, -2667.8, 1577.7, "A-25"),
    (-2600.0, -2203.1, 1400.0, 1800.0, -2398.2, 1612.2, "A-26"),
    (-2203.1, -1800.0, 1400.0, 1800.0, -2001.4, 1596.0, "A-27"),
    (-1800.0, -1400.0, 1400.0, 1800.0, -1601.2, 1600.0, "A-28"),
    (-1400.0, -1000.0, 1400.0, 1800.0, -1200.4, 1595.4, "A-29"),
    (-1000.0, -600.0, 1400.0, 1800.0, -675.5, 1620.8, "A-30"),
    (-600.0, -200.0, 1400.0, 1800.0, -415.2, 1565.6, "A-31"),
    (-200.0, 200.0, 1400.0, 1800.0, -26.1, 1678.5, "A-32"),
    (-3000.0, -2600.0, 1000.0, 1400.0, -2709.6, 1179.2, "A-33"),
    (-2600.0, -2200.0, 1000.0, 1400.0, -2408.6, 1214.6, "A-34"),
    (-2200.0, -1800.0, 1000.0, 1400.0, -1984.6, 1187.4, "A-35"),
    (-1800.0, -1400.0, 1000.0, 1400.0, -1599.8, 1203.8, "A-36"),
    (-1400.0, -1000.0, 1000.0, 1400.0, -1201.6, 1201.0, "A-37"),
    (-1000.0, -600.0, 1000.0, 1400.0, -732.1, 1259.4, "A-38"),
    (-600.0, -200.0, 1000.0, 1400.0, -446.2, 1260.9, "A-39"),
    (-200.0, 200.0, 1000.0, 1400.0, 45.3, 1188.7, "A-40"),
    (-3000.0, -2600.0, 600.0, 1000.0, -2723.1, 800.5, "A-41"),
    (-2600.0, -2200.0, 600.0, 1000.0, -2403.0, 799.1, "A-42"),
    (-2200.0, -1800.0, 600.0, 1000.0, -2006.2, 817.8, "A-43"),
    (-1800.0, -1400.0, 600.0, 1000.0, -1595.0, 798.2, "A-44"),
    (-1400.0, -1000.0, 600.0, 1000.0, -1198.2, 802.8, "A-45"),
    (-1000.0, -600.0, 600.0, 1000.0, -678.5, 854.3, "A-46"),
    (-600.0, -200.0, 600.0, 1000.0, -415.5, 844.3, "A-47"),
    (-200.0, 200.0, 600.0, 1000.0, -17.2, 871.4, "A-48"),
    (-3000.0, -2600.0, 200.0, 600.0, -2717.7, 438.1, "A-49"),
    (-2600.0, -2200.0, 199.2, 600.0, -2409.3, 390.8, "A-50"),
    (-2200.0, -1800.0, 200.0, 600.0, -1983.8, 398.5, "A-51"),
    (-1800.0, -1400.0, 200.0, 600.0, -1600.0, 403.9, "A-52"),
    (-1400.0, -1000.0, 199.2, 600.0, -1206.1, 407.1, "A-53"),
    (-1000.0, -600.0, 200.0, 600.0, -800.8, 403.7, "A-54"),
    (-600.0, -200.0, 200.0, 600.0, -410.8, 405.0, "A-55"),
    (-200.0, 200.0, 199.2, 600.0, 3.8, 407.5, "A-56"),
    (-200.0, 200.0, -200.0, 200.0, -8.2, 6.8, "A-57"),
    (200.0, 600.0, 2600.0, 3000.0, 298.6, 2701.8, "B-01"),
    (600.0, 1000.0, 2600.0, 3000.0, 870.3, 2655.9, "B-02"),
    (1000.0, 1400.0, 2600.0, 3000.0, 1147.8, 2673.1, "B-03"),
    (1400.0, 1800.0, 2600.0, 3000.0, 1692.9, 2690.0, "B-04"),
    (1800.0, 2200.0, 2600.0, 3000.0, 1937.8, 2688.9, "B-05"),
    (2200.0, 2600.0, 2600.0, 3000.0, 2352.4, 2817.9, "B-06"),
    (2600.0, 3000.0, 2600.0, 3000.0, 2802.6, 2799.1, "B-07"),
    (199.2, 600.0, 2200.0, 2600.0, 400.8, 2374.3, "B-08"),
    (600.0, 1000.0, 2200.0, 2600.0, 829.7, 2368.4, "B-09"),
    (1000.0, 1400.0, 2200.0, 2600.0, 1195.7, 2373.9, "B-10"),
    (1400.0, 1800.0, 2200.0, 2600.0, 1643.6, 2377.0, "B-11"),
    (1800.0, 2200.0, 2200.0, 2600.0, 1973.7, 2387.7, "B-12"),
    (2200.0, 2601.6, 2200.0, 2600.0, 2338.9, 2294.8, "B-13"),
    (2600.0, 3000.0, 2200.0, 2600.0, 2810.2, 2374.6, "B-14"),
    (200.0, 600.0, 1800.0, 2200.0, 387.8, 1915.6, "B-15"),
    (600.0, 1000.0, 1800.0, 2200.0, 702.3, 1893.9, "B-16"),
    (1000.0, 1400.0, 1799.2, 2200.0, 1209.4, 1995.7, "B-17"),
    (1400.0, 1800.0, 1800.0, 2200.0, 1602.7, 1995.5, "B-18"),
    (1800.0, 2200.0, 1800.0, 2200.0, 1993.1, 1993.1, "B-19"),
    (2200.0, 2600.0, 1800.0, 2200.0, 2395.1, 1988.8, "B-20"),
    (2600.0, 3000.0, 1800.0, 2200.0, 2717.2, 1953.9, "B-21"),
    (200.0, 600.0, 1400.0, 1800.0, 375.7, 1640.5, "B-22"),
    (600.0, 1000.0, 1400.0, 1800.0, 734.9, 1579.8, "B-23"),
    (1000.0, 1400.0, 1400.0, 1800.0, 1216.7, 1589.8, "B-24"),
    (1400.0, 1800.0, 1399.2, 1800.0, 1598.2, 1595.9, "B-25"),
    (1800.0, 2200.0, 1400.0, 1800.0, 1994.5, 1616.2, "B-26"),
    (2200.0, 2600.0, 1400.0, 1800.0, 2381.8, 1604.2, "B-27"),
    (2600.0, 3000.0, 1400.0, 1800.0, 2719.0, 1646.1, "B-28"),
    (200.0, 600.0, 1000.0, 1400.0, 474.8, 1218.2, "B-29"),
    (600.0, 1000.0, 1000.0, 1400.0, 730.7, 1194.1, "B-30"),
    (1000.0, 1400.0, 1000.0, 1400.0, 1203.6, 1198.9, "B-31"),
    (1400.0, 1800.0, 1000.0, 1400.0, 1583.2, 1206.1, "B-32"),
    (1800.0, 2200.0, 1000.0, 1400.0, 1993.5, 1209.7, "B-33"),
    (2200.0, 2600.0, 1000.0, 1400.0, 2399.6, 1196.2, "B-34"),
    (2600.0, 3000.0, 1000.0, 1400.0, 2707.5, 1183.8, "B-35"),
    (200.0, 600.0, 600.0, 1000.0, 438.3, 853.8, "B-36"),
    (600.0, 1000.0, 600.0, 1000.0, 773.9, 823.1, "B-37"),
    (1000.0, 1400.0, 600.0, 1000.0, 1197.4, 828.1, "B-38"),
    (1400.0, 1800.0, 600.0, 1000.0, 1576.3, 800.2, "B-39"),
    (1800.0, 2200.0, 600.0, 1000.0, 2014.4, 801.8, "B-40"),
    (2200.0, 2600.0, 600.0, 1000.0, 2399.6, 811.3, "B-41"),
    (2600.0, 3000.0, 597.7, 1000.0, 2708.9, 829.6, "B-42"),
    (200.0, 600.0, 200.0, 600.0, 407.4, 406.4, "B-43"),
    (600.0, 1000.0, 200.0, 600.0, 795.5, 386.1, "B-44"),
    (1000.0, 1400.0, 200.0, 600.0, 1225.0, 382.5, "B-45"),
    (1400.0, 1800.0, 200.0, 600.0, 1599.1, 407.0, "B-46"),
    (1800.0, 2200.0, 200.0, 600.0, 1989.0, 333.2, "B-47"),
    (2200.0, 2600.0, 200.0, 600.0, 2404.4, 392.3, "B-48"),
    (2600.0, 3000.0, 200.0, 600.0, 2727.5, 388.2, "B-49"),
    (200.0, 600.0, -200.0, 200.0, 390.5, 15.7, "B-50"),
    (600.0, 996.1, -200.0, 200.0, 789.1, 5.0, "B-51"),
    (996.1, 1400.0, -200.0, 200.0, 1197.2, 1.0, "B-52"),
    (1400.0, 1800.0, -200.0, 200.0, 1646.6, 13.7, "B-53"),
    (1800.0, 2200.0, -200.0, 200.0, 1983.7, 31.8, "B-54"),
    (2200.0, 2600.0, -200.0, 200.0, 2397.3, -1.3, "B-55"),
    (2600.0, 3000.0, -200.0, 200.0, 2725.1, 5.5, "B-56"),
    (-3000.0, -2600.0, -200.0, 200.0, -2731.1, -24.9, "C-01"),
    (-2600.0, -2200.0, -200.0, 200.0, -2392.7, -0.2, "C-02"),
    (-2200.0, -1800.0, -200.0, 200.0, -1999.1, -0.8, "C-03"),
    (-1800.0, -1400.0, -200.0, 200.0, -1605.5, 1.2, "C-04"),
    (-1400.0, -1000.0, -200.0, 200.0, -1182.7, 29.7, "C-05"),
    (-1000.0, -600.0, -200.0, 200.0, -803.3, 4.1, "C-06"),
    (-600.0, -200.0, -200.0, 200.0, -403.1, 1.9, "C-07"),
    (-3000.0, -2600.0, -600.0, -200.0, -2708.9, -403.7, "C-08"),
    (-2600.0, -2200.0, -600.0, -200.0, -2414.6, -355.7, "C-09"),
    (-2200.0, -1800.0, -600.0, -200.0, -1994.3, -394.6, "C-10"),
    (-1800.0, -1400.0, -600.0, -200.0, -1614.8, -384.3, "C-11"),
    (-1400.0, -1000.0, -601.6, -200.0, -1219.6, -434.0, "C-12"),
    (-1000.0, -600.0, -600.0, -200.0, -793.3, -397.1, "C-13"),
    (-600.0, -200.0, -600.0, -200.0, -403.1, -407.3, "C-14"),
    (-3000.0, -2600.0, -1000.0, -600.0, -2709.2, -759.4, "C-15"),
    (-2600.0, -2200.0, -1000.0, -600.0, -2466.7, -811.2, "C-16"),
    (-2200.0, -1800.0, -1000.8, -600.0, -1943.8, -796.2, "C-17"),
    (-1800.0, -1400.0, -1000.0, -600.0, -1610.0, -790.1, "C-18"),
    (-1400.0, -1000.0, -1000.0, -600.0, -1199.6, -794.1, "C-19"),
    (-1000.0, -600.0, -1000.0, -600.0, -810.1, -780.6, "C-20"),
    (-600.0, -200.0, -1000.0, -600.0, -396.8, -797.6, "C-21"),
    (-3000.0, -2600.0, -1400.0, -1000.0, -2690.5, -1217.1, "C-22"),
    (-2600.0, -2199.2, -1400.0, -1000.0, -2392.5, -1209.7, "C-23"),
    (-2200.0, -1800.0, -1400.0, -1000.0, -2004.9, -1204.7, "C-24"),
    (-1800.0, -1400.0, -1400.8, -1000.0, -1606.8, -1213.1, "C-25"),
    (-1400.0, -1000.0, -1400.0, -1000.0, -1205.1, -1206.4, "C-26"),
    (-1000.0, -600.0, -1400.0, -1000.0, -821.1, -1231.3, "C-27"),
    (-600.0, -200.0, -1400.0, -1000.0, -384.5, -1215.0, "C-28"),
    (-3000.0, -2600.0, -1800.0, -1400.0, -2678.5, -1624.1, "C-29"),
    (-2600.0, -2200.0, -1800.0, -1400.0, -2404.4, -1591.7, "C-30"),
    (-2200.0, -1800.0, -1800.0, -1400.0, -1995.6, -1599.2, "C-31"),
    (-1800.0, -1400.0, -1800.0, -1400.0, -1610.8, -1582.4, "C-32"),
    (-1400.0, -1000.0, -1800.0, -1400.0, -1205.7, -1572.4, "C-33"),
    (-1000.0, -600.0, -1800.0, -1400.0, -791.9, -1570.5, "C-34"),
    (-600.0, -200.0, -1800.0, -1400.0, -411.7, -1601.6, "C-35"),
    (-3000.0, -2600.0, -2200.0, -1800.0, -2687.5, -1963.5, "C-36"),
    (-2600.0, -2200.0, -2200.0, -1800.0, -2399.8, -2003.5, "C-37"),
    (-2200.0, -1800.0, -2200.0, -1800.0, -2007.4, -2003.2, "C-38"),
    (-1800.0, -1400.0, -2200.0, -1800.0, -1593.5, -2020.9, "C-39"),
    (-1400.0, -1000.0, -2200.0, -1800.0, -1181.5, -2020.1, "C-40"),
    (-1000.0, -600.0, -2200.0, -1800.0, -802.4, -2029.0, "C-41"),
    (-600.0, -200.0, -2200.0, -1800.0, -410.8, -2000.9, "C-42"),
    (-3000.0, -2600.0, -2600.0, -2200.0, -2761.1, -2303.8, "C-43"),
    (-2600.0, -2200.0, -2600.0, -2200.0, -2334.1, -2345.0, "C-44"),
    (-2200.0, -1800.0, -2600.0, -2200.0, -1999.4, -2393.1, "C-45"),
    (-1800.0, -1400.0, -2600.0, -2200.0, -1618.0, -2383.7, "C-46"),
    (-1400.0, -1000.0, -2600.0, -2200.0, -1192.5, -2391.0, "C-47"),
    (-1000.0, -600.0, -2600.0, -2200.0, -806.2, -2400.0, "C-48"),
    (-600.8, -200.0, -2600.0, -2200.0, -401.9, -2386.0, "C-49"),
    (-3000.0, -2600.0, -3000.0, -2600.0, -2765.2, -2843.2, "C-50"),
    (-2600.0, -2200.0, -3000.0, -2600.0, -2326.9, -2698.6, "C-51"),
    (-2200.0, -1800.0, -3000.0, -2600.0, -2012.6, -2654.0, "C-52"),
    (-1800.0, -1400.0, -3000.0, -2600.0, -1642.9, -2704.7, "C-53"),
    (-1400.0, -1000.0, -3000.0, -2600.0, -1167.0, -2693.2, "C-54"),
    (-1000.0, -600.0, -3000.0, -2600.0, -798.9, -2673.0, "C-55"),
    (-600.0, -200.0, -3000.0, -2600.0, -409.6, -2687.3, "C-56"),
    (-200.0, 200.0, -600.0, -200.0, -16.6, -407.5, "D-01"),
    (200.0, 600.0, -600.0, -200.0, 414.9, -359.3, "D-02"),
    (597.7, 1000.0, -600.0, -200.0, 807.9, -401.9, "D-03"),
    (1000.0, 1400.0, -600.0, -200.0, 1197.5, -401.7, "D-04"),
    (1400.0, 1800.0, -600.0, -200.0, 1611.2, -354.0, "D-05"),
    (1800.0, 2200.0, -600.0, -200.0, 2001.7, -398.1, "D-06"),
    (2200.0, 2600.0, -600.0, -200.0, 2396.7, -400.9, "D-07"),
    (2600.0, 3000.0, -600.0, -200.0, 2716.9, -399.9, "D-08"),
    (-200.0, 200.0, -1000.0, -600.0, -11.7, -815.3, "D-09"),
    (200.0, 600.0, -1000.0, -600.0, 404.5, -825.0, "D-10"),
    (600.0, 1000.0, -1000.0, -600.0, 803.4, -802.1, "D-11"),
    (1000.0, 1400.0, -1000.0, -600.0, 1198.4, -799.8, "D-12"),
    (1400.0, 1800.0, -1000.0, -600.0, 1602.4, -807.1, "D-13"),
    (1800.0, 2200.0, -1000.0, -600.0, 1996.2, -816.4, "D-14"),
    (2200.0, 2600.0, -1000.0, -600.0, 2383.5, -829.6, "D-15"),
    (2600.0, 3000.0, -1000.0, -600.0, 2766.7, -821.3, "D-16"),
    (-200.0, 200.0, -1400.0, -1000.0, -2.0, -1202.5, "D-17"),
    (200.0, 600.0, -1400.0, -1000.0, 370.2, -1194.4, "D-18"),
    (600.0, 1000.0, -1400.0, -1000.0, 859.1, -1225.0, "D-19"),
    (1000.0, 1400.0, -1400.0, -1000.0, 1169.0, -1242.2, "D-20"),
    (1400.0, 1570.3, -1400.0, -1000.0, 1443.0, -1218.0, "D-21"),
    (1800.0, 2200.0, -1400.0, -1000.0, 1994.9, -1189.8, "D-22"),
    (2200.0, 2600.0, -1400.0, -1000.0, 2405.7, -1195.5, "D-23"),
    (2600.0, 3000.0, -1400.0, -1000.0, 2710.6, -1249.3, "D-24"),
    (-200.0, 200.0, -1800.0, -1400.0, 0.8, -1585.5, "D-25"),
    (200.0, 600.0, -1800.0, -1400.0, 411.7, -1587.9, "D-26"),
    (600.0, 1000.0, -1800.0, -1400.0, 841.7, -1559.6, "D-27"),
    (1000.0, 1400.0, -1800.0, -1400.0, 1151.3, -1554.5, "D-28"),
    (1400.0, 1800.0, -1800.0, -1400.0, 1607.9, -1645.9, "D-29"),
    (1800.0, 2200.0, -1800.0, -1400.0, 1924.7, -1609.1, "D-30"),
    (2200.0, 2600.0, -1800.0, -1400.0, 2391.8, -1603.6, "D-31"),
    (2600.0, 3000.0, -1800.0, -1400.0, 2707.0, -1542.1, "D-32"),
    (-200.0, 200.0, -2200.0, -1800.0, -17.3, -2004.6, "D-33"),
    (200.0, 600.0, -2200.0, -1800.0, 438.1, -1984.2, "D-34"),
    (600.0, 996.1, -2200.0, -1800.0, 784.2, -1985.2, "D-35"),
    (996.1, 1400.0, -2200.0, -1800.0, 1203.6, -1996.1, "D-36"),
    (1400.0, 1800.0, -2200.0, -1800.0, 1581.9, -1998.6, "D-37"),
    (1800.0, 2200.0, -2200.0, -1800.0, 1949.6, -1948.7, "D-38"),
    (2200.0, 2600.0, -2200.0, -1800.0, 2391.0, -1994.6, "D-39"),
    (2600.0, 3000.0, -2200.0, -1800.0, 2686.3, -1984.2, "D-40"),
    (-200.0, 200.0, -2600.0, -2200.0, 3.1, -2395.9, "D-41"),
    (200.0, 600.0, -2600.0, -2200.0, 370.3, -2401.6, "D-42"),
    (600.0, 1000.0, -2600.0, -2200.0, 798.7, -2400.1, "D-43"),
    (1000.0, 1400.0, -2600.0, -2200.0, 1187.6, -2357.6, "D-44"),
    (1400.0, 1800.0, -2600.0, -2200.0, 1614.6, -2390.4, "D-45"),
    (1800.0, 2200.0, -2600.0, -2200.0, 1984.1, -2388.0, "D-46"),
    (2200.0, 2600.0, -2600.0, -2200.0, 2354.4, -2300.4, "D-47"),
    (2600.0, 3000.0, -2600.0, -2200.0, 2812.7, -2390.1, "D-48"),
    (-200.0, 200.0, -3000.0, -2600.0, 22.3, -2683.2, "D-49"),
    (200.0, 600.0, -3000.0, -2600.0, 396.5, -2664.3, "D-50"),
    (600.0, 1000.0, -3000.0, -2600.0, 835.9, -2706.8, "D-51"),
    (1000.0, 1400.0, -3000.0, -2600.0, 1088.5, -2728.6, "D-52"),
    (1400.0, 1800.0, -3000.0, -2600.0, 1649.9, -2682.0, "D-53"),
    (1800.0, 2200.0, -3000.0, -2600.0, 1911.2, -2688.7, "D-54"),
    (2200.0, 2600.0, -3000.0, -2600.0, 2407.1, -2761.6, "D-55"),
    (2600.0, 3000.0, -3000.0, -2600.0, 2846.4, -2783.3, "D-56"),
    (-3400.3, -3000.0, -1000.0, -599.2, -3317.4, -690.2, "MIL-01"),
    (-3800.5, -3400.0, -1000.0, -599.7, -3610.5, -699.3, "MIL-02"),
    (-4200.2, -3800.0, -1000.0, -599.8, -3995.5, -667.1, "MIL-03"),
    (-4600.0, -4200.0, -1000.0, -599.7, -4245.1, -647.8, "MIL-04"),
    (-3400.6, -3000.0, -600.0, -200.0, -3304.8, -395.8, "MIL-05"),
    (-3800.4, -3400.0, -600.0, -200.0, -3550.5, -513.6, "MIL-06"),
    (-4200.3, -3800.0, -600.0, -200.0, -4083.7, -480.1, "MIL-07"),
    (-4600.0, -4200.0, -600.0, -199.7, -4275.2, -388.2, "MIL-08"),
    (-3400.7, -3000.0, -200.0, 200.7, -3328.0, -47.2, "MIL-09"),
    (-3800.0, -3400.0, -200.4, 200.4, -3506.8, 64.3, "MIL-10"),
    (-4200.4, -3800.0, -200.0, 200.3, -4109.8, 60.8, "MIL-11"),
    (-4600.0, -4200.0, -200.0, 200.0, -4292.1, 21.3, "MIL-12"),
    (-3400.0, -3000.0, 200.0, 600.0, -3351.4, 266.8, "MIL-13"),
    (-3801.3, -3400.0, 200.0, 600.0, -3587.8, 284.4, "MIL-14"),
    (-4200.3, -3800.0, 200.0, 600.0, -3986.7, 297.2, "MIL-15"),
    (-4600.0, -4200.0, 200.0, 600.0, -4278.2, 270.0, "MIL-16"),
];

pub fn sector_label_at(pos: [f32; 3]) -> String {
    let px = pos[0];
    // The table records Unity scene coordinates. Canonical Bevy north is -Z.
    let pz = -pos[2];
    for &(min_x, max_x, min_z, max_z, _, _, name) in SECTOR_BOUNDS {
        if px >= min_x && px <= max_x && pz >= min_z && pz <= max_z {
            return name.to_string();
        }
    }
    let mut closest_name = "A-57";
    let mut min_dist_sq = f32::MAX;
    for &(_, _, _, _, cx, cz, name) in SECTOR_BOUNDS {
        let dx = px - cx;
        let dz = pz - cz;
        let dist_sq = dx * dx + dz * dz;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            closest_name = name;
        }
    }
    closest_name.to_string()
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
            tangents: Vec::new(),
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

/// Resolve the asset directory once, using the same rule in client and loaders.
pub fn asset_root() -> std::path::PathBuf {
    for candidate in ["assets", "../../assets", "../assets"] {
        let path = std::path::Path::new(candidate);
        if path.join("imported_map").is_dir() {
            return std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        }
    }
    if let Ok(executable) = std::env::current_exe() {
        let mut current = executable.parent();
        while let Some(parent) = current {
            let candidate = parent.join("assets");
            if candidate.join("imported_map").is_dir() {
                return candidate;
            }
            current = parent.parent();
        }
    }
    std::path::PathBuf::from("assets")
}

/// Runtime terrain has one versioned package. Legacy archives remain evidence,
/// and are never activated as a fallback for a missing or invalid package.
pub fn load_imported_terrain()
-> Result<Option<midnight_map_data::TerrainPackage>, midnight_map_data::ArchiveError> {
    let root = asset_root().join("imported_map/terrain");
    if !root.exists() {
        return Ok(None);
    }
    midnight_map_data::load_terrain_package(&root).map(Some)
}

pub fn load_imported_terrain_records()
-> Result<Vec<ImportedMeshRecord>, midnight_map_data::ArchiveError> {
    Ok(load_imported_terrain()?.map_or_else(Vec::new, |package| package.records))
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

pub fn load_imported_water_records() -> Vec<ImportedMeshRecord> {
    for candidate in [
        "assets/imported_map/water.bin",
        "../../assets/imported_map/water.bin",
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

pub fn load_imported_loading_tips() -> Vec<String> {
    for candidate in [
        "assets/imported_map/loading_tips.json",
        "../../assets/imported_map/loading_tips.json",
    ] {
        if let Ok(data) = std::fs::read_to_string(candidate)
            && let Ok(tips) = serde_json::from_str::<Vec<String>>(&data)
        {
            return tips;
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
        assert_eq!(sector_label_at([-2767.4, 0.0, -2783.8]), "A-01");
        assert_eq!(sector_label_at([-1306.5, 0.0, -794.9]), "A-45");
        assert_eq!(sector_label_at([0.0, 0.0, 400.0]), "D-01");
        assert_eq!(sector_label_at([400.0, 0.0, 0.0]), "B-50");
        assert_eq!(sector_label_at([-400.0, 0.0, 0.0]), "C-07");
        assert_eq!(sector_label_at([-4250.0, 0.0, -270.0]), "MIL-16");
    }

    #[test]
    fn imported_map_archives_parse_cleanly_if_present() {
        let terrain =
            load_imported_terrain_records().expect("canonical terrain must validate when present");
        if !terrain.is_empty() {
            assert!(terrain.len() >= 225);
            assert!(terrain.iter().any(|r| r.name == "A-57"));
            assert!(terrain.iter().any(|r| r.name == "A-01"));
            assert!(terrain.iter().any(|r| r.name == "D-56"));
        }
        let water = load_imported_water_records();
        if !water.is_empty() {
            assert!(water.len() >= 5);
            assert!(water.iter().any(|w| w.name.contains("water_01")));
            assert!(water.iter().any(|w| w.name.contains("water_neilushui")));
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
            assert!(tree_cfgs.len() >= 22);
            assert_eq!(tree_cfgs[0].name, "榆树");
            assert_eq!(tree_cfgs[1].name, "白桦树");
        }
        let loading_tips = load_imported_loading_tips();
        if !loading_tips.is_empty() {
            assert!(loading_tips.len() >= 100);
            assert!(loading_tips.iter().any(|t| t.contains("文明重启")));
        }
    }
}
