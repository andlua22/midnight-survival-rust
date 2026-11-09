//! Decode shipped texture bytes with the installed Rust codecs.
//!
//! unity-asset-decode 0.2 does not dispatch ASTC_RGB, and its ASTC path treats
//! the codec's BGRA words as RGBA. Keep channel order explicit here.

use image::RgbaImage;
use unity_asset_decode::texture::{Texture2D, TextureDecoder};

use crate::{fields::invalid, unity::Error};

pub fn decode(texture: &Texture2D) -> Result<RgbaImage, Error> {
    let format = texture.format as i32;
    let block = match format {
        48..=53 => Some([4, 5, 6, 8, 10, 12][(format - 48) as usize]),
        54..=59 => Some([4, 5, 6, 8, 10, 12][(format - 54) as usize]),
        _ => None,
    };
    let Some(block) = block else { return Ok(TextureDecoder::new().decode(texture)?); };
    let width = usize::try_from(texture.width)?;
    let height = usize::try_from(texture.height)?;
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(invalid("invalid ASTC image dimensions"));
    }
    let mut pixels = vec![0u32; width.checked_mul(height).ok_or_else(|| invalid("texture dimensions overflow"))?];
    texture2ddecoder::decode_astc(&texture.image_data, width, height, block, block, &mut pixels)
        .map_err(invalid)?;
    let rgba = pixels.into_iter().flat_map(|pixel| {
        let [b, g, r, a] = pixel.to_le_bytes();
        [r, g, b, a]
    }).collect();
    RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or_else(|| invalid("decoded texture size differs from dimensions"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use unity_asset_decode::texture::TextureFormat;

    #[test]
    fn asymmetric_astc_color_preserves_rgba_and_partial_edge_blocks() {
        // Standard LDR void-extent constant-colour block, with distinct RGBA.
        let block = vec![0xfc, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0x11, 0x11, 0x53, 0x53, 0xa7, 0xa7, 0xe9, 0xe9];
        for format in [TextureFormat::ASTC_RGB_6x6, TextureFormat::ASTC_RGBA_6x6] {
            let mut texture = Texture2D::new("asymmetric".into(), 5, 3, format);
            texture.image_data = block.clone();
            let image = decode(&texture).unwrap();
            assert_eq!(image.dimensions(), (5, 3));
            assert!(image.pixels().all(|pixel| pixel.0 == [0x11, 0x53, 0xa7, 0xe9]));
            texture.image_data.pop();
            assert!(decode(&texture).is_err());
        }
    }
}
