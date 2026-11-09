//! Uncompressed RGBA mip levels decoded from shipped texture payloads.

use std::io::Write;

use crate::{ArchiveError, invalid};

pub const RGBA_MIP_MAGIC: &[u8; 8] = b"MSRGBA01";

#[derive(Debug, PartialEq)]
pub struct RgbaMipChain<'a> {
    pub width: u32,
    pub height: u32,
    pub mip_count: u32,
    /// Top-left RGBA8 pixels, largest mip first, with no padding.
    pub pixels: &'a [u8],
}

pub fn rgba_mip_byte_len(width: u32, height: u32, mip_count: u32) -> Result<usize, ArchiveError> {
    if width == 0 || height == 0 || width > 16_384 || height > 16_384 {
        return Err(invalid("invalid RGBA mip dimensions"));
    }
    let maximum = u32::BITS - width.max(height).leading_zeros();
    if mip_count == 0 || mip_count > maximum {
        return Err(invalid("invalid RGBA mip count"));
    }
    (0..mip_count).try_fold(0usize, |bytes, level| {
        let width = (width >> level).max(1) as usize;
        let height = (height >> level).max(1) as usize;
        width
            .checked_mul(height)
            .and_then(|pixels| pixels.checked_mul(4))
            .and_then(|size| bytes.checked_add(size))
            .ok_or_else(|| invalid("RGBA mip byte count overflow"))
    })
}

pub fn read_rgba_mip_archive(bytes: &[u8]) -> Result<RgbaMipChain<'_>, ArchiveError> {
    let header = bytes
        .get(..20)
        .ok_or_else(|| invalid("truncated RGBA mip header"))?;
    if &header[..8] != RGBA_MIP_MAGIC {
        return Err(invalid("unsupported RGBA mip archive version"));
    }
    let number = |offset| {
        u32::from_le_bytes(
            header[offset..offset + 4]
                .try_into()
                .expect("bounded header word"),
        )
    };
    let (width, height, mip_count) = (number(8), number(12), number(16));
    let expected = rgba_mip_byte_len(width, height, mip_count)?;
    let pixels = &bytes[20..];
    if pixels.len() != expected {
        return Err(invalid(
            "RGBA mip payload length differs from declared levels",
        ));
    }
    Ok(RgbaMipChain {
        width,
        height,
        mip_count,
        pixels,
    })
}

pub fn write_rgba_mip_archive(
    writer: &mut impl Write,
    width: u32,
    height: u32,
    mip_count: u32,
    pixels: &[u8],
) -> Result<(), ArchiveError> {
    if pixels.len() != rgba_mip_byte_len(width, height, mip_count)? {
        return Err(invalid(
            "RGBA mip payload length differs from declared levels",
        ));
    }
    writer.write_all(RGBA_MIP_MAGIC)?;
    for number in [width, height, mip_count] {
        writer.write_all(&number.to_le_bytes())?;
    }
    writer.write_all(pixels)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_distinct_source_levels_with_non_square_dimensions() {
        let pixels: Vec<_> = [[0x11, 0x53, 0xa7, 0xe9]; 15]
            .into_iter()
            .chain([[0xf1, 0x27, 0x73, 0x99]; 2])
            .chain([[0x01, 0x12, 0x23, 0x34]])
            .flatten()
            .collect();
        let mut archive = Vec::new();
        write_rgba_mip_archive(&mut archive, 5, 3, 3, &pixels).unwrap();
        let loaded = read_rgba_mip_archive(&archive).unwrap();
        assert_eq!((loaded.width, loaded.height, loaded.mip_count), (5, 3, 3));
        assert_eq!(loaded.pixels, pixels);
        assert_eq!(rgba_mip_byte_len(5, 3, 3).unwrap(), 72);
    }

    #[test]
    fn refuses_missing_levels_trailing_bytes_and_invalid_headers() {
        let mut archive = Vec::new();
        write_rgba_mip_archive(&mut archive, 2, 1, 2, &[17; 12]).unwrap();
        for end in 0..archive.len() {
            assert!(read_rgba_mip_archive(&archive[..end]).is_err());
        }
        archive.push(0);
        assert!(read_rgba_mip_archive(&archive).is_err());
        for (width, height, count) in [(0, 1, 1), (1, 0, 1), (16_385, 1, 1), (2, 2, 0), (2, 2, 3)] {
            assert!(rgba_mip_byte_len(width, height, count).is_err());
        }
        archive[0] ^= 1;
        assert!(read_rgba_mip_archive(&archive).is_err());
    }
}
