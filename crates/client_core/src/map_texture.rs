//! Load source mip levels without regenerating or filtering them at runtime.

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    image::ImageLoaderSettings,
    prelude::*,
    render::render_resource::{Extent3d, TextureDataOrder, TextureDimension, TextureFormat},
};
use midnight_map_data::{ArchiveError, read_rgba_mip_archive};

#[derive(TypePath)]
struct SourceMipImageLoader;

fn mip_image(bytes: &[u8], settings: &ImageLoaderSettings) -> Result<Image, ArchiveError> {
    let chain = read_rgba_mip_archive(bytes)?;
    if settings.array_layout.is_some() {
        return Err(ArchiveError::Invalid(
            "source mip images are single 2D textures".into(),
        ));
    }
    let format = if settings.is_srgb {
        TextureFormat::Rgba8UnormSrgb
    } else {
        TextureFormat::Rgba8Unorm
    };
    if settings
        .texture_format
        .is_some_and(|override_format| override_format != format)
    {
        return Err(ArchiveError::Invalid(
            "source mip image format override would reinterpret RGBA pixels".into(),
        ));
    }
    let mut image = Image::new_uninit(
        Extent3d {
            width: chain.width,
            height: chain.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        format,
        settings.asset_usage,
    );
    image.texture_descriptor.mip_level_count = chain.mip_count;
    image.data = Some(chain.pixels.to_vec());
    image.data_order = TextureDataOrder::MipMajor;
    image.sampler = settings.sampler.clone();
    Ok(image)
}

impl AssetLoader for SourceMipImageLoader {
    type Asset = Image;
    type Settings = ImageLoaderSettings;
    type Error = ArchiveError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        _context: &mut LoadContext<'_>,
    ) -> Result<Image, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        mip_image(&bytes, settings)
    }

    fn extensions(&self) -> &[&str] {
        &["msrgba"]
    }
}

pub struct SourceMipImagePlugin;

impl Plugin for SourceMipImagePlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(SourceMipImageLoader);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
    use midnight_map_data::write_rgba_mip_archive;

    #[test]
    fn image_keeps_source_mips_color_space_and_sampler() {
        let pixels = [
            0x12, 0x34, 0x56, 0x78, 0x90, 0xab, 0xcd, 0xef, 0x21, 0x43, 0x65, 0x87,
        ];
        let mut archive = Vec::new();
        write_rgba_mip_archive(&mut archive, 2, 1, 2, &pixels).unwrap();
        for (is_srgb, format) in [
            (false, TextureFormat::Rgba8Unorm),
            (true, TextureFormat::Rgba8UnormSrgb),
        ] {
            let settings = ImageLoaderSettings {
                is_srgb,
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    mipmap_filter: ImageFilterMode::Linear,
                    ..default()
                }),
                ..default()
            };
            let image = mip_image(&archive, &settings).unwrap();
            assert_eq!(image.texture_descriptor.mip_level_count, 2);
            assert_eq!(image.texture_descriptor.format, format);
            assert_eq!(image.data.as_deref(), Some(pixels.as_slice()));
            let ImageSampler::Descriptor(sampler) = image.sampler else {
                panic!("source sampler was lost");
            };
            assert_eq!(sampler.mipmap_filter, ImageFilterMode::Linear);
        }
    }
}
