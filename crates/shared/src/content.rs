//! Stable content identities shared by protocol, persistence and runtime loaders.
use crate::level::MapId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainIdentity {
    pub source_apk_sha256: String,
    pub manifest_sha256: String,
    pub geometry_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameplayIdentity {
    pub source_apk_sha256: String,
    pub manifest_sha256: String,
    pub catalog_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldDescriptor {
    pub map: MapId,
    pub terrain: Option<TerrainIdentity>,
    pub gameplay: Option<GameplayIdentity>,
}

impl WorldDescriptor {
    pub fn fixture(map: MapId) -> Self {
        Self {
            map,
            terrain: None,
            gameplay: None,
        }
    }

    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
            && (self.map != MapId::ImportedIsland
                || self
                    .terrain
                    .as_ref()
                    .zip(self.gameplay.as_ref())
                    .is_some_and(|(terrain, gameplay)| {
                        terrain.source_apk_sha256 == gameplay.source_apk_sha256
                    }))
    }
}
