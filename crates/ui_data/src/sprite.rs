//! Source sprite geometry in RectTransform-local XY (Y up), sampled from full
//! decoded textures with top-left normalized UVs. This does not establish APK
//! runtime state or implement custom materials, masks, or canvas pixel snapping.

use serde::Serialize;

use crate::{ImageData, ReferenceState, UiError, UiSprite, invalid, layout::UiRect};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SpriteMesh {
    pub vertices: Vec<SpriteVertex>,
    /// Counterclockwise in source local XY; a caller's screen transform may reflect Y.
    pub indices: Vec<u32>,
}

/// `rect` is the node's local rectangle: its origin is the RectTransform pivot,
/// so `-rect.min / rect.size` supplies Unity's aspect-alignment pivot. The sprite
/// pivot does not reposition Simple/Sliced/Filled Image quads.
///
/// Supports Simple, Sliced, and horizontal/vertical Filled Images. Tiled/radial
/// Images, atlas variants, and Simple Images using retained sprite meshes return
/// errors rather than substituting approximate geometry. Canvas reference PPU
/// is explicit preview/runtime input, not an observed prefab CanvasScaler value.
pub fn image_mesh(
    image: &ImageData,
    sprite: &UiSprite,
    rect: UiRect,
    texture_size: [u32; 2],
    reference_pixels_per_unit: f32,
) -> Result<SpriteMesh, UiError> {
    let sampling = Sampling::new(image, sprite, rect, texture_size, reference_pixels_per_unit)?;
    if image.use_sprite_mesh && image.image_type == 0 {
        return Err(invalid(
            "Simple Image.use_sprite_mesh requires decoding retained sprite mesh data",
        ));
    }
    if !matches!(image.image_type, 0 | 1 | 3) {
        return Err(invalid(format!(
            "unsupported Unity Image type {}",
            image.image_type
        )));
    }
    if image.image_type == 3 {
        validate_fill(image)?;
    }
    if rect.size.contains(&0.0) {
        return Ok(SpriteMesh::default());
    }
    let mesh = match image.image_type {
        0 => simple(
            &sampling,
            drawing_rect(rect, sprite, image.preserve_aspect)?,
            [0.0, 0.0],
            [1.0, 1.0],
        ),
        1 => sliced(image, sprite, rect, &sampling, reference_pixels_per_unit)?,
        3 => filled(image, sprite, rect, &sampling)?,
        _ => unreachable!(),
    };
    for vertex in &mesh.vertices {
        if vertex
            .position
            .iter()
            .chain(&vertex.uv)
            .any(|value| !value.is_finite())
        {
            return Err(invalid(
                "sprite geometry overflowed to a non-finite position or UV",
            ));
        }
    }
    Ok(mesh)
}

struct Sampling {
    crop: [f32; 4],
    texture_size: [u32; 2],
    rotation: u32,
    original: [f32; 2],
    trim_offset: [f32; 2],
    trim_size: [f32; 2],
}

impl Sampling {
    fn new(
        image: &ImageData,
        sprite: &UiSprite,
        rect: UiRect,
        texture_size: [u32; 2],
        reference_ppu: f32,
    ) -> Result<Self, UiError> {
        if rect
            .min
            .iter()
            .chain(&rect.size)
            .any(|value| !value.is_finite())
            || rect.size.iter().any(|value| *value < 0.0)
        {
            return Err(invalid(
                "Image local rectangle must have finite coordinates and nonnegative size",
            ));
        }
        if texture_size.contains(&0)
            || !reference_ppu.is_finite()
            || reference_ppu <= 0.0
            || !sprite.pixels_per_unit.is_finite()
            || sprite.pixels_per_unit <= 0.0
            || !image.pixels_per_unit_multiplier.is_finite()
            || image.pixels_per_unit_multiplier <= 0.0
        {
            return Err(invalid(
                "texture dimensions and source/reference pixels-per-unit must be positive",
            ));
        }
        match &image.sprite.resolution {
            ReferenceState::Resolved {
                target,
                class_id: 213,
            } if target == &sprite.source => {}
            _ => {
                return Err(invalid(
                    "Image sprite reference does not identify the supplied source Sprite",
                ));
            }
        }
        if sprite.render.downscale_multiplier != 1.0 {
            return Err(invalid(
                "sprite atlas variant downscaling is not implemented",
            ));
        }
        if sprite.render.packing_rotation > 4 {
            return Err(invalid(format!(
                "unsupported Sprite packing rotation {}",
                sprite.render.packing_rotation
            )));
        }
        let crop = sprite.render.texture_rect;
        if crop
            .iter()
            .chain(&sprite.rect)
            .chain(&sprite.render.texture_rect_offset)
            .any(|value| !value.is_finite())
            || crop[0] < 0.0
            || crop[1] < 0.0
            || crop[2] <= 0.0
            || crop[3] <= 0.0
        {
            return Err(invalid("invalid source Sprite texture rectangle"));
        }
        for axis in 0..2 {
            if f64::from(crop[axis]) + f64::from(crop[axis + 2]) > f64::from(texture_size[axis]) {
                return Err(invalid(
                    "source Sprite crop exceeds its full decoded texture",
                ));
            }
        }
        let original = [sprite.rect[2], sprite.rect[3]];
        if original.iter().any(|value| *value <= 0.0) {
            return Err(invalid(
                "source Sprite rectangle dimensions must be positive",
            ));
        }
        let trim_size = if sprite.render.packing_rotation == 4 {
            [crop[3], crop[2]]
        } else {
            [crop[2], crop[3]]
        };
        let trim_offset = sprite.render.texture_rect_offset;
        for axis in 0..2 {
            // Float32 source fields can differ by an ULP when padding is zero.
            let tolerance = original[axis].abs().max(1.0) * 4.0 * f32::EPSILON;
            if trim_offset[axis] < 0.0
                || trim_offset[axis] + trim_size[axis] > original[axis] + tolerance
            {
                return Err(invalid(
                    "source Sprite trim lies outside its original rectangle",
                ));
            }
        }
        Ok(Self {
            crop,
            texture_size,
            rotation: sprite.render.packing_rotation,
            original,
            trim_offset,
            trim_size,
        })
    }

    fn uv(&self, point: [f32; 2]) -> [f32; 2] {
        let [s, t] = point;
        // Serialized SpritePackingRotation: None, FlipHorizontal, FlipVertical,
        // Rotate180, Rotate90. Rotate90 matches clockwise unpacking in PNG space.
        let packed = match self.rotation {
            0 => [s, t],
            1 => [1.0 - s, t],
            2 => [s, 1.0 - t],
            3 => [1.0 - s, 1.0 - t],
            4 => [1.0 - t, s],
            _ => unreachable!(),
        };
        // No integer crop bounds, half-pixel offsets, or duplicate vertical flip.
        [
            ((f64::from(self.crop[0]) + f64::from(self.crop[2]) * f64::from(packed[0]))
                / f64::from(self.texture_size[0])) as f32,
            (1.0 - (f64::from(self.crop[1]) + f64::from(self.crop[3]) * f64::from(packed[1]))
                / f64::from(self.texture_size[1])) as f32,
        ]
    }

    fn trimmed_rect(&self, rect: UiRect) -> UiRect {
        UiRect {
            min: [
                rect.min[0] + rect.size[0] * self.trim_offset[0] / self.original[0],
                rect.min[1] + rect.size[1] * self.trim_offset[1] / self.original[1],
            ],
            size: [
                rect.size[0] * self.trim_size[0] / self.original[0],
                rect.size[1] * self.trim_size[1] / self.original[1],
            ],
        }
    }

    fn padding(&self) -> [f32; 4] {
        [
            self.trim_offset[0],
            self.trim_offset[1],
            (self.original[0] - self.trim_offset[0] - self.trim_size[0]).max(0.0),
            (self.original[1] - self.trim_offset[1] - self.trim_size[1]).max(0.0),
        ]
    }
}

fn drawing_rect(
    mut rect: UiRect,
    sprite: &UiSprite,
    preserve_aspect: bool,
) -> Result<UiRect, UiError> {
    if preserve_aspect {
        let pivot = [-rect.min[0] / rect.size[0], -rect.min[1] / rect.size[1]];
        if pivot.iter().any(|value| !value.is_finite()) {
            return Err(invalid(
                "local rectangle does not encode a finite RectTransform pivot",
            ));
        }
        let sprite_aspect = sprite.rect[2] / sprite.rect[3];
        let rect_aspect = rect.size[0] / rect.size[1];
        if sprite_aspect > rect_aspect {
            let height = rect.size[0] / sprite_aspect;
            rect.min[1] += (rect.size[1] - height) * pivot[1];
            rect.size[1] = height;
        } else {
            let width = rect.size[1] * sprite_aspect;
            rect.min[0] += (rect.size[0] - width) * pivot[0];
            rect.size[0] = width;
        }
    }
    Ok(rect)
}

fn simple(
    sampling: &Sampling,
    rect: UiRect,
    sample_min: [f32; 2],
    sample_max: [f32; 2],
) -> SpriteMesh {
    let rect = sampling.trimmed_rect(rect);
    let points = [
        sample_min,
        [sample_max[0], sample_min[1]],
        sample_max,
        [sample_min[0], sample_max[1]],
    ];
    SpriteMesh {
        vertices: points
            .map(|point| SpriteVertex {
                position: [
                    rect.min[0] + rect.size[0] * point[0],
                    rect.min[1] + rect.size[1] * point[1],
                ],
                uv: sampling.uv(point),
            })
            .to_vec(),
        indices: vec![0, 1, 2, 2, 3, 0],
    }
}

fn sliced(
    image: &ImageData,
    sprite: &UiSprite,
    rect: UiRect,
    sampling: &Sampling,
    reference_ppu: f32,
) -> Result<SpriteMesh, UiError> {
    if sprite
        .border
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(invalid(
            "source Sprite borders must be finite and nonnegative",
        ));
    }
    if sprite.border.iter().all(|value| *value == 0.0) {
        // Unity's no-border Sliced fallback ignores preserveAspect/fillCenter.
        return Ok(simple(sampling, rect, [0.0, 0.0], [1.0, 1.0]));
    }
    let multiplied_ppu = sprite.pixels_per_unit / reference_ppu * image.pixels_per_unit_multiplier;
    if !multiplied_ppu.is_finite() || multiplied_ppu <= 0.0 {
        return Err(invalid("source Image border pixels-per-unit overflow"));
    }
    let mut border = sprite.border.map(|value| value / multiplied_ppu);
    let mut collapsed_split = [None; 2];
    for axis in 0..2 {
        let combined = border[axis] + border[axis + 2];
        if combined >= rect.size[axis] && combined > 0.0 {
            // A fully occupied axis has one shared inner edge. Computing the
            // two positions independently can cross them by an ULP after
            // Float32 border scaling, even with valid symmetric source borders.
            collapsed_split[axis] = Some(
                (f64::from(rect.min[axis])
                    + f64::from(rect.size[axis]) * f64::from(border[axis]) / f64::from(combined))
                    as f32,
            );
            let scale = rect.size[axis] / combined;
            border[axis] *= scale;
            border[axis + 2] *= scale;
        }
    }
    let padding = sampling.padding().map(|value| value / multiplied_ppu);
    let max = rect.max();
    let x = [
        rect.min[0] + padding[0],
        collapsed_split[0].unwrap_or(rect.min[0] + border[0]),
        collapsed_split[0].unwrap_or(max[0] - border[2]),
        max[0] - padding[2],
    ];
    let y = [
        rect.min[1] + padding[1],
        collapsed_split[1].unwrap_or(rect.min[1] + border[1]),
        collapsed_split[1].unwrap_or(max[1] - border[3]),
        max[1] - padding[3],
    ];
    let sx = [
        0.0,
        (sprite.border[0] - sampling.trim_offset[0]) / sampling.trim_size[0],
        (sampling.original[0] - sprite.border[2] - sampling.trim_offset[0]) / sampling.trim_size[0],
        1.0,
    ];
    let sy = [
        0.0,
        (sprite.border[1] - sampling.trim_offset[1]) / sampling.trim_size[1],
        (sampling.original[1] - sprite.border[3] - sampling.trim_offset[1]) / sampling.trim_size[1],
        1.0,
    ];
    for grid in [&x, &y, &sx, &sy] {
        if grid.iter().any(|value| !value.is_finite())
            || grid.windows(2).any(|pair| pair[0] > pair[1])
        {
            return Err(invalid(
                "source sliced Sprite padding/borders produce an inverted grid",
            ));
        }
    }
    if sx
        .iter()
        .chain(&sy)
        .any(|value| !(0.0..=1.0).contains(value))
    {
        return Err(invalid(
            "source sliced Sprite inner UVs lie outside its retained crop",
        ));
    }
    let mut mesh = SpriteMesh::default();
    for row in 0..4 {
        for column in 0..4 {
            mesh.vertices.push(SpriteVertex {
                position: [x[column], y[row]],
                uv: sampling.uv([sx[column], sy[row]]),
            });
        }
    }
    for row in 0..3 {
        for column in 0..3 {
            if (row == 1 && column == 1 && !image.fill_center)
                || x[column] == x[column + 1]
                || y[row] == y[row + 1]
            {
                continue;
            }
            let bottom_left = (row * 4 + column) as u32;
            mesh.indices.extend_from_slice(&[
                bottom_left,
                bottom_left + 1,
                bottom_left + 5,
                bottom_left + 5,
                bottom_left + 4,
                bottom_left,
            ]);
        }
    }
    Ok(mesh)
}

fn filled(
    image: &ImageData,
    sprite: &UiSprite,
    rect: UiRect,
    sampling: &Sampling,
) -> Result<SpriteMesh, UiError> {
    // Unity UGUI GenerateFilledSprite's lower threshold; genuine zero stays empty.
    if image.fill_amount < 0.001 {
        return Ok(SpriteMesh::default());
    }
    let mut min = [0.0, 0.0];
    let mut max = [1.0, 1.0];
    let axis = image.fill_method as usize;
    if image.fill_origin == 1 {
        min[axis] = 1.0 - image.fill_amount;
    } else {
        max[axis] = image.fill_amount;
    }
    Ok(simple(
        sampling,
        drawing_rect(rect, sprite, image.preserve_aspect)?,
        min,
        max,
    ))
}

fn validate_fill(image: &ImageData) -> Result<(), UiError> {
    if !image.fill_amount.is_finite() || !(0.0..=1.0).contains(&image.fill_amount) {
        return Err(invalid("Image.fill_amount must be finite and within 0..1"));
    }
    if !matches!(image.fill_method, 0 | 1) {
        return Err(invalid(format!(
            "unsupported radial/unknown Image fill method {}",
            image.fill_method
        )));
    }
    if !matches!(image.fill_origin, 0 | 1) {
        return Err(invalid(
            "horizontal/vertical Image fill origin must be 0 or 1",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GraphicData, SourceReference, SpriteRenderData};
    use midnight_map_data::{FileIdentity, SourceObject};

    fn source(path_id: &str) -> SourceObject {
        SourceObject {
            bundle: "assets/test.ab".into(),
            cab: "CAB-test".into(),
            path_id: path_id.into(),
        }
    }

    fn reference(path_id: &str, class_id: i32) -> SourceReference {
        SourceReference {
            owner: source("1"),
            file_id: 0,
            path_id: path_id.into(),
            resolution: ReferenceState::Resolved {
                target: source(path_id),
                class_id,
            },
        }
    }

    fn fixture() -> (ImageData, UiSprite) {
        let image = ImageData {
            graphic: GraphicData {
                color: [1.0; 4],
                material: reference("21", 21),
                raycast_target: false,
                raycast_padding: [0.0; 4],
                maskable: true,
            },
            sprite: reference("213", 213),
            image_type: 0,
            preserve_aspect: false,
            fill_center: true,
            fill_method: 0,
            fill_origin: 0,
            fill_clockwise: true,
            fill_amount: 1.0,
            use_sprite_mesh: false,
            pixels_per_unit_multiplier: 1.0,
        };
        let sprite = UiSprite {
            source: source("213"),
            name: "asymmetric".into(),
            rect: [0.0, 0.0, 50.0, 20.0],
            pivot: [0.5; 2],
            border: [0.0; 4],
            pixels_per_unit: 100.0,
            is_polygon: false,
            render: SpriteRenderData {
                owner: source("213"),
                atlas: Some(source("687078895")),
                render_data_key: serde_json::Value::Null,
                texture: reference("28", 28),
                alpha_texture: SourceReference {
                    owner: source("213"),
                    file_id: 0,
                    path_id: "0".into(),
                    resolution: ReferenceState::Null,
                },
                texture_rect: [20.25, 30.5, 50.0, 20.0],
                texture_rect_offset: [0.0; 2],
                atlas_rect_offset: [20.25, 30.5],
                uv_transform: [100.0, 45.25, 100.0, 40.5],
                downscale_multiplier: 1.0,
                settings_raw: 3,
                packed: true,
                packing_mode: 1,
                packing_rotation: 0,
                mesh_type: 0,
            },
            source_fields: FileIdentity {
                path: "sprite.json".into(),
                sha256: "source-test".into(),
            },
            preview: None,
        };
        (image, sprite)
    }

    fn close(actual: [f32; 2], expected: [f32; 2]) {
        for axis in 0..2 {
            assert!(
                (actual[axis] - expected[axis]).abs() < 2e-6,
                "{actual:?} != {expected:?}"
            );
        }
    }

    #[test]
    fn fractional_atlas_crop_and_uv_y_conversion_are_applied_once() {
        let (image, sprite) = fixture();
        let rect = UiRect {
            min: [-25.0, -10.0],
            size: [50.0, 20.0],
        };
        let mesh = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        assert_eq!(mesh.indices, [0, 1, 2, 2, 3, 0]);
        for (vertex, position) in mesh.vertices.iter().zip(rect.corners()) {
            close(vertex.position, position);
        }
        for (vertex, uv) in mesh.vertices.iter().zip([
            [0.10125, 0.695],
            [0.35125, 0.695],
            [0.35125, 0.495],
            [0.10125, 0.495],
        ]) {
            close(vertex.uv, uv);
        }
    }

    #[test]
    fn aspect_uses_the_recttransform_pivot_and_retains_asymmetric_trim() {
        let (mut image, mut sprite) = fixture();
        image.preserve_aspect = true;
        sprite.rect = [0.0, 0.0, 100.0, 50.0];
        sprite.render.texture_rect = [20.0, 30.0, 80.0, 35.0];
        sprite.render.texture_rect_offset = [10.0, 5.0];
        sprite.pivot = [0.1, 0.9];
        let bottom = image_mesh(
            &image,
            &sprite,
            UiRect {
                min: [0.0, 0.0],
                size: [200.0, 200.0],
            },
            [200, 100],
            100.0,
        )
        .unwrap();
        close(bottom.vertices[0].position, [20.0, 10.0]);
        close(bottom.vertices[2].position, [180.0, 80.0]);
        for (vertex, expected) in
            bottom
                .vertices
                .iter()
                .zip([[0.1, 0.7], [0.5, 0.7], [0.5, 0.35], [0.1, 0.35]])
        {
            close(vertex.uv, expected);
        }
        let top = image_mesh(
            &image,
            &sprite,
            UiRect {
                min: [-200.0, -200.0],
                size: [200.0, 200.0],
            },
            [200, 100],
            100.0,
        )
        .unwrap();
        close(top.vertices[0].position, [-180.0, -90.0]);
        close(top.vertices[2].position, [-20.0, -20.0]);
        sprite.pivot = [0.9, 0.1];
        let unchanged = image_mesh(
            &image,
            &sprite,
            UiRect {
                min: [0.0, 0.0],
                size: [200.0, 200.0],
            },
            [200, 100],
            100.0,
        )
        .unwrap();
        assert_eq!(bottom, unchanged);
    }

    #[test]
    fn sliced_trim_keeps_pixel_padding_and_exact_asymmetric_inner_uvs() {
        let (mut image, mut sprite) = fixture();
        image.image_type = 1;
        sprite.rect = [0.0, 0.0, 100.0, 80.0];
        sprite.render.texture_rect = [50.25, 100.75, 85.0, 70.0];
        sprite.render.texture_rect_offset = [5.0, 3.0];
        sprite.border = [12.0, 10.0, 20.0, 15.0];
        let rect = UiRect {
            min: [0.0; 2],
            size: [200.0, 160.0],
        };
        let mesh = image_mesh(&image, &sprite, rect, [400, 300], 100.0).unwrap();
        close(mesh.vertices[0].position, [5.0, 3.0]);
        close(mesh.vertices[15].position, [190.0, 153.0]);
        close(mesh.vertices[5].position, [12.0, 10.0]);
        close(mesh.vertices[5].uv, [57.25 / 400.0, 1.0 - 107.75 / 300.0]);
        close(mesh.vertices[10].uv, [125.25 / 400.0, 1.0 - 162.75 / 300.0]);
        image.image_type = 0;
        let simple = image_mesh(&image, &sprite, rect, [400, 300], 100.0).unwrap();
        close(simple.vertices[0].position, [10.0, 6.0]);
        close(simple.vertices[2].position, [180.0, 146.0]);
    }

    #[test]
    fn sliced_borders_keep_source_ppu_and_can_omit_the_center() {
        let (mut image, mut sprite) = fixture();
        image.image_type = 1;
        sprite.rect = [0.0, 0.0, 100.0, 80.0];
        sprite.render.texture_rect = [50.0, 100.0, 100.0, 80.0];
        sprite.border = [10.0, 20.0, 30.0, 10.0];
        let rect = UiRect {
            min: [-100.0, -50.0],
            size: [200.0, 100.0],
        };
        let mesh = image_mesh(&image, &sprite, rect, [400, 300], 100.0).unwrap();
        assert_eq!(mesh.vertices.len(), 16);
        assert_eq!(mesh.indices.len(), 54);
        close(mesh.vertices[5].position, [-90.0, -30.0]);
        close(mesh.vertices[10].position, [70.0, 40.0]);
        close(mesh.vertices[5].uv, [60.0 / 400.0, 1.0 - 120.0 / 300.0]);
        close(mesh.vertices[10].uv, [120.0 / 400.0, 1.0 - 170.0 / 300.0]);
        image.fill_center = false;
        let frame = image_mesh(&image, &sprite, rect, [400, 300], 100.0).unwrap();
        assert_eq!(frame.vertices, mesh.vertices);
        assert_eq!(frame.indices.len(), 48);
        assert!(
            !frame
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .any(|triangle| *triangle == [5, 6, 10] || *triangle == [10, 9, 5])
        );
        image.pixels_per_unit_multiplier = 2.0;
        let scaled = image_mesh(&image, &sprite, rect, [400, 300], 100.0).unwrap();
        close(scaled.vertices[5].position, [-95.0, -40.0]);
        close(scaled.vertices[10].position, [85.0, 45.0]);
    }

    #[test]
    fn sliced_borders_shrink_proportionally_and_zero_border_falls_back_to_simple() {
        let (mut image, mut sprite) = fixture();
        image.image_type = 1;
        sprite.border = [10.0, 8.0, 30.0, 4.0];
        let rect = UiRect {
            min: [0.0; 2],
            size: [20.0, 6.0],
        };
        let mesh = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        close(mesh.vertices[5].position, [5.0, 4.0]);
        close(mesh.vertices[10].position, [5.0, 4.0]);
        assert_eq!(mesh.indices.len(), 24);
        sprite.border = [0.0; 4];
        image.preserve_aspect = true;
        image.fill_center = false;
        let square = UiRect {
            min: [0.0; 2],
            size: [100.0; 2],
        };
        let simple = image_mesh(&image, &sprite, square, [200, 100], 100.0).unwrap();
        assert_eq!(simple.indices.len(), 6);
        close(simple.vertices[2].position, [100.0; 2]);
    }

    #[test]
    fn collapsed_slice_center_uses_one_edge_despite_float32_rounding() {
        let (mut image, mut sprite) = fixture();
        image.image_type = 1;
        sprite.rect = [0.0, 0.0, 40.0, 40.0];
        sprite.render.texture_rect = [20.25, 30.5, 40.0, 40.0];
        sprite.border = [15.0; 4];
        let height = 0.06169545_f32;
        let rect = UiRect {
            min: [-50.0, -height / 2.0],
            size: [100.0, height],
        };
        let mesh = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        assert_eq!(mesh.vertices[5].position[1], mesh.vertices[9].position[1]);
        assert_eq!(mesh.vertices[5].position[1], 0.0);
        assert_eq!(mesh.indices.len(), 36);
        assert!(mesh.vertices.iter().all(
            |vertex| vertex.position[1] >= -height / 2.0 && vertex.position[1] <= height / 2.0
        ));
    }

    #[test]
    fn filled_zero_is_empty_and_fractional_fill_cuts_positions_and_uvs() {
        let (mut image, sprite) = fixture();
        let rect = UiRect {
            min: [0.0; 2],
            size: [80.0, 40.0],
        };
        image.fill_amount = 0.0;
        assert_eq!(
            image_mesh(&image, &sprite, rect, [200, 100], 100.0)
                .unwrap()
                .indices
                .len(),
            6
        );
        image.image_type = 3;
        assert_eq!(
            image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap(),
            SpriteMesh::default()
        );
        image.fill_amount = 0.0005;
        assert_eq!(
            image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap(),
            SpriteMesh::default()
        );
        image.fill_amount = 0.25;
        let left = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        close(left.vertices[0].position, [0.0, 0.0]);
        close(left.vertices[2].position, [20.0, 40.0]);
        close(left.vertices[1].uv, [(20.25 + 50.0 * 0.25) / 200.0, 0.695]);
        image.fill_origin = 1;
        let right = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        close(right.vertices[0].position, [60.0, 0.0]);
        close(right.vertices[2].position, [80.0, 40.0]);
        image.fill_method = 1;
        image.fill_amount = 0.6;
        let top = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        close(top.vertices[0].position, [0.0, 16.0]);
        close(top.vertices[2].position, [80.0, 40.0]);
        image.fill_origin = 0;
        let bottom = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        close(bottom.vertices[2].position, [80.0, 24.0]);
    }

    #[test]
    fn packing_flips_and_quarter_rotation_map_into_the_full_texture() {
        let (image, mut sprite) = fixture();
        sprite.rect = [0.0, 0.0, 40.0, 20.0];
        sprite.render.texture_rect = [20.0, 30.0, 40.0, 20.0];
        let rect = UiRect {
            min: [0.0; 2],
            size: [40.0, 20.0],
        };
        for (rotation, expected) in [
            (1, [[0.3, 0.7], [0.1, 0.7], [0.1, 0.5], [0.3, 0.5]]),
            (2, [[0.1, 0.5], [0.3, 0.5], [0.3, 0.7], [0.1, 0.7]]),
            (3, [[0.3, 0.5], [0.1, 0.5], [0.1, 0.7], [0.3, 0.7]]),
        ] {
            sprite.render.packing_rotation = rotation;
            let mesh = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
            for (vertex, expected) in mesh.vertices.iter().zip(expected) {
                close(vertex.uv, expected);
            }
        }
        sprite.render.packing_rotation = 4;
        sprite.render.texture_rect = [40.0, 10.0, 20.0, 40.0];
        let mesh = image_mesh(&image, &sprite, rect, [200, 100], 100.0).unwrap();
        for (vertex, expected) in
            mesh.vertices
                .iter()
                .zip([[0.3, 0.9], [0.3, 0.5], [0.2, 0.5], [0.2, 0.9]])
        {
            close(vertex.uv, expected);
        }
    }

    #[test]
    fn unsupported_geometry_and_invalid_source_parameters_return_errors() {
        let (mut image, mut sprite) = fixture();
        let rect = UiRect {
            min: [0.0; 2],
            size: [50.0, 20.0],
        };
        image.use_sprite_mesh = true;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        image.use_sprite_mesh = false;
        image.image_type = 2;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        image.image_type = 3;
        image.fill_method = 4;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        assert!(
            image_mesh(
                &image,
                &sprite,
                UiRect {
                    min: [0.0; 2],
                    size: [0.0; 2]
                },
                [200, 100],
                100.0
            )
            .is_err()
        );
        image.fill_method = 0;
        image.fill_amount = f32::NAN;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        image.image_type = 0;
        sprite.render.texture_rect[0] = 199.5;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        sprite.render.texture_rect[0] = 20.25;
        sprite.render.downscale_multiplier = 0.5;
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
        sprite.render.downscale_multiplier = 1.0;
        assert!(image_mesh(&image, &sprite, rect, [0, 100], 100.0).is_err());
        assert!(
            image_mesh(
                &image,
                &sprite,
                UiRect {
                    min: [0.0; 2],
                    size: [-1.0, 20.0]
                },
                [200, 100],
                100.0
            )
            .is_err()
        );
        sprite.source = source("999");
        assert!(image_mesh(&image, &sprite, rect, [200, 100], 100.0).is_err());
    }
}
