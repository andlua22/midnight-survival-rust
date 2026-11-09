//! Independently compare shipped-font character coverage with the egui parser.
//! No system font fallback or fabricated glyph is used.

use std::{collections::BTreeMap, path::Path};

use midnight_ui_data::{ComponentProjection, HierarchyNode, load_ui_package};
use skrifa::MetadataProvider;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("provide the UI package path")?;
    let package = load_ui_package(Path::new(&root))?;
    let panel = package.load_panel("BattleMinimap")?;
    let mut strings = Vec::new();
    fn collect(node: &HierarchyNode, result: &mut Vec<(String, midnight_ui_data::TextData)>) {
        for component in &node.components {
            if let ComponentProjection::Text(text) = &component.projection {
                result.push((node.name.clone(), text.clone()));
            }
        }
        for child in &node.children {
            collect(child, result);
        }
    }
    collect(&panel.root, &mut strings);
    let mut fonts = Vec::new();
    for font in &package.manifest.fonts {
        let Some(identity) = &font.embedded_font else {
            continue;
        };
        let bytes = package.read_verified(identity)?;
        let skrifa = skrifa::FontRef::from_index(&bytes, 0)?;
        let independent = ttf_parser::Face::parse(&bytes, 0)?;
        let mut coverage = BTreeMap::new();
        let mut affected = Vec::new();
        for (name, text) in strings
            .iter()
            .filter(|(_, text)| text.font.target() == Some(&font.source))
        {
            let mut missing = Vec::new();
            for ch in text.text.chars().filter(|ch| !ch.is_whitespace()) {
                let actual = skrifa.charmap().map(ch).map(|glyph| glyph.to_u32());
                let independent = independent.glyph_index(ch).map(|glyph| glyph.0);
                coverage.insert(ch.to_string(), serde_json::json!({"codepoint":format!("U+{:04X}",ch as u32),"skrifa_glyph":actual,"ttf_parser_glyph":independent}));
                if actual.is_none() {
                    missing.push(ch.to_string());
                }
            }
            if !missing.is_empty() {
                affected.push(
                    serde_json::json!({"node":name,"text":text.text,"missing_characters":missing}),
                );
            }
        }
        fonts.push(serde_json::json!({"source":font.source,"name":font.name,"embedded":identity,"byte_count":bytes.len(),"glyph_count":independent.number_of_glyphs(),
            "skrifa_unicode_map":skrifa.charmap().has_map(),"skrifa_symbol_map":skrifa.charmap().is_symbol(),"coverage":coverage,"affected_text":affected}));
    }
    serde_json::to_writer_pretty(
        std::io::stdout().lock(),
        &serde_json::json!({"source_apk_sha256":panel.source_apk_sha256,"panel":panel.name,"fonts":fonts,
        "limitations":["This compares nominal mappings in the retained embedded font. Unity font fallback and runtime glyph atlas behavior are not established."]}),
    )?;
    Ok(())
}
