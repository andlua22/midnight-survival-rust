use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    ArchiveError, MeshRecord, TERRAIN_MAGIC, TerrainCollisionMesh, TerrainManifest,
    TerrainParameters, invalid, package_path, read_collision_archive, read_mesh_archive,
    read_rgba_mip_archive,
};

#[derive(Debug)]
pub struct TerrainPackage {
    pub root: PathBuf,
    pub manifest: TerrainManifest,
    pub records: Vec<MeshRecord>,
    /// Independent Bevy-local collision-only buffers, absent in legacy packages.
    pub collision_meshes: Vec<TerrainCollisionMesh>,
}

impl TerrainPackage {
    pub fn collision_records(&self) -> impl Iterator<Item = &MeshRecord> {
        self.records.iter().filter(|record| {
            self.manifest
                .meshes
                .get(&record.name)
                .and_then(|mesh| mesh.collider.as_ref())
                .is_some_and(|collider| collider.enabled && !collider.is_trigger)
        })
    }
}

pub fn mesh_bounds(record: &MeshRecord) -> [f32; 6] {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for position in &record.positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(position[axis]);
            max[axis] = max[axis].max(position[axis]);
        }
    }
    [min[0], min[1], min[2], max[0], max[1], max[2]]
}

pub fn file_sha256(path: &Path) -> Result<String, ArchiveError> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 256 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

/// The importer, client and collision loader use the same validation contract.
/// A broken canonical package fails instead of silently activating older data.
pub fn load_terrain_package(package: &Path) -> Result<TerrainPackage, ArchiveError> {
    let manifest: TerrainManifest =
        serde_json::from_reader(File::open(package.join("manifest.json"))?)
            .map_err(|error| invalid(format!("terrain manifest: {error}")))?;
    manifest.validate_collision_metadata()?;
    if manifest.coordinate_frame != "bevy_right_handed_y_up" || manifest.uv_origin != "top_left" {
        return Err(invalid("unsupported terrain manifest conventions/version"));
    }
    if manifest.source_apk.sha256.len() != 64
        || !manifest
            .source_apk
            .sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(invalid("invalid source APK identity"));
    }
    let geometry_path = package_path(package, &manifest.geometry.path)?;
    if file_sha256(&geometry_path)? != manifest.geometry.sha256 {
        return Err(invalid("terrain geometry checksum differs from manifest"));
    }
    let records = read_mesh_archive(&std::fs::read(geometry_path)?, TERRAIN_MAGIC)?;
    if records.is_empty() || records.len() != manifest.meshes.len() {
        return Err(invalid(
            "geometry and manifest record counts differ or are empty",
        ));
    }
    let mut names = BTreeSet::new();
    let mut textures = BTreeMap::new();
    let mut mip_metadata = BTreeMap::new();
    for record in &records {
        if !names.insert(&record.name) {
            return Err(invalid(format!("duplicate scene mesh {}", record.name)));
        }
        let source = manifest
            .meshes
            .get(&record.name)
            .ok_or_else(|| invalid(format!("{}: no source manifest", record.name)))?;
        if mesh_bounds(record) != source.bounds
            || source
                .world_transform
                .iter()
                .any(|value| !value.is_finite())
        {
            return Err(invalid(format!(
                "{}: manifest bounds/transform differ from geometry",
                record.name
            )));
        }
        if let Some(collider) = &source.collider {
            if collider.mesh != source.source || collider.convex {
                return Err(invalid(format!(
                    "{}: separate/convex collision mesh requires its own import",
                    record.name
                )));
            }
        }
        if source.material.terrain_layer_count().is_some() {
            TerrainParameters::from_material(&source.material)?;
            // A saved keyword does not establish the runtime shader LOD or
            // supply a missing vertex channel. Preserve source absence; the
            // renderer reports its geometric-normal fallback for these meshes.
            for name in source.material.terrain_sampler_names() {
                if source
                    .material
                    .textures
                    .get(&name)
                    .and_then(|binding| binding.texture.as_ref())
                    .is_none()
                    && source.material.default_texture_color(&name).is_none()
                {
                    return Err(invalid(format!(
                        "{}: required sampler {name} is absent",
                        record.name
                    )));
                }
            }
        }
        for binding in source.material.textures.values() {
            if let Some(texture) = &binding.texture {
                let gpu = texture.gpu.as_ref().ok_or_else(|| {
                    invalid(format!(
                        "{}: decoded source mip chain is missing",
                        texture.name
                    ))
                })?;
                for identity in [&texture.file, &texture.encoded, gpu] {
                    if let Some(previous) = textures.insert(&identity.path, &identity.sha256) {
                        if previous != &identity.sha256 {
                            return Err(invalid("conflicting texture file identities"));
                        }
                    }
                }
                let metadata = (texture.width, texture.height, texture.mip_count);
                if let Some(previous) = mip_metadata.insert(&gpu.path, metadata) {
                    if previous != metadata {
                        return Err(invalid("conflicting decoded mip metadata"));
                    }
                }
            }
        }
    }
    for (path, expected_hash) in textures {
        if file_sha256(&package_path(package, path)?)? != *expected_hash {
            return Err(invalid(format!("texture checksum differs: {path}")));
        }
    }
    for (path, metadata) in mip_metadata {
        let bytes = std::fs::read(package_path(package, path)?)?;
        let chain = read_rgba_mip_archive(&bytes)?;
        if (chain.width, chain.height, chain.mip_count) != metadata {
            return Err(invalid(format!(
                "{path}: decoded source mip metadata differs from texture"
            )));
        }
    }
    let collision_meshes = if let Some(collision) = &manifest.collision {
        let path = package_path(package, &collision.geometry.path)?;
        if file_sha256(&path)? != collision.geometry.sha256 {
            return Err(invalid(
                "terrain collision geometry checksum differs from manifest",
            ));
        }
        read_collision_archive(&std::fs::read(path)?)?
    } else {
        Vec::new()
    };
    let package = TerrainPackage {
        root: package.to_path_buf(),
        manifest,
        records,
        collision_meshes,
    };
    package.validate_collision_geometry()?;
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FileIdentity, SceneMesh, SourceApk, SourceCollider, SourceMaterial, SourceObject,
        TERRAIN_MANIFEST_VERSION, TerrainCollisionEntry, TerrainCollisionManifest,
        TerrainCollisionShape, TextureAsset, TextureBinding, TextureSampler,
        write_collision_archive, write_mesh_archive,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture {
        root: PathBuf,
        manifest: TerrainManifest,
        records: Vec<MeshRecord>,
    }

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "midnight-map-package-{}-{stamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&root).unwrap();
            let record = MeshRecord {
                name: "source surface".into(),
                texture: String::new(),
                center: [2.0, 4.0, -3.0],
                positions: vec![[0.0, 4.0, 0.0], [4.0, 4.0, 0.0], [0.0, 4.0, -6.0]],
                normals: vec![[0.0, 1.0, 0.0]; 3],
                tangents: Vec::new(),
                uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                indices: vec![0, 1, 2],
            };
            let source = SourceObject {
                bundle: "assets/scene/a-01.ab".into(),
                cab: "CAB-source".into(),
                path_id: "-9223372036854775807".into(),
            };
            let material = SourceMaterial {
                source: source.clone(),
                name: "surface".into(),
                shader: source.clone(),
                shader_name: "_URP/LambertLit".into(),
                textures: BTreeMap::new(),
                floats: BTreeMap::new(),
                colors: BTreeMap::new(),
                shader_defaults: BTreeMap::new(),
                texture_defaults: BTreeMap::new(),
                valid_keywords: Vec::new(),
                render_queue: -1,
            };
            let scene = SceneMesh {
                source: source.clone(),
                game_object: source.clone(),
                layer: Some(0),
                world_transform: [
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ],
                bounds: mesh_bounds(&record),
                material,
                collider: Some(SourceCollider {
                    source: source.clone(),
                    mesh: source,
                    enabled: true,
                    is_trigger: false,
                    convex: false,
                    cooking_options: 30,
                }),
            };
            let manifest = TerrainManifest {
                schema_version: 3,
                source_apk: SourceApk {
                    path: "fixture.apk".into(),
                    sha256: "a".repeat(64),
                    byte_count: 1,
                },
                coordinate_frame: "bevy_right_handed_y_up".into(),
                uv_origin: "top_left".into(),
                geometry: FileIdentity {
                    path: "meshes.bin".into(),
                    sha256: String::new(),
                },
                meshes: BTreeMap::from([(record.name.clone(), scene)]),
                collision: None,
                limitations: vec!["Test fixture".into()],
            };
            let mut fixture = Self {
                root,
                manifest,
                records: vec![record],
            };
            fixture.save();
            fixture
        }

        fn save(&mut self) {
            write_mesh_archive(
                &mut File::create(self.root.join("meshes.bin")).unwrap(),
                TERRAIN_MAGIC,
                &self.records,
            )
            .unwrap();
            self.manifest.geometry.sha256 = file_sha256(&self.root.join("meshes.bin")).unwrap();
            self.save_manifest();
        }

        fn save_manifest(&self) {
            serde_json::to_writer(
                File::create(self.root.join("manifest.json")).unwrap(),
                &self.manifest,
            )
            .unwrap();
        }

        fn add_collision_table(&mut self) {
            let scene = &self.manifest.meshes["source surface"];
            let retained = scene.collider.as_ref().unwrap();
            let collision_mesh = TerrainCollisionMesh {
                name: "source local collider".into(),
                positions: vec![[1., 2., -3.], [4., 2., -3.], [1., 2., -7.]],
                indices: vec![0, 1, 2],
            };
            write_collision_archive(
                &mut File::create(self.root.join("collision.bin")).unwrap(),
                std::slice::from_ref(&collision_mesh),
            )
            .unwrap();
            let entry = TerrainCollisionEntry {
                source: retained.source.clone(),
                game_object: scene.game_object.clone(),
                transform: SourceObject {
                    path_id: "-19".into(),
                    ..scene.game_object.clone()
                },
                layer: scene.layer.unwrap(),
                active_self: true,
                active_in_hierarchy: true,
                enabled: retained.enabled,
                is_trigger: retained.is_trigger,
                world_transform: scene.world_transform,
                shape: TerrainCollisionShape::Mesh {
                    mesh: retained.mesh.clone(),
                    record: collision_mesh.name,
                    convex: retained.convex,
                    cooking_options: retained.cooking_options,
                },
            };
            self.manifest.schema_version = TERRAIN_MANIFEST_VERSION;
            self.manifest.collision = Some(TerrainCollisionManifest {
                geometry: FileIdentity {
                    path: "collision.bin".into(),
                    sha256: file_sha256(&self.root.join("collision.bin")).unwrap(),
                },
                colliders: vec![entry],
            });
            self.save_manifest();
        }

        fn add_texture(&mut self) {
            std::fs::create_dir_all(self.root.join("textures")).unwrap();
            std::fs::write(
                self.root.join("textures/layer.png"),
                b"decoded fixture bytes",
            )
            .unwrap();
            std::fs::write(
                self.root.join("textures/layer.texture.bin"),
                b"encoded fixture bytes and mips",
            )
            .unwrap();
            crate::write_rgba_mip_archive(
                &mut File::create(self.root.join("textures/layer.msrgba")).unwrap(),
                1,
                1,
                1,
                &[0x12, 0x34, 0x56, 0x78],
            )
            .unwrap();
            let scene = self.manifest.meshes.values_mut().next().unwrap();
            scene.material.textures.insert(
                "_MainTex".into(),
                TextureBinding {
                    texture: Some(TextureAsset {
                        file: FileIdentity {
                            path: "textures/layer.png".into(),
                            sha256: file_sha256(&self.root.join("textures/layer.png")).unwrap(),
                        },
                        gpu: Some(FileIdentity {
                            path: "textures/layer.msrgba".into(),
                            sha256: file_sha256(&self.root.join("textures/layer.msrgba")).unwrap(),
                        }),
                        encoded: FileIdentity {
                            path: "textures/layer.texture.bin".into(),
                            sha256: file_sha256(&self.root.join("textures/layer.texture.bin"))
                                .unwrap(),
                        },
                        source: scene.source.clone(),
                        name: "layer".into(),
                        width: 1,
                        height: 1,
                        unity_format: 4,
                        unity_color_space: 1,
                        unity_version: "2021.3.23f1c1".into(),
                        mip_count: 1,
                        sampler: TextureSampler {
                            filter_mode: 1,
                            wrap: [0; 3],
                            aniso: 1,
                            mip_bias: 0.0,
                        },
                    }),
                    scale: [1.0; 2],
                    offset: [0.0; 2],
                },
            );
            self.save_manifest();
        }

        fn terrain_defaults(&mut self) {
            let material = &mut self.manifest.meshes.values_mut().next().unwrap().material;
            material.shader_name = "_URP/TerrainMeshBase/3 Textures".into();
            for layer in 0..3 {
                material
                    .shader_defaults
                    .insert(format!("_Splat{layer}_S"), [60.0, 0.0, 0.0, 0.0]);
                material
                    .shader_defaults
                    .insert(format!("_Splat{layer}_Smoothness"), [0.5, 0.0, 0.0, 0.0]);
                material
                    .texture_defaults
                    .insert(format!("_Splat{layer}"), "white".into());
            }
            material
                .texture_defaults
                .insert("_Control".into(), "red".into());
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn loaded_geometry_keeps_its_world_coordinates_and_qualified_source_ids() {
        let fixture = Fixture::new();
        let package = load_terrain_package(&fixture.root).unwrap();
        assert_eq!(package.records, fixture.records);
        assert_eq!(
            package.manifest.meshes["source surface"].source.path_id,
            "-9223372036854775807"
        );
        assert_eq!(package.collision_records().count(), 1);
    }

    #[test]
    fn legacy_unknown_layers_and_schema_three_zero_preserve_mesh_archive_bytes() {
        let mut fixture = Fixture::new();
        let geometry = std::fs::read(fixture.root.join("meshes.bin")).unwrap();
        let geometry_hash = fixture.manifest.geometry.sha256.clone();
        assert_eq!(&geometry[..8], b"MSMAP003");
        for (version, layer) in [(2, None), (3, Some(0)), (3, Some(31))] {
            fixture.manifest.schema_version = version;
            fixture.manifest.meshes.values_mut().next().unwrap().layer = layer;
            fixture.save_manifest();
            let package = load_terrain_package(&fixture.root).unwrap();
            assert_eq!(package.manifest.meshes["source surface"].layer, layer);
            assert_eq!(package.records, fixture.records);
            assert_eq!(package.manifest.geometry.sha256, geometry_hash);
            assert_eq!(
                std::fs::read(fixture.root.join("meshes.bin")).unwrap(),
                geometry
            );
            if version == 2 {
                let serialized: serde_json::Value = serde_json::from_reader(
                    File::open(fixture.root.join("manifest.json")).unwrap(),
                )
                .unwrap();
                assert!(
                    serialized["meshes"]["source surface"]
                        .get("layer")
                        .is_none()
                );
            }
        }
    }

    #[test]
    fn independent_collider_table_preserves_render_geometry_and_collision_only_buffers() {
        let mut fixture = Fixture::new();
        let original_archive = std::fs::read(fixture.root.join("meshes.bin")).unwrap();
        fixture.add_collision_table();
        let table = fixture.manifest.collision.as_mut().unwrap();
        let mut box_entry = table.colliders[0].clone();
        box_entry.source.path_id = "-34".into();
        box_entry.game_object.path_id = "-20".into();
        box_entry.transform.path_id = "-21".into();
        box_entry.shape = TerrainCollisionShape::Box {
            center: [1., 2., 3.],
            size: [4., 5., 6.],
        };
        table.colliders.push(box_entry);
        fixture.save_manifest();
        let package = load_terrain_package(&fixture.root).unwrap();
        assert_eq!(package.records, fixture.records);
        assert_eq!(package.manifest.meshes.len(), 1);
        assert_eq!(
            package.manifest.collision.as_ref().unwrap().colliders.len(),
            2
        );
        assert_eq!(package.collision_meshes.len(), 1);
        assert_eq!(
            package
                .collision_mesh("source local collider")
                .unwrap()
                .positions,
            [[1., 2., -3.], [4., 2., -3.], [1., 2., -7.]]
        );
        assert_eq!(
            std::fs::read(fixture.root.join("meshes.bin")).unwrap(),
            original_archive
        );
        // Collision-only buffers never appear in the renderer's archive/record list.
        assert!(
            !package
                .records
                .iter()
                .any(|record| record.name == "source local collider")
        );
    }

    #[test]
    fn independent_collision_schema_and_metadata_reject_invalid_admission() {
        let mut fixture = Fixture::new();
        fixture.manifest.schema_version = TERRAIN_MANIFEST_VERSION;
        assert!(fixture.manifest.validate_collision_metadata().is_err());
        fixture.add_collision_table();
        let admitted = fixture.manifest.clone();
        for corruption in 0..10 {
            let mut malformed = admitted.clone();
            let table = malformed.collision.as_mut().unwrap();
            match corruption {
                0 => table.colliders.push(table.colliders[0].clone()),
                1 => table.colliders[0].layer = 32,
                2 => table.colliders[0].source.path_id = "01".into(),
                3 => table.colliders[0].world_transform[15] = 0.,
                4 => table.colliders[0].world_transform[0] = 0.,
                5 => table.colliders[0].active_self = false,
                6 => table.geometry.path = "../collision.bin".into(),
                7 => table.geometry.sha256 = "bad".into(),
                8 => table.colliders[0].enabled = false,
                9 => table.colliders[0].game_object.path_id = "-18".into(),
                _ => unreachable!(),
            }
            assert!(
                malformed.validate_collision_metadata().is_err(),
                "corruption {corruption}"
            );
        }
        for version in [2, 3] {
            let mut legacy = admitted.clone();
            legacy.schema_version = version;
            if version == 2 {
                for mesh in legacy.meshes.values_mut() {
                    mesh.layer = None;
                }
            }
            assert!(
                legacy
                    .validate_collision_metadata()
                    .unwrap_err()
                    .to_string()
                    .contains("legacy terrain schema")
            );
        }
        let mut malformed = admitted.clone();
        let table = malformed.collision.as_mut().unwrap();
        let mut box_entry = table.colliders[0].clone();
        box_entry.source.path_id = "-34".into();
        box_entry.game_object.path_id = "-20".into();
        box_entry.shape = TerrainCollisionShape::Box {
            center: [0.; 3],
            size: [1., 0., 1.],
        };
        table.colliders.push(box_entry);
        assert!(malformed.validate_collision_metadata().is_err());
    }

    #[test]
    fn independent_collision_payload_checksum_and_reference_coverage_are_enforced() {
        let mut fixture = Fixture::new();
        fixture.add_collision_table();
        let path = fixture.root.join("collision.bin");
        let original = std::fs::read(&path).unwrap();
        let mut changed = original.clone();
        let last = changed.len() - 4;
        changed[last..].copy_from_slice(&1u32.to_le_bytes());
        std::fs::write(&path, &changed).unwrap();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("collision geometry checksum")
        );
        std::fs::write(&path, &original).unwrap();
        let mut meshes = read_collision_archive(&original).unwrap();
        let mut orphan = meshes[0].clone();
        orphan.name = "unreferenced collider mesh".into();
        meshes.push(orphan);
        write_collision_archive(&mut File::create(&path).unwrap(), &meshes).unwrap();
        fixture.manifest.collision.as_mut().unwrap().geometry.sha256 = file_sha256(&path).unwrap();
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("mesh records and references differ")
        );
        let mut package = TerrainPackage {
            root: fixture.root.clone(),
            manifest: fixture.manifest.clone(),
            records: fixture.records.clone(),
            collision_meshes: vec![],
        };
        assert!(package.validate_collision_geometry().is_err());
        package.manifest.schema_version = 3;
        package.manifest.collision = None;
        package.collision_meshes = meshes;
        assert!(
            package
                .validate_collision_geometry()
                .unwrap_err()
                .to_string()
                .contains("legacy terrain")
        );
    }

    #[test]
    fn schema_layers_reject_legacy_claims_missing_metadata_and_invalid_ranges() {
        let mut fixture = Fixture::new();
        for (version, layer, expected_error) in [
            (2, Some(0), "must remain unknown"),
            (3, None, "layer is missing"),
            (3, Some(32), "layer must be in 0..31"),
            (3, Some(255), "layer must be in 0..31"),
            (1, None, "unsupported terrain manifest"),
            (5, Some(0), "unsupported terrain manifest"),
        ] {
            fixture.manifest.schema_version = version;
            fixture.manifest.meshes.values_mut().next().unwrap().layer = layer;
            assert!(
                fixture
                    .manifest
                    .validate_layers()
                    .unwrap_err()
                    .to_string()
                    .contains(expected_error)
            );
            fixture.save_manifest();
            assert!(
                load_terrain_package(&fixture.root)
                    .unwrap_err()
                    .to_string()
                    .contains(expected_error)
            );
        }
    }

    #[test]
    fn layer_json_rejects_negative_overflow_fractional_and_non_numeric_values() {
        let fixture = Fixture::new();
        for invalid_layer in [
            serde_json::json!(-1),
            serde_json::json!(256),
            serde_json::json!(0.5),
            serde_json::json!("0"),
            serde_json::json!(true),
        ] {
            let mut manifest = serde_json::to_value(&fixture.manifest).unwrap();
            manifest["meshes"]["source surface"]["layer"] = invalid_layer;
            serde_json::to_writer(
                File::create(fixture.root.join("manifest.json")).unwrap(),
                &manifest,
            )
            .unwrap();
            assert!(
                load_terrain_package(&fixture.root)
                    .unwrap_err()
                    .to_string()
                    .contains("terrain manifest:")
            );
        }
    }

    #[test]
    fn submesh_layers_agree_for_qualified_game_objects_and_collider_components() {
        let mut fixture = Fixture::new();
        let mut record = fixture.records[0].clone();
        record.name = "second submesh".into();
        let scene = fixture.manifest.meshes["source surface"].clone();
        fixture.manifest.meshes.insert(record.name.clone(), scene);
        fixture.records.push(record);
        fixture.save();
        load_terrain_package(&fixture.root).unwrap();
        fixture
            .manifest
            .meshes
            .get_mut("second submesh")
            .unwrap()
            .layer = Some(19);
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("conflicting terrain layers for GameObject")
        );
        // Qualify the GameObject separately while retaining the same collider.
        fixture
            .manifest
            .meshes
            .get_mut("second submesh")
            .unwrap()
            .game_object
            .cab = "CAB-other-scene".into();
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("conflicting terrain layers for collider")
        );
        fixture
            .manifest
            .meshes
            .get_mut("second submesh")
            .unwrap()
            .collider
            .as_mut()
            .unwrap()
            .source
            .cab = "CAB-other-scene".into();
        fixture.save_manifest();
        let package = load_terrain_package(&fixture.root).unwrap();
        assert_eq!(package.manifest.meshes["source surface"].layer, Some(0));
        assert_eq!(package.manifest.meshes["second submesh"].layer, Some(19));
        fixture
            .manifest
            .meshes
            .get_mut("second submesh")
            .unwrap()
            .layer = None;
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("layer is missing")
        );
    }

    #[test]
    fn refuses_changed_geometry_even_when_the_archive_still_parses() {
        let fixture = Fixture::new();
        let mut changed = fixture.records.clone();
        changed[0].positions[0][0] = 1.0;
        write_mesh_archive(
            &mut File::create(fixture.root.join("meshes.bin")).unwrap(),
            TERRAIN_MAGIC,
            &changed,
        )
        .unwrap();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("geometry checksum")
        );
    }

    #[test]
    fn checks_decoded_preview_source_mips_and_complete_encoded_payload() {
        let mut fixture = Fixture::new();
        fixture.add_texture();
        load_terrain_package(&fixture.root).unwrap();
        let gpu_bytes = std::fs::read(fixture.root.join("textures/layer.msrgba")).unwrap();
        for (path, bytes) in [
            ("textures/layer.png", b"decoded fixture bytes".as_slice()),
            (
                "textures/layer.texture.bin",
                b"encoded fixture bytes and mips".as_slice(),
            ),
            ("textures/layer.msrgba", gpu_bytes.as_slice()),
        ] {
            std::fs::write(fixture.root.join(path), b"changed").unwrap();
            let error = load_terrain_package(&fixture.root).unwrap_err().to_string();
            assert!(error.contains(path), "{error}");
            std::fs::write(fixture.root.join(path), bytes).unwrap();
        }
    }

    #[test]
    fn source_mip_archive_must_match_declared_texture_dimensions_and_count() {
        let mut fixture = Fixture::new();
        fixture.add_texture();
        fixture
            .manifest
            .meshes
            .values_mut()
            .next()
            .unwrap()
            .material
            .textures
            .get_mut("_MainTex")
            .unwrap()
            .texture
            .as_mut()
            .unwrap()
            .width = 2;
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("mip metadata differs")
        );
        let texture = fixture
            .manifest
            .meshes
            .values_mut()
            .next()
            .unwrap()
            .material
            .textures
            .get_mut("_MainTex")
            .unwrap()
            .texture
            .as_mut()
            .unwrap();
        texture.width = 1;
        texture.gpu = None;
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("source mip chain is missing")
        );
    }

    #[test]
    fn refuses_missing_records_wrong_bounds_and_unsupported_frame() {
        let mut fixture = Fixture::new();
        let original = fixture.manifest.clone();
        fixture.manifest.meshes.clear();
        fixture.save_manifest();
        assert!(load_terrain_package(&fixture.root).is_err());
        fixture.manifest = original.clone();
        fixture.manifest.meshes.values_mut().next().unwrap().bounds[1] += 1.0;
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("bounds/transform")
        );
        fixture.manifest = original;
        fixture.manifest.coordinate_frame = "unity_left_handed".into();
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("conventions/version")
        );
    }

    #[test]
    fn collisions_follow_source_component_flags_including_roads() {
        let mut fixture = Fixture::new();
        for (name, enabled, is_trigger) in [
            ("road_enabled", true, false),
            ("terrain_disabled", false, false),
            ("terrain_trigger", true, true),
        ] {
            let mut record = fixture.records[0].clone();
            record.name = name.into();
            let mut source = fixture.manifest.meshes["source surface"].clone();
            let collider = source.collider.as_mut().unwrap();
            collider.enabled = enabled;
            collider.is_trigger = is_trigger;
            fixture.manifest.meshes.insert(name.into(), source);
            fixture.records.push(record);
        }
        fixture.save();
        let package = load_terrain_package(&fixture.root).unwrap();
        let names = package
            .collision_records()
            .map(|record| record.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["source surface", "road_enabled"]);
    }

    #[test]
    fn null_terrain_samplers_require_a_supported_shader_default() {
        let mut fixture = Fixture::new();
        fixture.terrain_defaults();
        fixture.save_manifest();
        load_terrain_package(&fixture.root).unwrap();
        fixture
            .manifest
            .meshes
            .values_mut()
            .next()
            .unwrap()
            .material
            .texture_defaults
            .remove("_Splat1");
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("required sampler _Splat1")
        );
    }

    #[test]
    fn normal_map_keywords_preserve_absent_tangents_and_still_validate_samplers() {
        let mut fixture = Fixture::new();
        fixture.terrain_defaults();
        let material = &mut fixture
            .manifest
            .meshes
            .values_mut()
            .next()
            .unwrap()
            .material;
        material.valid_keywords.push("_NORMALMAP".into());
        for layer in 0..3 {
            material
                .texture_defaults
                .insert(format!("_Splat{layer}_Normal"), "bump".into());
        }
        fixture.save_manifest();
        let package = load_terrain_package(&fixture.root).unwrap();
        assert!(package.records[0].tangents.is_empty());
        assert!(
            package.manifest.meshes["source surface"]
                .material
                .uses_terrain_normal_maps()
        );

        fixture
            .manifest
            .meshes
            .values_mut()
            .next()
            .unwrap()
            .material
            .texture_defaults
            .remove("_Splat1_Normal");
        fixture.save_manifest();
        assert!(
            load_terrain_package(&fixture.root)
                .unwrap_err()
                .to_string()
                .contains("required sampler _Splat1_Normal")
        );
    }
}
