// Synthetic decoder/replica scenario, never a captured packet or starter bag.
use std::{fs::OpenOptions, io::Write, path::Path};
fn int(out: &mut Vec<u8>, value: i32, code: u8) {
    out.extend(value.to_le_bytes().map(|byte| byte ^ code));
}
fn octets(out: &mut Vec<u8>, value: &[u8], code: u8) {
    int(out, value.len() as i32, code);
    out.extend(value.iter().map(|byte| byte ^ code));
}
fn item(out: &mut Vec<u8>, code: u8, instance: i32, id: i32, count: i32, bound: bool) {
    for value in [instance, id, count, -1, 20] { int(out, value, code); }
    out.push(u8::from(bound) ^ code);
    int(out, 0, code);
    octets(out, &[255,0,42], code);
}
fn write(name: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true)
        .open(Path::new(".local/validation-source-inventory").join(name))?;
    file.write_all(bytes)
}
fn main() -> std::io::Result<()> {
    let mut snapshot = Vec::new();
    int(&mut snapshot, 2, 123);
    item(&mut snapshot, 123, 101, 1, 17, false);
    item(&mut snapshot, 123, 102, 4, 2, true);
    int(&mut snapshot, 0, 14);
    int(&mut snapshot, 1, 7);
    int(&mut snapshot, 8, 7);
    item(&mut snapshot, 7, 103, 1, 3, false);
    int(&mut snapshot, 0, 63);
    snapshot.push(1 ^ 81);
    int(&mut snapshot, 1, 106);
    write("snapshot.bin", &snapshot)?;
    let mut changed = Vec::new();
    int(&mut changed, 0, 99);
    changed.push(1 ^ 109);
    for (value, code) in [(999,100),(4,115),(55,82),(101,0)] { int(&mut changed, value, code); }
    octets(&mut changed, &[4,0,6], 4);
    int(&mut changed, 5, 109);
    write("changed.bin", &changed)?;
    let mut quick = Vec::new();
    int(&mut quick, 999, 46); int(&mut quick, 2, 118); quick.push(1 ^ 97);
    for (value, code) in [(101,49),(0,47),(55,126),(4,67),(4,7)] { int(&mut quick, value, code); }
    octets(&mut quick, &[8,0,9], 104);
    write("quick.bin", &quick)?;
    let mut size = Vec::new(); int(&mut size, 0, 77); write("size.bin", &size)?;
    let mut duration = Vec::new();
    int(&mut duration, 8, 101); int(&mut duration, 101, 88); write("duration.bin", &duration)?;
    let mut alert = vec![0 ^ 124]; int(&mut alert, -2, 118); int(&mut alert, 1, 61);
    write("alert.bin", &alert)?;
    let mut hand = Vec::new(); int(&mut hand, 101, 42); write("hand.bin", &hand)?;
    write("truncated.bin", &[0])?;
    Ok(())
}
