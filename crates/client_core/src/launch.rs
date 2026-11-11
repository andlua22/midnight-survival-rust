use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ClientOptions {
    pub config_dir: Option<PathBuf>,
    pub no_vsync: bool,
    pub benchmark: bool,
    pub benchmark_report: Option<PathBuf>,
    pub connect: Option<String>,
    pub join_code_file: Option<PathBuf>,
    pub network_report: Option<PathBuf>,
    /// None selects the canonical APK island; test fixtures require an explicit flag.
    pub map: Option<crate::level::MapId>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LaunchCommand {
    Run(ClientOptions),
    Help,
}

pub const HELP: &str = "Midnight Survival — Linux prototype\n\
Usage: client_runner [OPTIONS]\n\n\
  --config-dir PATH        Override the HUD configuration directory\n\
  --no-vsync               Disable vertical synchronization\n\
  --map MAP               imported-island (default), polygon, or load-arena\n\
  --benchmark              Fixed camera, 1920x1080, no VSync; 10s warmup + 60s capture\n\
  --benchmark-report PATH  Save the benchmark JSON (requires --benchmark)\n\
  --connect HOST:PORT      Open a server connection (default port 5000)\n\
  --join-code-file PATH    Read an invitation and connect automatically\n\
  --network-report PATH    Save online FPS, connection and correction metrics\n\
  -h, --help               Show this help\n\n\
Controls: WASD, mouse, Space jump, Shift sprint, Escape menu, F2 HUD editor.\n\
Benchmark mode requires a release build; Escape cancels the run.";

pub fn parse_args(arguments: impl IntoIterator<Item = OsString>) -> Result<LaunchCommand, String> {
    let mut options = ClientOptions::default();
    let mut args = arguments.into_iter();
    while let Some(argument) = args.next() {
        match argument.to_str() {
            Some("--help" | "-h") => return Ok(LaunchCommand::Help),
            Some("--config-dir") => {
                options.config_dir = Some(path_argument(&mut args, "--config-dir")?)
            }
            Some("--no-vsync") => options.no_vsync = true,
            Some("--map") => {
                let value = path_argument(&mut args, "--map")?;
                options.map = Some(match value.to_str() {
                    Some("imported-island") => crate::level::MapId::ImportedIsland,
                    Some("polygon") => crate::level::MapId::Polygon,
                    Some("load-arena") => crate::level::MapId::LoadArena,
                    _ => return Err("--map must be imported-island, polygon, or load-arena".into()),
                });
            }
            Some("--connect") => {
                options.connect = Some(
                    path_argument(&mut args, "--connect")?
                        .into_os_string()
                        .into_string()
                        .map_err(|_| "--connect requires a UTF-8 address")?,
                );
            }
            Some("--join-code-file") => {
                options.join_code_file = Some(path_argument(&mut args, "--join-code-file")?)
            }
            Some("--network-report") => {
                options.network_report = Some(path_argument(&mut args, "--network-report")?)
            }
            Some("--benchmark") => options.benchmark = true,
            Some("--benchmark-report") => {
                options.benchmark_report = Some(path_argument(&mut args, "--benchmark-report")?)
            }
            _ => {
                return Err(format!(
                    "Unknown argument: {}. Use --help.",
                    argument.to_string_lossy()
                ));
            }
        }
    }
    if options.benchmark_report.is_some() && !options.benchmark {
        return Err("--benchmark-report requires --benchmark".into());
    }
    if options.join_code_file.is_some() && options.connect.is_none() {
        return Err("--join-code-file requires --connect".into());
    }
    if options.benchmark && (options.connect.is_some() || options.network_report.is_some()) {
        return Err(
            "--benchmark is an offline measurement; --connect cannot be combined with it".into(),
        );
    }
    if options.benchmark {
        options.no_vsync = true;
    }
    Ok(LaunchCommand::Run(options))
}

fn path_argument(args: &mut impl Iterator<Item = OsString>, flag: &str) -> Result<PathBuf, String> {
    let value = args
        .next()
        .ok_or_else(|| format!("{flag} requires a path"))?;
    if value.is_empty() || value.to_string_lossy().starts_with("--") {
        return Err(format!("{flag} requires a path"));
    }
    Ok(PathBuf::from(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<LaunchCommand, String> {
        parse_args(args.iter().map(OsString::from))
    }

    #[test]
    fn benchmark_enforces_no_vsync_and_preserves_paths_with_spaces() {
        let LaunchCommand::Run(options) = parse(&[
            "--config-dir",
            "/tmp/my game",
            "--benchmark",
            "--benchmark-report",
            "/tmp/report.json",
        ])
        .unwrap() else {
            panic!();
        };
        assert!(options.no_vsync);
        assert_eq!(options.config_dir, Some(PathBuf::from("/tmp/my game")));
        assert!(options.benchmark_report.is_some());
    }

    #[test]
    fn invalid_or_incomplete_arguments_do_not_start_the_client() {
        for args in [
            &["--config-dir"][..],
            &["--config-dir", "--benchmark"],
            &["--unknown"],
            &["--benchmark-report", "report.json"],
        ] {
            assert!(parse(args).is_err());
        }
        assert_eq!(parse(&["--help"]).unwrap(), LaunchCommand::Help);
    }
}
