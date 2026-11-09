use clap::Parser;
use std::path::{Path, PathBuf};

mod fields;
mod inspect;
mod mesh;
mod package_publish;
mod packed_mesh;
mod pass1_index;
mod pass2_terrain;
mod pass3_ui;
mod resolver;
mod terrain_verification;
mod texture;
mod unity;

#[derive(Parser, Debug)]
#[command(
    name = "asset_unpacker",
    about = "Native Rust offline asset unpacker & global PathID resolver"
)]
pub struct Cli {
    #[arg(short, long, default_value = "export/lite4.0.apk")]
    pub apk: PathBuf,

    #[arg(short, long, default_value = "assets")]
    pub out_dir: PathBuf,

    #[arg(long)]
    pub pass1: bool,

    #[arg(long)]
    pub pass2: bool,

    #[arg(long)]
    pub pass3: bool,

    #[arg(long)]
    pub all: bool,

    /// Inspect one exact bundle member without changing extracted assets.
    #[arg(long)]
    pub inspect_bundle: Option<String>,

    /// Unity class IDs to include in the inspection report (comma separated).
    #[arg(long, value_delimiter = ',')]
    pub inspect_classes: Vec<i32>,

    /// Save inspection JSON here instead of writing it to stdout.
    #[arg(long)]
    pub report: Option<PathBuf>,

    /// Verify the canonical terrain archive, source coverage and texture checksums.
    #[arg(long)]
    pub verify: bool,

    /// Verify a completed candidate without importing or activating it.
    #[arg(long, conflicts_with = "promote_terrain")]
    pub verify_package: Option<PathBuf>,

    /// Verify and activate a completed candidate, retaining any prior canonical package.
    #[arg(long)]
    pub promote_terrain: Option<PathBuf>,

    /// Verify a retained UI source package against the APK, without importing or activating it.
    #[arg(long, value_name = "PACKAGE", conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "verify", "verify_package", "promote_terrain"])]
    pub verify_ui: Option<PathBuf>,
}

fn write_report<T: serde::Serialize>(
    path: Option<&Path>,
    report: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(std::fs::File::create(path)?, report)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Cli::parse();
    if let Some(package) = &args.verify_ui {
        let verification = pass3_ui::verify(package, &args.apk)?;
        write_report(args.report.as_deref(), &verification)?;
        println!(
            "Verified {} UI panels, {} sprites, {} decoded textures and {} source objects ({} deferred textures); runtime equivalence remains unverified",
            verification.panels.len(),
            verification.sprites,
            verification.decoded_textures,
            verification.source_objects,
            verification.deferred_textures.len()
        );
        return Ok(());
    }
    if let Some(bundle) = &args.inspect_bundle {
        return inspect::run(
            &args.apk,
            bundle,
            &args.inspect_classes,
            args.report.as_deref(),
        );
    }
    if args.verify || args.verify_package.is_some() || args.promote_terrain.is_some() {
        let verification = if let Some(candidate) = &args.promote_terrain {
            pass2_terrain::promote(&args.apk, candidate, &args.out_dir)
        } else {
            let package = args
                .verify_package
                .clone()
                .unwrap_or_else(|| args.out_dir.join("imported_map/terrain"));
            terrain_verification::verify(&package, &args.apk)
        }
        .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &verification)?;
        println!(
            "Verified {} terrain instances and {} other instances ({} render parts) from APK SHA-256 {}",
            verification.terrain_source_instances,
            verification.other_source_instances,
            verification.render_records,
            verification.source_apk.sha256
        );
        return Ok(());
    }
    if !args.pass1 && !args.pass2 && !args.pass3 && !args.all {
        args.all = true;
    }

    println!("=======================================================");
    println!("  Midnight Survival Rust Asset Unpacker                ");
    println!("=======================================================");
    println!("APK path: {}", args.apk.display());
    println!("Out dir:  {}", args.out_dir.display());

    if args.pass1 || args.all {
        pass1_index::run(&args.apk, &args.out_dir)?;
    }
    if args.pass2 || args.all {
        pass2_terrain::run(&args.apk, &args.out_dir)?;
    }
    if args.pass3 || args.all {
        pass3_ui::run(&args.apk, &args.out_dir)?;
    }

    println!("\nAll requested passes finished successfully!");
    Ok(())
}
