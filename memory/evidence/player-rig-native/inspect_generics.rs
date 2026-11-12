//! Read selected unresolved generic method tokens from the retained counterpart.
//! Reports source/counterpart byte equality on method specs; never executes either.
#[allow(dead_code)]
#[path = "inspect_slots.rs"]
mod elf;
use elf::{file_offset, relative, u32_at, u64_at, word};
use std::{env, error::Error, fs};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn string(meta: &[u8], index: u32) -> Result<&str> {
    if index >= u32_at(meta, 28)? { return Err("metadata string index outside table".into()); }
    let offset = u32_at(meta, 24)? as usize + index as usize;
    let tail = meta.get(offset..).ok_or("metadata string offset")?;
    let length = tail.iter().position(|x| *x == 0).ok_or("unterminated metadata string")?;
    Ok(std::str::from_utf8(&tail[..length])?)
}
fn definition(meta: &[u8], index: u32) -> Result<String> {
    if index >= u32_at(meta, 164)? / 88 { return Err(format!("type definition {index} out of range").into()); }
    let at = u32_at(meta, 160)? as usize + index as usize * 88;
    let name = string(meta, u32_at(meta, at)?)?;
    let namespace = string(meta, u32_at(meta, at + 4)?)?;
    Ok(if namespace.is_empty() { name.to_owned() } else { format!("{namespace}.{name}") })
}
fn native_type(pair: &[u8], dump: &[u8], meta: &[u8], address: u64, depth: u32) -> Result<String> {
    if depth > 12 { return Err("type depth limit".into()); }
    let at = file_offset(pair, address, 12)?;
    let data = u64_at(pair, at)?;
    let attributes = u32_at(pair, at + 8)?;
    let kind = (attributes >> 16) & 255;
    let source_at = file_offset(dump, address, 12)?;
    let source_data = u64_at(dump, source_at)?;
    let source_attributes = u32_at(dump, source_at + 8)?;
    println!("type-record address={address:#x} counterpart-data={data:#x} counterpart-attributes={attributes:#x} source-data={source_data:#x} source-attributes={source_attributes:#x}");
    Ok(match kind {
        0x11 | 0x12 => definition(meta, data as u32)?,
        0x1d => format!("{}[]", native_type(pair, dump, meta, data, depth + 1)?),
        0x01 => "void".into(), 0x02 => "bool".into(), 0x08 => "int".into(), 0x0c => "float".into(), 0x0e => "string".into(), 0x1c => "object".into(),
        _ => format!("unresolved-kind-{kind:#x}-at-{address:#x}"),
    })
}
fn registration(pair: &[u8], meta: &[u8]) -> Result<u64> {
    let count = u32_at(meta, 164)? / 88;
    let mut found = None;
    for at in (0..pair.len().saturating_sub(112)).step_by(8) {
        if u32_at(pair, at + 80)? != count || u32_at(pair, at + 96)? != count { continue; }
        let phoff = u64_at(pair, 32)? as usize;
        let phsize = u16::from_le_bytes(pair[54..56].try_into()?) as usize;
        let phcount = u16::from_le_bytes(pair[56..58].try_into()?) as usize;
        for ph in (0..phcount).map(|i|phoff + phsize * i) {
            if u32_at(pair, ph)? != 1 { continue; }
            let offset = u64_at(pair, ph + 8)? as usize;
            let size = u64_at(pair, ph + 32)? as usize;
            if at >= offset && at + 112 <= offset + size {
                let address = u64_at(pair, ph + 16)? + (at - offset) as u64;
                if found.replace(address).is_some() { return Err("ambiguous MetadataRegistration".into()); }
            }
        }
    }
    found.ok_or_else(|| "MetadataRegistration not found".into())
}
fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 5 { return Err("usage: inspect_generics DUMP_ELF PAIRED_ELF METADATA SPEC_INDEX...".into()); }
    let dump = fs::read(&args[1])?;
    let pair = fs::read(&args[2])?;
    let meta = fs::read(&args[3])?;
    if u32_at(&meta, 0)? != 0xfab11baf || u32_at(&meta, 4)? != 29 { return Err("requires metadata29".into()); }
    let reg = registration(&pair, &meta)?;
    let specs = word(&pair, reg + 72)?;
    let insts = word(&pair, reg + 24)?;
    if relative(&dump, reg + 72)? != specs || relative(&dump, reg + 24)? != insts { return Err("registration pointers differ from source relocations".into()); }
    let specs_count = word(&pair, reg + 64)?;
    println!("MetadataRegistration={reg:#x} methodSpecs={specs:#x} specsCount={specs_count} genericInsts={insts:#x}");
    for arg in &args[4..] {
        let index = arg.parse::<u32>()?;
        if u64::from(index) >= specs_count { return Err("method spec index out of range".into()); }
        let address = specs + u64::from(index) * 12;
        let at = file_offset(&pair, address, 12)?;
        let source_at = file_offset(&dump, address, 12)?;
        let method = u32_at(&dump, source_at)?;
        let class_inst = u32_at(&dump, source_at + 4)? as i32;
        let method_inst = u32_at(&dump, source_at + 8)? as i32;
        println!("raw-spec index={index} address={address:#x} source=({method},{class_inst},{method_inst}) counterpart=({},{},{})", u32_at(&pair, at)?, u32_at(&pair, at + 4)? as i32, u32_at(&pair, at + 8)? as i32);
        if method >= u32_at(&meta, 52)? / 32 { return Err("method definition index out of range".into()); }
        let method_at = u32_at(&meta, 48)? as usize + method as usize * 32;
        let owner = definition(&meta, u32_at(&meta, method_at + 4)?)?;
        let method_name = string(&meta, u32_at(&meta, method_at)?)?;
        println!("spec-index={index} address={address:#x} source-equal={} method-index={method} name={owner}::{method_name} class-inst={class_inst} method-inst={method_inst}", pair[at..at + 12] == dump[source_at..source_at + 12]);
        for (label, inst_index) in [("class", class_inst), ("method", method_inst)] {
            if inst_index < 0 { continue; }
            if inst_index as u64 >= word(&pair, reg + 16)? { return Err("generic inst index out of range".into()); }
            let pointer_slot = insts + inst_index as u64 * 8;
            let inst = word(&pair, pointer_slot)?;
            if relative(&dump, pointer_slot)? != inst { return Err("generic instance pointer differs".into()); }
            let count = word(&pair, inst)?;
            if count > 16 { return Err("generic argument count exceeds bound".into()); }
            let argv = word(&pair, inst + 8)?;
            if relative(&dump, inst + 8)? != argv { return Err("generic argument vector pointer differs".into()); }
            println!("{label}-inst index={inst_index} pointer-slot={pointer_slot:#x} address={inst:#x} count={count} argv={argv:#x}");
            for i in 0..count {
                let address = word(&pair, argv + i * 8)?;
                if relative(&dump, argv + i * 8)? != address { return Err("generic argument pointer differs".into()); }
                println!("{label}-argument[{i}]={}", native_type(&pair, &dump, &meta, address, 0)?);
            }
        }
    }
    Ok(())
}
