// Synthetic bodies from the reviewed native mask table, never captured packets.
// Preserve the earlier alert.bin/hand.bin; their generator used incorrect masks.
use std::{fs::OpenOptions, io::Write, path::Path};

fn int(out: &mut Vec<u8>, value: i32, code: u8) {
    out.extend(value.to_le_bytes().map(|byte| byte ^ code));
}

fn write(name: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(".local/validation-source-inventory").join(name))?;
    file.write_all(bytes)
}

fn main() -> std::io::Result<()> {
    let mut alert = vec![36]; // false ^ 36
    int(&mut alert, -2, 57);
    int(&mut alert, 1, 123);
    write("alert-reviewed.bin", &alert)?;
    let mut hand = Vec::new();
    int(&mut hand, 101, 127);
    write("hand-reviewed.bin", &hand)?;
    write("clear.bin", &[])?;
    Ok(())
}
