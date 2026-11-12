//! Terrain query geometry and source layer metadata share one commit boundary.

use super::source_geometry::{source_box_builder, source_trimesh_builder};
use super::*;
use midnight_map_data::{
    ArchiveError, SourceObject, TerrainCollisionShape, TerrainPackage, validate_mesh,
};

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

struct PreparedCollider {
    builder: ColliderBuilder,
    layer: Option<u8>,
    source: Option<SourceObject>,
}

struct PreparedTerrain {
    colliders: Vec<PreparedCollider>,
    trigger_layers: Vec<Option<u8>>,
}

fn prepare_legacy(package: &TerrainPackage) -> Result<PreparedTerrain, ArchiveError> {
    let colliders = package
        .collision_records()
        .map(|chunk| {
            validate_mesh(chunk)?;
            let source = package
                .manifest
                .meshes
                .get(&chunk.name)
                .ok_or_else(|| invalid(format!("{}: missing terrain metadata", chunk.name)))?;
            let vertices = chunk
                .positions
                .iter()
                .map(|position| Vec3::from_array(*position))
                .collect();
            let triangles = chunk.indices.as_chunks::<3>().0.to_vec();
            let builder = ColliderBuilder::trimesh(vertices, triangles)
                .map_err(|error| invalid(format!("{}: {error}", chunk.name)))?;
            Ok(PreparedCollider {
                builder,
                layer: source.layer,
                source: None,
            })
        })
        .collect::<Result<Vec<_>, ArchiveError>>()?;
    let mut triggers = BTreeMap::new();
    for source in package.manifest.meshes.values() {
        let Some(collider) = &source.collider else {
            continue;
        };
        if !collider.enabled || !collider.is_trigger {
            continue;
        }
        if triggers
            .insert(&collider.source, source.layer)
            .is_some_and(|previous| previous != source.layer)
        {
            return Err(invalid("terrain source trigger has conflicting layers"));
        }
    }
    Ok(PreparedTerrain {
        colliders,
        trigger_layers: triggers.into_values().collect(),
    })
}

fn prepare_table(package: &TerrainPackage) -> Result<PreparedTerrain, ArchiveError> {
    let table = package
        .manifest
        .collision
        .as_ref()
        .ok_or_else(|| invalid("independent terrain collision table is missing"))?;
    let mut colliders = Vec::new();
    let mut trigger_layers = Vec::new();
    for entry in &table.colliders {
        if !entry.enabled || !entry.active_in_hierarchy || !entry.active_self {
            continue;
        }
        if entry.is_trigger {
            trigger_layers.push(Some(entry.layer));
            continue;
        }
        let builder = match &entry.shape {
            TerrainCollisionShape::Box { center, size } => {
                source_box_builder(*center, *size, &entry.world_transform)?
            }
            TerrainCollisionShape::Mesh { record, convex, .. } => {
                if *convex {
                    return Err(invalid(
                        "Unity convex terrain mesh cooking has not been authenticated",
                    ));
                }
                let mesh = package
                    .collision_mesh(record)
                    .ok_or_else(|| invalid(format!("missing terrain collision mesh {record}")))?;
                source_trimesh_builder(&mesh.positions, &mesh.indices, &entry.world_transform)?
            }
        };
        colliders.push(PreparedCollider {
            builder,
            layer: Some(entry.layer),
            source: Some(entry.source.clone()),
        });
    }
    Ok(PreparedTerrain {
        colliders,
        trigger_layers,
    })
}

impl CollisionWorld {
    /// Prepare all source solid shapes and layer/trigger metadata before
    /// replacing terrain. Schema 4 uses independent component records, so
    /// collider-only objects and multiple colliders do not become render meshes
    /// or alias each other. Legacy schemas retain their renderer-derived query
    /// geometry; schema-2 layers remain unknown. Enabled triggers remain
    /// physically unrepresented and retain distinct source query metadata.
    pub fn insert_canonical_terrain(
        &mut self,
        package: &TerrainPackage,
    ) -> Result<(), ArchiveError> {
        package.manifest.validate_layers()?;
        package.validate_collision_geometry()?;
        let prepared = if package.manifest.collision.is_some() {
            prepare_table(package)?
        } else {
            prepare_legacy(package)?
        };
        if prepared.colliders.is_empty() {
            return Err(invalid("canonical map has no enabled source colliders"));
        }
        let removed = std::mem::take(&mut self.initial_colliders);
        for &handle in &removed {
            self.source_collision_layers.remove(&handle);
            self.colliders.remove(
                handle,
                &mut self.islands,
                &mut self.bodies,
                &mut self.soft_bodies,
                false,
            );
        }
        let mut solid_handles = BTreeMap::new();
        let handles = prepared
            .colliders
            .into_iter()
            .map(|prepared| {
                let handle = self.colliders.insert(prepared.builder);
                if let Some(layer) = prepared.layer {
                    self.source_collision_layers.insert(handle, layer);
                }
                if let Some(source) = prepared.source {
                    solid_handles.insert(source, handle);
                }
                handle
            })
            .collect::<Vec<_>>();
        self.initial_colliders = handles.clone();
        self.source_terrain_trigger_layers = prepared.trigger_layers;
        self.source_terrain_solid_handles = solid_handles;
        self.imported_ground = true;
        self.refresh(&handles, &removed);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        source_prefabs::tests::{IDENTITY, package as prefab_package, source},
        tests::terrain_package,
    };
    use super::*;
    use midnight_game_data::clearance::{SourceStandClearanceCast, SourceStandClearanceKind};
    use midnight_map_data::{
        FileIdentity, TerrainCollisionEntry, TerrainCollisionManifest, TerrainCollisionMesh,
    };

    fn cast() -> SourceStandClearanceCast {
        SourceStandClearanceCast::from_bevy_root_bone_position(
            SourceStandClearanceKind::CrouchUp,
            [15.0, 9.0, 0.0],
        )
        .unwrap()
    }

    fn set_layers(package: &mut TerrainPackage, layer: Option<u8>) {
        for mesh in package.manifest.meshes.values_mut() {
            mesh.layer = layer;
        }
    }

    fn table_entry(
        id: i64,
        owner: i64,
        layer: u8,
        shape: TerrainCollisionShape,
    ) -> TerrainCollisionEntry {
        TerrainCollisionEntry {
            source: source(id),
            game_object: source(owner),
            transform: source(owner + 1_000),
            layer,
            active_self: true,
            active_in_hierarchy: true,
            enabled: true,
            is_trigger: false,
            world_transform: IDENTITY,
            shape,
        }
    }

    fn independent_table_package() -> TerrainPackage {
        let mut package = terrain_package();
        package.manifest.schema_version = midnight_map_data::TERRAIN_MANIFEST_VERSION;
        for mesh in package.manifest.meshes.values_mut() {
            mesh.collider = None;
        }
        package.collision_meshes = vec![TerrainCollisionMesh {
            name: "ceiling".into(),
            positions: vec![[-4., 0., -4.], [4., 0., -4.], [4., 0., 4.], [-4., 0., 4.]],
            indices: vec![0, 2, 1, 0, 3, 2],
        }];
        let mesh_shape = TerrainCollisionShape::Mesh {
            mesh: source(300),
            record: "ceiling".into(),
            convex: false,
            cooking_options: 30,
        };
        let mut first = table_entry(100, 200, 19, mesh_shape.clone());
        first.world_transform[13] = 1.;
        let mut duplicate = first.clone();
        duplicate.source = source(101);
        let mut box_entry = table_entry(
            102,
            201,
            0,
            TerrainCollisionShape::Box {
                center: [0., 0., 1.],
                size: [2., 2., 2.],
            },
        );
        box_entry.world_transform = [
            0., 0., -2., 0., 0., 1., 0., 0., 3., 0., 0., 0., 10., 4., 6., 1.,
        ];
        let mut trigger = table_entry(
            103,
            202,
            2,
            TerrainCollisionShape::Box {
                center: [0.; 3],
                size: [1.; 3],
            },
        );
        trigger.is_trigger = true;
        let mut inactive = table_entry(
            104,
            203,
            0,
            TerrainCollisionShape::Box {
                center: [0.; 3],
                size: [3., 0.2, 3.],
            },
        );
        inactive.world_transform[12] = 50.;
        inactive.world_transform[13] = 1.;
        inactive.active_self = false;
        inactive.active_in_hierarchy = false;
        let mut disabled = inactive.clone();
        disabled.source = source(105);
        disabled.game_object = source(204);
        disabled.transform = source(1204);
        disabled.world_transform[12] = 60.;
        disabled.enabled = false;
        disabled.active_self = true;
        disabled.active_in_hierarchy = true;
        let mut parent_inactive = disabled.clone();
        parent_inactive.source = source(106);
        parent_inactive.game_object = source(205);
        parent_inactive.transform = source(1205);
        parent_inactive.world_transform[12] = 70.;
        parent_inactive.enabled = true;
        parent_inactive.active_in_hierarchy = false;
        package.manifest.collision = Some(TerrainCollisionManifest {
            geometry: FileIdentity {
                path: "collision.bin".into(),
                sha256: "c".repeat(64),
            },
            colliders: vec![
                first,
                duplicate,
                box_entry,
                trigger,
                inactive,
                disabled,
                parent_inactive,
            ],
        });
        package
    }

    fn root_cast(kind: SourceStandClearanceKind, root: [f32; 3]) -> SourceStandClearanceCast {
        SourceStandClearanceCast::from_bevy_root_bone_position(kind, root).unwrap()
    }

    #[test]
    fn independent_table_keeps_identical_components_separate_and_does_not_add_render_records() {
        let package = independent_table_package();
        let render_records = package.records.clone();
        let mut first = CollisionWorld::new(MapId::ImportedIsland);
        let mut second = CollisionWorld::new(MapId::ImportedIsland);
        first.insert_canonical_terrain(&package).unwrap();
        second.insert_canonical_terrain(&package).unwrap();
        assert_eq!(first.initial_colliders.len(), 3);
        assert_eq!(first.source_terrain_solid_handles.len(), 3);
        assert_eq!(first.source_terrain_trigger_layers, [Some(2)]);
        let a = first.source_terrain_solid_handles[&source(100)];
        let b = first.source_terrain_solid_handles[&source(101)];
        assert_ne!(a, b);
        assert_eq!(
            first.colliders[a].compute_aabb(),
            first.colliders[b].compute_aabb()
        );
        assert_eq!(package.records, render_records);
        assert_eq!(package.manifest.meshes.len(), render_records.len());
        let request = root_cast(SourceStandClearanceKind::CrouchUp, [0.; 3]);
        assert_eq!(
            first.check_source_stand_clearance(request, false),
            second.check_source_stand_clearance(request, false)
        );
        assert!(
            !first
                .check_source_stand_clearance(request, false)
                .unwrap()
                .allows_standing()
        );
        for root in [[50., 0., 0.], [60., 0., 0.], [70., 0., 0.]] {
            assert!(
                first
                    .check_source_stand_clearance(
                        root_cast(SourceStandClearanceKind::CrouchUp, root),
                        false
                    )
                    .unwrap()
                    .allows_standing()
            );
        }
    }

    #[test]
    fn independent_box_uses_raw_center_reflection_and_complete_affine_transform() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world
            .insert_canonical_terrain(&independent_table_package())
            .unwrap();
        let handle = world.source_terrain_solid_handles[&source(102)];
        let bounds = world.colliders[handle].compute_aabb();
        assert_eq!(bounds.mins, Vec3::new(4., 3., 4.));
        assert_eq!(bounds.maxs, Vec3::new(10., 5., 8.));
        let root = [7., 1., 6.];
        assert!(
            world
                .check_source_stand_clearance(
                    root_cast(SourceStandClearanceKind::CrouchUp, root),
                    false
                )
                .unwrap()
                .allows_standing()
        );
        let SourceClearanceResult::Blocked { distance } = world
            .check_source_stand_clearance(
                root_cast(SourceStandClearanceKind::PaToStand, root),
                false,
            )
            .unwrap()
        else {
            panic!("longer cast must hit the transformed box");
        };
        assert!((distance - 1.8).abs() < 0.001);
    }

    #[test]
    fn independent_mesh_reflection_preserves_vertices_and_reverses_winding_once() {
        let mut package = independent_table_package();
        let table = package.manifest.collision.as_mut().unwrap();
        for entry in &mut table.colliders[..2] {
            entry.world_transform[0] = -2.;
            entry.world_transform[10] = 3.;
        }
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.insert_canonical_terrain(&package).unwrap();
        let collider = &world.colliders[world.source_terrain_solid_handles[&source(100)]];
        let shape = collider.shape().as_trimesh().unwrap();
        assert_eq!(shape.vertices()[0], Vec3::new(8., 1., -12.));
        assert_eq!(shape.indices()[0], [0, 1, 2]);
        assert!(
            world
                .cast_ray([0., 3., 0.], [0., -1., 0.], 3., None)
                .is_some()
        );
    }

    #[test]
    fn independent_table_rolls_back_invalid_or_convex_shapes_and_preserves_other_state() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world
            .replace_source_prefab("preserved", &prefab_package(), &source(1), IDENTITY)
            .unwrap();
        let player = PlayerId(88);
        let state = PlayerState::at([100., 100., 100.]);
        world.set_player(player, state);
        world
            .insert_canonical_terrain(&independent_table_package())
            .unwrap();
        let handles = world.initial_colliders.clone();
        let layers = world.source_collision_layers.clone();
        let identities = world.source_terrain_solid_handles.clone();
        let triggers = world.source_terrain_trigger_layers.clone();
        let prefab_handles = world.source_prefabs["preserved"].clone();
        for failure in 0..3 {
            let mut broken = independent_table_package();
            match failure {
                0 => broken.collision_meshes[0].indices[0] = 99,
                1 => broken.manifest.collision.as_mut().unwrap().colliders[2].layer = 32,
                _ => {
                    let shape = &mut broken.manifest.collision.as_mut().unwrap().colliders[1].shape;
                    if let TerrainCollisionShape::Mesh { convex, .. } = shape {
                        *convex = true;
                    }
                }
            }
            assert!(world.insert_canonical_terrain(&broken).is_err());
            assert_eq!(world.initial_colliders, handles);
            assert_eq!(world.source_collision_layers, layers);
            assert_eq!(world.source_terrain_solid_handles, identities);
            assert_eq!(world.source_terrain_trigger_layers, triggers);
            assert_eq!(world.source_prefabs["preserved"], prefab_handles);
            assert_eq!(world.state(player), Some(state));
        }
        world.insert_canonical_terrain(&terrain_package()).unwrap();
        assert!(world.source_terrain_solid_handles.is_empty());
        for handle in handles {
            assert!(world.colliders.get(handle).is_none());
            assert!(!world.source_collision_layers.contains_key(&handle));
        }
        assert_eq!(world.source_prefabs["preserved"], prefab_handles);
        assert_eq!(world.state(player), Some(state));
    }

    #[test]
    fn independent_table_filters_solid_and_trigger_layers_by_the_native_mask() {
        for layer in [0, 19, 30, 31] {
            let mut package = independent_table_package();
            for entry in &mut package.manifest.collision.as_mut().unwrap().colliders[..2] {
                entry.layer = layer;
            }
            let mut world = CollisionWorld::new(MapId::ImportedIsland);
            world.insert_canonical_terrain(&package).unwrap();
            let request = root_cast(SourceStandClearanceKind::CrouchUp, [0.; 3]);
            assert_eq!(
                world
                    .check_source_stand_clearance(request, true)
                    .unwrap()
                    .allows_standing(),
                layer == 31
            );
        }
        let mut package = independent_table_package();
        package.manifest.collision.as_mut().unwrap().colliders[3].layer = 19;
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.insert_canonical_terrain(&package).unwrap();
        let request = root_cast(SourceStandClearanceKind::CrouchUp, [20., 0., 0.]);
        assert_eq!(
            world.check_source_stand_clearance(request, true),
            Err(SourceClearanceError::UnrepresentedTriggers { count: 1 })
        );
        assert!(
            world
                .check_source_stand_clearance(request, false)
                .unwrap()
                .allows_standing()
        );
    }

    #[test]
    fn installed_terrain_preserves_unity_mask_in_independent_query_worlds() {
        for layer in [0, 19, 30, 31] {
            let mut terrain = terrain_package();
            set_layers(&mut terrain, Some(layer));
            let mut first = CollisionWorld::new(MapId::ImportedIsland);
            let mut second = CollisionWorld::new(MapId::ImportedIsland);
            first.insert_canonical_terrain(&terrain).unwrap();
            second.insert_canonical_terrain(&terrain).unwrap();
            let result = first.check_source_stand_clearance(cast(), false).unwrap();
            assert_eq!(
                result,
                second.check_source_stand_clearance(cast(), false).unwrap()
            );
            assert_eq!(first.source_collision_layers.len(), 2);
            if layer == 31 {
                assert!(result.allows_standing());
            } else {
                let SourceClearanceResult::Blocked { distance } = result else {
                    panic!("included terrain layer should block standing")
                };
                assert!((distance - 0.8).abs() < 0.001);
            }
        }
    }

    #[test]
    fn legacy_terrain_keeps_solid_and_skipped_trigger_layers_explicitly_unknown() {
        let mut terrain = terrain_package();
        terrain.manifest.schema_version = 2;
        set_layers(&mut terrain, None);
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.insert_canonical_terrain(&terrain).unwrap();
        assert!(world.source_collision_layers.is_empty());
        assert_eq!(world.source_terrain_trigger_layers, [None]);
        assert_eq!(
            world.check_source_stand_clearance(cast(), false),
            Err(SourceClearanceError::UnknownColliderLayers { count: 2 })
        );
        assert_eq!(
            world.check_source_stand_clearance(cast(), true),
            Err(SourceClearanceError::UnknownTriggerLayers { count: 1 })
        );
        set_layers(&mut terrain, Some(0));
        terrain.manifest.schema_version = 3;
        world.insert_canonical_terrain(&terrain).unwrap();
        assert_eq!(world.source_collision_layers.len(), 2);
        assert_eq!(world.source_terrain_trigger_layers, [Some(0)]);
        assert!(
            !world
                .check_source_stand_clearance(cast(), false)
                .unwrap()
                .allows_standing()
        );
    }

    #[test]
    fn terrain_replacement_cleans_its_metadata_and_preserves_prefabs_and_players() {
        let mut world = CollisionWorld::new(MapId::Polygon);
        let demo = world.initial_colliders.clone();
        world
            .replace_source_prefab("retained", &prefab_package(), &source(1), IDENTITY)
            .unwrap();
        let prefab_handles = world.source_prefabs["retained"].clone();
        let player = PlayerId(81);
        let state = PlayerState::at([100.0, 100.0, 100.0]);
        world.set_player(player, state);
        let mut terrain = terrain_package();
        world.insert_canonical_terrain(&terrain).unwrap();
        assert!(
            demo.iter()
                .all(|handle| world.colliders.get(*handle).is_none())
        );
        let previous = world.initial_colliders.clone();
        let old_triggers = world.source_terrain_trigger_layers.clone();
        set_layers(&mut terrain, Some(31));
        world.insert_canonical_terrain(&terrain).unwrap();
        for handle in previous {
            assert!(world.colliders.get(handle).is_none());
            assert!(!world.source_collision_layers.contains_key(&handle));
        }
        assert_ne!(world.source_terrain_trigger_layers, old_triggers);
        assert_eq!(world.source_terrain_trigger_layers, [Some(31)]);
        assert!(
            world
                .initial_colliders
                .iter()
                .all(|handle| { world.source_collision_layers.get(handle) == Some(&31) })
        );
        assert_eq!(world.source_prefabs["retained"], prefab_handles);
        assert_eq!(world.state(player), Some(state));
        assert!(prefab_handles.iter().all(|handle| {
            world.colliders.get(*handle).is_some()
                && world.source_collision_layers.get(handle) == Some(&0)
        }));
    }

    #[test]
    fn invalid_layer_or_late_geometry_failure_rolls_back_shapes_and_metadata() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.insert_canonical_terrain(&terrain_package()).unwrap();
        let handles = world.initial_colliders.clone();
        let layers = world.source_collision_layers.clone();
        let triggers = world.source_terrain_trigger_layers.clone();
        let before = world.check_source_stand_clearance(cast(), false);
        let mut terrain = terrain_package();
        terrain.manifest.meshes.get_mut("trigger").unwrap().layer = Some(32);
        assert!(world.insert_canonical_terrain(&terrain).is_err());
        assert_eq!(world.initial_colliders, handles);
        assert_eq!(world.source_collision_layers, layers);
        assert_eq!(world.source_terrain_trigger_layers, triggers);
        let mut terrain = terrain_package();
        set_layers(&mut terrain, Some(19));
        terrain.records[1].indices[0] = 99;
        assert!(world.insert_canonical_terrain(&terrain).is_err());
        assert_eq!(world.initial_colliders, handles);
        assert_eq!(world.source_collision_layers, layers);
        assert_eq!(world.source_terrain_trigger_layers, triggers);
        assert_eq!(world.check_source_stand_clearance(cast(), false), before);
    }

    #[test]
    fn skipped_terrain_triggers_are_filtered_and_deduplicated_by_source_identity() {
        let mut terrain = terrain_package();
        set_layers(&mut terrain, Some(31));
        let mut duplicate = terrain.records[3].clone();
        duplicate.name = "trigger@submesh-1".into();
        terrain.manifest.meshes.insert(
            duplicate.name.clone(),
            terrain.manifest.meshes["trigger"].clone(),
        );
        terrain.records.push(duplicate);
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        world.insert_canonical_terrain(&terrain).unwrap();
        assert_eq!(world.source_terrain_trigger_layers, [Some(31)]);
        assert!(
            world
                .check_source_stand_clearance(cast(), true)
                .unwrap()
                .allows_standing()
        );
        for name in ["trigger", "trigger@submesh-1"] {
            terrain.manifest.meshes.get_mut(name).unwrap().layer = Some(19);
        }
        world.insert_canonical_terrain(&terrain).unwrap();
        assert_eq!(world.source_terrain_trigger_layers, [Some(19)]);
        assert!(
            world
                .check_source_stand_clearance(cast(), false)
                .unwrap()
                .allows_standing()
        );
        assert_eq!(
            world.check_source_stand_clearance(cast(), true),
            Err(SourceClearanceError::UnrepresentedTriggers { count: 1 })
        );
        for name in ["trigger", "trigger@submesh-1"] {
            terrain
                .manifest
                .meshes
                .get_mut(name)
                .unwrap()
                .collider
                .as_mut()
                .unwrap()
                .enabled = false;
        }
        world.insert_canonical_terrain(&terrain).unwrap();
        assert!(world.source_terrain_trigger_layers.is_empty());
        assert!(
            world
                .check_source_stand_clearance(cast(), true)
                .unwrap()
                .allows_standing()
        );
    }

    #[test]
    #[ignore = "requires the separately verified terrain layer metadata package"]
    fn retained_terrain_layers_populate_shared_collision() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/imported_map/terrain");
        let terrain = midnight_map_data::load_terrain_package(&root).unwrap();
        assert!(matches!(
            terrain.manifest.schema_version,
            3 | midnight_map_data::TERRAIN_MANIFEST_VERSION
        ));
        let mut first = CollisionWorld::new(MapId::ImportedIsland);
        let mut second = CollisionWorld::new(MapId::ImportedIsland);
        first.insert_canonical_terrain(&terrain).unwrap();
        second.insert_canonical_terrain(&terrain).unwrap();
        let expected = terrain.manifest.collision.as_ref().map_or_else(
            || terrain.collision_records().count(),
            |table| {
                table
                    .colliders
                    .iter()
                    .filter(|entry| {
                        entry.active_in_hierarchy
                            && entry.active_self
                            && entry.enabled
                            && !entry.is_trigger
                    })
                    .count()
            },
        );
        assert_eq!(first.initial_colliders.len(), expected);
        assert_eq!(
            first.source_collision_layers.len(),
            first.initial_colliders.len()
        );
        for (a, b) in first
            .initial_colliders
            .iter()
            .zip(&second.initial_colliders)
        {
            assert_eq!(
                first.source_collision_layers[a],
                second.source_collision_layers[b]
            );
            assert_eq!(
                first.colliders[*a].compute_aabb(),
                second.colliders[*b].compute_aabb()
            );
        }
        assert_eq!(
            first.source_terrain_trigger_layers,
            second.source_terrain_trigger_layers
        );
        eprintln!(
            "Retained terrain layers: {} source collision records, {} classified solid handles, {} skipped source triggers",
            first.initial_colliders.len(),
            first.source_collision_layers.len(),
            first.source_terrain_trigger_layers.len()
        );
    }

    #[test]
    #[ignore = "requires the verified independent source terrain collision table"]
    fn retained_independent_terrain_collider_table_is_complete() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/imported_map/terrain");
        let terrain = midnight_map_data::load_terrain_package(&root).unwrap();
        let table = terrain.manifest.collision.as_ref().unwrap();
        assert_eq!(
            terrain.manifest.schema_version,
            midnight_map_data::TERRAIN_MANIFEST_VERSION
        );
        assert_eq!(table.colliders.len(), 404);
        assert_eq!(terrain.records.len(), 400);
        let mut first = CollisionWorld::new(MapId::ImportedIsland);
        let mut second = CollisionWorld::new(MapId::ImportedIsland);
        first.insert_canonical_terrain(&terrain).unwrap();
        second.insert_canonical_terrain(&terrain).unwrap();
        assert_eq!(first.initial_colliders.len(), 389);
        assert_eq!(first.source_terrain_solid_handles.len(), 389);
        assert_eq!(first.source_collision_layers.len(), 389);
        assert_eq!(first.source_terrain_trigger_layers, [Some(2); 12]);
        for entry in &table.colliders {
            if !entry.enabled
                || !entry.active_in_hierarchy
                || !entry.active_self
                || entry.is_trigger
            {
                assert!(
                    !first
                        .source_terrain_solid_handles
                        .contains_key(&entry.source)
                );
                continue;
            }
            let a = first.source_terrain_solid_handles[&entry.source];
            let b = second.source_terrain_solid_handles[&entry.source];
            assert_eq!(first.source_collision_layers[&a], entry.layer);
            assert_eq!(
                first.source_collision_layers[&a],
                second.source_collision_layers[&b]
            );
            assert_eq!(
                first.colliders[a].compute_aabb(),
                second.colliders[b].compute_aabb()
            );
        }
        let b32 = |id: &str| SourceObject {
            bundle: "assets/scene/b-32.ab".into(),
            cab: "BuildPlayer-B-32".into(),
            path_id: id.into(),
        };
        let a = first.source_terrain_solid_handles[&b32("10")];
        let b = first.source_terrain_solid_handles[&b32("11")];
        assert_ne!(a, b);
        assert_eq!(
            first.colliders[a].compute_aabb(),
            first.colliders[b].compute_aabb()
        );
        eprintln!(
            "Independent terrain collision: {} visible records, {} solid components, {} skipped source triggers, {} source entries; B-32 components10/11 both represented",
            terrain.records.len(),
            first.initial_colliders.len(),
            first.source_terrain_trigger_layers.len(),
            table.colliders.len()
        );
    }
}
