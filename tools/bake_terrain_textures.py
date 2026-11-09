#!/usr/bin/env python3
"""
Bakes all 225 island terrain sector textures (A-01..D-56) from raw Unity splatmaps
into authentic, vibrant, crisp 1024x1024 terrain textures using exact Unity material layers,
seamless micro-tiling, and gamma 2.2 color correction without stretching 2D overview maps.
"""

import json
import os
import time
from concurrent.futures import ProcessPoolExecutor
import numpy as np
from PIL import Image

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
MAP_DIR = os.path.join(PROJECT_ROOT, "assets", "imported_map")
SPLAT_DIR = os.path.join(MAP_DIR, "splatmaps")
TEX_DIR = os.path.join(MAP_DIR, "textures")
MATS_DIR = os.path.join(MAP_DIR, "terrain_mats")
SECTOR_MATS_JSON = os.path.join(MAP_DIR, "sector_materials.json")

# Global worker cache
g_tex_cache = None
g_sector_mats = None
g_grid_y = None
g_grid_x = None


def init_worker(mats_dir, sector_mats_json, out_size):
    global g_tex_cache, g_sector_mats, g_grid_y, g_grid_x
    g_sector_mats = {}
    if os.path.exists(sector_mats_json):
        with open(sector_mats_json, "r", encoding="utf-8") as f:
            g_sector_mats = json.load(f)

    g_tex_cache = {}
    if os.path.exists(mats_dir):
        for f in os.listdir(mats_dir):
            if f.endswith(".png"):
                name = os.path.splitext(f)[0]
                p = os.path.join(mats_dir, f)
                im = Image.open(p).convert("RGB")
                arr = np.array(im, dtype=np.float32) / 255.0
                # Convert linear Unity texture colors to sRGB space (gamma 2.2)
                arr = (arr ** (1.0 / 2.2)) * 255.0
                g_tex_cache[name] = arr

    g_grid_y, g_grid_x = np.indices((out_size, out_size))


def sample_tiled(tex, num_tiles, out_size):
    th, tw, _ = tex.shape
    ix = ((g_grid_x * num_tiles * tw) // out_size) % tw
    iy = ((g_grid_y * num_tiles * th) // out_size) % th
    return tex[iy, ix]


def process_sector(args):
    fname, out_size = args
    splat_path = os.path.join(SPLAT_DIR, fname)
    out_path = os.path.join(TEX_DIR, fname)
    sec_name = os.path.splitext(fname)[0].upper()

    layers = g_sector_mats.get(sec_name, [])
    fallback_tex = g_tex_cache.get("terrain_kucao_01")
    if fallback_tex is None and g_tex_cache:
        fallback_tex = next(iter(g_tex_cache.values()))

    # Splat weight blending - resize channels independently to avoid PIL alpha premultiplication bug
    if os.path.exists(splat_path):
        splat = Image.open(splat_path)
        bands = splat.getbands()
        channels = [
            np.array(
                splat.getchannel(i).resize((out_size, out_size), Image.Resampling.BILINEAR),
                dtype=np.float32,
            )
            for i in range(min(4, len(bands)))
        ]
        while len(channels) < 4:
            channels.append(np.zeros((out_size, out_size), dtype=np.float32))
        arr = np.stack(channels, axis=-1)
        # Unity Terrain splatmaps store only RGB; alpha is unused ALB/roughness
        # and previously flattened whole sectors into a single material.
        arr[..., 3] = 0.0
        tot = arr.sum(axis=-1, keepdims=True)
        tot[tot == 0] = 1.0
        weights = arr / tot
    else:
        weights = np.zeros((out_size, out_size, 4), dtype=np.float32)
        weights[..., 0] = 1.0

    result = np.zeros((out_size, out_size, 3), dtype=np.float32)
    for i in range(4):
        cfg = layers[i] if i < len(layers) else {}
        tname = cfg.get("texture", "")
        scale = cfg.get("scale", 100.0)
        tex = g_tex_cache.get(tname, fallback_tex)
        if tex is not None:
            num_tiles = max(8, int(round(scale * 0.32)))
            tiled = sample_tiled(tex, num_tiles, out_size)
            result += weights[..., i : i + 1] * tiled

    final = np.clip(result, 0, 255).astype(np.uint8)
    Image.fromarray(final).save(out_path, compress_level=1)
    return fname


def main():
    if not os.path.exists(TEX_DIR):
        os.makedirs(TEX_DIR, exist_ok=True)

    files = sorted(
        [
            f
            for f in os.listdir(SPLAT_DIR)
            if f.endswith(".png")
            and ((f[0] in "ABCD" and "-" in f) or f.upper().startswith("JUNSHIJIDI_TERRAIN_"))
        ]
    )
    print(f"Baking {len(files)} sectors at 1024x1024 with authentic Unity materials...")

    out_size = 1024
    t0 = time.time()
    tasks = [(fname, out_size) for fname in files]

    with ProcessPoolExecutor(initializer=init_worker, initargs=(MATS_DIR, SECTOR_MATS_JSON, out_size)) as executor:
        for idx, _ in enumerate(executor.map(process_sector, tasks)):
            if (idx + 1) % 45 == 0 or (idx + 1) == len(tasks):
                print(f"  Progress: {idx + 1}/{len(tasks)} sectors baked ({time.time() - t0:.1f}s)...")

    elapsed = time.time() - t0
    print(f"Successfully baked {len(files)} authentic terrain textures in {elapsed:.1f}s!")


if __name__ == "__main__":
    main()
