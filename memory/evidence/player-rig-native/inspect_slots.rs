//! Bounded offline inspection of paired IL2CPP metadata usage slots. No target execution.
use std::{collections::BTreeSet, env, error::Error, fs};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) fn u32_at(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(bytes.get(offset..offset + 4).ok_or("truncated u32")?.try_into()?))
}
pub(crate) fn u64_at(bytes: &[u8], offset: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(bytes.get(offset..offset + 8).ok_or("truncated u64")?.try_into()?))
}
pub(crate) fn file_offset(bytes: &[u8], address: u64, length: usize) -> Result<usize> {
    if bytes.get(..6) != Some(b"\x7fELF\x02\x01") { return Err("expected little-endian ELF64".into()); }
    let phoff = usize::try_from(u64_at(bytes, 32)?)?;
    let stride = u16::from_le_bytes(bytes.get(54..56).ok_or("ELF header")?.try_into()?) as usize;
    let count = u16::from_le_bytes(bytes.get(56..58).ok_or("ELF header")?.try_into()?) as usize;
    for index in 0..count {
        let at = phoff + index * stride;
        if u32_at(bytes, at)? != 1 { continue; }
        let start = u64_at(bytes, at + 16)?;
        let end = start.checked_add(u64_at(bytes, at + 32)?).ok_or("segment end overflow")?;
        if address >= start && address.checked_add(length as u64).ok_or("range overflow")? <= end {
            let offset = usize::try_from(u64_at(bytes, at + 8)? + address - start)?;
            bytes.get(offset..offset + length).ok_or("truncated PT_LOAD")?;
            return Ok(offset);
        }
    }
    Err(format!("unmapped file-backed address {address:#x}").into())
}
pub(crate) fn word(bytes: &[u8], address: u64) -> Result<u64> { u64_at(bytes, file_offset(bytes, address, 8)?) }
pub(crate) fn relative(bytes: &[u8], slot: u64) -> Result<u64> {
    let phoff = usize::try_from(u64_at(bytes, 32)?)?;
    let stride = u16::from_le_bytes(bytes.get(54..56).ok_or("ELF header")?.try_into()?) as usize;
    let count = u16::from_le_bytes(bytes.get(56..58).ok_or("ELF header")?.try_into()?) as usize;
    let (mut rela, mut size, mut entry_size) = (0, 0, 0);
    for i in 0..count {
        let at = phoff + i * stride;
        if u32_at(bytes, at)? != 2 { continue; }
        let start = usize::try_from(u64_at(bytes, at + 8)?)?;
        let len = usize::try_from(u64_at(bytes, at + 32)?)?;
        for j in (start..start + len).step_by(16) {
            let tag = u64_at(bytes, j)?;
            let value = u64_at(bytes, j + 8)?;
            match tag { 7 => rela = value, 8 => size = value, 9 => entry_size = value, 0 => break, _ => () }
        }
    }
    if entry_size != 24 || size > 16 * 1024 * 1024 { return Err("unsupported relocation table".into()); }
    let start = file_offset(bytes, rela, usize::try_from(size)?)?;
    for i in (start..start + usize::try_from(size)?).step_by(24) {
        if u64_at(bytes, i)? == slot && u64_at(bytes, i + 8)? == 1027 { return u64_at(bytes, i + 16); }
    }
    Err(format!("no R_AARCH64_RELATIVE relocation at {slot:#x}").into())
}
fn literal(meta: &[u8], index: u32) -> Result<&str> {
    if u32_at(meta, 0)? != 0xfab11baf || u32_at(meta, 4)? != 29 { return Err("requires IL2CPP metadata 29".into()); }
    let count_bytes = u32_at(meta, 12)?;
    if index >= count_bytes / 8 { return Err("string literal index out of range".into()); }
    let entry = u32_at(meta, 8)? as usize + index as usize * 8;
    let len = u32_at(meta, entry)? as usize;
    let offset = u32_at(meta, entry + 4)? as usize;
    if offset.checked_add(len).ok_or("literal overflow")? > u32_at(meta, 20)? as usize { return Err("literal data out of range".into()); }
    let start = u32_at(meta, 16)? as usize + offset;
    Ok(std::str::from_utf8(meta.get(start..start + len).ok_or("truncated literal")?)?)
}
fn slots(argument: &str) -> Result<BTreeSet<u64>> {
    let Some(path) = argument.strip_prefix("asm:") else {
        return Ok(BTreeSet::from([u64::from_str_radix(argument.trim_start_matches("0x"), 16)?]));
    };
    if fs::metadata(path)?.len() > 262144 { return Err("assembly input exceeds 256 KiB".into()); }
    let text = fs::read_to_string(path)?;
    let mut pages = [None; 31];
    let mut values = BTreeSet::new();
    for (line_index, line) in text.lines().enumerate() {
        let words: Vec<_> = line.split_whitespace().collect();
        if words.len() == 5 && words[2] == "adrp" {
            let register = words[3].trim_start_matches('x').trim_end_matches(',').parse::<usize>()?;
            if register < 31 { pages[register] = Some((u64::from_str_radix(words[4].trim_start_matches("#0x"), 16)?, line_index)); }
        } else if words.len() == 6 && words[2] == "ldr" {
            let register = words[4].trim_start_matches("[x").trim_end_matches(',').parse::<usize>();
            if let Ok(register) = register {
                if let Some(Some((page, page_line))) = pages.get(register) {
                    if line_index - page_line <= 6 {
                        let offset = u64::from_str_radix(words[5].trim_start_matches("#0x").trim_end_matches(']'), 16)?;
                        let value = page + offset;
                        if (0x45ad000..0x4640000).contains(&value) { values.insert(value); }
                    }
                }
            }
        }
    }
    Ok(values)
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 5 { return Err("usage: inspect_slots DUMP_ELF PAIRED_ELF METADATA SLOT...".into()); }
    let dump = fs::read(&args[1])?;
    let pair = fs::read(&args[2])?;
    let meta = fs::read(&args[3])?;
    println!("slot\trelocation_addend\tdump_pointer\tpointer_bias\tpaired_pointer\tdump_usage\tpaired_usage\tusage_kind\tusage_index\tliteral");
    let mut selected_slots = BTreeSet::new();
    for argument in &args[4..] { selected_slots.extend(slots(argument)?); }
    for slot in selected_slots {
        let addend = relative(&dump, slot)?;
        if relative(&pair, slot)? != addend { return Err(format!("paired relocation differs at {slot:#x}").into()); }
        let dump_pointer = word(&dump, slot)?;
        let pair_pointer = word(&pair, slot)?;
        if pair_pointer != addend { return Err(format!("unexpected paired slot pointer at {slot:#x}").into()); }
        let dump_usage = word(&dump, addend)?;
        let usage = word(&pair, addend)?;
        if usage > u64::from(u32::MAX) || usage & 1 == 0 { return Err(format!("paired value is not unresolved encoded metadata usage at {addend:#x}").into()); }
        let kind = usage >> 29;
        let index = (usage as u32 & 0x1fffffff) >> 1;
        let text = if kind == 5 { format!("{:?}", literal(&meta, index)?) } else { String::from("unresolved_nonliteral") };
        println!("{slot:#x}\t{addend:#x}\t{dump_pointer:#x}\t{:#x}\t{pair_pointer:#x}\t{dump_usage:#x}\t{usage:#x}\t{kind}\t{index}\t{text}", dump_pointer.checked_sub(addend).ok_or("pointer bias underflow")?);
    }
    Ok(())
}
