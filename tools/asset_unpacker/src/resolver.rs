use std::{collections::BTreeMap, path::Path, sync::Arc};

use glam::{Mat4, Quat, Vec3};
use midnight_map_data::{
    FileIdentity, SourceMaterial, SourceObject, TextureAsset, TextureBinding, TextureSampler,
};
use sha2::{Digest, Sha256};
use unity_asset_binary::{bundle::AssetBundle, object::UnityObject};
use unity_asset_core::UnityValue;
use unity_asset_decode::texture::Texture2DConverter;

use crate::{
    fields::{array, field, integer, invalid, number, pairs, property, text, vector},
    pass1_index::GlobalIndex,
    unity::{Apk, Error, cab_name, read_object, texture_version},
};

/// Object identity is always (serialized file, signed PathID), never PathID alone.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectId {
    pub bundle: String,
    pub cab: String,
    pub path_id: i64,
}

#[derive(Clone)]
struct ShaderInfo {
    name: String,
    defaults: BTreeMap<String, [f32; 4]>,
    texture_defaults: BTreeMap<String, String>,
}

impl ObjectId {
    pub fn source(&self) -> SourceObject {
        SourceObject {
            bundle: self.bundle.clone(),
            cab: self.cab.clone(),
            path_id: self.path_id.to_string(),
        }
    }
}

pub struct Resolver<'a> {
    pub apk: Apk,
    index: &'a GlobalIndex,
    bundles: BTreeMap<String, Arc<AssetBundle>>,
    textures: BTreeMap<ObjectId, TextureAsset>,
    shaders: BTreeMap<ObjectId, ShaderInfo>,
}

impl<'a> Resolver<'a> {
    pub fn new(apk: &Path, index: &'a GlobalIndex) -> Result<Self, Error> {
        Ok(Self {
            apk: Apk::open(apk)?,
            index,
            bundles: BTreeMap::new(),
            textures: BTreeMap::new(),
            shaders: BTreeMap::new(),
        })
    }

    pub fn bundle(&mut self, member: &str) -> Result<Arc<AssetBundle>, Error> {
        if let Some(bundle) = self.bundles.get(member) {
            return Ok(bundle.clone());
        }
        let bundle = Arc::new(self.apk.bundle(member)?);
        self.bundles.insert(member.to_owned(), bundle.clone());
        Ok(bundle)
    }

    pub fn object(&mut self, id: &ObjectId) -> Result<UnityObject, Error> {
        let bundle = self.bundle(&id.bundle)?;
        let asset_index = bundle
            .asset_names
            .iter()
            .position(|name| cab_name(name) == id.cab)
            .ok_or_else(|| invalid(format!("{}: missing CAB {}", id.bundle, id.cab)))?;
        let asset = &bundle.assets[asset_index];
        let info = asset
            .objects
            .iter()
            .find(|object| object.path_id == id.path_id)
            .ok_or_else(|| invalid(format!("{}:{}: missing object", id.cab, id.path_id)))?;
        read_object(asset, info)
    }

    pub fn resolve(
        &mut self,
        owner: &ObjectId,
        pointer: &UnityValue,
    ) -> Result<Option<ObjectId>, Error> {
        let path_id = integer(field(pointer, "m_PathID")?)?;
        if path_id == 0 {
            return Ok(None);
        }
        let file_id = integer(field(pointer, "m_FileID")?)?;
        if file_id == 0 {
            return Ok(Some(ObjectId {
                path_id,
                ..owner.clone()
            }));
        }
        let bundle = self.bundle(&owner.bundle)?;
        let asset_index = bundle
            .asset_names
            .iter()
            .position(|name| cab_name(name) == owner.cab)
            .ok_or_else(|| invalid("pointer owner CAB is missing"))?;
        let external_index =
            usize::try_from(file_id - 1).map_err(|_| invalid("negative external file ID"))?;
        let external = bundle.assets[asset_index]
            .externals
            .get(external_index)
            .ok_or_else(|| invalid("external file ID is out of range"))?;
        let cab = cab_name(&external.path).to_owned();
        let member = if bundle.asset_names.iter().any(|name| cab_name(name) == cab) {
            owner.bundle.clone()
        } else {
            self.index
                .cabs
                .get(&cab)
                .cloned()
                .ok_or_else(|| invalid(format!("unresolved external CAB {cab}")))?
        };
        Ok(Some(ObjectId {
            bundle: member,
            cab,
            path_id,
        }))
    }

    pub fn required_ref(
        &mut self,
        owner: &ObjectId,
        pointer: &UnityValue,
    ) -> Result<ObjectId, Error> {
        self.resolve(owner, pointer)?
            .ok_or_else(|| invalid("unexpected null object reference"))
    }

    pub fn transform(
        &mut self,
        id: &ObjectId,
        ancestors: &mut Vec<ObjectId>,
    ) -> Result<Mat4, Error> {
        if ancestors.len() >= 128 || ancestors.contains(id) {
            return Err(invalid("transform hierarchy cycle/depth limit"));
        }
        ancestors.push(id.clone());
        let object = self.object(id)?;
        let position = vector(property(&object, "m_LocalPosition")?, ["x", "y", "z"])?;
        let rotation = vector(property(&object, "m_LocalRotation")?, ["x", "y", "z", "w"])?;
        let scale = vector(property(&object, "m_LocalScale")?, ["x", "y", "z"])?;
        let rotation = Quat::from_array(rotation);
        if rotation.length_squared() < 1e-8 {
            return Err(invalid("zero source quaternion"));
        }
        let local = Mat4::from_scale_rotation_translation(
            Vec3::from_array(scale),
            rotation.normalize(),
            Vec3::from_array(position),
        );
        let world = if let Some(parent) = self.resolve(id, property(&object, "m_Father")?)? {
            self.transform(&parent, ancestors)? * local
        } else {
            local
        };
        ancestors.pop();
        Ok(world)
    }

    fn shader_info(&mut self, shader: &ObjectId) -> Result<ShaderInfo, Error> {
        if let Some(info) = self.shaders.get(shader) {
            return Ok(info.clone());
        }
        let object = self.object(shader)?;
        let form = property(&object, "m_ParsedForm")?;
        let name = text(field(form, "m_Name")?)?.to_owned();
        let mut defaults = BTreeMap::new();
        let mut texture_defaults = BTreeMap::new();
        for property in array(field(field(form, "m_PropInfo")?, "m_Props")?)? {
            let name = text(field(property, "m_Name")?)?.to_owned();
            let mut values = [0.0; 4];
            for (index, value) in values.iter_mut().enumerate() {
                *value = number(field(property, &format!("m_DefValue[{index}]"))?)?;
            }
            if integer(field(property, "m_Type")?)? == 4 {
                texture_defaults.insert(
                    name.clone(),
                    text(field(field(property, "m_DefTexture")?, "m_DefaultName")?)?.to_owned(),
                );
            }
            defaults.insert(name, values);
        }
        let info = ShaderInfo {
            name,
            defaults,
            texture_defaults,
        };
        self.shaders.insert(shader.clone(), info.clone());
        Ok(info)
    }

    pub fn texture(&mut self, id: &ObjectId, output: &Path) -> Result<TextureAsset, Error> {
        if let Some(texture) = self.textures.get(id) {
            return Ok(texture.clone());
        }
        let object = self.object(id)?;
        if object.info.type_id != 28 {
            return Err(invalid("material sampler does not reference Texture2D"));
        }
        let bundle = self.bundle(&id.bundle)?;
        let asset_index = bundle
            .asset_names
            .iter()
            .position(|name| cab_name(name) == id.cab)
            .ok_or_else(|| invalid("texture CAB missing"))?;
        let version = texture_version(&bundle.assets[asset_index].unity_version)?;
        let mut texture = Texture2DConverter::new(version).from_unity_object(&object)?;
        if texture.image_data.is_empty() {
            let stream = &texture.stream_info;
            if stream.size == 0 || stream.path.is_empty() {
                return Err(invalid("texture has neither embedded nor streamed bytes"));
            }
            let file = bundle
                .files
                .iter()
                .find(|file| {
                    file.name == stream.path || cab_name(&file.name) == cab_name(&stream.path)
                })
                .ok_or_else(|| invalid(format!("missing texture stream {}", stream.path)))?;
            let data = bundle.extract_file_slice(file)?;
            let begin = usize::try_from(stream.offset)?;
            let end = begin
                .checked_add(usize::try_from(stream.size)?)
                .ok_or_else(|| invalid("texture stream overflow"))?;
            texture.image_data = data
                .get(begin..end)
                .ok_or_else(|| invalid("truncated texture stream"))?
                .to_vec();
            texture.data_size = i32::try_from(texture.image_data.len())?;
        }
        let images = crate::texture::decode_mips(&texture)?;
        let image = images
            .first()
            .ok_or_else(|| invalid("source texture has no mip levels"))?;
        let name = texture.name.clone();
        let identity = format!("{}:{}", id.cab, id.path_id);
        let filename = format!("textures/{:x}.png", Sha256::digest(identity.as_bytes()));
        let path = output.join(&filename);
        std::fs::create_dir_all(
            path.parent()
                .ok_or_else(|| invalid("texture output has no parent"))?,
        )?;
        image.save(&path)?;
        let hash = format!("{:x}", Sha256::digest(std::fs::read(&path)?));
        let gpu_path = filename.replace(".png", ".msrgba");
        let pixels: Vec<_> = images
            .iter()
            .flat_map(|image| image.as_raw().iter().copied())
            .collect();
        midnight_map_data::write_rgba_mip_archive(
            &mut std::fs::File::create(output.join(&gpu_path))?,
            image.width(),
            image.height(),
            u32::try_from(texture.mip_count)?,
            &pixels,
        )?;
        let gpu = Some(FileIdentity {
            path: gpu_path.clone(),
            sha256: format!(
                "{:x}",
                Sha256::digest(std::fs::read(output.join(gpu_path))?)
            ),
        });
        let encoded_path = filename.replace(".png", ".texture.bin");
        std::fs::write(output.join(&encoded_path), &texture.image_data)?;
        let encoded = FileIdentity {
            path: encoded_path,
            sha256: format!("{:x}", Sha256::digest(&texture.image_data)),
        };
        let settings = property(&object, "m_TextureSettings")?;
        let sampler = TextureSampler {
            filter_mode: i32::try_from(integer(field(settings, "m_FilterMode")?)?)?,
            wrap: ["m_WrapU", "m_WrapV", "m_WrapW"]
                .map(|name| {
                    i32::try_from(integer(field(settings, name)?)?).map_err(|error| error.into())
                })
                .into_iter()
                .collect::<Result<Vec<_>, Error>>()?
                .try_into()
                .map_err(|_| invalid("invalid sampler axes"))?,
            aniso: i32::try_from(integer(field(settings, "m_Aniso")?)?)?,
            mip_bias: number(field(settings, "m_MipBias")?)?,
        };
        let extracted = TextureAsset {
            file: FileIdentity {
                path: filename,
                sha256: hash,
            },
            gpu,
            source: id.source(),
            name,
            width: image.width(),
            height: image.height(),
            unity_format: integer(property(&object, "m_TextureFormat")?)? as i32,
            unity_color_space: integer(property(&object, "m_ColorSpace")?)? as i32,
            unity_version: bundle.assets[asset_index].unity_version.clone(),
            mip_count: u32::try_from(integer(property(&object, "m_MipCount")?)?)?,
            sampler,
            encoded,
        };
        self.textures.insert(id.clone(), extracted.clone());
        Ok(extracted)
    }

    pub fn material(&mut self, id: &ObjectId, output: &Path) -> Result<SourceMaterial, Error> {
        let object = self.object(id)?;
        let name = text(property(&object, "m_Name")?)?.to_owned();
        let shader = self.required_ref(id, property(&object, "m_Shader")?)?;
        let shader_info = self.shader_info(&shader)?;
        let saved = property(&object, "m_SavedProperties")?;
        let floats = pairs(field(saved, "m_Floats")?)?
            .into_iter()
            .map(|(name, value)| Ok((name.to_owned(), number(value)?)))
            .collect::<Result<_, Error>>()?;
        let colors = pairs(field(saved, "m_Colors")?)?
            .into_iter()
            .map(|(name, value)| Ok((name.to_owned(), vector(value, ["r", "g", "b", "a"])?)))
            .collect::<Result<_, Error>>()?;
        let mut textures = BTreeMap::new();
        for (name, environment) in pairs(field(saved, "m_TexEnvs")?)? {
            let texture = self
                .resolve(id, field(environment, "m_Texture")?)?
                .map(|texture| self.texture(&texture, output))
                .transpose()?;
            textures.insert(
                name.to_owned(),
                TextureBinding {
                    texture,
                    scale: vector(field(environment, "m_Scale")?, ["x", "y"])?,
                    offset: vector(field(environment, "m_Offset")?, ["x", "y"])?,
                },
            );
        }
        let valid_keywords = array(property(&object, "m_ValidKeywords")?)?
            .iter()
            .map(|value| Ok(text(value)?.to_owned()))
            .collect::<Result<_, Error>>()?;
        let render_queue = i32::try_from(integer(property(&object, "m_CustomRenderQueue")?)?)?;
        let material = SourceMaterial {
            source: id.source(),
            name,
            shader: shader.source(),
            shader_name: shader_info.name,
            textures,
            floats,
            colors,
            shader_defaults: shader_info.defaults,
            texture_defaults: shader_info.texture_defaults,
            valid_keywords,
            render_queue,
        };
        if material.terrain_layer_count().is_some() {
            for property in material.terrain_sampler_names() {
                if material
                    .textures
                    .get(&property)
                    .and_then(|binding| binding.texture.as_ref())
                    .is_none()
                    && material.default_texture_color(&property).is_none()
                {
                    return Err(invalid(format!(
                        "{}: required terrain sampler {property} is unresolved/null",
                        material.name
                    )));
                }
            }
        }
        Ok(material)
    }
}
