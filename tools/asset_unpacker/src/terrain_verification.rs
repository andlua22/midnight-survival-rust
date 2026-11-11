//! Source coverage is counted by qualified Unity instance identity, not submesh count.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use midnight_map_data::{
    FileIdentity, SceneMesh, SourceApk, SourceObject, TerrainPackage, load_terrain_package,
};
use serde::Serialize;

use crate::{
    fields::invalid,
    pass2_terrain::is_chunk,
    unity::{Apk, Error, source_apk},
};

#[derive(Debug, Default, Serialize)]
pub struct ShaderCoverage {
    pub render_records: usize,
    pub source_instances: usize,
}

#[derive(Debug, Serialize)]
pub struct TerrainVerification {
    pub source_apk: SourceApk,
    pub geometry: FileIdentity,
    pub coordinate_frame: String,
    pub uv_origin: String,
    pub bounds: [f32; 6],
    pub terrain_scene_bundles: usize,
    pub terrain_instances_by_scene: BTreeMap<String, usize>,
    pub source_instances: usize,
    pub terrain_source_instances: usize,
    pub other_source_instances: usize,
    pub render_records: usize,
    pub terrain_render_records: usize,
    pub normal_map_records: usize,
    pub records_with_source_tangents: usize,
    pub normal_map_records_without_source_tangents: Vec<String>,
    pub other_render_records: usize,
    pub vertices: usize,
    pub triangles: usize,
    pub enabled_collision_records: usize,
    pub enabled_source_colliders: usize,
    pub materials: usize,
    pub textures: usize,
    pub texture_formats: BTreeMap<i32, usize>,
    pub texture_color_spaces: BTreeMap<i32, usize>,
    pub texture_mip_counts: BTreeMap<u32, usize>,
    pub null_required_samplers: BTreeMap<String, usize>,
    pub shader_variants: BTreeMap<String, ShaderCoverage>,
    pub limitations: Vec<String>,
}

type Instance<'a> = (&'a SourceObject, &'a SourceObject);

fn instances_by_scene<'a>(
    meshes: impl Iterator<Item = &'a SceneMesh>,
) -> BTreeMap<&'a str, BTreeSet<Instance<'a>>> {
    let mut scenes = BTreeMap::<&str, BTreeSet<Instance<'a>>>::new();
    for mesh in meshes {
        scenes
            .entry(&mesh.game_object.bundle)
            .or_default()
            .insert((&mesh.game_object, &mesh.source));
    }
    scenes
}

pub fn verify_loaded(
    package: &TerrainPackage,
    source: &SourceApk,
    members: &BTreeSet<String>,
) -> Result<TerrainVerification, Error> {
    if package.manifest.source_apk.sha256 != source.sha256
        || package.manifest.source_apk.byte_count != source.byte_count
    {
        return Err(invalid(
            "candidate terrain package comes from a different APK",
        ));
    }
    if members.len() != 242 {
        return Err(invalid(format!(
            "expected 242 terrain bundles in the source APK, found {}",
            members.len()
        )));
    }
    let manifest = &package.manifest;
    let terrain = instances_by_scene(
        manifest
            .meshes
            .values()
            .filter(|mesh| mesh.material.terrain_layer_count().is_some()),
    );
    let actual = terrain
        .keys()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    if &actual != members {
        return Err(invalid(format!(
            "terrain scene coverage differs from the APK: missing {:?}, unexpected {:?}",
            members.difference(&actual).collect::<Vec<_>>(),
            actual.difference(members).collect::<Vec<_>>()
        )));
    }
    let all_instances = instances_by_scene(manifest.meshes.values())
        .values()
        .map(BTreeSet::len)
        .sum();
    let terrain_source_instances = terrain.values().map(BTreeSet::len).sum();
    let terrain_render_records = manifest
        .meshes
        .values()
        .filter(|mesh| mesh.material.terrain_layer_count().is_some())
        .count();
    let materials = manifest
        .meshes
        .values()
        .map(|mesh| (&mesh.material.source, &mesh.material))
        .collect::<BTreeMap<_, _>>();
    let textures = materials
        .values()
        .flat_map(|material| material.textures.values())
        .filter_map(|binding| binding.texture.as_ref())
        .map(|texture| (&texture.source, texture))
        .collect::<BTreeMap<_, _>>();
    let mut texture_formats = BTreeMap::new();
    let mut texture_color_spaces = BTreeMap::new();
    let mut texture_mip_counts = BTreeMap::new();
    for texture in textures.values() {
        *texture_formats.entry(texture.unity_format).or_default() += 1;
        *texture_color_spaces
            .entry(texture.unity_color_space)
            .or_default() += 1;
        *texture_mip_counts.entry(texture.mip_count).or_default() += 1;
    }
    let mut null_required_samplers = BTreeMap::new();
    for material in materials.values() {
        if material.terrain_layer_count().is_some() {
            for sampler in material.terrain_sampler_names() {
                if material
                    .textures
                    .get(&sampler)
                    .and_then(|binding| binding.texture.as_ref())
                    .is_none()
                {
                    *null_required_samplers.entry(sampler).or_default() += 1;
                }
            }
        }
    }
    let mut shader_records = BTreeMap::<&str, Vec<&SceneMesh>>::new();
    for mesh in manifest.meshes.values() {
        shader_records
            .entry(&mesh.material.shader_name)
            .or_default()
            .push(mesh);
    }
    let shader_variants = shader_records
        .into_iter()
        .map(|(shader, meshes)| {
            let instances = instances_by_scene(meshes.iter().copied())
                .values()
                .map(BTreeSet::len)
                .sum();
            (
                shader.to_owned(),
                ShaderCoverage {
                    render_records: meshes.len(),
                    source_instances: instances,
                },
            )
        })
        .collect();
    let source_colliders = manifest
        .meshes
        .values()
        .filter_map(|mesh| mesh.collider.as_ref())
        .filter(|collider| collider.enabled && !collider.is_trigger)
        .map(|collider| &collider.source)
        .collect::<BTreeSet<_>>();
    let mut bounds = [
        f32::INFINITY,
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    ];
    for mesh in manifest.meshes.values() {
        for axis in 0..3 {
            bounds[axis] = bounds[axis].min(mesh.bounds[axis]);
            bounds[axis + 3] = bounds[axis + 3].max(mesh.bounds[axis + 3]);
        }
    }
    Ok(TerrainVerification {
        source_apk: source.clone(),
        geometry: manifest.geometry.clone(),
        coordinate_frame: manifest.coordinate_frame.clone(),
        uv_origin: manifest.uv_origin.clone(),
        bounds,
        terrain_scene_bundles: terrain.len(),
        terrain_instances_by_scene: terrain
            .iter()
            .map(|(scene, instances)| ((*scene).to_owned(), instances.len()))
            .collect(),
        source_instances: all_instances,
        terrain_source_instances,
        other_source_instances: all_instances - terrain_source_instances,
        render_records: package.records.len(),
        terrain_render_records,
        normal_map_records: manifest
            .meshes
            .values()
            .filter(|mesh| mesh.material.uses_terrain_normal_maps())
            .count(),
        records_with_source_tangents: package
            .records
            .iter()
            .filter(|record| !record.tangents.is_empty())
            .count(),
        normal_map_records_without_source_tangents: package
            .records
            .iter()
            .filter(|record| {
                record.tangents.is_empty()
                    && manifest.meshes[&record.name]
                        .material
                        .uses_terrain_normal_maps()
            })
            .map(|record| record.name.clone())
            .collect(),
        other_render_records: package.records.len() - terrain_render_records,
        vertices: package
            .records
            .iter()
            .map(|record| record.positions.len())
            .sum(),
        triangles: package
            .records
            .iter()
            .map(|record| record.indices.len() / 3)
            .sum(),
        enabled_collision_records: package.collision_records().count(),
        enabled_source_colliders: source_colliders.len(),
        materials: materials.len(),
        textures: textures.len(),
        texture_formats,
        texture_color_spaces,
        texture_mip_counts,
        null_required_samplers,
        shader_variants,
        limitations: manifest.limitations.clone(),
    })
}

pub fn verify(package: &Path, apk: &Path) -> Result<TerrainVerification, Error> {
    let package = load_terrain_package(package)?;
    let source = source_apk(apk)?;
    let members = Apk::open(apk)?
        .members()?
        .into_iter()
        .filter(|member| is_chunk(member))
        .collect();
    verify_loaded(&package, &source, &members)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mesh(bundle: &str, game_object_id: &str, mesh_id: &str) -> SceneMesh {
        serde_json::from_value(serde_json::json!({
            "source": {"bundle": bundle, "cab": "CAB-mesh", "path_id": mesh_id},
            "game_object": {"bundle": bundle, "cab": "CAB-scene", "path_id": game_object_id},
            "world_transform": [1.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,1.0],
            "bounds": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0], "collider": null,
            "material": {"source": {"bundle": bundle, "cab": "CAB-material", "path_id": "-1"},
                "name": "terrain", "shader": {"bundle": bundle, "cab": "CAB-shader", "path_id": "-2"},
                "shader_name": "_URP/TerrainMeshBase/4 Textures", "textures": {}, "floats": {}, "colors": {},
                "shader_defaults": {}, "texture_defaults": {}, "valid_keywords": [], "render_queue": -1}
        })).unwrap()
    }

    #[test]
    fn submeshes_are_one_source_instance_but_distinct_placements_remain_distinct() {
        let first = mesh("assets/scene/a-1.ab", "-17", "-90");
        let second_part = first.clone();
        let other_placement = mesh("assets/scene/a-1.ab", "-18", "-90");
        let scenes = instances_by_scene([&first, &second_part, &other_placement].into_iter());
        assert_eq!(scenes["assets/scene/a-1.ab"].len(), 2);
    }

    #[test]
    fn identical_path_ids_in_different_cabs_or_bundles_do_not_alias() {
        let first = mesh("assets/scene/a-1.ab", "-17", "-90");
        let mut other_cab = first.clone();
        other_cab.game_object.cab = "CAB-other-scene".into();
        let other_bundle = mesh("assets/scene/a-2.ab", "-17", "-90");
        let scenes = instances_by_scene([&first, &other_cab, &other_bundle].into_iter());
        assert_eq!(scenes["assets/scene/a-1.ab"].len(), 2);
        assert_eq!(scenes["assets/scene/a-2.ab"].len(), 1);
    }
}
