//! Source prefab collision preparation is shared by server and prediction.
//! Callers supply a separately authenticated placement; packages supply local geometry.
use super::*;
use midnight_map_data::{ArchiveError, PrefabPackage, SourceObject, prefab_transform_point};

#[derive(Debug, Default)]
pub struct PrefabCollisionReport {
    pub solid_colliders: usize,
    pub inactive_or_disabled: usize,
    pub triggers: usize,
    /// Source component identities, in source node/component order.
    pub source_colliders: Vec<SourceObject>,
}

fn invalid(message: impl Into<String>) -> ArchiveError {
    ArchiveError::Invalid(message.into())
}

impl CollisionWorld {
    /// Prepare all shapes before replacing one instance. Root TRS is assigned
    /// exactly as by the source placement loader; local mesh positions are already
    /// Bevy-space. A failed replacement leaves the previous instance/BVH intact.
    /// This API does not authenticate arbitrary caller-provided coordinates.
    pub fn replace_source_prefab(
        &mut self,
        instance: &str,
        package: &PrefabPackage,
        root: &SourceObject,
        placement: [f32; 16],
    ) -> Result<PrefabCollisionReport, ArchiveError> {
        if instance.is_empty() {
            return Err(invalid("source prefab instance requires a stable ID"));
        }
        let poses = package.placed_poses(root, placement)?;
        let meshes = package
            .manifest
            .collision_meshes
            .iter()
            .map(|m| (&m.source, m))
            .collect::<BTreeMap<_, _>>();
        let mut builders = Vec::new();
        let mut report = PrefabCollisionReport::default();
        for (node, pose) in package.manifest.nodes.iter().zip(&poses) {
            let Some(pose) = pose else {
                continue;
            };
            if pose.active_in_hierarchy && node.components.iter().any(|c| c.class_id == 54) {
                return Err(invalid(
                    "source rigid-body prefab needs authoritative dynamic-body conversion",
                ));
            }
            for component in node
                .components
                .iter()
                .filter(|c| matches!(c.class_id, 64 | 65 | 135 | 136 | 143))
            {
                if !pose.active_in_hierarchy || !component.boolean("m_Enabled")? {
                    report.inactive_or_disabled += 1;
                    continue;
                }
                if component.boolean("m_IsTrigger")? {
                    report.triggers += 1;
                    continue;
                }
                let builder = match component.class_id {
                    65 => {
                        let mut center = component.vector3("m_Center")?;
                        center[2] = -center[2]; // Raw primitive fields are still Unity-local.
                        let size = component.vector3("m_Size")?;
                        if size.iter().any(|v| *v <= 0.) {
                            return Err(invalid("source box has nonpositive size"));
                        }
                        // Baking all eight corners preserves parent rotation and
                        // nonuniform scale without reducing the box to world AABB.
                        let mut points = Vec::with_capacity(8);
                        for x in [-0.5, 0.5] {
                            for y in [-0.5, 0.5] {
                                for z in [-0.5, 0.5] {
                                    let point = [
                                        center[0] + x * size[0],
                                        center[1] + y * size[1],
                                        center[2] + z * size[2],
                                    ];
                                    points.push(Vec3::from_array(prefab_transform_point(
                                        &pose.world_transform,
                                        point,
                                    )?));
                                }
                            }
                        }
                        ColliderBuilder::convex_hull(&points)
                            .ok_or_else(|| invalid("source box convex hull failed"))?
                    }
                    64 => {
                        if component.boolean("m_Convex")? {
                            return Err(invalid(
                                "Unity convex mesh cooking has not been authenticated",
                            ));
                        }
                        let mesh_id = component.reference("m_Mesh")?;
                        let mesh = meshes
                            .get(&mesh_id)
                            .ok_or_else(|| invalid("source mesh collider is missing"))?;
                        if mesh.indices.is_empty()
                            || !mesh.indices.len().is_multiple_of(3)
                            || mesh
                                .indices
                                .iter()
                                .any(|&i| i as usize >= mesh.positions.len())
                        {
                            return Err(invalid("invalid source collider triangle indices"));
                        }
                        let vertices = mesh
                            .positions
                            .iter()
                            .map(|&p| {
                                prefab_transform_point(&pose.world_transform, p)
                                    .map(Vec3::from_array)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let mut triangles = mesh.indices.as_chunks::<3>().0.to_vec();
                        let m = pose.world_transform;
                        let determinant = Vec3::new(m[0], m[1], m[2])
                            .cross(Vec3::new(m[4], m[5], m[6]))
                            .dot(Vec3::new(m[8], m[9], m[10]));
                        if determinant < 0. {
                            for triangle in &mut triangles {
                                triangle.swap(1, 2);
                            }
                        }
                        ColliderBuilder::trimesh(vertices, triangles).map_err(|error| {
                            invalid(format!(
                                "source collider {}: {error}",
                                component.source.path_id
                            ))
                        })?
                    }
                    _ => {
                        return Err(invalid(format!(
                            "source collider class {} requires its own verified conversion",
                            component.class_id
                        )));
                    }
                };
                builders.push(builder);
                report.source_colliders.push(component.source.clone());
            }
        }
        report.solid_colliders = builders.len();
        let removed = self.source_prefabs.remove(instance).unwrap_or_default();
        for &handle in &removed {
            self.colliders.remove(
                handle,
                &mut self.islands,
                &mut self.bodies,
                &mut self.soft_bodies,
                false,
            );
        }
        let handles = builders
            .into_iter()
            .map(|builder| self.colliders.insert(builder))
            .collect::<Vec<_>>();
        self.source_prefabs
            .insert(instance.to_owned(), handles.clone());
        self.refresh(&handles, &removed);
        Ok(report)
    }

    pub fn remove_source_prefab(&mut self, instance: &str) {
        if let Some(handles) = self.source_prefabs.remove(instance) {
            for &handle in &handles {
                self.colliders.remove(
                    handle,
                    &mut self.islands,
                    &mut self.bodies,
                    &mut self.soft_bodies,
                    false,
                );
            }
            self.refresh(&[], &handles);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midnight_map_data::{
        FileIdentity, PrefabCollisionMesh, PrefabManifest, PrefabNode, SourceApk, SourceComponent,
    };
    use serde_json::json;

    const IDENTITY: [f32; 16] = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
    fn source(id: i64) -> SourceObject {
        SourceObject {
            bundle: "source.ab".into(),
            cab: "CAB-source".into(),
            path_id: id.to_string(),
        }
    }
    fn package() -> PrefabPackage {
        let fields = json!({"m_Enabled":true, "m_IsTrigger":false, "m_Center":{"x":0.,"y":0.,"z":1.}, "m_Size":{"x":2.,"y":2.,"z":2.}});
        let component = SourceComponent {
            source: source(3),
            class_id: 65,
            serialized: FileIdentity {
                path: "source.bin".into(),
                sha256: "a".repeat(64),
            },
            fields,
        };
        let mut local = IDENTITY;
        local[0] = 2.;
        local[5] = 3.;
        local[10] = 4.;
        local[12] = 1000.;
        let node = PrefabNode {
            source: source(1),
            name: "source box".into(),
            active_self: true,
            layer: 0,
            transform: source(2),
            parent: None,
            children: vec![],
            bevy_local_transform: local,
            components: vec![component],
        };
        PrefabPackage {
            root: "/unused".into(),
            manifest_sha256: "a".repeat(64),
            records: vec![],
            poses: vec![],
            manifest: PrefabManifest {
                schema_version: 1,
                source_apk: SourceApk {
                    path: "source.apk".into(),
                    sha256: "a".repeat(64),
                    byte_count: 1,
                },
                bundle: "source.ab".into(),
                coordinate_frame: "bevy_right_handed_y_up".into(),
                geometry: FileIdentity {
                    path: "mesh.bin".into(),
                    sha256: "a".repeat(64),
                },
                nodes: vec![node],
                roots: vec![source(1)],
                renderers: vec![],
                collision_meshes: vec![],
                class_counts: BTreeMap::new(),
                retained_files: BTreeMap::new(),
                limitations: vec![],
            },
        }
    }

    #[test]
    fn box_collision_uses_placement_scale_and_reflects_raw_center_once() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        let package = package();
        let mut placement = IDENTITY;
        placement[0] = 2.;
        placement[5] = 3.;
        placement[10] = 4.;
        placement[12] = 10.;
        let report = world
            .replace_source_prefab("source-span-1", &package, &source(1), placement)
            .unwrap();
        assert_eq!(report.solid_colliders, 1);
        assert_eq!(report.source_colliders, [source(3)]);
        let bounds = world.colliders[world.source_prefabs["source-span-1"][0]].compute_aabb();
        assert!((bounds.center() - Vec3::new(10., 0., -4.)).length() < 1e-5);
        assert!((bounds.half_extents() - Vec3::new(2., 3., 4.)).length() < 1e-5);
        assert!(!world.free_at([10., 0., -4.], None));
        assert!(world.free_at([1000., 0., 0.], None));
    }

    #[test]
    fn skips_inactive_disabled_and_trigger_collision() {
        for case in 0..3 {
            let mut world = CollisionWorld::new(MapId::ImportedIsland);
            let mut package = package();
            match case {
                0 => package.manifest.nodes[0].active_self = false,
                1 => package.manifest.nodes[0].components[0].fields["m_Enabled"] = json!(false),
                _ => package.manifest.nodes[0].components[0].fields["m_IsTrigger"] = json!(true),
            }
            let report = world
                .replace_source_prefab("flags", &package, &source(1), IDENTITY)
                .unwrap();
            assert_eq!(report.solid_colliders, 0);
            assert_eq!(report.inactive_or_disabled + report.triggers, 1);
            assert!(world.free_at([0., 0., -1.], None));
        }
    }

    #[test]
    fn rotated_child_box_retains_shape_and_inherits_parent_activity() {
        let mut package = package();
        let mut child = package.manifest.nodes[0].clone();
        child.source = source(10);
        child.transform = source(11);
        child.parent = Some(source(1));
        child.bevy_local_transform = [
            0., 0., -1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 5., 0., 0., 1.,
        ];
        package.manifest.nodes[0].components.clear();
        package.manifest.nodes[0].children.push(source(10));
        package.manifest.nodes.push(child);
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        let report = world
            .replace_source_prefab("child", &package, &source(1), IDENTITY)
            .unwrap();
        assert_eq!(report.solid_colliders, 1);
        let bounds = world.colliders[world.source_prefabs["child"][0]].compute_aabb();
        assert!((bounds.center() - Vec3::new(4., 0., 0.)).length() < 1e-5);
        package.manifest.nodes[0].active_self = false;
        let report = world
            .replace_source_prefab("child", &package, &source(1), IDENTITY)
            .unwrap();
        assert_eq!(report.solid_colliders, 0);
        assert_eq!(report.inactive_or_disabled, 1);
        assert!(world.free_at([4., 0., 0.], None));
    }

    #[test]
    fn failed_replacement_preserves_collision_and_success_refreshes_bvh() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        let mut package = package();
        world
            .replace_source_prefab("stable", &package, &source(1), IDENTITY)
            .unwrap();
        let handles = world.source_prefabs["stable"].clone();
        package.manifest.nodes[0].components[0].fields["m_Size"]["x"] = json!(0.);
        assert!(
            world
                .replace_source_prefab("stable", &package, &source(1), IDENTITY)
                .is_err()
        );
        assert_eq!(world.source_prefabs["stable"], handles);
        assert!(!world.free_at([0., 0., -1.], None));
        package.manifest.nodes[0].components[0].fields["m_Size"]["x"] = json!(2.);
        let mut moved = IDENTITY;
        moved[12] = 50.;
        world
            .replace_source_prefab("stable", &package, &source(1), moved)
            .unwrap();
        assert!(world.free_at([0., 0., -1.], None));
        assert!(!world.free_at([50., 0., -1.], None));
        let moved_handles = world.source_prefabs["stable"].clone();
        let mut unsupported = package.manifest.nodes[0].components[0].clone();
        unsupported.source = source(99);
        unsupported.class_id = 136;
        package.manifest.nodes[0].components.push(unsupported);
        assert!(
            world
                .replace_source_prefab("stable", &package, &source(1), IDENTITY)
                .is_err()
        );
        assert_eq!(world.source_prefabs["stable"], moved_handles);
        assert!(!world.free_at([50., 0., -1.], None));
        world.remove_source_prefab("stable");
        assert!(world.free_at([50., 0., -1.], None));
    }

    #[test]
    fn separate_mesh_collision_applies_converted_vertices_once() {
        let mut world = CollisionWorld::new(MapId::ImportedIsland);
        let mut package = package();
        let component = &mut package.manifest.nodes[0].components[0];
        component.class_id = 64;
        component.fields["m_Convex"] = json!(false);
        component.fields["m_Mesh"] = json!({"state":"resolved","target":source(4)});
        package.manifest.collision_meshes.push(PrefabCollisionMesh {
            source: source(4),
            positions: vec![[-3., 0., -4.], [3., 0., -4.], [0., 0., 2.]],
            indices: vec![0, 2, 1],
        });
        let mut moved = IDENTITY;
        moved[12] = 50.;
        moved[13] = 5.;
        world
            .replace_source_prefab("mesh", &package, &source(1), moved)
            .unwrap();
        let bounds = world.colliders[world.source_prefabs["mesh"][0]].compute_aabb();
        assert_eq!(bounds.mins, Vec3::new(47., 5., -4.));
        assert_eq!(bounds.maxs, Vec3::new(53., 5., 2.));
        let handles = world.source_prefabs["mesh"].clone();
        package.manifest.collision_meshes[0].indices[0] = 99;
        assert!(
            world
                .replace_source_prefab("mesh", &package, &source(1), moved)
                .is_err()
        );
        assert_eq!(world.source_prefabs["mesh"], handles);
    }

    #[test]
    #[ignore = "requires retained supermarket source prefab package"]
    fn retained_supermarket_builds_identical_shared_collision() {
        let package=midnight_map_data::load_prefab_package(&crate::level::asset_root().join("imported_prefabs/2e480f42335ef1eb73fa54743f7511757211f3bc62af7114820f6410387ba3df/building_chaoshi_01-3be4bce3aeca")).unwrap();
        let root = &package.manifest.roots[0];
        let mut server = CollisionWorld::new(MapId::ImportedIsland);
        let mut client = CollisionWorld::new(MapId::ImportedIsland);
        let a = server
            .replace_source_prefab("verified-preview", &package, root, IDENTITY)
            .unwrap();
        let b = client
            .replace_source_prefab("verified-preview", &package, root, IDENTITY)
            .unwrap();
        assert!(a.solid_colliders > 0);
        assert_eq!(a.source_colliders, b.source_colliders);
        for (a, b) in server.source_prefabs["verified-preview"]
            .iter()
            .zip(&client.source_prefabs["verified-preview"])
        {
            assert_eq!(
                server.colliders[*a].compute_aabb(),
                client.colliders[*b].compute_aabb()
            );
        }
        for position in [[0., 1., 0.], [5., 1., 5.], [-5., 1., -5.], [0., 10., 0.]] {
            assert_eq!(
                server.free_at(position, None),
                client.free_at(position, None)
            );
        }
        eprintln!(
            "verified supermarket: {} solid, {} inactive/disabled, {} triggers",
            a.solid_colliders, a.inactive_or_disabled, a.triggers
        );
    }
}
