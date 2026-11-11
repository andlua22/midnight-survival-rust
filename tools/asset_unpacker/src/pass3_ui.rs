//! APK-backed UI extraction. Legacy `assets/ui` exports are deliberately not inputs.

#[path = "ui_source_font.rs"]
mod font;
#[path = "ui_source_import.rs"]
mod importer;
#[path = "ui_source_sprite.rs"]
mod sprite;
#[path = "ui_source_types.rs"]
mod types;
#[path = "ui_source_verify.rs"]
mod verifier;

pub use verifier::UiVerification;

pub fn verify(
    package: &std::path::Path,
    apk: &std::path::Path,
) -> Result<UiVerification, Box<dyn std::error::Error>> {
    verifier::verify(package, apk).map_err(|error| error as Box<dyn std::error::Error>)
}

pub fn run(
    apk_path: &std::path::Path,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    importer::run(apk_path, out_dir).map_err(|error| error as Box<dyn std::error::Error>)
}

pub fn run_selected(
    apk: &std::path::Path,
    output: &std::path::Path,
    panels: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    importer::run_selected(apk, output, panels).map_err(|error| error as Box<dyn std::error::Error>)
}
