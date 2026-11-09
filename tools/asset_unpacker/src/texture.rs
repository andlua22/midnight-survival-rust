//! Decode shipped texture bytes with the installed Rust codecs.
//!
//! unity-asset-decode 0.2 does not dispatch ASTC_RGB, and its ASTC path treats
//! the codec's BGRA words as RGBA. Keep channel order explicit here.

use image::RgbaImage;
use unity_asset_decode::texture::{Texture2D, TextureDecoder};

use crate::{fields::invalid, unity::Error};

fn astc_block(format: i32) -> Option<usize> {
    match format {
        48..=53 => Some([4, 5, 6, 8, 10, 12][(format - 48) as usize]),
        54..=59 => Some([4, 5, 6, 8, 10, 12][(format - 54) as usize]),
        _ => None,
    }
}

pub fn decode(texture: &Texture2D) -> Result<RgbaImage, Error> {
    let block = astc_block(texture.format as i32);
    let Some(block) = block else {
        return Ok(TextureDecoder::new().decode(texture)?);
    };
    let width = usize::try_from(texture.width)?;
    let height = usize::try_from(texture.height)?;
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(invalid("invalid ASTC image dimensions"));
    }
    let mut pixels = vec![
        0u32;
        width
            .checked_mul(height)
            .ok_or_else(|| invalid("texture dimensions overflow"))?
    ];
    texture2ddecoder::decode_astc(
        &texture.image_data,
        width,
        height,
        block,
        block,
        &mut pixels,
    )
    .map_err(invalid)?;
    let rgba = pixels
        .into_iter()
        .flat_map(|pixel| {
            let [b, g, r, a] = pixel.to_le_bytes();
            [r, g, b, a]
        })
        .collect();
    RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or_else(|| invalid("decoded texture size differs from dimensions"))
}

/// Decode every shipped mip independently. These levels are source data,
/// not downsampled copies of the largest decoded image.
pub fn decode_mips(texture: &Texture2D) -> Result<Vec<RgbaImage>, Error> {
    let width = u32::try_from(texture.width)?;
    let height = u32::try_from(texture.height)?;
    let mip_count = u32::try_from(texture.mip_count)?;
    midnight_map_data::rgba_mip_byte_len(width, height, mip_count)?;
    if texture.image_count != 1
        || texture.texture_dimension != 2
        || texture.mips_stripped.is_some_and(|count| count != 0)
    {
        return Err(invalid(
            "source mip decoding requires an unstripped single 2D image",
        ));
    }
    let block = astc_block(texture.format as i32);
    if block.is_none() && !texture.format.is_supported() {
        return Err(invalid(format!(
            "source mip size is unsupported for {:?}",
            texture.format
        )));
    }
    let mut offset = 0usize;
    let mut images = Vec::new();
    for level in 0..mip_count {
        let width = (width >> level).max(1);
        let height = (height >> level).max(1);
        let size = if let Some(block) = block {
            (width as usize).div_ceil(block) * (height as usize).div_ceil(block) * 16
        } else {
            texture.format.calculate_data_size(width, height) as usize
        };
        let end = offset
            .checked_add(size)
            .ok_or_else(|| invalid("source mip offset overflow"))?;
        let data = texture
            .image_data
            .get(offset..end)
            .ok_or_else(|| invalid(format!("{}: truncated source mip {level}", texture.name)))?;
        let mut mip = Texture2D::new(
            texture.name.clone(),
            width as i32,
            height as i32,
            texture.format,
        );
        mip.image_data = data.to_vec();
        let mut image = decode(&mip)?;
        image::imageops::flip_vertical_in_place(&mut image);
        images.push(image);
        offset = end;
    }
    if offset != texture.image_data.len() {
        return Err(invalid(format!(
            "{}: source mip payload has unexplained trailing bytes",
            texture.name
        )));
    }
    Ok(images)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unity_asset_decode::texture::TextureFormat;

    #[test]
    fn asymmetric_astc_color_preserves_rgba_and_partial_edge_blocks() {
        // Standard LDR void-extent constant-colour block, with distinct RGBA.
        let block = vec![
            0xfc, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x11, 0x11, 0x53, 0x53, 0xa7, 0xa7,
            0xe9, 0xe9,
        ];
        for format in [TextureFormat::ASTC_RGB_6x6, TextureFormat::ASTC_RGBA_6x6] {
            let mut texture = Texture2D::new("asymmetric".into(), 5, 3, format);
            texture.image_data = block.clone();
            let image = decode(&texture).unwrap();
            assert_eq!(image.dimensions(), (5, 3));
            assert!(
                image
                    .pixels()
                    .all(|pixel| pixel.0 == [0x11, 0x53, 0xa7, 0xe9])
            );
            texture.image_data.pop();
            assert!(decode(&texture).is_err());
        }
    }

    #[test]
    fn source_astc_mips_keep_distinct_colors_and_partial_blocks() {
        let block = |color: [u8; 4]| {
            let mut bytes = vec![0xfc, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
            bytes.extend(color.into_iter().flat_map(|channel| [channel, channel]));
            bytes
        };
        let colors = [
            [0x11, 0x53, 0xa7, 0xe9],
            [0xd2, 0x63, 0x14, 0x85],
            [0x31, 0x42, 0x53, 0x64],
        ];
        let mut texture = Texture2D::new("source mips".into(), 5, 3, TextureFormat::ASTC_RGB_6x6);
        texture.mip_count = 3;
        texture.image_data = colors.into_iter().flat_map(block).collect();
        let images = decode_mips(&texture).unwrap();
        for ((image, dimensions), color) in images.iter().zip([(5, 3), (2, 1), (1, 1)]).zip(colors)
        {
            assert_eq!(image.dimensions(), dimensions);
            assert!(image.pixels().all(|pixel| pixel.0 == color));
        }
        texture.image_data.pop();
        assert!(decode_mips(&texture).is_err());
    }

    #[test]
    fn rgba_mips_flip_each_source_level_without_regenerating_pixels() {
        let mut texture = Texture2D::new("source rgba".into(), 2, 2, TextureFormat::RGBA32);
        texture.mip_count = 2;
        texture.image_data = [1, 2, 3, 4, 77]
            .into_iter()
            .flat_map(|red| [red, 0, 0, 255])
            .collect();
        let images = decode_mips(&texture).unwrap();
        assert_eq!(
            images[0]
                .pixels()
                .map(|pixel| pixel.0[0])
                .collect::<Vec<_>>(),
            [3, 4, 1, 2]
        );
        assert_eq!(images[1].get_pixel(0, 0).0, [77, 0, 0, 255]);
        texture.image_data.push(0);
        assert!(decode_mips(&texture).is_err());
    }
}
