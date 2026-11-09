//! Retain large Font byte arrays without raising the parser's general array limit.

use unity_asset_binary::{
    object::{ObjectHandle, UnityObject},
    reader::{BinaryReader, ByteOrder},
    typetree::{TypeTree, TypeTreeParseMode, TypeTreeParseOptions, TypeTreeSerializer},
};
use unity_asset_core::UnityValue;

use crate::{
    fields::invalid,
    resolver::{ObjectId, Resolver},
    unity::{Error, cab_name},
};

const MAX_FONT_OBJECT_BYTES: usize = 64 * 1024 * 1024;

fn parse_with_font_blob(
    tree: &TypeTree,
    bytes: &[u8],
    order: ByteOrder,
) -> Result<UnityValue, Error> {
    if bytes.len() > MAX_FONT_OBJECT_BYTES {
        return Err(invalid("Font object exceeds the bounded blob reader limit"));
    }
    let root = tree
        .nodes
        .first()
        .ok_or_else(|| invalid("Font has no shipped type tree"))?;
    if root.type_name != "Font" {
        return Err(invalid("large blob reader requires the Font type tree"));
    }
    let matches: Vec<_> = root
        .children
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name == "m_FontData")
        .collect();
    let [(field_index, field)] = matches.as_slice() else {
        return Err(invalid(
            "Font must declare exactly one root m_FontData field",
        ));
    };
    let array = field
        .children
        .iter()
        .find(|child| child.type_name == "Array")
        .ok_or_else(|| invalid("Font data is not a declared byte array"))?;
    let element = array
        .children
        .get(1)
        .ok_or_else(|| invalid("Font byte array has no element type"))?;
    if array.children.len() != 2
        || !matches!(element.type_name.as_str(), "UInt8" | "char")
        || element.byte_size != 1
        || !element.children.is_empty()
    {
        return Err(invalid(
            "Font blob reader only accepts a declared one-byte element",
        ));
    }
    let options = TypeTreeParseOptions {
        mode: TypeTreeParseMode::Strict,
    };
    let serializer = TypeTreeSerializer::new(tree);
    let mut prefix = BinaryReader::new(bytes, order);
    let parsed = serializer.parse_object_prefix_detailed(&mut prefix, options, *field_index)?;
    if root.children[..*field_index]
        .iter()
        .any(|field| !field.name.is_empty() && !parsed.properties.contains_key(&field.name))
    {
        return Err(invalid("Font prefix is truncated"));
    }
    let count_offset = usize::try_from(prefix.position())?;
    let count =
        usize::try_from(prefix.read_i32()?).map_err(|_| invalid("negative Font byte count"))?;
    if array.children[0].is_aligned() {
        prefix.align()?;
    }
    let begin = usize::try_from(prefix.position())?;
    let end = begin
        .checked_add(count)
        .ok_or_else(|| invalid("Font blob offset overflow"))?;
    let data = bytes
        .get(begin..end)
        .ok_or_else(|| invalid("Font blob extends beyond its source object"))?;
    let aligned = array.is_aligned() || field.is_aligned();
    let suffix = if aligned {
        end.next_multiple_of(4)
    } else {
        end
    };
    let remaining = bytes
        .get(suffix..)
        .ok_or_else(|| invalid("Font alignment extends beyond its source object"))?;

    // Parse the exact prefix/suffix with a zero-length placeholder. The original
    // field bytes are restored afterwards, and the source object remains intact.
    let mut compact = bytes[..begin].to_vec();
    compact[count_offset..count_offset + 4].copy_from_slice(&[0; 4]);
    if aligned {
        compact.resize(begin.next_multiple_of(4), 0);
    }
    compact.extend_from_slice(remaining);
    let mut reader = BinaryReader::new(&compact, order);
    let mut result = serializer.parse_object_detailed(&mut reader, options)?;
    if reader.remaining() != 0
        || root
            .children
            .iter()
            .any(|field| !field.name.is_empty() && !result.properties.contains_key(&field.name))
    {
        return Err(invalid(
            "Font fields do not consume the complete bounded source layout",
        ));
    }
    match result.properties.get("m_FontData") {
        Some(UnityValue::Bytes(placeholder)) if placeholder.is_empty() => {}
        _ => {
            return Err(invalid(
                "Font placeholder did not parse as an empty byte array",
            ));
        }
    }
    result
        .properties
        .insert("m_FontData".into(), UnityValue::Bytes(data.to_vec()));
    Ok(UnityValue::Object(result.properties))
}

pub(super) fn read(resolver: &mut Resolver<'_>, id: &ObjectId) -> Result<UnityObject, Error> {
    let original_error = match resolver.object(id) {
        Ok(object) => return Ok(object),
        Err(error) => error,
    };
    if !original_error.to_string().contains("Array size too large") {
        return Err(original_error);
    }
    let bundle = resolver.bundle(&id.bundle)?;
    let asset_index = bundle
        .asset_names
        .iter()
        .position(|name| cab_name(name) == id.cab)
        .ok_or_else(|| invalid("Font source CAB is absent"))?;
    let asset = &bundle.assets[asset_index];
    let info = asset
        .objects
        .iter()
        .find(|object| object.path_id == id.path_id)
        .ok_or_else(|| invalid("Font source object is absent"))?;
    if info.type_id != 128 || !asset.enable_type_tree {
        return Err(original_error);
    }
    let serialized = if info.type_index >= 0 {
        asset.types.get(info.type_index as usize)
    } else {
        asset
            .types
            .iter()
            .find(|serialized| serialized.class_id == 128)
    }
    .ok_or_else(|| invalid("Font serialized type is absent"))?;
    let handle = ObjectHandle::new(asset, info);
    let raw = handle.raw_data()?;
    let UnityValue::Object(properties) =
        parse_with_font_blob(&serialized.type_tree, raw, asset.header.byte_order())?
    else {
        return Err(invalid("Font fields did not form an object"));
    };
    // Lenient construction retains the original raw source bytes. Only its
    // temporary preview properties are replaced by the fully parsed fields.
    let mut object = handle.read_with_options(TypeTreeParseOptions {
        mode: TypeTreeParseMode::Lenient,
    })?;
    object.class.properties_mut().clear();
    object.class.update_properties(properties);
    if object.raw_data() != raw {
        return Err(invalid("Font raw object identity was not retained"));
    }
    Ok(object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unity_asset_binary::typetree::TypeTreeNode;

    fn fixture(count: usize, order: ByteOrder) -> (TypeTree, Vec<u8>) {
        let node =
            |kind: &str, name: &str, size| TypeTreeNode::with_info(kind.into(), name.into(), size);
        let mut array = node("Array", "Array", -1);
        array.meta_flags = 0x4000;
        array.children = vec![node("int", "size", 4), node("UInt8", "data", 1)];
        let mut data = node("vector", "m_FontData", -1);
        data.children.push(array);
        let mut root = node("Font", "Base", -1);
        root.children = vec![node("int", "before", 4), data, node("int", "after", 4)];
        let mut tree = TypeTree::new();
        tree.nodes.push(root);
        let word = |value: u32| {
            if order == ByteOrder::Little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            }
        };
        let mut bytes = word(0x1234).to_vec();
        bytes.extend_from_slice(&word(count as u32));
        bytes.extend(std::iter::repeat_n(0x57, count));
        bytes.resize(bytes.len().next_multiple_of(4), 0);
        bytes.extend_from_slice(&word(0x5678));
        (tree, bytes)
    }

    #[test]
    fn large_font_bytes_and_fields_survive_without_a_global_limit_change() {
        for (count, order) in [(1_000_003, ByteOrder::Little), (3, ByteOrder::Big)] {
            let (tree, bytes) = fixture(count, order);
            let UnityValue::Object(fields) = parse_with_font_blob(&tree, &bytes, order).unwrap()
            else {
                panic!("fields missing");
            };
            assert_eq!(fields["before"].as_i64(), Some(0x1234));
            assert_eq!(fields["after"].as_i64(), Some(0x5678));
            let UnityValue::Bytes(font) = &fields["m_FontData"] else {
                panic!("font bytes missing");
            };
            assert_eq!(font, &vec![0x57; count]);
        }
    }

    #[test]
    fn font_reader_rejects_truncated_data_and_incorrect_element_types() {
        let (mut tree, mut bytes) = fixture(3, ByteOrder::Little);
        bytes.truncate(10);
        assert!(parse_with_font_blob(&tree, &bytes, ByteOrder::Little).is_err());
        let (_, bytes) = fixture(3, ByteOrder::Little);
        tree.nodes[0].children[1].children[0].children[1].type_name = "float".into();
        assert!(parse_with_font_blob(&tree, &bytes, ByteOrder::Little).is_err());
    }
}
