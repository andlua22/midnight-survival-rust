//! Inspect explicit source inventory bodies without inferring their transport provenance.
use midnight_game_data::{
    inventory_wire::{
        BagItem, InventoryBody, ItemNumberChange, MAX_INVENTORY_BODY_BYTES, S_ITEMS_TYPE_ID,
        read_inventory_body,
    },
    package::{PackageError, load_package},
    source_bag::{SourceBagState, SourceInventoryClock},
};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const MAX_PACKETS: usize = 32;
const PREVIEW_ENTRIES: usize = 20;
const HELP: &str = "inspect_inventory --catalog VERIFIED_GAMEPLAY_PACKAGE \
    --packet TYPE_ID REALTIME_SECONDS BODY_PATH [--packet TYPE_ID REALTIME_SECONDS BODY_PATH ...] \
    [--report NEW_PATH]\n\
Inspect at most 32 explicit message bodies (16 MiB each), with SItems first and\n\
nondecreasing Unity realtimeSinceStartup seconds. Verify the source catalog,\n\
apply all bodies, and emit compact JSON with at most 20 entries per collection.\n\
Existing reports are refused. Original framing/authentication and captured-body\n\
provenance are not established by decoding or hashing.";

struct PacketInput {
    type_id: i32,
    realtime_seconds: f32,
    clock: SourceInventoryClock,
    path: PathBuf,
}

struct Options {
    catalog: PathBuf,
    packets: Vec<PacketInput>,
    report: Option<PathBuf>,
}

#[derive(Serialize)]
struct PacketReport {
    type_id: i32,
    message: &'static str,
    body_path: PathBuf,
    body_sha256: String,
    body_bytes: usize,
    realtime_seconds: f32,
    applied_changed: bool,
    count_alert: Option<ItemNumberChange>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("inspect_inventory: {error}");
            ExitCode::FAILURE
        }
    }
}

fn next_argument(
    args: &mut impl Iterator<Item = OsString>,
    expected: &str,
) -> Result<OsString, PackageError> {
    args.next()
        .ok_or_else(|| format!("missing {expected}; see --help").into())
}

fn next_path(
    args: &mut impl Iterator<Item = OsString>,
    expected: &str,
) -> Result<PathBuf, PackageError> {
    let value = next_argument(args, expected)?;
    let path = value
        .to_str()
        .ok_or_else(|| format!("{expected} must be UTF-8 for JSON reporting"))?;
    Ok(PathBuf::from(path))
}

fn options() -> Result<Option<Options>, PackageError> {
    let mut args = std::env::args_os().skip(1);
    let mut catalog = None;
    let mut report = None;
    let mut packets: Vec<PacketInput> = Vec::new();
    while let Some(flag) = args.next() {
        match flag.to_str() {
            Some("--help" | "-h") => {
                println!("{HELP}");
                return Ok(None);
            }
            Some("--catalog") if catalog.is_none() => {
                catalog = Some(next_path(&mut args, "--catalog path")?);
            }
            Some("--report") if report.is_none() => {
                report = Some(next_path(&mut args, "--report path")?);
            }
            Some("--catalog" | "--report") => {
                return Err(format!("repeated argument {}", flag.to_string_lossy()).into());
            }
            Some("--packet") => {
                if packets.len() == MAX_PACKETS {
                    return Err("at most 32 --packet arguments are allowed".into());
                }
                let type_arg = next_argument(&mut args, "--packet TYPE_ID")?;
                let type_id = type_arg
                    .to_str()
                    .ok_or("TYPE_ID must be UTF-8")?
                    .parse::<i32>()
                    .map_err(|_| "TYPE_ID must be a signed decimal i32")?;
                let time_arg = next_argument(&mut args, "--packet REALTIME_SECONDS")?;
                let realtime_seconds = time_arg
                    .to_str()
                    .ok_or("REALTIME_SECONDS must be UTF-8")?
                    .parse::<f32>()
                    .map_err(|_| "REALTIME_SECONDS must be a finite nonnegative f32")?;
                let clock = SourceInventoryClock::from_realtime_seconds(realtime_seconds)?;
                if packets
                    .last()
                    .is_some_and(|previous| realtime_seconds < previous.realtime_seconds)
                {
                    return Err("packet receive realtime seconds must be nondecreasing".into());
                }
                packets.push(PacketInput {
                    type_id,
                    realtime_seconds,
                    clock,
                    path: next_path(&mut args, "--packet BODY_PATH")?,
                });
            }
            _ => return Err(format!("unknown argument {}", flag.to_string_lossy()).into()),
        }
    }
    let catalog = catalog.ok_or("--catalog is required; see --help")?;
    if packets.first().map(|packet| packet.type_id) != Some(S_ITEMS_TYPE_ID) {
        return Err(
            "at least one --packet is required, and the first must be SItems (59771964)".into(),
        );
    }
    Ok(Some(Options {
        catalog,
        packets,
        report,
    }))
}

fn read_body(path: &Path) -> Result<Vec<u8>, PackageError> {
    // Refuse streams/devices before opening, then bound the actual file read too.
    if !std::fs::metadata(path)?.is_file() {
        return Err("body path must identify a regular file".into());
    }
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > MAX_INVENTORY_BODY_BYTES as u64 {
        return Err("body file is not regular or exceeds 16 MiB".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(MAX_INVENTORY_BODY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_INVENTORY_BODY_BYTES
        || bytes.len() as u64 != metadata.len()
        || file.metadata()?.len() != metadata.len()
    {
        return Err("body file exceeded its bound or changed length while reading".into());
    }
    Ok(bytes)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn message_name(body: &InventoryBody) -> &'static str {
    match body {
        InventoryBody::Items(_) => "SItems",
        InventoryBody::ItemChanged(_) => "SItemChanged",
        InventoryBody::QuickUseItemChanged(_) => "SQuickUseItemChanged",
        InventoryBody::BagSizeChange(_) => "SBagSizeChange",
        InventoryBody::ItemDurationChanged(_) => "SItemDurationChanged",
        InventoryBody::ItemNumberChange(_) => "SItemNumberChange",
        InventoryBody::SetQuickUseItem(_) => "SSetQuickUseItem",
        InventoryBody::PutToHand(_) => "SPutToHand",
        InventoryBody::SyncBagItem(_) => "SSyncBagItem",
        InventoryBody::ClearAllBag => "SClearAllBag",
    }
}

fn item_preview(item: &BagItem) -> Value {
    json!({
        "instance_id": item.instance_id,
        "item_id": item.item_id,
        "number": item.number,
        "duration": item.duration,
        "timeout": item.timeout,
        "is_bind": item.is_bind,
        "bind_timeout": item.bind_timeout,
        "extra_info_bytes": item.extra_info.len(),
        "extra_info_sha256": hash(&item.extra_info),
    })
}

fn write_new_report(path: &Path, bytes: &[u8]) -> Result<(), PackageError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    let result = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = result {
        if let Err(cleanup) = std::fs::remove_file(path) {
            return Err(format!(
                "report write failed: {error}; incomplete report cleanup failed: {cleanup}"
            )
            .into());
        }
        return Err(error.into());
    }
    Ok(())
}

fn run() -> Result<(), PackageError> {
    let Some(options) = options()? else {
        return Ok(());
    };
    if let Some(path) = &options.report {
        match std::fs::symlink_metadata(path) {
            Ok(_) => return Err(format!("report already exists: {}", path.display()).into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let (manifest, catalog) = load_package(&options.catalog)
        .map_err(|error| format!("catalog {}: {error}", options.catalog.display()))?;
    let mut state = SourceBagState::default();
    let mut packets = Vec::with_capacity(options.packets.len());
    for (index, input) in options.packets.iter().enumerate() {
        let context = format!(
            "packet {} ({}, type {})",
            index + 1,
            input.path.display(),
            input.type_id
        );
        let bytes = read_body(&input.path).map_err(|error| format!("{context}: {error}"))?;
        let body = read_inventory_body(input.type_id, &bytes)
            .map_err(|error| format!("{context}: decode failed: {error}"))?;
        let outcome = state
            .apply(&catalog, &body, input.clock)
            .map_err(|error| format!("{context}: apply failed: {error}"))?;
        packets.push(PacketReport {
            type_id: input.type_id,
            message: message_name(&body),
            body_path: input.path.clone(),
            body_sha256: hash(&bytes),
            body_bytes: bytes.len(),
            realtime_seconds: input.realtime_seconds,
            applied_changed: outcome.changed,
            count_alert: outcome.count_alert,
        });
    }
    let snapshot = state.snapshot().ok_or("no source snapshot applied")?;
    let preview = |items: &[BagItem]| {
        items
            .iter()
            .take(PREVIEW_ENTRIES)
            .map(item_preview)
            .collect::<Vec<_>>()
    };
    let quick_preview = snapshot
        .quick_use_items
        .iter()
        .take(PREVIEW_ENTRIES)
        .map(|(&index, item)| json!({ "index": index, "item": item_preview(item) }))
        .collect::<Vec<_>>();
    let report = json!({
        "schema_version": 1,
        "catalog": {
            "package_path": options.catalog,
            "source_apk_sha256": manifest.source_apk_sha256,
            "catalog_sha256": manifest.catalog.sha256,
            "catalog_bytes": manifest.catalog.bytes,
        },
        "packets": packets,
        "final_state": {
            "stored_capacity": snapshot.capacity,
            "visible_capacity": state.visible_capacity().ok_or("no visible source capacity")?,
            "hand_instance_id": state.hand_instance_id(),
            "show_equip": snapshot.show_equip,
            "counts": {
                "bag": snapshot.bag_items.len(), "skin": snapshot.skin_items.len(),
                "quick": snapshot.quick_use_items.len(), "equip": snapshot.equip_items.len(),
            },
            "preview_limit_per_collection": PREVIEW_ENTRIES,
            "entries": {
                "bag": preview(&snapshot.bag_items), "skin": preview(&snapshot.skin_items),
                "quick": quick_preview, "equip": preview(&snapshot.equip_items),
            },
        },
        "limitations": [
            "Original network framing/session authentication and captured-body provenance are not established by decoding or hashing these explicit bodies.",
            "Native IDs and handlers describe static unpatched base methods; IFix state, expiry and weapon effects remain unobserved or unported.",
            "Snapshot-first sequencing, nondecreasing receive time, stored-count/identity validation and allocation bounds are reconstruction admission policies.",
        ],
    });
    let mut bytes = serde_json::to_vec(&report)?;
    bytes.push(b'\n');
    if let Some(path) = &options.report {
        write_new_report(path, &bytes)
            .map_err(|error| format!("report {}: {error}", path.display()))?;
    }
    std::io::stdout().lock().write_all(&bytes)?;
    Ok(())
}
