#!/usr/bin/env python3
"""
Extracts the 225 island terrain chunks (A-01..D-56), road meshes, 2D map overview,
sceneinfo object placements, and 14 core 3D models + textures from
/home/andlua/Завантажене/last-day-rules-survival-1-0.xapk into assets/imported_map/.
"""

import io
import math
import os
import struct
import shutil
import sys
import time
import zipfile
import numpy as np
import UnityPy
from UnityPy.export.MeshExporter import MeshHandler

XAPK_CANDIDATES = [
    os.path.expanduser("~/Завантажене/last-day-rules-survival-1-0.xapk"),
    os.path.expanduser("~/Downloads/last-day-rules-survival-1-0.xapk"),
]
XAPK_DEFAULT = next((p for p in XAPK_CANDIDATES if os.path.exists(p)), XAPK_CANDIDATES[0])
OBB_ENTRY = "Android/Obb/com.herogame.gplay.lastdayrulessurvival/main.25.com.herogame.gplay.lastdayrulessurvival.obb"
OUT_DIR = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "assets", "imported_map")
TEX_DIR = os.path.join(OUT_DIR, "textures")
OBB_CACHE = "/tmp/lastday-main.25.obb"


class SliceFile(io.RawIOBase):
    def __init__(self, f, start, length):
        self.f = f
        self.start = start
        self.length = length
        self.pos = 0

    def seekable(self):
        return True

    def readable(self):
        return True

    def seek(self, offset, whence=io.SEEK_SET):
        if whence == io.SEEK_SET:
            self.pos = offset
        elif whence == io.SEEK_CUR:
            self.pos += offset
        elif whence == io.SEEK_END:
            self.pos = self.length + offset
        return self.pos

    def tell(self):
        return self.pos

    def readinto(self, b):
        rem = max(0, self.length - self.pos)
        n = min(len(b), rem)
        if n == 0:
            return 0
        self.f.seek(self.start + self.pos)
        data = self.f.read(n)
        b[: len(data)] = data
        self.pos += len(data)
        return len(data)


def terrain_height_adjust(wy_raw: float, is_road: bool) -> float:
    # sceneinfo / mesh Y is already world height; only lift roads slightly.
    wy = wy_raw
    if is_road:
        wy += 0.05
    return wy


def unity_euler_to_quat(rx_deg: float, ry_deg: float, rz_deg: float):
    """
    Converts Unity ZXY Euler rotation angles (degrees) into a normalized quaternion.
    In Unity, Quaternion.Euler(rx, ry, rz) applies rotations in order Z, then X, then Y.
    Our Bevy coordinate system inverts the X axis (left-to-right handedness conversion),
    which transforms the quaternion according to M * R * M = [qx, -qy, -qz, qw].
    """
    rx = math.radians(rx_deg)
    ry = math.radians(ry_deg)
    rz = math.radians(rz_deg)
    cx, sx = math.cos(rx * 0.5), math.sin(rx * 0.5)
    cy, sy = math.cos(ry * 0.5), math.sin(ry * 0.5)
    cz, sz = math.cos(rz * 0.5), math.sin(rz * 0.5)
    qx = sx * cy * cz + cx * sy * sz
    qy = cx * sy * cz - sx * cy * sz
    qz = cx * cy * sz - sx * sy * cz
    qw = cx * cy * cz + sx * sy * sz
    return (qx, -qy, -qz, qw)


def drape_road_to_terrain(road_positions, terrain_positions):
    """
    Drapes road strip spline vertices directly onto the terrain surface (+0.03m clearance),
    preventing road meshes from floating up to 1.6m above the ground collider in gullies and dips.
    """
    t_arr = np.array(terrain_positions, dtype=np.float32)
    r_arr = np.array(road_positions, dtype=np.float32)
    if len(t_arr) < 3 or len(r_arr) == 0:
        return road_positions
    d2 = (r_arr[:, [0, 2]][:, None, :] - t_arr[:, [0, 2]][None, :, :]) ** 2
    d2_sum = d2[:, :, 0] + d2[:, :, 1]
    k = min(3, len(t_arr))
    topk_idx = np.argpartition(d2_sum, k, axis=-1)[:, :k]
    topk_d2 = np.take_along_axis(d2_sum, topk_idx, axis=-1)
    topk_y = t_arr[topk_idx, 1]
    topk_w = 1.0 / (np.sqrt(topk_d2) + 1e-4)
    weighted_y = np.sum(topk_y * topk_w, axis=-1) / np.sum(topk_w, axis=-1)
    r_arr[:, 1] = weighted_y + 0.03
    return r_arr.tolist()


def extract_tree_configs(xapk_path: str, out_json_path: str):
    """
    Extracts all authentic tree types and properties from cfg.TreeCfg.oc inside the APK.
    """
    import json
    with zipfile.ZipFile(xapk_path, "r") as xz:
        apk_name = next(n for n in xz.namelist() if n.lower().endswith(".apk"))
        apk_data = xz.read(apk_name)
    with zipfile.ZipFile(io.BytesIO(apk_data), "r") as az:
        raw = az.read("assets/cfg/cfg.TreeCfg.oc")

    pos = 0
    records = []
    while pos + 8 <= len(raw):
        rec_start = pos
        tid = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        nlen = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        if nlen == 0 or nlen > 64 or pos + nlen > len(raw):
            print(f"Warning: TreeCfg bad name length {nlen} at offset {rec_start}, stopping")
            break
        name = raw[pos : pos + nlen].decode("utf-8", errors="replace")
        pos += nlen
        if pos + 12 + 20 + 4 + 4 + 4 + 4 > len(raw):
            print(f"Warning: TreeCfg truncated after name {name!r}")
            break
        hp, drop_id, drop_count = struct.unpack_from("<3I", raw, pos)
        pos += 12
        sounds = list(struct.unpack_from("<5i", raw, pos))
        pos += 20
        unk1 = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        ilen = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        if ilen > 128 or pos + ilen + 8 > len(raw):
            print(f"Warning: TreeCfg bad icon length {ilen} at {name!r}")
            break
        icon = raw[pos : pos + ilen].decode("utf-8", errors="replace")
        pos += ilen
        mlen = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        if mlen > 128 or pos + mlen + 4 > len(raw):
            print(f"Warning: TreeCfg bad map_icon length {mlen} at {name!r}")
            break
        map_icon = raw[pos : pos + mlen].decode("utf-8", errors="replace")
        pos += mlen
        marker = struct.unpack_from("<I", raw, pos)[0]
        pos += 4
        # Current TreeCfg.oc has an extra 8-byte tail after marker on every record.
        if pos + 8 <= len(raw):
            pos += 8
        records.append({
            "id": tid,
            "name": name,
            "health": hp,
            "drop_item_id": drop_id,
            "drop_count_per_hit": drop_count,
            "sound_stages": sounds,
            "icon": icon,
            "map_icon": map_icon,
            "marker_type": marker,
        })
    with open(out_json_path, "w", encoding="utf-8") as f:
        json.dump(records, f, indent=2, ensure_ascii=False)
    print(f"Extracted {len(records)} authentic tree configurations to {out_json_path}")


def pack_mesh_record(f_out, name: str, tex_name: str, positions, normals, uvs, indices):
    name_b = name.encode("utf-8")
    tex_b = tex_name.encode("utf-8")
    if positions:
        cx = sum(p[0] for p in positions) / len(positions)
        cy = sum(p[1] for p in positions) / len(positions)
        cz = sum(p[2] for p in positions) / len(positions)
    else:
        cx = cy = cz = 0.0
    vcount = len(positions)
    icount = len(indices)
    f_out.write(struct.pack("<H", len(name_b)))
    f_out.write(name_b)
    f_out.write(struct.pack("<H", len(tex_b)))
    f_out.write(tex_b)
    f_out.write(struct.pack("<3fII", cx, cy, cz, vcount, icount))
    pos_flat = [coord for p in positions for coord in p]
    norm_flat = [coord for n in normals for coord in n]
    uv_flat = [coord for u in uvs for coord in u]
    f_out.write(struct.pack(f"<{vcount * 3}f", *pos_flat))
    f_out.write(struct.pack(f"<{vcount * 3}f", *norm_flat))
    f_out.write(struct.pack(f"<{vcount * 2}f", *uv_flat))
    f_out.write(struct.pack(f"<{icount}I", *indices))


def extract_mesh_arrays(mesh, z_up: bool, is_terrain: bool = False, is_road: bool = False, target_height: float = 0.0, preserve_pivot: bool = False):
    h = MeshHandler(mesh)
    h.process()
    if h.m_VertexCount <= 0 or not h.m_Vertices:
        return None
    vcount = h.m_VertexCount
    raw_verts = h.m_Vertices
    raw_norms = h.m_Normals if h.m_Normals and len(h.m_Normals) == vcount else [(0.0, 1.0, 0.0)] * vcount
    raw_uvs = h.m_UV0 if h.m_UV0 and len(h.m_UV0) == vcount else [(0.0, 0.0)] * vcount

    positions = []
    normals = []
    uvs = []
    for i in range(vcount):
        vx, vy, vz = raw_verts[i][:3]
        nx, ny, nz = raw_norms[i][:3]
        if is_terrain:
            # Terrain chunks in Unity: X is East (+vx), Z is North (-vy), Y is Height (+vz)
            wx = vx
            wy = terrain_height_adjust(vz, is_road)
            wz = -vy
            wnx, wny, wnz = nx, nz, -ny
        elif z_up:
            wx, wy, wz = -vx, vz, -vy
            wnx, wny, wnz = -nx, nz, -ny
        else:
            wx, wy, wz = -vx, vy, vz
            wnx, wny, wnz = -nx, ny, nz

        if not (math.isfinite(wx) and math.isfinite(wy) and math.isfinite(wz)):
            wx = wy = wz = 0.0
        if not (math.isfinite(wnx) and math.isfinite(wny) and math.isfinite(wnz)):
            wnx, wny, wnz = 0.0, 1.0, 0.0

        u, v = raw_uvs[i][0], 1.0 - raw_uvs[i][1]
        if not (math.isfinite(u) and math.isfinite(v)):
            u = v = 0.0

        positions.append([wx, wy, wz])
        normals.append((wnx, wny, wnz))
        uvs.append((u, v))

    if not is_terrain and positions:
        min_x = min(p[0] for p in positions)
        max_x = max(p[0] for p in positions)
        min_y = min(p[1] for p in positions)
        max_y = max(p[1] for p in positions)
        min_z = min(p[2] for p in positions)
        max_z = max(p[2] for p in positions)
        cx = 0.5 * (min_x + max_x)
        cz = 0.5 * (min_z + max_z)
        height = max(0.01, max_y - min_y)
        scale = (target_height / height) if target_height > 0.0 else 1.0
        if not preserve_pivot:
            for p in positions:
                p[0] = (p[0] - cx) * scale
                p[1] = (p[1] - min_y) * scale
                p[2] = (p[2] - cz) * scale
        else:
            # Preserves native Unity pivot so subterranean rock bases and tree roots remain underground
            for p in positions:
                p[0] = p[0] * scale
                p[1] = p[1] * scale
                p[2] = p[2] * scale

    indices = []
    for sub in h.get_triangles():
        for a, b, c in sub:
            if a < vcount and b < vcount and c < vcount:
                if is_terrain:
                    indices.extend((a, b, c))
                else:
                    indices.extend((c, b, a))

    if not indices:
        return None
    return positions, normals, uvs, indices


def main():
    xapk_path = sys.argv[1] if len(sys.argv) > 1 else XAPK_DEFAULT
    if not os.path.exists(xapk_path):
        print(f"XAPK not found at {xapk_path}")
        sys.exit(1)

    os.makedirs(TEX_DIR, exist_ok=True)
    t0 = time.time()

    def ensure_obb_extracted():
        if os.path.exists(OBB_CACHE) and os.path.getsize(OBB_CACHE) > 100_000_000:
            print(f"Using cached OBB at {OBB_CACHE}")
            return OBB_CACHE
        with zipfile.ZipFile(xapk_path, "r") as xz:
            obb_name = next(n for n in xz.namelist() if n.lower().endswith(".obb"))
            info = xz.getinfo(obb_name)
            if info.compress_type == zipfile.ZIP_STORED:
                return None
            print(f"Extracting compressed OBB ({info.file_size} bytes) to {OBB_CACHE}...")
            with xz.open(obb_name) as fsrc, open(OBB_CACHE, "wb") as fdst:
                shutil.copyfileobj(fsrc, fdst, 4 * 1024 * 1024)
            return OBB_CACHE

    obb_file_path = ensure_obb_extracted()
    if obb_file_path:
        obb_src = open(obb_file_path, "rb")
    else:
        raw_xapk = open(xapk_path, "rb")
        xapk_zip = zipfile.ZipFile(raw_xapk, "r")
        obb_info = xapk_zip.getinfo(next(n for n in xapk_zip.namelist() if n.lower().endswith(".obb")))
        raw_xapk.seek(obb_info.header_offset + 26)
        fname_len = int.from_bytes(raw_xapk.read(2), "little")
        extra_len = int.from_bytes(raw_xapk.read(2), "little")
        data_offset = obb_info.header_offset + 30 + fname_len + extra_len
        raw_xapk.seek(data_offset)
        obb_src = io.BufferedReader(SliceFile(raw_xapk, data_offset, obb_info.file_size))

    # Extract authentic tree configuration and drop tables from APK
    try:
        extract_tree_configs(xapk_path, os.path.join(OUT_DIR, "tree_configs.json"))
    except Exception as e:
        print(f"Warning: tree configs extraction failed: {e}")

    with obb_src, zipfile.ZipFile(obb_src, "r") as obb:
            # 1. Extract 2048x2048 Map_map + 128x128 RGB grid for egui minimap/tactical map
            print("Extracting Map_map overview texture and 128x128 minimap grid...")
            env_map = UnityPy.load(obb.read("assets/texture/map_map.ab"))
            for obj in env_map.objects:
                if obj.type.name == "Texture2D":
                    img = obj.read().image
                    img.save(os.path.join(TEX_DIR, "map_overview.png"), compress_level=1)
                    small = img.convert("RGB").resize((128, 128))
                    with open(os.path.join(OUT_DIR, "map_grid.bin"), "wb") as gf:
                        gf.write(small.tobytes())
                    break

            # 2. Extract road_01 texture
            env_road = UnityPy.load(obb.read("assets/scenetexture/road.ab"))
            for obj in env_road.objects:
                if obj.type.name == "Texture2D":
                    tex = obj.read()
                    if tex.m_Name == "road_01":
                        im = tex.image.convert("RGBA")
                        arr = np.array(im, dtype=np.float32)
                        rgb = (arr[:, :, :3] / 255.0) ** (1.0 / 2.2) * 255.0
                        corrected = np.dstack([rgb, arr[:, :, 3]])
                        from PIL import Image as PILImage
                        PILImage.fromarray(np.clip(corrected, 0, 255).astype(np.uint8)).save(
                            os.path.join(TEX_DIR, "road_01.png"), compress_level=1
                        )
                        break

            # 3. Extract all 225 island sectors (A-01..D-56) + western military
            # launch-site terrain (junshijidi_terrain_01..16) + road meshes.
            terrain_bin_path = os.path.join(OUT_DIR, "island_terrain.bin")
            if not os.path.exists(terrain_bin_path):
                scene_files = sorted(
                    [
                        n
                        for n in obb.namelist()
                        if n.startswith("assets/scene/")
                        and n.endswith(".ab")
                        and (
                            (
                                n.split("/")[-1][0] in "abcd"
                                and "-" in n
                                and "_" not in n
                            )
                            or n.split("/")[-1].startswith("junshijidi_terrain_")
                        )
                    ]
                )
                print(f"Extracting {len(scene_files)} terrain chunks + road meshes...")
                chunk_records = []
                for idx, sf in enumerate(scene_files):
                    env = UnityPy.load(obb.read(sf))
                    sector_tex_name = ""
                    for obj in env.objects:
                        if obj.type.name == "Texture2D":
                            tex = obj.read()
                            sector_tex_name = f"{tex.m_Name.upper()}.png"
                            out_png = os.path.join(TEX_DIR, sector_tex_name)
                            if not os.path.exists(out_png):
                                tex.image.save(out_png, compress_level=1)
                    sector_terrain_res = None
                    sector_roads = []
                    junk_names = {
                        'jichang', 'gangkou02', 'feiqichangfang', 'shiyanshi',
                        'object006', 'maoyiqu_01', 'feijichang', 'default1',
                    }
                    expected_sector = os.path.basename(sf).rsplit('.', 1)[0].upper()
                    is_west_chunk = os.path.basename(sf).startswith("junshijidi_terrain_")
                    for obj in env.objects:
                        if obj.type.name == "Mesh":
                            m = obj.read()
                            mname = m.m_Name
                            if mname.lower() in junk_names and not (
                                is_west_chunk and mname.startswith("junshijidi_terrain_")
                            ):
                                continue
                            is_road = "road" in mname.lower()
                            if not is_road and not (
                                mname.upper() == expected_sector
                                or (is_west_chunk and mname == expected_sector)
                            ):
                                # Keep only the matching sector / west-chunk terrain
                                # mesh; skip hupo_* / duanya_* decorations.
                                continue
                            is_road = "road" in mname.lower()
                            res = extract_mesh_arrays(m, z_up=True, is_terrain=True, is_road=is_road)
                            if res:
                                if is_road:
                                    sector_roads.append((mname, "road_01.png", res))
                                else:
                                    sector_terrain_res = res
                                    actual_tex = f"{mname.upper()}.png" if os.path.exists(os.path.join(TEX_DIR, f"{mname.upper()}.png")) else sector_tex_name
                                    chunk_records.append((mname, actual_tex, *res))
                    # Drape road ribbon vertices directly onto the terrain mesh so roads never levitate or sink
                    for rname, rtex, (rpos, rnorms, ruvs, rindices) in sector_roads:
                        if sector_terrain_res:
                            draped_rpos = drape_road_to_terrain(rpos, sector_terrain_res[0])
                        else:
                            draped_rpos = rpos
                        chunk_records.append((rname, rtex, draped_rpos, rnorms, ruvs, rindices))
                    if (idx + 1) % 45 == 0 or idx + 1 == len(scene_files):
                        print(f"  Processed {idx + 1}/{len(scene_files)} sectors ({len(chunk_records)} meshes)...")

                with open(terrain_bin_path, "wb") as tf:
                    tf.write(b"MSMAP001")
                    tf.write(struct.pack("<I", len(chunk_records)))
                    for rec in chunk_records:
                        pack_mesh_record(tf, *rec)
                print(f"Saved {terrain_bin_path} ({os.path.getsize(terrain_bin_path) // 1024} KB)")
            else:
                print(f"Terrain already extracted at {terrain_bin_path} ({os.path.getsize(terrain_bin_path) // 1024} KB)")

            # 4. Extract core 3D prop/vehicle/NPC/monument/foliage models + textures
            # Format: (key, bpath, tbpath, mname, tname, z_up, target_h, preserve_pivot)
            prop_specs = [
                ("tree_temperate", "assets/sceneprefab/tree_baihuashu_01.ab", None, "tree_baihuashu_01", "tree_baihuashu_01", False, 0.0, True),
                ("tree_jungle", "assets/sceneprefab/tree_redaishu_01.ab", None, "tree_redaishu_01", "tree_redaishu_01_d", False, 0.0, True),
                ("rock_node", "assets/sceneprefab/stone_dayanshi_01.ab", None, "stone_dayanshi_01", "stone_dayanshi_01_LOD1", False, 0.0, True),
                ("rock_boulder", "assets/scenetexture/stone_dayanshi_03_lod2.ab", "assets/scenetexture/stone-mat.ab", "stone_dayanshi_03_LOD2", "stone-mat-_MainTex-atlas0", False, 0.0, True),
                ("grass_clump", "assets/sceneprefab/grass_lvdi_01.ab", "assets/scenetexture/grass_lvdi_01_d.ab", "grass_lvdi_01", "grass_lvdi_01_d", False, 0.0, True),
                ("rock_small", "assets/sceneprefab/project_suishi_01.ab", "assets/scenetexture/stone-mat.ab", "project_suishi_01", "stone-mat-_MainTex-atlas0", False, 0.0, True),
                ("supply_crate", "assets/model/perfab_bujixiang_01.ab", None, "bujixiang", "perfab_bujixiang_01_d", False, 0.95, False),
                ("military_crate", "assets/model/perfab_gaojidanyaoxiang_01.ab", None, "wupin_xz3_01", "wupin_xz3_d", False, 0.85, False),
                ("hazard_barrel", "assets/model/perfab_hongyoutong_01.ab", None, "perfab_hongyoutong_01", "perfab_hongyoutong_01_d", False, 1.15, False),
                ("monument_gas_station", "assets/sceneprefab/building_jiayouzhan_01.ab", None, "Building_jiayouzhan_01", "Building_jiayouzhan_01_d", True, 9.5, False),
                ("cargo_ship", "assets/scenetexture/building_huolun_lod02.ab", None, "building_huolun_01_lod02", "building_huolun_01_lod02", False, 14.0, False),
                ("vehicle_buggy", "assets/model/car_jiaoche_1.ab", None, "car_jiaoche_1_low", "car_jiaoche_1_d", True, 1.55, False),
                ("vehicle_humvee", "assets/model/hanma.ab", None, "car_hanma_low", "car_hanma_01_low_d", True, 2.15, False),
                ("vehicle_bike", "assets/model/moto_saiche.ab", None, "moto_saiche", "moto_saiche_d", True, 1.15, False),
                ("npc_zombie", "assets/role/npc_js_1.ab", None, "npc_js_1", "npc_js_1_d", True, 1.80, False),
                ("npc_scientist", "assets/role/npc_1.ab", None, "npc_1", "npc_1_d", True, 1.85, False),
                ("item_c4", "assets/model/wq_c4.ab", None, "wq_c4", "wq_c4_d", False, 0.35, False),
                ("player_survivor", "assets/role/man_clothes_pj_01.ab", None, "man_clothes_pj_01", "man_clothes_pj_01_d", True, 1.82, False),
                ("tool_hatchet", "assets/model/wq_shifu.ab", None, "wq_shifu", "wq_shifu_d", False, 0.46, False),
                ("tool_pickaxe", "assets/model/wq_shigao.ab", None, "wq_shigao", "wq_shigao_d", False, 0.52, False),
                ("tool_hammer", "assets/model/wq_jianzaochui.ab", None, "wq_jianzaochui", "wq_jianzaochui", False, 0.38, False),
                ("weapon_rifle", "assets/model/wq_tuzhibuqiang_1.ab", None, "wq_tuzhibuqiang", "wq_tuzhibuqiang_d", False, 0.78, False),
                ("weapon_pistol", "assets/model/wq_m92_1.ab", None, "M92", "M92_d", False, 0.28, False),
                ("weapon_rpg", "assets/model/wq_rpg.ab", None, "wq_rpg", "wq_rpg_d", False, 0.98, False),
                ("building_gas_station", "assets/sceneprefab/building_jiayouzhan_01.ab", None, "Building_jiayouzhandiji_01", "Building_jiayouzhan_01_d", False, 0.0, True),
                ("building_supermarket", "assets/sceneprefab/building_chaoshi_01.ab", None, "Building_chaoshi_01", "Building_chaoshi_01_d", False, 0.0, True),
                ("building_warehouse", "assets/sceneprefab/building_cangku_01.ab", None, "building_cangku_01_LOD2", "building_cangku_01", False, 0.0, True),
                ("building_old_warehouse", "assets/sceneprefab/building_pocangku_01.ab", None, "building_pocangku_01_LOD2", "building_pocangku_01_d", False, 0.0, True),
                ("building_barracks", "assets/sceneprefab/project_jidifangzi_01.ab", None, "Project_jidifangzi_01", "Project_jidifangzi_01", False, 0.0, True),
                ("building_garage", "assets/sceneprefab/building_daxingcheku_01.ab", None, "building_daxingcheku_01", "building_daxingcheku_01_d", False, 0.0, True),
                ("building_pylon", "assets/sceneprefab/project_tieta_01.ab", None, "project_tieta_01", "project_tieta_01_d", False, 0.0, True),
                ("building_watchtower", "assets/sceneprefab/project_shaota_01.ab", None, "project_shaota_01", "project_shaota_01_d", False, 0.0, True),
                ("prop_fence", "assets/sceneprefab/project_weiqiang_01.ab", None, "project_weiqiang_01", "project_weiqiang_01_LOD2", False, 0.0, True),
                ("prop_military_box", "assets/sceneprefab/project_junshixiangzi_01.ab", None, "Project_junshixiangzi_01_LOD2", "Project_junshixiangzi_01_LOD2", False, 0.0, True),
            ]
            model_records = []
            for item in prop_specs:
                if len(item) == 8:
                    key, bpath, tbpath, mname, tname, z_up, target_h, pres_piv = item
                elif len(item) == 7:
                    key, bpath, tbpath, mname, tname, z_up, target_h = item
                    pres_piv = False
                else:
                    key, bpath, mname, tname, z_up, target_h = item
                    tbpath = None
                    pres_piv = False
                tex_filename = f"{key}.png"
                tex_bundle = tbpath if tbpath else bpath
                env_t = UnityPy.load(obb.read(tex_bundle))
                for obj in env_t.objects:
                    if obj.type.name == "Texture2D":
                        tex = obj.read()
                        if tex.m_Name == tname:
                            tex.image.save(os.path.join(TEX_DIR, tex_filename), compress_level=1)
                            break
                        if tex.m_Name.endswith("_d") and tname.endswith("_d"):
                            pass
                env_m = UnityPy.load(obb.read(bpath)) if tbpath else env_t
                found_mesh = False
                for obj in env_m.objects:
                    if obj.type.name == "Mesh":
                        m = obj.read()
                        if m.m_Name == mname or m.m_Name.startswith(mname):
                            res = extract_mesh_arrays(m, z_up=z_up, is_terrain=False, target_height=target_h, preserve_pivot=pres_piv)
                            if res:
                                model_records.append((key, tex_filename, *res))
                                found_mesh = True
                                break
                if not found_mesh:
                    print(f"Warning: mesh {mname!r} not found in {bpath}")

            models_bin_path = os.path.join(OUT_DIR, "models.bin")
            with open(models_bin_path, "wb") as mf:
                mf.write(b"MSMDL001")
                mf.write(struct.pack("<I", len(model_records)))
                for rec in model_records:
                    pack_mesh_record(mf, *rec)
            print(f"Saved {len(model_records)} 3D models to {models_bin_path} ({os.path.getsize(models_bin_path) // 1024} KB)")

            # 5. Extract grass clusters, trees, pebble clusters, large boulders, and authentic building/prop structures
            # from all sceneinfo/*.oc files (covering the main island and military base sectors).
            all_ocs = sorted([f for f in obb.namelist() if f.startswith("assets/sceneinfo/") and f.endswith(".oc")])
            placements = []  # (kind_u8, x, y, z, qx, qy, qz, qw, scale)
            for oc_path in all_ocs:
                d = obb.read(oc_path)
                if len(d) < 5 or d[0] != 255:
                    continue
                ncount = struct.unpack_from("<I", d, 1)[0]
                if ncount > 5000:
                    continue
                pos = 5
                names = []
                ok = True
                for _ in range(ncount):
                    if pos + 4 > len(d):
                        ok = False
                        break
                    slen = struct.unpack_from("<I", d, pos)[0]
                    pos += 4
                    if pos + slen > len(d):
                        ok = False
                        break
                    names.append(d[pos : pos + slen].decode("utf-8", errors="ignore"))
                    pos += slen
                if not ok:
                    continue

                # Group 1: grass tuft clusters and small stone pebbles (stride 40 bytes)
                if pos + 4 <= len(d):
                    g1 = struct.unpack_from("<I", d, pos)[0]
                    pos += 4
                    for i in range(g1):
                        if pos + 40 > len(d):
                            break
                        px, py, pz, sx, sy, sz, rx, ry, rz = struct.unpack_from("<9f", d, pos)
                        pidx = struct.unpack_from("<I", d, pos + 36)[0]
                        pos += 40
                        pname = names[pidx].lower() if pidx < len(names) else ""
                        wy = terrain_height_adjust(py, False)
                        qx, qy, qz, qw = unity_euler_to_quat(rx, ry, rz)
                        if "suishi" in pname or "stone" in pname or "rock" in pname:
                            if i % 3 == 0:
                                placements.append((4, px, wy, pz, qx, qy, qz, qw, max(0.5, min(1.4, sy))))
                        else:
                            if i % 15 == 0:
                                placements.append((3, px, wy, pz, qx, qy, qz, qw, max(0.65, min(1.35, sy))))

                # Group 2: scenic trees (stride 43 bytes)
                if pos + 4 <= len(d):
                    g2 = struct.unpack_from("<I", d, pos)[0]
                    pos += 4
                    for i in range(g2):
                        if pos + 43 > len(d):
                            break
                        px, py, pz, sx, sy, sz, rx, ry, rz = struct.unpack_from("<9f", d, pos)
                        pidx = struct.unpack_from("<H", d, pos + 36)[0]
                        pos += 43
                        pname = names[pidx].lower() if pidx < len(names) else ""
                        wy = terrain_height_adjust(py, False)
                        kind = 1 if ("redai" in pname or px > 80.0) else 0
                        qx, qy, qz, qw = unity_euler_to_quat(rx, ry, rz)
                        if i % 3 == 0:
                            placements.append((kind, px, wy, pz, qx, qy, qz, qw, max(0.65, min(1.45, sy))))

                # Skip Group 3 & 4
                if pos + 8 <= len(d):
                    g3 = struct.unpack_from("<I", d, pos)[0]
                    pos += 4
                    g4 = struct.unpack_from("<I", d, pos)[0]
                    pos += 4

                # Group 5: boulders & buildings (stride 55 bytes)
                if pos + 4 <= len(d):
                    g5 = struct.unpack_from("<I", d, pos)[0]
                    pos += 4
                    for i in range(g5):
                        if pos + 55 > len(d):
                            break
                        px, py, pz, sx, sy, sz, rx, ry, rz = struct.unpack_from("<9f", d, pos)
                        pidx = struct.unpack_from("<H", d, pos + 36)[0]
                        pos += 55
                        if pidx >= len(names):
                            continue
                        pname = names[pidx].lower()
                        wy = terrain_height_adjust(py, False)
                        bkind = None
                        if "jiayouzhan" in pname:
                            bkind = 5
                        elif "chaoshi" in pname:
                            bkind = 6
                        elif "cangku" in pname and "po" not in pname:
                            bkind = 7
                        elif "pocangku" in pname:
                            bkind = 8
                        elif any(k in pname for k in ["jidifangzi", "junyongzhangpeng", "dazhangpeng", "fangzi", "muwu"]):
                            bkind = 9
                        elif "cheku" in pname:
                            bkind = 10
                        elif "tieta" in pname:
                            bkind = 11
                        elif any(k in pname for k in ["shaota", "kongzhita"]):
                            bkind = 12
                        elif any(k in pname for k in ["weiqiang", "zhalan", "tiesiwang"]):
                            bkind = 13
                        elif any(k in pname for k in ["xiangzi", "jizhuangxiang", "youguan", "shuiniguan"]):
                            bkind = 14
                        elif any(k in pname for k in ["stone", "rock", "yanshi", "kuang"]):
                            if i % 6 == 0:
                                bkind = 2

                        if bkind is not None:
                            if bkind >= 5:
                                if abs(rx) < 5.0:
                                    rx = 270.0
                                sc = max(0.5, min(3.0, sy))
                            else:
                                sc = max(0.7, min(2.2, sy))
                            qx, qy, qz, qw = unity_euler_to_quat(rx, ry, rz)
                            placements.append((bkind, px, wy, pz, qx, qy, qz, qw, sc))

            placements_path = os.path.join(OUT_DIR, "placements.bin")
            with open(placements_path, "wb") as pf:
                pf.write(b"MSPLC002")
                pf.write(struct.pack("<I", len(placements)))
                for kind, wx, wy, wz, qx, qy, qz, qw, sc in placements:
                    pf.write(struct.pack("<B8f", kind, wx, wy, wz, qx, qy, qz, qw, sc))
            print(f"Saved {len(placements)} scenic placements with full 3D Quaternions to {placements_path}")

    # Automatically invoke terrain texture baker to turn raw splatmaps into realistic terrain textures
    try:
        import bake_terrain_textures
        bake_terrain_textures.main()
    except Exception as e:
        print(f"Warning: could not run bake_terrain_textures: {e}")

    print(f"Done in {time.time() - t0:.2f}s!")


if __name__ == "__main__":
    main()
