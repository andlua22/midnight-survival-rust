//! Validated content selection shared by local simulation and the server.
//! Source packages are loaded before actors are spawned; fixture geometry is explicit.

use std::{path::PathBuf, sync::Arc};

pub use crate::content::{GameplayIdentity, TerrainIdentity, WorldDescriptor};
use bevy::prelude::Resource;
use midnight_game_data::{GameplayCatalog, package::load_package};
use midnight_map_data::{ArchiveError, TerrainPackage, file_sha256, load_terrain_package};

pub const SOURCE_GAMEPLAY_PACKAGE: &str =
    "imported_gameplay/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df-v2";

use crate::{
    level::{MapId, SPAWN_POSITION, asset_root},
    physics::CollisionWorld,
};

#[derive(Clone, Debug, Resource)]
pub struct WorldSelection {
    pub map: MapId,
    pub terrain_root: PathBuf,
    pub gameplay_root: PathBuf,
}

impl Default for WorldSelection {
    fn default() -> Self {
        Self::for_map(MapId::Polygon)
    }
}

impl WorldSelection {
    pub fn for_map(map: MapId) -> Self {
        Self {
            map,
            terrain_root: asset_root().join("imported_map/terrain"),
            gameplay_root: asset_root().join(SOURCE_GAMEPLAY_PACKAGE),
        }
    }
}

#[derive(Clone, Debug, Resource)]
pub struct LoadedWorld {
    pub descriptor: WorldDescriptor,
    pub terrain: Option<Arc<TerrainPackage>>,
    pub gameplay: Option<Arc<GameplayCatalog>>,
    pub spawn: [f32; 3],
}

#[derive(Debug, Resource)]
pub struct WorldLoadFailure(pub String);

impl LoadedWorld {
    pub fn load(selection: &WorldSelection) -> Result<(Self, CollisionWorld), ArchiveError> {
        let mut collision = CollisionWorld::new(selection.map);
        if selection.map != MapId::ImportedIsland {
            return Ok((
                Self {
                    descriptor: WorldDescriptor::fixture(selection.map),
                    terrain: None,
                    gameplay: None,
                    spawn: SPAWN_POSITION,
                },
                collision,
            ));
        }
        let manifest_path = selection.terrain_root.join("manifest.json");
        let before = file_sha256(&manifest_path)?;
        let terrain = load_terrain_package(&selection.terrain_root)?;
        let after = file_sha256(&manifest_path)?;
        if before != after {
            return Err(ArchiveError::Invalid(
                "terrain manifest changed while loading".into(),
            ));
        }
        let gameplay_manifest_path = selection.gameplay_root.join("manifest.json");
        let gameplay_before = file_sha256(&gameplay_manifest_path)?;
        let (gameplay_manifest, gameplay) = load_package(&selection.gameplay_root)
            .map_err(|error| ArchiveError::Invalid(format!("source gameplay package: {error}")))?;
        let gameplay_after = file_sha256(&gameplay_manifest_path)?;
        if gameplay_before != gameplay_after {
            return Err(ArchiveError::Invalid(
                "gameplay manifest changed while loading".into(),
            ));
        }
        if gameplay_manifest.source_apk_sha256 != terrain.manifest.source_apk.sha256 {
            return Err(ArchiveError::Invalid(
                "terrain and gameplay packages come from different APKs".into(),
            ));
        }
        collision.insert_canonical_terrain(&terrain)?;
        let spawn = collision
            .surface_spawn_at([SPAWN_POSITION[0], SPAWN_POSITION[2]], None)
            .or_else(|| collision.spawn_position(None))
            .ok_or_else(|| {
                ArchiveError::Invalid("canonical terrain has no free spawn surface".into())
            })?;
        let descriptor = WorldDescriptor {
            map: selection.map,
            terrain: Some(TerrainIdentity {
                source_apk_sha256: terrain.manifest.source_apk.sha256.clone(),
                manifest_sha256: after,
                geometry_sha256: terrain.manifest.geometry.sha256.clone(),
            }),
            gameplay: Some(GameplayIdentity {
                source_apk_sha256: gameplay_manifest.source_apk_sha256,
                manifest_sha256: gameplay_after,
                catalog_sha256: gameplay_manifest.catalog.sha256,
            }),
        };
        Ok((
            Self {
                descriptor,
                terrain: Some(Arc::new(terrain)),
                gameplay: Some(Arc::new(gameplay)),
                spawn,
            },
            collision,
        ))
    }

    /// Reconnection/prediction rebuilds collision from the retained validated package.
    pub fn collision(&self) -> Result<CollisionWorld, ArchiveError> {
        let mut collision = CollisionWorld::new(self.descriptor.map);
        if let Some(terrain) = &self.terrain {
            collision.insert_canonical_terrain(terrain)?;
        } else if self.descriptor.map == MapId::ImportedIsland {
            return Err(ArchiveError::Invalid(
                "imported world is missing its validated terrain".into(),
            ));
        }
        Ok(collision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imported_selection_never_substitutes_fixture_geometry() {
        let selection = WorldSelection {
            map: MapId::ImportedIsland,
            terrain_root: PathBuf::from("/nonexistent-midnight-terrain-package"),
            gameplay_root: PathBuf::from("/nonexistent-midnight-gameplay-package"),
        };
        assert!(LoadedWorld::load(&selection).is_err());
        assert!(crate::level::map_geometry(selection.map).is_empty());
        assert!(
            !WorldDescriptor::fixture(selection.map)
                .compatible_with(&WorldDescriptor::fixture(selection.map))
        );
    }

    #[test]
    fn fixtures_are_explicit_and_content_identity_is_checked() {
        let (loaded, _) = LoadedWorld::load(&WorldSelection::for_map(MapId::Polygon)).unwrap();
        assert!(
            loaded
                .descriptor
                .compatible_with(&WorldDescriptor::fixture(MapId::Polygon))
        );
        assert!(
            !loaded
                .descriptor
                .compatible_with(&WorldDescriptor::fixture(MapId::LoadArena))
        );
        let a = WorldDescriptor {
            map: MapId::ImportedIsland,
            terrain: Some(TerrainIdentity {
                source_apk_sha256: "a".repeat(64),
                manifest_sha256: "b".repeat(64),
                geometry_sha256: "c".repeat(64),
            }),
            gameplay: Some(GameplayIdentity {
                source_apk_sha256: "a".repeat(64),
                manifest_sha256: "e".repeat(64),
                catalog_sha256: "f".repeat(64),
            }),
        };
        let mut b = a.clone();
        b.terrain.as_mut().unwrap().manifest_sha256 = "d".repeat(64);
        assert!(!a.compatible_with(&b));
        let mut b = a.clone();
        b.gameplay.as_mut().unwrap().catalog_sha256 = "0".repeat(64);
        assert!(!a.compatible_with(&b));
        let mut b = a.clone();
        b.gameplay.as_mut().unwrap().source_apk_sha256 = "0".repeat(64);
        assert!(!b.compatible_with(&b));
    }

    #[test]
    #[ignore = "requires the retained APK terrain package"]
    fn canonical_world_rebuilds_identical_collision_and_spawn() {
        let (loaded, collision) =
            LoadedWorld::load(&WorldSelection::for_map(MapId::ImportedIsland)).unwrap();
        let rebuilt = loaded.collision().unwrap();
        assert_eq!(loaded.terrain.as_ref().unwrap().records.len(), 400);
        assert_eq!(
            loaded.terrain.as_ref().unwrap().collision_records().count(),
            383
        );
        assert_eq!(loaded.gameplay.as_ref().unwrap().items.len(), 16861);
        assert!(loaded.descriptor.compatible_with(&loaded.descriptor));
        for xz in [[0.0, 12.0], [-8.0, 9.0], [7.0, 18.0], [-3383.0, -1013.0]] {
            assert_eq!(
                collision.surface_spawn_at(xz, None),
                rebuilt.surface_spawn_at(xz, None)
            );
        }
        let z01_spawn = collision
            .surface_spawn_at([-3383.0, -1013.0], None)
            .unwrap();
        assert!(collision.free_at(z01_spawn, None));
        assert!(collision.free_at(loaded.spawn, None));
    }
}
