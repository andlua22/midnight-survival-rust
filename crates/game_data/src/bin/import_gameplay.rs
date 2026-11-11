use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut apk = PathBuf::from("export/lite4.0.apk");
    let mut output = None;
    let mut verify = None;
    let mut report = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--apk" => apk = args.next().ok_or("--apk requires a path")?.into(),
            "--out" => output = Some(PathBuf::from(args.next().ok_or("--out requires a path")?)),
            "--verify" => {
                verify = Some(PathBuf::from(
                    args.next().ok_or("--verify requires a path")?,
                ))
            }
            "--report" => {
                report = Some(PathBuf::from(
                    args.next().ok_or("--report requires a path")?,
                ))
            }
            "--help" | "-h" => {
                println!(
                    "import_gameplay --apk PATH --out NEW_PACKAGE [--report PATH]\nimport_gameplay --verify PACKAGE [--report PATH]\nDecodes six selected source tables in Rust; existing packages are preserved."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument {flag}").into()),
        }
    }
    let manifest = match (output, verify) {
        (Some(output), None) => midnight_game_data::package::import_apk(&apk, &output)?,
        (None, Some(root)) => midnight_game_data::package::load_package(&root)?.0,
        _ => return Err("choose exactly one of --out or --verify".into()),
    };
    let json = serde_json::to_string_pretty(&manifest)?;
    if let Some(report) = report {
        if let Some(parent) = report.parent().filter(|path| !path.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(report, &json)?;
    }
    println!("{json}");
    Ok(())
}
