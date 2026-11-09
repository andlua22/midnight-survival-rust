use crate::{ArchiveError, SourceMaterial, invalid};

/// Scalar uniforms used by the compiled APK terrain shaders. Saved `_Layer4`,
/// texture ST offsets and `_Tiling3` do not select layers or transform their UVs.
#[derive(Debug, Clone, Copy)]
pub struct TerrainParameters {
    pub layer_count: usize,
    pub tiling: [f32; 4],
    pub smoothness: [f32; 4],
}

impl TerrainParameters {
    pub fn from_material(material: &SourceMaterial) -> Result<Self, ArchiveError> {
        let layer_count = material
            .terrain_layer_count()
            .ok_or_else(|| invalid(format!("{}: unsupported terrain shader", material.name)))?;
        let mut parameters = Self {
            layer_count,
            tiling: [1.0; 4],
            smoothness: [0.0; 4],
        };
        for layer in 0..layer_count {
            let tiling = material.float(&format!("_Splat{layer}_S")).ok_or_else(|| {
                invalid(format!("{}: missing layer {layer} tiling", material.name))
            })?;
            let smoothness = material
                .float(&format!("_Splat{layer}_Smoothness"))
                .ok_or_else(|| {
                    invalid(format!(
                        "{}: missing layer {layer} smoothness",
                        material.name
                    ))
                })?;
            if !tiling.is_finite() || !smoothness.is_finite() {
                return Err(invalid("non-finite terrain shader parameter"));
            }
            parameters.tiling[layer] = tiling;
            parameters.smoothness[layer] = smoothness;
        }
        Ok(parameters)
    }
}

/// CPU reference for the pre-lighting albedo in the complete GLES3 programs.
/// Both shader variants use raw RGBA in the residual; only the four-layer
/// variant adds the fourth colour. The residual interpolates towards layer 0.
pub fn terrain_albedo(control: [f32; 4], layers: [[f32; 3]; 4], layer_count: usize) -> [f32; 3] {
    let residual = (1.0 - control.into_iter().sum::<f32>()).clamp(0.0, 1.0);
    std::array::from_fn(|channel| {
        let blended: f32 = (0..layer_count)
            .map(|layer| control[layer] * layers[layer][channel])
            .sum();
        blended + residual * (layers[0][channel] - blended)
    })
}

/// Pre-TBN normal vector from the shipped GLES3 normal-map variants. Layer
/// contributions are decoded RGB, but the residual targets encoded layer 0.
pub fn terrain_tangent_normal(
    control: [f32; 4],
    encoded: [[f32; 3]; 4],
    layer_count: usize,
) -> [f32; 3] {
    let residual = (1.0 - control.into_iter().sum::<f32>()).clamp(0.0, 1.0);
    std::array::from_fn(|channel| {
        let blended: f32 = (0..layer_count)
            .map(|layer| control[layer] * (2.0 * encoded[layer][channel] - 1.0))
            .sum();
        blended + residual * (encoded[0][channel] - blended)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYERS: [[f32; 3]; 4] = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ];

    #[test]
    fn residual_is_an_interpolation_and_weights_are_not_normalized() {
        assert_eq!(
            terrain_albedo([0.25, 0.25, 0.0, 0.0], LAYERS, 4),
            [0.625, 0.125, 0.0]
        );
        assert_eq!(
            terrain_albedo([0.75, 0.75, 0.0, 0.0], LAYERS, 4),
            [0.75, 0.75, 0.0]
        );
        assert_eq!(terrain_albedo([0.0; 4], LAYERS, 4), LAYERS[0]);
    }

    #[test]
    fn three_layer_alpha_changes_the_residual_without_adding_a_color() {
        assert_eq!(
            terrain_albedo([0.0, 0.5, 0.0, 0.5], LAYERS, 3),
            [0.0, 0.5, 0.0]
        );
        assert_eq!(
            terrain_albedo([0.0, 0.5, 0.0, 0.0], LAYERS, 3),
            [0.5, 0.25, 0.0]
        );
        assert_eq!(terrain_albedo([0.0, 0.0, 0.0, 1.0], LAYERS, 3), [0.0; 3]);
        assert_eq!(terrain_albedo([0.0, 0.0, 0.0, 1.0], LAYERS, 4), LAYERS[3]);
    }

    #[test]
    fn normal_residual_targets_encoded_layer_zero_and_preserves_three_layer_alpha() {
        let encoded = [[0.5, 0.5, 1.0]; 4];
        assert_eq!(
            terrain_tangent_normal([0.0; 4], encoded, 4),
            [0.5, 0.5, 1.0]
        );
        assert_eq!(
            terrain_tangent_normal([1.0, 0.0, 0.0, 0.0], encoded, 4),
            [0.0, 0.0, 1.0]
        );
        assert_eq!(
            terrain_tangent_normal([0.0, 0.5, 0.0, 0.0], encoded, 3),
            [0.25, 0.25, 0.75]
        );
        assert_eq!(
            terrain_tangent_normal([0.0, 0.5, 0.0, 0.5], encoded, 3),
            [0.0, 0.0, 0.5]
        );
    }
}
