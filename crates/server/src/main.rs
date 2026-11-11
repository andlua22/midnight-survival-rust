use bevy::prelude::*;
use shared::{CharacterMotion, Player, SimulationClock, build_headless_app};

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("server: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments: Vec<_> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("serve") => {
            let options = parse_server_options(arguments.into_iter().skip(1))?;
            let report = midnight_network::runtime()?.block_on(server::serve(options))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            return Ok(());
        }
        Some("invite") => {
            let mut directory = std::path::PathBuf::from("server-data");
            let mut args = arguments.into_iter().skip(1);
            while let Some(flag) = args.next() {
                if flag != "--data-dir" {
                    return Err(format!("Unknown invitation option: {flag}"));
                }
                directory = args.next().ok_or("--data-dir requires a path")?.into();
            }
            let identity = midnight_network::identity::ServerIdentity::load_or_create(&directory)?;
            println!("{}", identity.invitation().code());
            return Ok(());
        }
        _ => {}
    }
    let mut ticks = 120_u64;
    let mut args = arguments.into_iter();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!(
                    "Midnight Survival server\n\nserver [--ticks N] — finite offline smoke test (default: 120)\nserver serve [OPTIONS] — authoritative QUIC server\nserver invite [--data-dir PATH] — print the private invitation code\n\nServe options:\n  --listen IP:PORT       Default: 0.0.0.0:5000 (UDP)\n  --data-dir PATH        Persistent certificate and access secret (server-data)\n  --max-players N        1..150, default 150\n  --map MAP             imported-island, polygon, or load-arena\n  --layout LAYOUT       crowd or spread (spread needs load-arena)\n  --seconds N           Stop after N seconds; otherwise run until Ctrl+C\n  --report PATH         Write periodic and final JSON metrics"
                );
                return Ok(());
            }
            "--ticks" => {
                ticks = args
                    .next()
                    .ok_or("--ticks needs a positive integer")?
                    .parse()
                    .map_err(|_| "invalid --ticks value")?;
                if ticks == 0 {
                    return Err("--ticks must be positive".into());
                }
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    let mut app = build_headless_app();
    while app.world().resource::<SimulationClock>().tick < ticks {
        app.update();
    }
    let world = app.world_mut();
    let completed = world.resource::<SimulationClock>().tick;
    let mut query = world.query_filtered::<(&Transform, &CharacterMotion), With<Player>>();
    let (transform, motion) = query.single(world).map_err(|error| error.to_string())?;
    println!(
        "ticks={completed} position={:.3},{:.3},{:.3} grounded={}",
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
        motion.0.grounded
    );
    Ok(())
}

fn parse_server_options(
    arguments: impl Iterator<Item = String>,
) -> Result<server::ServerOptions, String> {
    let mut options = server::ServerOptions::default();
    let mut args = arguments;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--listen" => options.listen = value.parse().map_err(|_| "Invalid --listen address")?,
            "--data-dir" => options.data_dir = value.into(),
            "--max-players" => {
                options.capacity = value.parse().map_err(|_| "Invalid --max-players")?
            }
            "--map" => {
                options.map = match value.as_str() {
                    "polygon" => shared::level::MapId::Polygon,
                    "load-arena" => shared::level::MapId::LoadArena,
                    "imported-island" => shared::level::MapId::ImportedIsland,
                    _ => return Err("--map must be polygon, load-arena, or imported-island".into()),
                }
            }
            "--layout" => {
                options.spread = match value.as_str() {
                    "crowd" => false,
                    "spread" => true,
                    _ => return Err("--layout must be crowd or spread".into()),
                }
            }
            "--seconds" => {
                let seconds = value.parse::<f64>().map_err(|_| "Invalid --seconds")?;
                if !seconds.is_finite() || seconds <= 0.0 {
                    return Err("--seconds must be positive and finite".into());
                }
                options.seconds = Some(seconds);
            }
            "--report" => options.report = Some(value.into()),
            _ => return Err(format!("Unknown server option: {flag}")),
        }
    }
    Ok(options)
}
