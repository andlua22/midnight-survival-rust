//! APK-backed UI extraction. Legacy `assets/ui` exports are deliberately not inputs.

#[path = "ui_source_import.rs"]
mod importer;
#[path = "ui_source_sprite.rs"]
mod sprite;
#[path = "ui_source_types.rs"]
mod types;

pub fn run(
    apk_path: &std::path::Path,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    importer::run(apk_path, out_dir).map_err(|error| error as Box<dyn std::error::Error>)
}
