use clap::{ArgGroup, Parser};
use std::path::{Path, PathBuf};

mod bundle_cache;
mod fields;
mod inspect;
mod mesh;
mod package_publish;
mod packed_mesh;
mod pass1_index;
mod pass2_terrain;
mod pass3_ui;
mod player;
mod player_animation;
mod prefab;
mod resolver;
mod terrain_collisions;
mod terrain_layers;
mod terrain_verification;
mod texture;
mod unity;
mod world;
mod world_scene;

#[derive(Parser, Debug)]
#[command(
    name = "asset_unpacker",
    about = "Native Rust offline asset unpacker & global PathID resolver",
    group(ArgGroup::new("action").args(["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab", "augment_terrain_scene", "enrich_terrain_layers", "import_terrain_collision", "import_player", "verify_player"]).required(true).multiple(true))
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

    /// Add an exact source UI panel (Name=assets/ui/member.ab) to a separate verified package revision.
    #[arg(long, requires = "pass3", conflicts_with = "all")]
    pub ui_panel: Vec<String>,

    #[arg(long)]
    pub all: bool,

    /// Restrict pass 1 to exact APK bundle members; writes a separate selected index.
    #[arg(long, requires = "pass1", conflicts_with_all = ["all", "pass2", "pass3"])]
    pub index_bundle: Vec<String>,

    /// Maximum parallel bundle parsers for pass 1 (1-8).
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=8))]
    pub index_workers: u8,

    /// Inspect one exact bundle member without changing extracted assets.
    #[arg(long)]
    pub inspect_bundle: Option<String>,

    /// Extract one complete prefab hierarchy and local mesh/material package.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "verify", "verify_package", "promote_terrain", "verify_ui"])]
    pub prefab_bundle: Option<String>,

    /// Report bounded APK placement discovery without importing or activating world objects.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "verify_prefab"])]
    pub discover_world: bool,

    /// Inspect at most eight exact candidate Octets members using the recovered plain SceneInfo layout.
    #[arg(long, requires = "discover_world")]
    pub world_member: Vec<String>,

    /// Verify a retained prefab package without importing or activating it.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui"])]
    pub verify_prefab: Option<PathBuf>,

    /// Add only the authenticated omitted assets/scene/z-01.ab terrain to the canonical package.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab"])]
    pub augment_terrain_scene: Option<String>,

    /// Verify and retain source terrain GameObject layers without decoding meshes/textures.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab", "augment_terrain_scene"])]
    pub enrich_terrain_layers: bool,

    /// Import an independent source collider table, preserving visible terrain and its predecessor.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab", "augment_terrain_scene", "enrich_terrain_layers"])]
    pub import_terrain_collision: bool,

    /// Import the exact role/role_new/rolehand roots and their default animation dependencies.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab", "augment_terrain_scene", "enrich_terrain_layers", "import_terrain_collision", "verify_player"])]
    pub import_player: bool,

    /// Add an exact controller-linked clip (APK member#signed-PathID); names can be ambiguous.
    #[arg(long, requires = "import_player")]
    pub player_clip: Vec<String>,

    /// Verify a separate player package against the exact APK without activating it.
    #[arg(long, conflicts_with_all = ["pass1", "pass2", "pass3", "all", "inspect_bundle", "prefab_bundle", "verify", "verify_package", "promote_terrain", "verify_ui", "discover_world", "verify_prefab", "augment_terrain_scene", "enrich_terrain_layers", "import_terrain_collision", "import_player"])]
    pub verify_player: Option<PathBuf>,

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
    let args = Cli::parse();
    if args.import_player || args.verify_player.is_some() {
        let report = if let Some(package) = &args.verify_player {
            player::verify(&args.apk, package)
        } else {
            player::import(&args.apk, &args.out_dir, &args.player_clip)
        }.map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)?;
        return Ok(());
    }
    if args.import_terrain_collision {
        let report = terrain_collisions::import(&args.apk, &args.out_dir)
            .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        println!(
            "Verified independent terrain collision: {} components, {} active solids, {} triggers, {} inactive/disabled; already imported: {}",
            report.source_entries,
            report.active_solid_entries,
            report.active_trigger_entries,
            report.inactive_or_disabled_entries,
            report.already_imported
        );
        return Ok(());
    }
    if args.enrich_terrain_layers {
        let report = terrain_layers::enrich(&args.apk, &args.out_dir)
            .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        println!(
            "Verified terrain layers: {} GameObjects, {} render records, {} omitted source colliders; already enriched: {}",
            report.source_audit.game_objects.len(),
            report.verification.known_layer_records,
            report.source_audit.omitted_colliders.len(),
            report.already_enriched
        );
        return Ok(());
    }
    if let Some(member) = &args.augment_terrain_scene {
        let report = pass2_terrain::augment(&args.apk, &args.out_dir, member)
            .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        println!(
            "Verified terrain augmentation: {} added records, {} terrain scenes, {} total source instances; previous package retained",
            report.added_records.len(),
            report.verification.terrain_scene_bundles,
            report.verification.source_instances
        );
        return Ok(());
    }
    if args.discover_world {
        let report = world::discover(&args.apk, &args.world_member)
            .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        println!(
            "World discovery: {} direct sceneinfo members, {} listed sceneinfo paths, {} selected payloads; {} authenticated placements, no world package published",
            report.direct_sceneinfo_members.len(),
            report.file_list_sceneinfo_paths.len(),
            report.selected_payloads.len(),
            report.authenticated_placements
        );
        return Ok(());
    }
    if let Some(package) = &args.verify_prefab {
        let prefab = midnight_map_data::load_prefab_package(package)?;
        write_report(
            args.report.as_deref(),
            &serde_json::json!({
                "package": package,
                "manifest_sha256": prefab.manifest_sha256,
                "source_apk": prefab.manifest.source_apk,
                "bundle": prefab.manifest.bundle,
                "nodes": prefab.manifest.nodes.len(),
                "renderers": prefab.manifest.renderers.len(),
                "mesh_parts": prefab.records.len(),
                "collision_meshes": prefab.manifest.collision_meshes.len(),
                "verified_retained_files": prefab.manifest.retained_files.len(),
                "runtime_equivalence_verified": false,
                "limitations": prefab.manifest.limitations,
            }),
        )?;
        println!(
            "Verified prefab {}: {} nodes, {} renderers, {} mesh parts; runtime equivalence remains unverified",
            prefab.manifest.bundle,
            prefab.manifest.nodes.len(),
            prefab.manifest.renderers.len(),
            prefab.records.len()
        );
        return Ok(());
    }
    if let Some(member) = &args.prefab_bundle {
        let report = prefab::run(&args.apk, member, &args.out_dir)
            .map_err(|error| error as Box<dyn std::error::Error>)?;
        write_report(args.report.as_deref(), &report)?;
        serde_json::to_writer_pretty(std::io::stdout().lock(), &report)?;
        return Ok(());
    }
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
    println!("=======================================================");
    println!("  Midnight Survival Rust Asset Unpacker                ");
    println!("=======================================================");
    println!("APK path: {}", args.apk.display());
    println!("Out dir:  {}", args.out_dir.display());

    if args.pass1 || args.all {
        pass1_index::run_selected(
            &args.apk,
            &args.out_dir,
            &pass1_index::ScanOptions {
                workers: usize::from(args.index_workers),
                members: args.index_bundle.clone(),
                ..Default::default()
            },
            args.report.as_deref(),
        )
        .map_err(|error| error as Box<dyn std::error::Error>)?;
    }
    if args.pass2 || args.all {
        pass2_terrain::run(&args.apk, &args.out_dir)?;
    }
    if args.pass3 || args.all {
        if args.ui_panel.is_empty() {
            pass3_ui::run(&args.apk, &args.out_dir)?;
        } else {
            pass3_ui::run_selected(&args.apk, &args.out_dir, &args.ui_panel)?;
        }
    }

    println!("\nAll requested passes finished successfully!");
    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn extraction_requires_an_explicit_action() {
        assert!(Cli::try_parse_from(["asset_unpacker"]).is_err());
        assert!(Cli::try_parse_from(["asset_unpacker", "--out-dir", "assets"]).is_err());
        assert!(Cli::try_parse_from(["asset_unpacker", "--pass1", "--pass2"]).is_ok());
    }

    #[test]
    fn selected_index_cannot_replace_a_full_import_dependency_index() {
        assert!(
            Cli::try_parse_from(["asset_unpacker", "--all", "--index-bundle", "x.ab"]).is_err()
        );
        assert!(
            Cli::try_parse_from(["asset_unpacker", "--pass1", "--index-bundle", "x.ab"]).is_ok()
        );
        assert!(
            Cli::try_parse_from(["asset_unpacker", "--pass1", "--index-workers", "9"]).is_err()
        );
    }

    #[test]
    fn explicit_actions_are_scoped_and_reject_mixed_imports() {
        assert!(Cli::try_parse_from(["asset_unpacker", "--import-terrain-collision"]).is_ok());
        for action in ["--all", "--pass2", "--verify", "--enrich-terrain-layers"] {
            assert!(
                Cli::try_parse_from(["asset_unpacker", "--import-terrain-collision", action])
                    .is_err()
            );
        }
        assert!(Cli::try_parse_from(["asset_unpacker", "--enrich-terrain-layers"]).is_ok());
        for action in ["--all", "--pass2", "--verify", "--discover-world"] {
            assert!(
                Cli::try_parse_from(["asset_unpacker", "--enrich-terrain-layers", action]).is_err()
            );
        }
        assert!(Cli::try_parse_from(["asset_unpacker", "--discover-world"]).is_ok());
        assert!(Cli::try_parse_from(["asset_unpacker", "--discover-world", "--pass2"]).is_err());
        assert!(
            Cli::try_parse_from(["asset_unpacker", "--world-member", "assets/candidate.oc"])
                .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "asset_unpacker",
                "--discover-world",
                "--world-member",
                "assets/candidate.oc"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from(["asset_unpacker", "--verify-prefab", "assets/package"]).is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "asset_unpacker",
                "--verify-prefab",
                "assets/package",
                "--prefab-bundle",
                "x.ab"
            ])
            .is_err()
        );
        assert!(
            Cli::try_parse_from([
                "asset_unpacker",
                "--augment-terrain-scene",
                "assets/scene/z-01.ab"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from([
                "asset_unpacker",
                "--augment-terrain-scene",
                "assets/scene/z-01.ab",
                "--all"
            ])
            .is_err()
        );
    }
}
