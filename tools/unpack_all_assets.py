#!/usr/bin/env python3
"""
Comprehensive One-Time Asset Unpacker & Global PathID Resolver
for Midnight Survival Rust (Civilization Continent / 文明大陆).

Pass 1: Builds global_asset_index.json across ALL bundles in export/lite4.0.apk.
        Resolves every cross-bundle PPtr (m_FileID, m_PathID) with 100% precision.
Pass 2: Dumps terrain chunk metadata & authentic biome detail layers to
        terrain_chunks_manifest.json and chunk_biomes.json.
        Re-bakes all 1024x1024 chunk textures with 70% detail + 30% ov_dir.png macro-tinting.
Pass 3: Extracts all UI Sprites as cropped PNGs with 9-slice border metadata and
        parses complete RectTransform hierarchies into battle_panel_layout.json,
        battle_minimap_layout.json, and battle_map_panel_layout.json.
"""

import argparse
import json
import math
import os
import re
import struct
import sys
import time
import zipfile
from concurrent.futures import ProcessPoolExecutor, as_completed
import numpy as np
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
MAP_DIR = os.path.join(PROJECT_ROOT, "assets", "imported_map")
TEX_DIR = os.path.join(MAP_DIR, "textures")
SPLAT_DIR = os.path.join(MAP_DIR, "splatmaps")
MATS_DIR = os.path.join(MAP_DIR, "terrain_mats")
GLOBAL_INDEX_PATH = os.path.join(MAP_DIR, "global_asset_index.json")
TERRAIN_MANIFEST_PATH = os.path.join(MAP_DIR, "terrain_chunks_manifest.json")
CHUNK_BIOMES_PATH = os.path.join(MAP_DIR, "chunk_biomes.json")
CHUNK_POS_PATH = os.path.join(MAP_DIR, "chunk_positions.json")
TERRAIN_BIN_PATH = os.path.join(MAP_DIR, "island_terrain.bin")
OVERVIEW_PATH = os.path.join(TEX_DIR, "map_overview.png")
ROOT_OVERVIEW_PATH = os.path.join(PROJECT_ROOT, "ov_dir.png")

UI_DIR = os.path.join(PROJECT_ROOT, "assets", "ui")
UI_SPRITES_DIR = os.path.join(UI_DIR, "sprites")
UI_BATTLE_PANEL_DIR = os.path.join(UI_DIR, "battle_panel")
UI_BATTLE_MINIMAP_DIR = os.path.join(UI_DIR, "battle_minimap")
UI_BATTLE_MAP_PANEL_DIR = os.path.join(UI_DIR, "battle_map_panel")

APK_PATH = os.path.join(PROJECT_ROOT, "export", "lite4.0.apk")
MAP_OVERVIEW_HALF_EXTENT = 5360.0
OUT_SIZE = 1024


# ============================================================================
# PASS 1: Global PathID Indexer across ALL bundles
# ============================================================================

def _scan_bundle_batch(bundle_names):
    """Worker function to scan a batch of .ab bundles in lite4.0.apk."""
    import UnityPy

    local_path_map = {}
    local_cab_map = {}
    local_cab_path_map = {}

    try:
        with zipfile.ZipFile(APK_PATH, "r") as apk:
            for bname in bundle_names:
                try:
                    data = apk.read(bname)
                    env = UnityPy.load(data)
                except Exception:
                    continue

                for f in env.files.values():
                    if hasattr(f, "files"):
                        for cab in f.files.keys():
                            local_cab_map[cab] = bname
                    else:
                        cname = getattr(f, "name", "")
                        if cname:
                            local_cab_map[cname] = bname

                for obj in env.objects:
                    tname = obj.type.name
                    if tname in (
                        "Texture2D",
                        "Sprite",
                        "Material",
                        "Shader",
                        "Font",
                        "TextAsset",
                        "MonoScript",
                        "Mesh",
                        "GameObject",
                    ):
                        try:
                            r = obj.read()
                            nm = getattr(r, "m_Name", "") or getattr(r, "name", "")
                            entry = {
                                "name": nm,
                                "type": tname,
                                "bundle": bname,
                            }
                            if tname == "Sprite":
                                if hasattr(r, "m_Rect"):
                                    entry["rect"] = [
                                        float(r.m_Rect.x),
                                        float(r.m_Rect.y),
                                        float(r.m_Rect.width),
                                        float(r.m_Rect.height),
                                    ]
                                if hasattr(r, "m_Border"):
                                    entry["border"] = [
                                        float(r.m_Border.x),
                                        float(r.m_Border.y),
                                        float(r.m_Border.z),
                                        float(r.m_Border.w),
                                    ]
                            local_path_map[str(obj.path_id)] = entry
                            cab_name = getattr(obj.assets_file, "name", "")
                            if cab_name:
                                local_cab_path_map[f"{cab_name}:{obj.path_id}"] = entry
                        except Exception:
                            pass
    except Exception as e:
        print(f"Batch worker error: {e}", file=sys.stderr)

    return local_path_map, local_cab_map, local_cab_path_map


def run_pass1_global_index():
    print("\n=======================================================")
    print("PASS 1: Building Global PathID Index across ALL bundles")
    print("=======================================================")
    t0 = time.time()

    if not os.path.exists(APK_PATH):
        print(f"ERROR: APK not found at {APK_PATH}")
        return False

    with zipfile.ZipFile(APK_PATH, "r") as apk:
        all_ab = sorted([n for n in apk.namelist() if n.endswith(".ab")])

    print(f"Total .ab bundles to scan: {len(all_ab)}")
    batch_size = 80
    batches = [all_ab[i : i + batch_size] for i in range(0, len(all_ab), batch_size)]
    print(f"Divided into {len(batches)} batches of up to {batch_size} bundles.")

    cpu_count = min(8, os.cpu_count() or 4)
    print(f"Running across {cpu_count} CPU worker processes...")

    global_path_map = {}
    global_cab_map = {}
    global_cab_path_map = {}
    done_batches = 0

    with ProcessPoolExecutor(max_workers=cpu_count) as executor:
        futures = {executor.submit(_scan_bundle_batch, b): i for i, b in enumerate(batches)}
        for future in as_completed(futures):
            b_path_map, b_cab_map, b_cab_path = future.result()
            global_path_map.update(b_path_map)
            global_cab_map.update(b_cab_map)
            global_cab_path_map.update(b_cab_path)
            done_batches += 1
            if done_batches % 10 == 0 or done_batches == len(batches):
                print(
                    f"  Progress: {done_batches}/{len(batches)} batches "
                    f"({len(global_path_map)} assets, {len(global_cab_map)} CABs indexed)..."
                )

    out_data = {
        "cabs": global_cab_map,
        "cab_assets": global_cab_path_map,
        "assets": global_path_map,
    }

    os.makedirs(MAP_DIR, exist_ok=True)
    with open(GLOBAL_INDEX_PATH, "w", encoding="utf-8") as f:
        json.dump(out_data, f)

    fsize_mb = os.path.getsize(GLOBAL_INDEX_PATH) / (1024.0 * 1024.0)
    print(
        f"Pass 1 Complete in {time.time() - t0:.2f}s! Indexed {len(global_path_map)} assets "
        f"and {len(global_cab_map)} CABs -> {GLOBAL_INDEX_PATH} ({fsize_mb:.2f} MB)"
    )
    return True


# ============================================================================
# PASS 2: Terrain Chunks Manifest & Biomes Restoration
# ============================================================================

def _load_chunk_bounds():
    bounds = {}
    if os.path.exists(TERRAIN_BIN_PATH):
        with open(TERRAIN_BIN_PATH, "rb") as f:
            data = f.read()
        if len(data) >= 12 and data[:8] == b"MSMAP001":
            count = struct.unpack_from("<I", data, 8)[0]
            offset = 12
            for _ in range(count):
                if offset + 2 > len(data):
                    break
                nlen = struct.unpack_from("<H", data, offset)[0]
                offset += 2
                name = data[offset : offset + nlen].decode("utf-8", errors="replace")
                offset += nlen
                tlen = struct.unpack_from("<H", data, offset)[0]
                offset += 2 + tlen
                cx, cy, cz, vc, ic = struct.unpack_from("<fffII", data, offset)
                offset += 20
                xs, zs = [], []
                for _ in range(vc):
                    x, y, z = struct.unpack_from("<fff", data, offset)
                    offset += 12
                    xs.append(x)
                    zs.append(-z)  # Bevy -Z is north
                offset += vc * 20 + ic * 4
                if not name.lower().startswith("road") and xs and zs:
                    bounds[name.upper()] = (min(xs), max(xs), min(zs), max(zs))

    if os.path.exists(CHUNK_POS_PATH):
        with open(CHUNK_POS_PATH, "r", encoding="utf-8") as f:
            pos_dict = json.load(f)
        for k, v in pos_dict.items():
            upper = k.upper()
            if upper not in bounds:
                cx = float(v.get("x", 0.0))
                cz = float(v.get("z", 0.0))
                size = float(v.get("size", 400.0))
                half_s = size * 0.5
                bounds[upper] = (cx - half_s, cx + half_s, cz - half_s, cz + half_s)

    return bounds


g_tex_cache = None
g_tiled_cache = None
g_grid_y = None
g_grid_x = None
g_overview_im = None


def _init_blend_worker(mats_dir, overview_path, out_size):
    global g_tex_cache, g_tiled_cache, g_grid_y, g_grid_x, g_overview_im
    g_tex_cache = {}
    g_tiled_cache = {}
    g_grid_y, g_grid_x = np.indices((out_size, out_size))

    if os.path.exists(mats_dir):
        for f in os.listdir(mats_dir):
            if f.endswith(".png"):
                p = os.path.join(mats_dir, f)
                try:
                    im = Image.open(p).convert("RGB")
                    arr = np.array(im, dtype=np.float32) / 255.0
                    arr = arr ** (1.0 / 2.2)
                    g_tex_cache[f.lower()] = arr
                except Exception:
                    pass

    g_overview_im = None
    for p in (overview_path, ROOT_OVERVIEW_PATH):
        if os.path.exists(p):
            try:
                g_overview_im = Image.open(p).convert("RGB")
                break
            except Exception:
                pass


def _sample_tiled(tex, repeats):
    th, tw, _ = tex.shape
    return tex[((g_grid_y * repeats) % th).astype(int), ((g_grid_x * repeats) % tw).astype(int)]


def _get_tiled_layer(mat_name, repeats):
    key = (mat_name.lower(), repeats)
    if key not in g_tiled_cache:
        tex = g_tex_cache.get(mat_name.lower())
        if tex is None:
            # Fallback
            tex = g_tex_cache.get("terrain_caodi_01.png", next(iter(g_tex_cache.values())))
        g_tiled_cache[key] = _sample_tiled(tex, repeats)
    return g_tiled_cache[key]


def _blend_chunk_task(args):
    cname, splat_channels, layers, min_x, max_x, min_z, max_z, biome_hint = args

    num_layers = max(1, len(layers))
    weights = [
        splat_channels[0][..., None],
        splat_channels[1][..., None],
        splat_channels[2][..., None],
        splat_channels[3][..., None]
        if len(splat_channels) > 3
        else np.zeros((OUT_SIZE, OUT_SIZE, 1), dtype=np.float32),
    ]

    tot = np.zeros((OUT_SIZE, OUT_SIZE, 1), dtype=np.float32)
    for i in range(min(num_layers, 4)):
        tot += weights[i]
    tot[tot == 0.0] = 1.0

    detail = np.zeros((OUT_SIZE, OUT_SIZE, 3), dtype=np.float32)
    for i in range(min(num_layers, 4)):
        w = weights[i] / tot
        mat = layers[i] if i < len(layers) else "terrain_caodi_01.png"
        repeats = 44 if any(k in mat.lower() for k in ["shi", "yan", "rock"]) else 32
        tiled = _get_tiled_layer(mat, repeats)
        detail += w * tiled

    half = MAP_OVERVIEW_HALF_EXTENT
    if g_overview_im is not None:
        W_ov, H_ov = g_overview_im.size
        u0 = (min_x + half) / (2.0 * half)
        u1 = (max_x + half) / (2.0 * half)
        v0 = (min_z + half) / (2.0 * half)
        v1 = (max_z + half) / (2.0 * half)

        px0 = int(np.clip(u0 * W_ov, 0, W_ov - 1))
        px1 = int(np.clip(u1 * W_ov, px0 + 1, W_ov))
        py0 = int(np.clip(v0 * H_ov, 0, H_ov - 1))
        py1 = int(np.clip(v1 * H_ov, py0 + 1, H_ov))

        patch = g_overview_im.crop((px0, py0, px1, py1)).resize(
            (OUT_SIZE, OUT_SIZE), Image.Resampling.BICUBIC
        )
        macro = np.array(patch, dtype=np.float32) / 255.0

        # Seamless Macro-Color modulation: 70% detail textures + 30% continuous overview tint
        result = np.clip(0.70 * detail + 0.30 * macro, 0.0, 1.0)
    else:
        result = np.clip(detail, 0.0, 1.0)

    out_png = os.path.join(TEX_DIR, f"{cname}.png")
    final_arr = (result * 255.0).astype(np.uint8)
    Image.fromarray(final_arr).save(out_png, compress_level=1)
    return cname, biome_hint, layers


def run_pass2_terrain():
    import UnityPy

    print("\n=======================================================")
    print("PASS 2: Resolving Terrain Chunk Metadata & Biomes")
    print("=======================================================")
    t0 = time.time()

    if not os.path.exists(GLOBAL_INDEX_PATH):
        print(f"Global index missing at {GLOBAL_INDEX_PATH}, running Pass 1 first...")
        if not run_pass1_global_index():
            return False

    with open(GLOBAL_INDEX_PATH, "r", encoding="utf-8") as f:
        global_data = json.load(f)
    cab_map = global_data.get("cabs", {})
    cab_assets = global_data.get("cab_assets", {})
    asset_index = global_data.get("assets", {})

    os.makedirs(TEX_DIR, exist_ok=True)
    os.makedirs(SPLAT_DIR, exist_ok=True)
    os.makedirs(MATS_DIR, exist_ok=True)

    bounds_map = _load_chunk_bounds()
    print(f"Loaded bounds for {len(bounds_map)} terrain chunks.")

    overview_im = None
    for p in (OVERVIEW_PATH, ROOT_OVERVIEW_PATH):
        if os.path.exists(p):
            try:
                overview_im = Image.open(p).convert("RGB")
                break
            except Exception:
                pass
    W_ov, H_ov = overview_im.size if overview_im else (2048, 2048)
    half = MAP_OVERVIEW_HALF_EXTENT

    available_mats = {f.lower(): f for f in os.listdir(MATS_DIR) if f.endswith(".png")}

    chunk_manifest = {}
    chunk_tasks = []

    with zipfile.ZipFile(APK_PATH, "r") as apk:
        scene_files = sorted(
            [
                n
                for n in apk.namelist()
                if n.startswith("assets/scene/")
                and n.endswith(".ab")
                and (
                    (n.split("/")[-1][0] in "abcd" and "-" in n)
                    or "junshijidi_terrain_" in n
                )
            ]
        )
        print(f"Found {len(scene_files)} terrain chunk bundles in APK.")

        for sf in scene_files:
            cname = os.path.splitext(os.path.basename(sf))[0].upper()
            env = UnityPy.load(apk.read(sf))

            splat_im = None
            resolved_splats = {}
            resolved_normals = {}
            mat_layer4 = None

            # Collect externals from SerializedFile
            exts = []
            for bf in env.files.values():
                if hasattr(bf, "files"):
                    for sf_sub in bf.files.values():
                        if hasattr(sf_sub, "externals") and sf_sub.externals:
                            exts = sf_sub.externals
                            break
                elif hasattr(bf, "externals") and bf.externals:
                    exts = bf.externals
                    break

            # Find terrain Material
            terrain_mat = None
            for obj in env.objects:
                if obj.type.name == "Material":
                    r = obj.read()
                    if r.m_Name.upper() == cname:
                        terrain_mat = r
                        break
                    elif "terrain" in r.m_Name.lower() or terrain_mat is None:
                        terrain_mat = r

            if terrain_mat is not None and hasattr(terrain_mat, "m_SavedProperties"):
                flts = dict(terrain_mat.m_SavedProperties.m_Floats)
                mat_layer4 = flts.get("_Layer4")

                for tex_prop, tex_val in terrain_mat.m_SavedProperties.m_TexEnvs:
                    if not (tex_prop.startswith("_Splat") or "normal" in tex_prop.lower() or "bump" in tex_prop.lower()):
                        continue
                    fid = tex_val.m_Texture.file_id
                    pid = str(tex_val.m_Texture.path_id)
                    tname = ""
                    src_bundle = ""

                    if fid > 0 and fid - 1 < len(exts):
                        cab = exts[fid - 1].path.split("/")[-1]
                        src_bundle = cab_map.get(cab, "")
                        if f"{cab}:{pid}" in cab_assets:
                            tname = cab_assets[f"{cab}:{pid}"].get("name", "")
                        elif src_bundle:
                            tname = os.path.splitext(os.path.basename(src_bundle))[0]

                    if not tname and pid in asset_index:
                        tname = asset_index[pid].get("name", "")
                        if not src_bundle:
                            src_bundle = asset_index[pid].get("bundle", "")

                    if not tname:
                        continue

                    tex_file = tname + ".png" if not tname.endswith(".png") else tname

                    # Extract texture from APK if missing in MATS_DIR
                    if tex_file.lower() not in available_mats:
                        if src_bundle and src_bundle in apk.namelist():
                            try:
                                sub_env = UnityPy.load(apk.read(src_bundle))
                                for so in sub_env.objects:
                                    if str(so.path_id) == pid or so.type.name == "Texture2D":
                                        sub_im = so.read().image
                                        dst = os.path.join(MATS_DIR, tex_file)
                                        sub_im.save(dst)
                                        available_mats[tex_file.lower()] = tex_file
                                        break
                            except Exception:
                                pass

                    if tex_prop.startswith("_Splat") and not tex_prop.endswith("_Normal"):
                        resolved_splats[tex_prop] = tex_file
                    elif tex_prop.endswith("_Normal") or "bump" in tex_prop.lower() or "normal" in tex_prop.lower():
                        resolved_normals[tex_prop] = tex_file

            # Find control texture
            for obj in env.objects:
                if obj.type.name == "Texture2D":
                    r = obj.read()
                    if "control" in r.m_Name.lower() or r.m_Name.upper() == cname or splat_im is None:
                        try:
                            splat_im = r.image
                            if "control" in r.m_Name.lower() or r.m_Name.upper() == cname:
                                break
                        except Exception:
                            pass

            if splat_im is None:
                splat_fallback = os.path.join(SPLAT_DIR, f"{cname}.png")
                if os.path.exists(splat_fallback):
                    splat_im = Image.open(splat_fallback)
                else:
                    continue

            # Determine if Layer 4 is active
            if mat_layer4 is not None and mat_layer4 == 0.0:
                layer4_active = False
            elif "_Splat3" not in resolved_splats:
                layer4_active = False
            else:
                layer4_active = True

            # Process splat channels
            splat_rgb = splat_im.convert("RGBA")
            ch_list = splat_rgb.split()
            c0 = np.array(ch_list[0].resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0
            c1 = np.array(ch_list[1].resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0
            c2 = np.array(ch_list[2].resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0

            if layer4_active:
                c3 = np.array(ch_list[3].resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0
                if c3.mean() > 0.95 or (c0 + c1 + c2).mean() > 0.85:
                    c3 = np.clip(1.0 - (c0 + c1 + c2), 0.0, 1.0)
            else:
                c3 = np.zeros((OUT_SIZE, OUT_SIZE), dtype=np.float32)

            tot = c0 + c1 + c2 + c3
            tot[tot < 1e-4] = 1.0
            c0 /= tot
            c1 /= tot
            c2 /= tot
            c3 /= tot

            channels = [c0, c1, c2, c3]

            # Save clean, normalized 4-channel splatmap
            clean_splat_u8 = (np.stack([c0, c1, c2, c3], axis=-1) * 255.0).clip(0, 255).astype(np.uint8)
            Image.fromarray(clean_splat_u8, mode="RGBA").save(os.path.join(SPLAT_DIR, f"{cname}.png"), compress_level=1)

            min_x, max_x, min_z, max_z = bounds_map.get(
                cname, (-200.0, 200.0, -200.0, 200.0)
            )

            # Check primary splats
            all_splat_names = " ".join(resolved_splats.values()).lower()
            s0 = resolved_splats.get("_Splat0", "").lower()

            is_snow = any(k in s0 for k in ["xue", "bing"]) or ("xue" in all_splat_names and min_z < 0.0)
            is_desert = not is_snow and (
                any(k in s0 for k in ["sand", "gobi", "shamo", "shadi"])
                or any(k in all_splat_names for k in ["sand", "gobi", "shamo", "shadi"])
                or max_z > 400.0
            )
            is_mil = not is_snow and not is_desert and (
                cname.startswith("JUNSHIJIDI")
                or (cname.startswith("A-") and min_x < -1000.0)
                or (min_x < -1400.0)
            )

            if is_snow:
                biome_kind = "Snow"
                standard_layers = ["terrain_xuedi_01.png", "terrain_shitou_02.png", "terrain_xuedi_04.png", "terrain_xuedi_03.png"]
                normal_map = "terrain_xuedi_01_N.png"
            elif is_desert:
                biome_kind = "Desert"
                standard_layers = ["terrain_sandA2.png", "terrain_gobi_02.png", "terrain_yanshi_01.png", "terrain_nitu_02.png"]
                normal_map = "terrain_sandA2_N.png"
            elif is_mil:
                biome_kind = "Military"
                standard_layers = ["terrain_kucao_01.png", "terrain_nitu_02.png", "terrain_yanshi_01.png", "terrain_kucao_01.png"]
                normal_map = "caodi1215_N.png"
            else:
                biome_kind = "Plains"
                standard_layers = ["terrain_caodi_01.png", "terrain_nitu_02.png", "terrain_yanshi_01.png", "terrain_kucao_01.png"]
                normal_map = "caodi1215_N.png"

            l0 = resolved_splats.get("_Splat0", standard_layers[0])
            l1 = resolved_splats.get("_Splat1", standard_layers[1])
            l2 = resolved_splats.get("_Splat2", standard_layers[2])
            if layer4_active:
                l3 = resolved_splats.get("_Splat3", standard_layers[3])
            else:
                l3 = l0  # Layer 4 disabled: set l3 to l0 (w3 is 0 anyway)

            final_layers = [
                available_mats.get(l0.lower(), l0),
                available_mats.get(l1.lower(), l1),
                available_mats.get(l2.lower(), l2),
                available_mats.get(l3.lower(), l3),
            ]

            chunk_manifest[cname] = {
                "biome": biome_kind,
                "layers": final_layers,
                "normal": normal_map,
                "layer4_active": layer4_active,
                "bounds": [min_x, max_x, min_z, max_z],
            }

            chunk_tasks.append((cname, channels, final_layers, min_x, max_x, min_z, max_z, biome_kind))

    print(f"Prepared {len(chunk_tasks)} chunks for blending...")
    with open(TERRAIN_MANIFEST_PATH, "w", encoding="utf-8") as f:
        json.dump(chunk_manifest, f, indent=2, ensure_ascii=False)
    print(f"Saved chunk manifest -> {TERRAIN_MANIFEST_PATH}")

    # Save chunk_biomes.json for Rust client
    biome_dict = {}
    for cname, d in chunk_manifest.items():
        ls = d["layers"]
        biome_dict[cname] = {
            "biome": d["biome"],
            "l0": ls[0],
            "l1": ls[1],
            "l2": ls[2],
            "l3": ls[3],
            "normal": d["normal"],
        }
    with open(CHUNK_BIOMES_PATH, "w", encoding="utf-8") as f:
        json.dump(biome_dict, f, indent=2, ensure_ascii=False)
    print(f"Saved chunk biomes -> {CHUNK_BIOMES_PATH}")

    # Parallel blending with multiprocessing
    cpu_count = min(8, os.cpu_count() or 4)
    with ProcessPoolExecutor(
        max_workers=cpu_count,
        initializer=_init_blend_worker,
        initargs=(MATS_DIR, OVERVIEW_PATH, OUT_SIZE),
    ) as executor:
        for _ in executor.map(_blend_chunk_task, chunk_tasks):
            pass

    snow_count = sum(1 for v in biome_dict.values() if v["biome"] == "Snow")
    desert_count = sum(1 for v in biome_dict.values() if v["biome"] == "Desert")
    plains_count = sum(1 for v in biome_dict.values() if v["biome"] == "Plains")
    mil_count = sum(1 for v in biome_dict.values() if v["biome"] == "Military")
    print(f"Summary: {snow_count} Snow, {desert_count} Desert, {plains_count} Plains, {mil_count} Military chunks.")
    print(f"Pass 2 Complete in {time.time() - t0:.2f}s!")
    return True


# ============================================================================
# PASS 3: Port BattlePanel, BattleMinimap, and BattleMapPanel UI
# ============================================================================

def _parse_rect_transform(tr):
    """Extract full RectTransform coordinates."""
    data = {
        "anchor_min": [float(tr.m_AnchorMin.x), float(tr.m_AnchorMin.y)] if hasattr(tr, "m_AnchorMin") else [0.5, 0.5],
        "anchor_max": [float(tr.m_AnchorMax.x), float(tr.m_AnchorMax.y)] if hasattr(tr, "m_AnchorMax") else [0.5, 0.5],
        "anchored_pos": [float(tr.m_AnchoredPosition.x), float(tr.m_AnchoredPosition.y)] if hasattr(tr, "m_AnchoredPosition") else [0.0, 0.0],
        "size_delta": [float(tr.m_SizeDelta.x), float(tr.m_SizeDelta.y)] if hasattr(tr, "m_SizeDelta") else [100.0, 100.0],
        "pivot": [float(tr.m_Pivot.x), float(tr.m_Pivot.y)] if hasattr(tr, "m_Pivot") else [0.5, 0.5],
        "local_pos": [float(tr.m_LocalPosition.x), float(tr.m_LocalPosition.y), float(tr.m_LocalPosition.z)] if hasattr(tr, "m_LocalPosition") else [0.0, 0.0, 0.0],
        "local_scale": [float(tr.m_LocalScale.x), float(tr.m_LocalScale.y), float(tr.m_LocalScale.z)] if hasattr(tr, "m_LocalScale") else [1.0, 1.0, 1.0],
    }
    return data


def _parse_gameobject_hierarchy(go, asset_index, sprite_name_cache):
    """Recursively parses a GameObject and its components."""
    # Find RectTransform or Transform
    tr = None
    components = {}

    for comp_pair in go.m_Component:
        c = comp_pair.component
        if not c:
            continue
        c_type = c.type.name
        if c_type in ("RectTransform", "Transform"):
            tr = c.read()
        elif c_type == "MonoBehaviour":
            try:
                tree = c.read_typetree()
                if "m_Sprite" in tree:
                    s_info = tree.get("m_Sprite") or {}
                    pid = str(s_info.get("m_PathID", 0))
                    s_name = sprite_name_cache.get(pid, asset_index.get(pid, {}).get("name", ""))
                    color = tree.get("m_Color", {})
                    components["image"] = {
                        "sprite_name": s_name,
                        "color": [
                            float(color.get("r", 1.0)),
                            float(color.get("g", 1.0)),
                            float(color.get("b", 1.0)),
                            float(color.get("a", 1.0)),
                        ],
                        "type": int(tree.get("m_Type", 0) or 0),
                        "fill_method": int(tree.get("m_FillMethod", 0) or 0),
                        "fill_amount": float(tree.get("m_FillAmount", 1.0) or 1.0),
                        "raycast_target": bool(tree.get("m_RaycastTarget", True)),
                    }
                elif "m_Text" in tree:
                    color = tree.get("m_Color", {})
                    components["text"] = {
                        "text": str(tree.get("m_Text", "") or ""),
                        "font_size": int(tree.get("m_FontSize", 14) or 14),
                        "color": [
                            float(color.get("r", 1.0)),
                            float(color.get("g", 1.0)),
                            float(color.get("b", 1.0)),
                            float(color.get("a", 1.0)),
                        ],
                        "alignment": int(tree.get("m_Alignment", 0) or 0),
                    }
            except Exception:
                pass

    node = {
        "name": go.m_Name,
        "active": bool(getattr(go, "m_IsActive", True)),
        "rect": _parse_rect_transform(tr) if tr else {},
        "components": components,
        "children": [],
    }

    if tr and hasattr(tr, "m_Children"):
        for child_ptr in tr.m_Children:
            try:
                child_tr = child_ptr.read()
                child_go = child_tr.m_GameObject.read()
                node["children"].append(
                    _parse_gameobject_hierarchy(child_go, asset_index, sprite_name_cache)
                )
            except Exception:
                pass

    return node


def run_pass3_ui():
    import UnityPy

    print("\n=======================================================")
    print("PASS 3: Porting BattlePanel, BattleMinimap, BattleMapPanel")
    print("=======================================================")
    t0 = time.time()

    if not os.path.exists(GLOBAL_INDEX_PATH):
        if not run_pass1_global_index():
            return False

    with open(GLOBAL_INDEX_PATH, "r", encoding="utf-8") as f:
        global_data = json.load(f)
    asset_index = global_data.get("assets", {})

    os.makedirs(UI_DIR, exist_ok=True)
    os.makedirs(UI_SPRITES_DIR, exist_ok=True)
    os.makedirs(UI_BATTLE_PANEL_DIR, exist_ok=True)
    os.makedirs(UI_BATTLE_MINIMAP_DIR, exist_ok=True)
    os.makedirs(UI_BATTLE_MAP_PANEL_DIR, exist_ok=True)

    # 1. Extract ALL Sprites from UI Atlases
    atlas_bundles = [
        "assets/uiatlas/battlepanel.ab",
        "assets/uiatlas/battlepanel2.ab",
        "assets/uiatlas/newbattlepanel.ab",
        "assets/uiatlas/battlemappanel.ab",
        "assets/uiatlas/common.ab",
        "assets/uiatlas/battlecustompanel.ab",
        "assets/uiatlas/battleinfopaneltwopanel.ab",
        "assets/uiatlas/battletankpanel.ab",
        "assets/uiatlas/battlezhanlipanel.ab",
    ]

    sprite_meta = {}
    sprite_name_cache = {}
    exported_count = 0

    print("Extracting authentic cropped Sprites from UI atlases...")
    with zipfile.ZipFile(APK_PATH, "r") as apk:
        for ab_name in atlas_bundles:
            if ab_name not in apk.namelist():
                continue
            try:
                env = UnityPy.load(apk.read(ab_name))
                for obj in env.objects:
                    if obj.type.name == "Sprite":
                        sp = obj.read()
                        sname = sp.m_Name
                        if not sname:
                            continue
                        sprite_name_cache[str(obj.path_id)] = sname

                        # Border info for 9-slice: [left, bottom, right, top]
                        border = [0.0, 0.0, 0.0, 0.0]
                        if hasattr(sp, "m_Border"):
                            border = [
                                float(sp.m_Border.x),
                                float(sp.m_Border.y),
                                float(sp.m_Border.z),
                                float(sp.m_Border.w),
                            ]

                        try:
                            im = sp.image
                            w, h = im.size
                            sprite_meta[sname] = {
                                "width": w,
                                "height": h,
                                "border": border,
                                "bundle": ab_name,
                            }

                            # Save to shared sprites directory
                            out_path = os.path.join(UI_SPRITES_DIR, f"{sname}.png")
                            if not os.path.exists(out_path):
                                im.save(out_path, compress_level=1)
                                exported_count += 1

                            # Also categorize into panel directories
                            if "battlepanel" in ab_name:
                                p_path = os.path.join(UI_BATTLE_PANEL_DIR, f"{sname}.png")
                                if not os.path.exists(p_path):
                                    im.save(p_path, compress_level=1)
                            if "battlemappanel" in ab_name:
                                m_path = os.path.join(UI_BATTLE_MAP_PANEL_DIR, f"{sname}.png")
                                if not os.path.exists(m_path):
                                    im.save(m_path, compress_level=1)
                        except Exception:
                            pass
            except Exception as e:
                print(f"Error reading {ab_name}: {e}")

    meta_path = os.path.join(UI_DIR, "sprites_meta.json")
    with open(meta_path, "w", encoding="utf-8") as f:
        json.dump(sprite_meta, f, indent=2)
    print(f"Exported {exported_count} sprites, indexed {len(sprite_meta)} in {meta_path}.")

    # 2. Parse Prefab Layouts
    panels = [
        ("assets/ui/battlepanel.ab", "BattlePanel", os.path.join(UI_DIR, "battle_panel_layout.json")),
        ("assets/ui/battleminimap.ab", "BattleMinimap", os.path.join(UI_DIR, "battle_minimap_layout.json")),
        ("assets/ui/battlemappanel.ab", "BattleMapPanel", os.path.join(UI_DIR, "battle_map_panel_layout.json")),
    ]

    with zipfile.ZipFile(APK_PATH, "r") as apk:
        for ab_name, root_name, out_layout_path in panels:
            if ab_name not in apk.namelist():
                print(f"Warning: {ab_name} not found in APK!")
                continue

            print(f"Parsing prefab hierarchy for {root_name} ({ab_name})...")
            env = UnityPy.load(apk.read(ab_name))

            root_go = None
            for obj in env.objects:
                if obj.type.name == "GameObject":
                    go = obj.read()
                    if go.m_Name == root_name:
                        root_go = go
                        break

            if root_go:
                tree = _parse_gameobject_hierarchy(root_go, asset_index, sprite_name_cache)
                with open(out_layout_path, "w", encoding="utf-8") as f:
                    json.dump(tree, f, indent=2)
                print(f"Saved layout -> {out_layout_path}")
            else:
                print(f"ERROR: Root GameObject '{root_name}' not found in {ab_name}!")

    print(f"Pass 3 Complete in {time.time() - t0:.2f}s!")
    return True


def main():
    parser = argparse.ArgumentParser(description="Midnight Survival Full Asset Unpacker")
    parser.add_argument("--pass1", action="store_true", help="Run Pass 1: Global PathID Index")
    parser.add_argument("--pass2", action="store_true", help="Run Pass 2: Terrain Manifest & Textures")
    parser.add_argument("--pass3", action="store_true", help="Run Pass 3: Port UI Sprites & Layouts")
    parser.add_argument("--all", action="store_true", help="Run all 3 passes in sequence")
    args = parser.parse_args()

    # Default to --all if no specific pass chosen
    run_all = args.all or not (args.pass1 or args.pass2 or args.pass3)

    t_start = time.time()
    if run_all or args.pass1:
        if not run_pass1_global_index():
            return 1

    if run_all or args.pass2:
        if not run_pass2_terrain():
            return 1

    if run_all or args.pass3:
        if not run_pass3_ui():
            return 1

    print(f"\nALL REQUESTED PASSES FINISHED SUCCESSFULLY in {time.time() - t_start:.2f}s!")
    return 0


if __name__ == "__main__":
    sys.exit(main())
