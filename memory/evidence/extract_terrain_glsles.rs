//! Extract intact length-prefixed GLES3 shader strings from an already
//! decompressed Unity 2021 shader subprogram blob. This bounded evidence probe
//! does not reinterpret non-text GPU programs or edit application assets.
//! Build: rustc --edition=2024 memory/evidence/extract_terrain_glsles.rs -o /tmp/extract-terrain-glsles
//! Run: /tmp/extract-terrain-glsles 4 /tmp/terrain-4-glsles-subprograms.bin memory/evidence/terrain-glsles

use std::{fs, io, path::Path};

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn read_u32(bytes: &[u8], offset: usize) -> io::Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| invalid("offset overflow"))?;
    let value: [u8; 4] = bytes
        .get(offset..end)
        .ok_or_else(|| invalid(format!("truncated u32 at {offset}")))?
        .try_into()
        .map_err(|_| invalid("u32 byte count"))?;
    Ok(u32::from_le_bytes(value))
}

fn main() -> io::Result<()> {
    let arguments: Vec<_> = std::env::args().collect();
    if arguments.len() != 4 || !["3", "4"].contains(&arguments[1].as_str()) {
        return Err(invalid(
            "usage: extract-terrain-glsles <3|4> <decompressed-blob> <output-directory>",
        ));
    }
    let layer_count = &arguments[1];
    let data = fs::read(&arguments[2])?;
    let output = Path::new(&arguments[3]);
    fs::create_dir_all(output)?;
    fs::write(
        output.join(format!("terrain-{layer_count}-subprograms.bin")),
        &data,
    )?;

    let count = read_u32(&data, 0)? as usize;
    if count == 0 || count > 65_536 {
        return Err(invalid(format!("implausible subprogram count {count}")));
    }
    let table_end = 4usize
        .checked_add(
            count
                .checked_mul(12)
                .ok_or_else(|| invalid("table overflow"))?,
        )
        .ok_or_else(|| invalid("table overflow"))?;
    if table_end > data.len() {
        return Err(invalid("truncated subprogram table"));
    }

    let mut manifest = String::from(
        "index\tentry_offset\tentry_length\tblob_index\tprogram_version\tgpu_type\tcode_offset\tcode_length\ttrailing_nul_bytes\tprefix_keywords\tsource_file\n",
    );
    let mut extracted = 0;
    let mut maximum_end = table_end;
    for index in 0..count {
        let table_offset = 4 + index * 12;
        let entry_offset = read_u32(&data, table_offset)? as usize;
        let entry_length = read_u32(&data, table_offset + 4)? as usize;
        let blob_index = read_u32(&data, table_offset + 8)?;
        let entry_end = entry_offset
            .checked_add(entry_length)
            .ok_or_else(|| invalid("entry overflow"))?;
        if blob_index != 0 || entry_offset < table_end || entry_end > data.len() || entry_length < 8
        {
            return Err(invalid(format!("invalid or external table entry {index}")));
        }
        maximum_end = maximum_end.max(entry_end);
        let version = read_u32(&data, entry_offset)?;
        let gpu_type = read_u32(&data, entry_offset + 4)?;
        let entry = &data[entry_offset..entry_end];
        let starts: Vec<_> = entry
            .windows(b"#ifdef VERTEX\n".len())
            .enumerate()
            .filter_map(|(at, value)| (value == b"#ifdef VERTEX\n").then_some(at))
            .collect();
        if starts.is_empty() {
            manifest.push_str(&format!("{index}\t{entry_offset}\t{entry_length}\t{blob_index}\t{version}\t{gpu_type}\t\t\t\t\t\n"));
            continue;
        }
        if starts.len() != 1 || starts[0] < 4 {
            return Err(invalid(format!(
                "ambiguous shader source in subprogram {index}"
            )));
        }
        let code_offset = entry_offset + starts[0];
        let code_length = read_u32(&data, code_offset - 4)? as usize;
        let code_end = code_offset
            .checked_add(code_length)
            .ok_or_else(|| invalid("source overflow"))?;
        if code_end > entry_end {
            return Err(invalid(format!("source exceeds subprogram {index}")));
        }
        let code = &data[code_offset..code_end];
        let trailing_nul_bytes = code.iter().rev().take_while(|&&value| value == 0).count();
        let text = &code[..code.len() - trailing_nul_bytes];
        if text.contains(&0)
            || !text
                .windows(b"#ifdef FRAGMENT\n".len())
                .any(|value| value == b"#ifdef FRAGMENT\n")
        {
            return Err(invalid(format!(
                "unexpected combined source layout in subprogram {index}"
            )));
        }
        std::str::from_utf8(text)
            .map_err(|error| invalid(format!("non-UTF8 source {index}: {error}")))?;

        // These are exact length-prefixed uppercase strings in the prefix.
        // Do not infer the distinction between local/global keyword tables.
        let mut prefix_keywords = Vec::new();
        for at in (entry_offset + 8..code_offset.saturating_sub(4)).step_by(4) {
            let length = read_u32(&data, at)? as usize;
            if !(1..=128).contains(&length) {
                continue;
            }
            let Some(end) = (at + 4).checked_add(length) else {
                continue;
            };
            if end > code_offset - 4 {
                continue;
            }
            let token = &data[at + 4..end];
            if token
                .iter()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_')
            {
                let token = std::str::from_utf8(token).map_err(|_| invalid("ASCII keyword"))?;
                prefix_keywords.push(token);
            }
        }
        let basename = format!("terrain-{layer_count}-program-{index:02}");
        let source_file = format!("{basename}.glsl");
        fs::write(output.join(&source_file), text)?;
        fs::write(output.join(format!("{basename}.program-code.bin")), code)?;
        manifest.push_str(&format!("{index}\t{entry_offset}\t{entry_length}\t{blob_index}\t{version}\t{gpu_type}\t{code_offset}\t{code_length}\t{trailing_nul_bytes}\t{}\t{source_file}\n", prefix_keywords.join(",")));
        extracted += 1;
    }
    if maximum_end != data.len() || extracted == 0 {
        return Err(invalid(format!(
            "uncovered bytes or missing GLSL: last={maximum_end} total={} sources={extracted}",
            data.len()
        )));
    }
    fs::write(
        output.join(format!("terrain-{layer_count}-manifest.tsv")),
        manifest,
    )?;
    println!(
        "terrain-{layer_count}: {count} table entries, {extracted} complete GLSL strings; all ranges bounded, final byte covered"
    );
    Ok(())
}
