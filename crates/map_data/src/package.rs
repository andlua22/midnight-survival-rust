use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    ArchiveError, MeshRecord, TERRAIN_MAGIC, TERRAIN_MANIFEST_VERSION, TerrainManifest,
    TerrainParameters, invalid, package_path, read_mesh_archive,
};

#[derive(Debug)]
pub struct TerrainPackage {
    pub root: PathBuf,
    pub manifest: TerrainManifest,
    pub records: Vec<MeshRecord>,
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
    if manifest.schema_version != TERRAIN_MANIFEST_VERSION
        || manifest.coordinate_frame != "bevy_right_handed_y_up"
        || manifest.uv_origin != "top_left"
    {
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
                for identity in [&texture.file, &texture.encoded] {
                    if let Some(previous) = textures.insert(&identity.path, &identity.sha256) {
                        if previous != &identity.sha256 {
                            return Err(invalid("conflicting texture file identities"));
                        }
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
    Ok(TerrainPackage {
        root: package.to_path_buf(),
        manifest,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FileIdentity, SceneMesh, SourceApk, SourceCollider, SourceMaterial, SourceObject,
        TextureAsset, TextureBinding, TextureSampler, write_mesh_archive,
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
                schema_version: TERRAIN_MANIFEST_VERSION,
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
            let scene = self.manifest.meshes.values_mut().next().unwrap();
            scene.material.textures.insert(
                "_MainTex".into(),
                TextureBinding {
                    texture: Some(TextureAsset {
                        file: FileIdentity {
                            path: "textures/layer.png".into(),
                            sha256: file_sha256(&self.root.join("textures/layer.png")).unwrap(),
                        },
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
    fn checks_both_decoded_texture_and_complete_encoded_mip_payload() {
        let mut fixture = Fixture::new();
        fixture.add_texture();
        load_terrain_package(&fixture.root).unwrap();
        for (path, bytes) in [
            ("textures/layer.png", b"decoded fixture bytes".as_slice()),
            (
                "textures/layer.texture.bin",
                b"encoded fixture bytes and mips".as_slice(),
            ),
        ] {
            std::fs::write(fixture.root.join(path), b"changed").unwrap();
            let error = load_terrain_package(&fixture.root).unwrap_err().to_string();
            assert!(error.contains(path), "{error}");
            std::fs::write(fixture.root.join(path), bytes).unwrap();
        }
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
