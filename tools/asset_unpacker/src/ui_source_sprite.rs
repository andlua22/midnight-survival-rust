//! Resolve SpriteAtlas keys without discarding FileID, then retain texture-crop previews.

use std::io::Cursor;

use image::{ImageFormat, RgbaImage, imageops};
use unity_asset_binary::object::UnityObject;
use unity_asset_core::UnityValue;

use crate::{
    fields::{array, field, integer, invalid, number, property, text, vector},
    resolver::ObjectId,
    unity::Error,
};

use super::{
    importer::{Importer, flag, rect, reference_id},
    types::*,
};

fn atlas_render_data<'a>(map: &'a UnityValue, key: &UnityValue) -> Result<&'a UnityValue, Error> {
    let mut found = None;
    for entry in array(map)? {
        let pair = array(entry)?;
        if pair.len() != 2 {
            return Err(invalid("malformed SpriteAtlas render-data pair"));
        }
        if &pair[0] == key {
            if found.is_some() {
                return Err(invalid("duplicate SpriteAtlas render-data key"));
            }
            found = Some(&pair[1]);
        }
    }
    found.ok_or_else(|| invalid("Sprite render-data key is absent from its referenced SpriteAtlas"))
}

/// Preserve every touched source pixel for fractional tight-sprite rectangles.
/// These bounds describe a preview; they do not round the retained Unity metadata.
fn crop_bounds(texture_size: (u32, u32), rect: [f32; 4]) -> Result<([u32; 4], bool), Error> {
    let [x, y, width, height] = rect;
    if rect.iter().any(|value| !value.is_finite())
        || x < 0.0
        || y < 0.0
        || width <= 0.0
        || height <= 0.0
    {
        return Err(invalid("invalid source sprite texture rectangle"));
    }
    let left = f64::from(x).floor();
    let bottom = f64::from(y).floor();
    let right = (f64::from(x) + f64::from(width)).ceil();
    let top = (f64::from(y) + f64::from(height)).ceil();
    if right > f64::from(texture_size.0) || top > f64::from(texture_size.1) {
        return Err(invalid(
            "sprite texture rectangle exceeds decoded Texture2D bounds",
        ));
    }
    let fractional = rect.iter().any(|value| value.fract() != 0.0);
    let left = left as u32;
    let bottom = bottom as u32;
    let right = right as u32;
    let top = top as u32;
    let png_top = texture_size
        .1
        .checked_sub(top)
        .ok_or_else(|| invalid("sprite PNG origin underflow"))?;
    Ok(([left, png_top, right - left, top - bottom], fractional))
}

fn unpack_rotation(mut image: RgbaImage, rotation: u32) -> Result<RgbaImage, Error> {
    match rotation {
        0 => Ok(image),
        1 => {
            imageops::flip_horizontal_in_place(&mut image);
            Ok(image)
        }
        2 => {
            imageops::flip_vertical_in_place(&mut image);
            Ok(image)
        }
        3 => Ok(imageops::rotate180(&image)),
        // Unity's kSPRRotate90 is unpacked by a clockwise quarter-turn in PNG coordinates.
        4 => Ok(imageops::rotate90(&image)),
        _ => Err(invalid(format!(
            "unsupported Unity sprite packing rotation {rotation}"
        ))),
    }
}

fn apply_alpha(image: &mut RgbaImage, alpha: &RgbaImage) -> Result<(), Error> {
    if image.dimensions() != alpha.dimensions() {
        return Err(invalid("separate sprite alpha dimensions differ"));
    }
    for (pixel, alpha) in image.pixels_mut().zip(alpha.pixels()) {
        pixel[3] = alpha[0];
    }
    Ok(())
}

fn preview(importer: &mut Importer<'_>, render: &SpriteRenderData) -> Result<SpritePreview, Error> {
    let texture_id = reference_id(&render.texture)?;
    let texture = importer.texture_image(&texture_id)?;
    let (bounds, fractional_crop_bounds) = crop_bounds(texture.dimensions(), render.texture_rect)?;
    let [x, y, width, height] = bounds;
    let mut image = imageops::crop_imm(texture.as_ref(), x, y, width, height).to_image();
    let separate_alpha_red_channel_applied =
        !matches!(render.alpha_texture.resolution, ReferenceState::Null);
    if separate_alpha_red_channel_applied {
        let alpha_id = reference_id(&render.alpha_texture)?;
        let alpha = importer.texture_image(&alpha_id)?;
        if alpha.dimensions() != texture.dimensions() {
            return Err(invalid(
                "sprite atlas alpha/color textures have different dimensions",
            ));
        }
        let alpha = imageops::crop_imm(alpha.as_ref(), x, y, width, height).to_image();
        apply_alpha(&mut image, &alpha)?;
    }
    let image = unpack_rotation(image, render.packing_rotation)?;
    let (width, height) = image.dimensions();
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).write_to(&mut encoded, ImageFormat::Png)?;
    let file = importer.store_bytes("sprite_previews", "png", &encoded.into_inner())?;
    Ok(SpritePreview {
        file,
        width,
        height,
        texture_crop_pixels: bounds,
        fractional_crop_bounds,
        packing_rotation_applied: render.packing_rotation,
        separate_alpha_red_channel_applied,
        source_mesh_applied: false,
        original_canvas_restored: false,
    })
}

pub(super) fn extract(
    importer: &mut Importer<'_>,
    id: &ObjectId,
    object: &UnityObject,
) -> Result<UiSprite, Error> {
    if object.info.type_id != 213 {
        return Err(invalid("UI sprite reference does not point to Sprite"));
    }
    let atlas_reference = importer.reference(id, property(object, "m_SpriteAtlas")?)?;
    let key = property(object, "m_RenderDataKey")?;
    let (owner, atlas, render_data) = if matches!(atlas_reference.resolution, ReferenceState::Null)
    {
        (id.clone(), None, property(object, "m_RD")?.clone())
    } else {
        let atlas_id = reference_id(&atlas_reference)?;
        let atlas_object = importer.resolver.object(&atlas_id)?;
        if atlas_object.info.type_id != 687078895 {
            return Err(invalid("Sprite.m_SpriteAtlas is not SpriteAtlas"));
        }
        importer.record(&atlas_id, &atlas_object, false)?;
        let data = atlas_render_data(property(&atlas_object, "m_RenderDataMap")?, key)?.clone();
        (atlas_id.clone(), Some(atlas_id.source()), data)
    };
    let settings_raw = u32::try_from(integer(field(&render_data, "settingsRaw")?)?)?;
    let downscale_multiplier = number(field(&render_data, "downscaleMultiplier")?)?;
    if downscale_multiplier <= 0.0 {
        return Err(invalid("sprite downscale multiplier is not positive"));
    }
    let render = SpriteRenderData {
        owner: owner.source(),
        atlas,
        render_data_key: serde_json::to_value(key)?,
        texture: importer.reference(&owner, field(&render_data, "texture")?)?,
        alpha_texture: importer.reference(&owner, field(&render_data, "alphaTexture")?)?,
        texture_rect: rect(field(&render_data, "textureRect")?)?,
        texture_rect_offset: vector(field(&render_data, "textureRectOffset")?, ["x", "y"])?,
        atlas_rect_offset: vector(field(&render_data, "atlasRectOffset")?, ["x", "y"])?,
        uv_transform: vector(field(&render_data, "uvTransform")?, ["x", "y", "z", "w"])?,
        downscale_multiplier,
        settings_raw,
        packed: settings_raw & 1 != 0,
        packing_mode: (settings_raw >> 1) & 1,
        packing_rotation: (settings_raw >> 2) & 15,
        mesh_type: (settings_raw >> 6) & 1,
    };
    let pixels_per_unit = number(property(object, "m_PixelsToUnits")?)?;
    if pixels_per_unit <= 0.0 {
        return Err(invalid("sprite pixels-per-unit is not positive"));
    }
    let mut sprite = UiSprite {
        source: id.source(),
        name: text(property(object, "m_Name")?)?.to_owned(),
        rect: rect(property(object, "m_Rect")?)?,
        pivot: vector(property(object, "m_Pivot")?, ["x", "y"])?,
        border: vector(property(object, "m_Border")?, ["x", "y", "z", "w"])?,
        pixels_per_unit,
        is_polygon: flag(property(object, "m_IsPolygon")?)?,
        render,
        source_fields: importer.source_fields_for(id)?,
        preview: None,
    };
    match preview(importer, &sprite.render) {
        Ok(preview) => {
            if preview.fractional_crop_bounds
                || sprite.is_polygon
                || sprite.render.texture_rect_offset != [0.0, 0.0]
                || (sprite.render.packed && sprite.render.packing_mode == 0)
                || sprite.render.downscale_multiplier != 1.0
            {
                importer.issue("warning", "sprite_preview_requires_geometry_or_trim", Some(id), Some("/m_RD"),
                    "PNG is a texture crop preview; source fractional trim, original canvas, packing, and mesh data remain authoritative for runtime reconstruction".into());
            }
            sprite.preview = Some(preview);
        }
        Err(error) => importer.issue(
            "error",
            "sprite_preview_failed",
            Some(id),
            None,
            error.to_string(),
        ),
    }
    Ok(sprite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use serde_json::json;

    fn value(json: serde_json::Value) -> UnityValue {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn atlas_lookup_uses_the_entire_guid_and_local_id_key() {
        let first = value(
            json!([[{"data[0]": 1, "data[1]": 2, "data[2]": 3, "data[3]": 4}, 21300000],
            {"texture": {"m_FileID": 1, "m_PathID": 42}}]),
        );
        let second = value(
            json!([[{"data[0]": 9, "data[1]": 2, "data[2]": 3, "data[3]": 4}, 21300000],
            {"texture": {"m_FileID": 7, "m_PathID": 42}}]),
        );
        let map = UnityValue::Array(vec![first.clone(), second.clone()]);
        let key = array(&second).unwrap()[0].clone();
        let data = atlas_render_data(&map, &key).unwrap();
        assert_eq!(
            integer(field(field(data, "texture").unwrap(), "m_FileID").unwrap()).unwrap(),
            7
        );
        let duplicate = UnityValue::Array(vec![second.clone(), second]);
        assert!(atlas_render_data(&duplicate, &key).is_err());
        assert!(atlas_render_data(&map, &value(json!([{}, 21300001]))).is_err());
    }

    #[test]
    fn bottom_left_rect_is_converted_once_for_an_asymmetric_texture() {
        let image = RgbaImage::from_fn(4, 3, |x, y| Rgba([x as u8, y as u8, 0, 255]));
        let (bounds, fractional) = crop_bounds(image.dimensions(), [1.0, 0.0, 2.0, 1.0]).unwrap();
        assert_eq!(bounds, [1, 2, 2, 1]);
        assert!(!fractional);
        let [x, y, w, h] = bounds;
        let crop = imageops::crop_imm(&image, x, y, w, h).to_image();
        assert_eq!(crop.get_pixel(0, 0).0, [1, 2, 0, 255]);
        assert_eq!(crop.get_pixel(1, 0).0, [2, 2, 0, 255]);
    }

    #[test]
    fn fractional_bounds_preserve_touched_pixels_and_invalid_bounds_fail() {
        assert_eq!(
            crop_bounds((810, 810), [5.07612, 5.07612, 799.8478, 799.8478]).unwrap(),
            ([5, 5, 800, 800], true)
        );
        assert!(crop_bounds((2, 2), [1.0, 1.0, 2.0, 2.0]).is_err());
        assert!(crop_bounds((2, 2), [0.0, 0.0, 0.0, 1.0]).is_err());
        assert!(crop_bounds((2, 2), [f32::NAN, 0.0, 1.0, 1.0]).is_err());
    }

    #[test]
    fn packing_rotation_and_separate_alpha_are_explicit() {
        let image = RgbaImage::from_fn(2, 3, |x, y| Rgba([(y * 2 + x) as u8, 0, 0, 100]));
        let rotated = unpack_rotation(image.clone(), 4).unwrap();
        assert_eq!(rotated.dimensions(), (3, 2));
        assert_eq!(rotated.get_pixel(0, 0)[0], 4);
        assert_eq!(rotated.get_pixel(2, 1)[0], 1);
        let flipped = unpack_rotation(image.clone(), 1).unwrap();
        assert_eq!(flipped.get_pixel(0, 0)[0], 1);
        assert!(unpack_rotation(image.clone(), 15).is_err());
        let mut with_alpha = image;
        let alpha = RgbaImage::from_pixel(2, 3, Rgba([17, 99, 201, 255]));
        apply_alpha(&mut with_alpha, &alpha).unwrap();
        assert_eq!(with_alpha.get_pixel(1, 2).0, [5, 0, 0, 17]);
        assert!(apply_alpha(&mut with_alpha, &RgbaImage::new(1, 1)).is_err());
    }
}
