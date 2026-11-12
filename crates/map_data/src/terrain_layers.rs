//! Terrain GameObject layer admission shared by package readers and physics.

use std::collections::BTreeMap;

use crate::{ArchiveError, TERRAIN_MANIFEST_VERSION, TerrainManifest, invalid};

impl TerrainManifest {
    /// Check schema/layer consistency without loading geometry or textures.
    ///
    /// Schema two did not retain GameObject layers. Its missing values remain
    /// unknown; adding a claimed layer requires an explicit schema-three import.
    /// Schemas three/four require every render record to retain a layer in 0..31,
    /// including records without an enabled collider. Submeshes referencing the
    /// same qualified GameObject or collider component must agree. Source
    /// completeness/provenance is validated separately by the importer, not
    /// inferred from this metadata.
    pub fn validate_layers(&self) -> Result<(), ArchiveError> {
        if !matches!(self.schema_version, 2 | 3 | TERRAIN_MANIFEST_VERSION) {
            return Err(invalid("unsupported terrain manifest conventions/version"));
        }
        let mut layers = BTreeMap::new();
        let mut collider_layers = BTreeMap::new();
        for (name, mesh) in &self.meshes {
            match (self.schema_version, mesh.layer) {
                (2, None) => {}
                (2, Some(_)) => {
                    return Err(invalid(format!(
                        "{name}: schema-two terrain GameObject layer must remain unknown"
                    )));
                }
                (_, Some(layer)) if layer < 32 => {}
                (_, Some(_)) => {
                    return Err(invalid(format!(
                        "{name}: terrain GameObject layer must be in 0..31"
                    )));
                }
                (_, None) => {
                    return Err(invalid(format!(
                        "{name}: schema-three terrain GameObject layer is missing"
                    )));
                }
            }
            if let Some(previous) = layers.insert(&mesh.game_object, mesh.layer)
                && previous != mesh.layer
            {
                return Err(invalid(format!(
                    "{name}: conflicting terrain layers for GameObject {}:{}:{}",
                    mesh.game_object.bundle, mesh.game_object.cab, mesh.game_object.path_id
                )));
            }
            if let Some(collider) = &mesh.collider
                && let Some(previous) = collider_layers.insert(&collider.source, mesh.layer)
                && previous != mesh.layer
            {
                return Err(invalid(format!(
                    "{name}: conflicting terrain layers for collider {}:{}:{}",
                    collider.source.bundle, collider.source.cab, collider.source.path_id
                )));
            }
        }
        Ok(())
    }
}
