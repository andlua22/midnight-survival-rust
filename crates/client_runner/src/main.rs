#[cfg(target_os = "linux")]
mod vulkan;

use client_core::{
    LaunchCommand, build_app, parse_args,
    storage::{FileStorage, config_directory},
};

fn main() -> std::process::ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Midnight Survival: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn run() -> Result<std::process::ExitCode, String> {
    let options = match parse_args(std::env::args_os().skip(1))? {
        LaunchCommand::Help => {
            println!("{}", client_core::launch::HELP);
            return Ok(std::process::ExitCode::SUCCESS);
        }
        LaunchCommand::Run(options) => options,
    };
    if options.benchmark && cfg!(debug_assertions) {
        return Err(
            "Замір потребує release-збірки: cargo run --release -p client_runner -- --benchmark"
                .into(),
        );
    }
    let directory = config_directory(
        options.config_dir.clone(),
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
    .map_err(|error| error.to_string())?;
    #[cfg(target_os = "linux")]
    vulkan::check_vulkan()?;
    let mut app = build_app(options, Box::new(FileStorage::new(directory)));
    app.add_plugins(client_core::desktop_input::DesktopInputPlugin);
    let exit = app.run();
    if exit.is_success() {
        Ok(std::process::ExitCode::SUCCESS)
    } else {
        Ok(std::process::ExitCode::FAILURE)
    }
}
