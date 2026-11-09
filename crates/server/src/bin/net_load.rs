use server::load::{LoadOptions, run};
use std::{net::SocketAddr, path::PathBuf};

fn main() -> std::process::ExitCode {
    match start() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("net_load: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn start() -> Result<(), String> {
    let mut address: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let mut code_file = None;
    let mut clients = 150;
    let mut seconds = 1800.0;
    let mut spread = false;
    let mut report = PathBuf::from("network-load.json");
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        if flag == "--help" || flag == "-h" {
            println!(
                "net_load --code-file PATH [--server IP:PORT] [--clients 150] [--seconds 1800] [--scenario crowd|spread] [--report PATH]\nUse a release build on a machine separate from the server. Invitations must match the server identity."
            );
            return Ok(());
        }
        let value = args
            .next()
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--server" => address = value.parse().map_err(|_| "Invalid server address")?,
            "--code-file" => code_file = Some(PathBuf::from(value)),
            "--clients" => clients = value.parse().map_err(|_| "Invalid client count")?,
            "--seconds" => seconds = value.parse().map_err(|_| "Invalid duration")?,
            "--scenario" => {
                spread = match value.as_str() {
                    "crowd" => false,
                    "spread" => true,
                    _ => return Err("--scenario must be crowd or spread".into()),
                }
            }
            "--report" => report = value.into(),
            _ => return Err(format!("Unknown option: {flag}")),
        }
    }
    let options = LoadOptions {
        server: address,
        code_file: code_file.ok_or("--code-file is required")?,
        clients,
        seconds,
        spread,
        report,
    };
    let result = midnight_network::runtime()?.block_on(run(options))?;
    println!(
        "Завершено: {} клієнтів, {:.1} с, отримано {:.2} MiB, втрачено {} QUIC-пакетів",
        result.clients,
        result.elapsed_seconds,
        result.traffic.received_bytes as f64 / 1048576.0,
        result.traffic.lost_packets
    );
    Ok(())
}
