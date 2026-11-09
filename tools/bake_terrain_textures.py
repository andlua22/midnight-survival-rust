#!/usr/bin/env python3
"""
Authentic Terrain Texture Baker for Civilization Continent (文明大陆).

Features:
1. Multi-Biome ground-truth restoration (Arctic Snow, Desert, Plains, Military Base).
2. Elimination of chunk seams via 70% high-frequency detail + 30% global overview macro-tinting.
3. Authentic layer resolution from PreloadData dependencies in lite4.0.apk.
4. Non-premultiplied splatmap channel handling (prevents PIL alpha corruption).
5. Fast multiprocessing execution with full 1024x1024 output for all 241 chunks.
"""

import json
import math
import os
import struct
import sys
import time
import zipfile
from concurrent.futures import ProcessPoolExecutor
import numpy as np
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
MAP_DIR = os.path.join(PROJECT_ROOT, "assets", "imported_map")
TEX_DIR = os.path.join(MAP_DIR, "textures")
SPLAT_DIR = os.path.join(MAP_DIR, "splatmaps")
MATS_DIR = os.path.join(MAP_DIR, "terrain_mats")
OVERVIEW_PATH = os.path.join(TEX_DIR, "map_overview.png")
TERRAIN_BIN = os.path.join(MAP_DIR, "island_terrain.bin")
CHUNK_POS_JSON = os.path.join(MAP_DIR, "chunk_positions.json")
CHUNK_BIOMES_JSON = os.path.join(MAP_DIR, "chunk_biomes.json")
APK_PATH = os.path.join(PROJECT_ROOT, "export", "lite4.0.apk")
OUT_SIZE = 1024
MAP_OVERVIEW_HALF_EXTENT = 5360.0

g_tex_cache = None
g_tiled_cache = None
g_grid_y = None
g_grid_x = None
g_overview_im = None


def init_worker(mats_dir, overview_path, out_size):
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
                    arr = (arr ** (1.0 / 2.2))
                    g_tex_cache[f.lower()] = arr
                except Exception:
                    pass

    if os.path.exists(overview_path):
        try:
            g_overview_im = Image.open(overview_path).convert("RGB")
        except Exception:
            g_overview_im = None


def sample_tiled(tex, repeats):
    th, tw, _ = tex.shape
    return tex[((g_grid_y * repeats) % th).astype(int), ((g_grid_x * repeats) % tw).astype(int)]


def get_tiled_layer(mat_name, repeats):
    key = (mat_name, repeats)
    if key not in g_tiled_cache:
        tex = g_tex_cache.get(mat_name.lower())
        if tex is None:
            tex = g_tex_cache.get("terrain_caodi_01.png", next(iter(g_tex_cache.values())))
        g_tiled_cache[key] = sample_tiled(tex, repeats)
    return g_tiled_cache[key]


def blend_chunk_task(args):
    cname, splat_channels, layers, min_x, max_x, min_z, max_z, biome_hint = args

    num_layers = max(1, len(layers))
    weights = [
        splat_channels[0][..., None],
        splat_channels[1][..., None],
        splat_channels[2][..., None],
        splat_channels[3][..., None] if len(splat_channels) > 3 else np.zeros((OUT_SIZE, OUT_SIZE, 1), dtype=np.float32),
    ]

    tot = np.zeros((OUT_SIZE, OUT_SIZE, 1), dtype=np.float32)
    for i in range(min(num_layers, 4)):
        tot += weights[i]
    tot[tot == 0.0] = 1.0

    detail = np.zeros((OUT_SIZE, OUT_SIZE, 3), dtype=np.float32)
    for i in range(min(num_layers, 4)):
        w = weights[i] / tot
        mat = layers[i] if i < len(layers) else "terrain_caodi_01.png"
        repeats = 44 if any(k in mat for k in ["shi", "yan", "rock"]) else 32
        tiled = get_tiled_layer(mat, repeats)
        detail += w * tiled

    # Extract matching chunk patch from global overview minimap (map_overview.png)
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

        patch = g_overview_im.crop((px0, py0, px1, py1)).resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BICUBIC)
        macro = np.array(patch, dtype=np.float32) / 255.0

        # Seamless Macro-Color modulation: 70% detail textures + 30% continuous overview tint
        result = np.clip(0.70 * detail + 0.30 * macro, 0.0, 1.0)
    else:
        result = np.clip(detail, 0.0, 1.0)

    out_png = os.path.join(TEX_DIR, f"{cname}.png")
    final_arr = (result * 255.0).astype(np.uint8)
    Image.fromarray(final_arr).save(out_png, compress_level=1)
    return cname, biome_hint, layers


def load_chunk_bounds():
    bounds = {}
    if os.path.exists(TERRAIN_BIN):
        with open(TERRAIN_BIN, "rb") as f:
            data = f.read()
        if len(data) >= 12 and data[:8] == b"MSMAP001":
            count = struct.unpack_from("<I", data, 8)[0]
            offset = 12
            for _ in range(count):
                if offset + 2 > len(data):
                    break
                nlen = struct.unpack_from("<H", data, offset)[0]
                offset += 2
                name = data[offset:offset+nlen].decode("utf-8", errors="replace")
                offset += nlen
                tlen = struct.unpack_from("<H", data, offset)[0]
                offset += 2
                offset += tlen
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

    if os.path.exists(CHUNK_POS_JSON):
        with open(CHUNK_POS_JSON, "r", encoding="utf-8") as f:
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


def extract_all_chunk_tasks(bounds_map):
    import UnityPy

    available_mats = {f.lower(): f for f in os.listdir(MATS_DIR) if f.endswith(".png")}
    chunk_tasks = []

    print(f"Reading chunks from {APK_PATH}...")
    with zipfile.ZipFile(APK_PATH, "r") as apk:
        scene_files = sorted([
            n for n in apk.namelist()
            if n.startswith("assets/scene/")
            and n.endswith(".ab")
            and (
                (n.split("/")[-1][0] in "abcd" and "-" in n)
                or "junshijidi_terrain_" in n
            )
        ])

        overview_im = None
        if os.path.exists(OVERVIEW_PATH):
            overview_im = Image.open(OVERVIEW_PATH).convert("RGB")
        W_ov, H_ov = overview_im.size if overview_im else (2048, 2048)
        half = MAP_OVERVIEW_HALF_EXTENT

        for sf in scene_files:
            cname = os.path.splitext(os.path.basename(sf))[0].upper()
            env = UnityPy.load(apk.read(sf))
            splat_im = None
            deps = []

            for obj in env.objects:
                if obj.type.name == "Texture2D":
                    splat_im = obj.read().image
                elif obj.type.name == "PreloadData":
                    pd = obj.read()
                    if hasattr(pd, "m_Dependencies"):
                        deps = pd.m_Dependencies

            if splat_im is None:
                # Try loading cached splatmap from SPLAT_DIR
                splat_fallback = os.path.join(SPLAT_DIR, f"{cname}.png")
                if os.path.exists(splat_fallback):
                    splat_im = Image.open(splat_fallback)
                else:
                    continue

            # Save raw splatmap
            raw_splat_path = os.path.join(SPLAT_DIR, f"{cname}.png")
            if not os.path.exists(raw_splat_path):
                splat_im.save(raw_splat_path, compress_level=1)

            # Split channels individually BEFORE resize to prevent PIL alpha premultiplication bug
            channels = [
                np.array(ch.resize((OUT_SIZE, OUT_SIZE), Image.Resampling.BILINEAR), dtype=np.float32) / 255.0
                for ch in splat_im.split()[:4]
            ]
            while len(channels) < 4:
                channels.append(np.zeros((OUT_SIZE, OUT_SIZE), dtype=np.float32))

            # Resolve authentic terrain dependencies from lite4.0.apk
            terrain_deps = []
            for d in deps:
                base = os.path.basename(d).replace(".ab", ".png").lower()
                if (
                    base in available_mats
                    and not base.endswith("_n.png")
                    and "_bump" not in base
                    and "_norm" not in base
                ):
                    terrain_deps.append(available_mats[base])

            # Get chunk world bounds
            min_x, max_x, min_z, max_z = bounds_map.get(
                cname, (-200.0, 200.0, -200.0, 200.0)
            )

            # Sample overview patch to determine ground-truth macroscopic biome
            macro_mean = [0.5, 0.5, 0.5]
            if overview_im:
                u0 = (min_x + half) / (2.0 * half)
                u1 = (max_x + half) / (2.0 * half)
                v0 = (min_z + half) / (2.0 * half)
                v1 = (max_z + half) / (2.0 * half)
                px0 = int(np.clip(u0 * W_ov, 0, W_ov - 1))
                px1 = int(np.clip(u1 * W_ov, px0 + 1, W_ov))
                py0 = int(np.clip(v0 * H_ov, 0, H_ov - 1))
                py1 = int(np.clip(v1 * H_ov, py0 + 1, H_ov))
                p = overview_im.crop((px0, py0, px1, py1)).resize((32, 32), Image.Resampling.BOX)
                macro_mean = np.array(p, dtype=np.float32).mean(axis=(0, 1)) / 255.0

            r, g, b = macro_mean[0], macro_mean[1], macro_mean[2]
            lum = 0.299 * r + 0.587 * g + 0.114 * b

            # Ground-truth biome classification
            is_snow = any("xue" in d or "bing" in d for d in terrain_deps) or lum > 0.62 or (r > 0.60 and g > 0.60 and b > 0.60)
            is_desert = any("sand" in d or "gobi" in d or "shamo" in d or "shadi" in d for d in terrain_deps) or (r > 0.35 and r > 1.25 * b and max_z > 300.0)
            is_mil = cname.startswith("JUNSHIJIDI")

            if is_snow:
                biome_kind = "Snow"
                standard_layers = ["terrain_xuedi_01.png", "terrain_xuedi_04.png", "terrain_shitou_02.png", "terrain_nitu_01.png"]
            elif is_desert:
                biome_kind = "Desert"
                standard_layers = ["terrain_sandA2.png", "terrain_gobi_02.png", "terrain_yanshi_01.png", "shamonitu.png" if "shamonitu.png" in available_mats else "terrain_nitu_02.png"]
            elif is_mil:
                biome_kind = "Military"
                standard_layers = ["terrain_kucao_01.png", "terrain_nitu_02.png", "terrain_sandA2.png", "terrain_yanshi_01.png"]
            else:
                biome_kind = "Plains"
                standard_layers = ["terrain_caodi_01.png", "terrain_nitu_02.png", "terrain_yanshi_01.png", "terrain_kucao_01.png"]

            # Merge authentic dependencies with standard layers to ensure all 4 channels have rich, authentic detail
            final_layers = list(terrain_deps)
            for sl in standard_layers:
                if sl not in final_layers and len(final_layers) < 4:
                    final_layers.append(sl)
            while len(final_layers) < 4:
                final_layers.append("terrain_caodi_01.png")

            chunk_tasks.append((cname, channels, final_layers, min_x, max_x, min_z, max_z, biome_kind))

    return chunk_tasks


def main():
    t0 = time.time()
    os.makedirs(TEX_DIR, exist_ok=True)
    os.makedirs(SPLAT_DIR, exist_ok=True)

    print("Loading chunk bounds from island_terrain.bin...")
    bounds_map = load_chunk_bounds()
    print(f"Loaded bounds for {len(bounds_map)} chunks.")

    print("Phase 1: Extracting raw splatmaps, authentic dependencies, and overview patches...")
    tasks = extract_all_chunk_tasks(bounds_map)
    print(f"Prepared {len(tasks)} chunk tasks in {time.time() - t0:.2f}s.")

    print("Phase 2: Blending 1024x1024 seamless terrain textures with macro color tinting...")
    cpu_count = min(8, os.cpu_count() or 4)
    success_count = 0
    biome_results = {}

    with ProcessPoolExecutor(
        max_workers=cpu_count,
        initializer=init_worker,
        initargs=(MATS_DIR, OVERVIEW_PATH, OUT_SIZE),
    ) as executor:
        for cname, bkind, layers in executor.map(blend_chunk_task, tasks):
            success_count += 1
            normal_map = (
                "terrain_xuedi_01_N.png" if bkind == "Snow"
                else "terrain_sandA2_N.png" if bkind == "Desert"
                else "caodi1215_N.png"
            )
            biome_results[cname] = {
                "biome": bkind,
                "l0": layers[0],
                "l1": layers[1],
                "l2": layers[2],
                "l3": layers[3],
                "normal": normal_map,
            }

    with open(CHUNK_BIOMES_JSON, "w", encoding="utf-8") as f:
        json.dump(biome_results, f, indent=2, ensure_ascii=False)
    print(f"Saved {len(biome_results)} updated chunk biomes to {CHUNK_BIOMES_JSON}.")

    snow_count = sum(1 for v in biome_results.values() if v["biome"] == "Snow")
    desert_count = sum(1 for v in biome_results.values() if v["biome"] == "Desert")
    plains_count = sum(1 for v in biome_results.values() if v["biome"] == "Plains")
    mil_count = sum(1 for v in biome_results.values() if v["biome"] == "Military")
    print(f"Summary: {snow_count} Snow, {desert_count} Desert, {plains_count} Plains, {mil_count} Military chunks.")

    print(f"Done! Successfully baked all {success_count}/{len(tasks)} authentic terrain textures in {time.time() - t0:.2f}s.")
    return 0 if success_count > 0 else 1


if __name__ == "__main__":
    sys.exit(main())
