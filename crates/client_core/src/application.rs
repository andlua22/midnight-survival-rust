use std::collections::VecDeque;

use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::tonemapping::Tonemapping,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    pbr::{DistanceFog, FogFalloff},
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        renderer::RenderAdapterInfo,
        settings::{Backends, PowerPreference, RenderCreation, WgpuSettings},
    },
    window::{PresentMode, PrimaryWindow, WindowResizeConstraints, WindowResolution},
};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use shared::{
    CharacterMotion, LevelGeometry, LocalPlayer, ResourceNodeMarker, SharedSimulationPlugin,
    SimulationClock, SimulationInput, SimulationPose, SimulationSurvival, SimulationWorld,
    StructureMarker,
    movement::{EYE_OFFSET, preview_look},
    protocol::{PlayerId, ServerTick},
    survival::{
        Biome, BuildingTier, FALLOUT_CRATE_POS, ItemKind, NUCLEAR_STRIKE_CENTER, NpcEntity,
        NpcKind, NuclearPhase, PlacedStructure, RADTOWN_CENTER, RADTOWN_FUSEBOX_POS,
        RADTOWN_MILITARY_CRATE_POS, RADTOWN_NUCLEAR_BUTTON_POS, RADTOWN_READER_POS,
        RADTOWN_RECYCLER_POS, RADTOWN_SUPPLY_CRATE_POS, ResourceKind, StructureKind, VehicleEntity,
        VehicleKind, WeatherKind, cargo_ship_transform, preview_socket_placement,
    },
};

use crate::{
    interface::{
        Interface, LookedObjectInfo, LookedTargetKind, PlatformStorage, Telemetry, UiMode,
        draw_interface,
    },
    launch::ClientOptions,
    metrics::{
        BenchmarkRecorder, BenchmarkReport, CAPTURE_SECONDS, FrameStatistics, WARMUP_SECONDS,
    },
    online::{Online, OnlinePlugin},
    online_metrics::{OnlineCapture, capture},
    storage::{StorageService, write_report},
};

use bevy::audio::{AudioPlayer, PlaybackSettings};

#[derive(Resource)]
struct RuntimeOptions(ClientOptions);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct PlayerBodyVisual;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum CameraPerspective {
    #[default]
    FirstPerson,
    ThirdPerson,
}

#[derive(Resource, Clone)]
pub struct MapTextures {
    pub overview: Handle<Image>,
    pub marker: Handle<Image>,
    pub cargo_ship: Handle<Image>,
    pub vehicle: Handle<Image>,
    pub loading_bg: Handle<Image>,
    pub loading_bar_bg: Handle<Image>,
    pub loading_bar_fill: Handle<Image>,
}

#[derive(Resource, Clone)]
pub struct AudioAssets {
    pub gunshot_ak: Handle<AudioSource>,
    pub gunshot_shotgun: Handle<AudioSource>,
    pub gunshot_pistol: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
    pub reload_out: Handle<AudioSource>,
    pub reload_in: Handle<AudioSource>,
    pub reload_bolt: Handle<AudioSource>,
    pub player_hit: Handle<AudioSource>,
    pub footstep_grass: Handle<AudioSource>,
    pub footstep_metal: Handle<AudioSource>,
    pub footstep_water: Handle<AudioSource>,
    pub chop_wood: Handle<AudioSource>,
    pub chop_rock: Handle<AudioSource>,
    pub keycard_swipe: Handle<AudioSource>,
    pub fuse_switch: Handle<AudioSource>,
    pub map_pin: Handle<AudioSource>,
    pub map_unpin: Handle<AudioSource>,
}

#[derive(Resource, Default)]
pub struct AudioRuntimeState {
    pub footstep_timer: f32,
    pub prev_health: f32,
    pub prev_swing_timer: f32,
    pub prev_shot_trace: bool,
    pub prev_monument_power: bool,
    pub prev_monument_door: bool,
    pub prev_nuke_active: bool,
}

#[derive(Component)]
struct HeldToolVisual;

#[derive(Component)]
struct ThirdPersonHeldToolVisual;

#[derive(Component)]
struct BuildingGhostVisual;

#[derive(Component)]
struct SunLight;

#[derive(Component)]
struct SunDiscVisual;

#[derive(Component)]
struct SkyDomeVisual;

#[derive(Component)]
struct CloudCanopyVisual;

#[derive(Component)]
struct CirrusCanopyVisual;

#[derive(Component)]
pub struct TerrainSectorChunk {
    pub center: Vec3,
    pub is_road: bool,
}

pub const CHUNK_CELL_SIZE: f32 = 180.0;
pub const CHUNK_ACTIVE_RADIUS: i32 = 2; // (2*2+1)^2 = 25 cells ~ 900m x 900m coverage
pub const CHUNK_DESPAWN_RADIUS: i32 = 3; // Despawn distance > 540m

#[derive(Resource, Default)]
pub struct SpatialPlacementGrid {
    pub cells: std::collections::BTreeMap<(i32, i32), Vec<shared::level::ImportedPlacement>>,
    pub active_entities: std::collections::BTreeMap<(i32, i32), Vec<Entity>>,
    pub current_player_cell: Option<(i32, i32)>,
}

#[derive(Component)]
struct MonumentPowerBeacon;

#[derive(Component)]
struct MonumentBunkerDoor;

#[derive(Component)]
struct BulletTracerVisual {
    remaining_secs: f32,
}

#[derive(Component)]
struct NpcVisual(u16);

#[derive(Component)]
struct VehicleVisual(u16);

#[derive(Component)]
struct CargoShipVisual;

#[derive(Component)]
struct NuclearMushroomVisual;

#[derive(Component)]
struct FalloutCrateVisual;

#[derive(Component, Clone, Copy, Debug)]
pub struct ScenicPlacementMarker {
    pub position: Vec3,
    pub scale: f32,
}

#[derive(Resource, Clone)]
struct ToolAssets {
    rock: (Handle<Mesh>, Handle<StandardMaterial>),
    hatchet: (Handle<Mesh>, Handle<StandardMaterial>),
    pickaxe: (Handle<Mesh>, Handle<StandardMaterial>),
    spear: (Handle<Mesh>, Handle<StandardMaterial>),
    bow: (Handle<Mesh>, Handle<StandardMaterial>),
    revolver: (Handle<Mesh>, Handle<StandardMaterial>),
    rifle: (Handle<Mesh>, Handle<StandardMaterial>),
    blueprint: (Handle<Mesh>, Handle<StandardMaterial>),
    hammer: (Handle<Mesh>, Handle<StandardMaterial>),
    consumable: (Handle<Mesh>, Handle<StandardMaterial>),
    unit_cube: Handle<Mesh>,
    sphere_mesh: Handle<Mesh>,
    cylinder_mesh: Handle<Mesh>,
    doorway_pillar: Handle<Mesh>,
    doorway_lintel: Handle<Mesh>,
    campfire_flame: Handle<StandardMaterial>,
    sky_zenith_material: Handle<StandardMaterial>,
    sky_horizon_material: Handle<StandardMaterial>,
    cloud_canopy_material: Handle<StandardMaterial>,
    cirrus_canopy_material: Handle<StandardMaterial>,
    sun_disc_material: Handle<StandardMaterial>,
    sun_corona_material: Handle<StandardMaterial>,
    power_on_material: Handle<StandardMaterial>,
    power_off_material: Handle<StandardMaterial>,
    tracer_mesh: Handle<Mesh>,
    tracer_material: Handle<StandardMaterial>,
    npc_zombie_mat: Handle<StandardMaterial>,
    npc_rad_zombie_mat: Handle<StandardMaterial>,
    npc_scientist_mat: Handle<StandardMaterial>,
    npc_bradley_mat: Handle<StandardMaterial>,
    npc_heli_mat: Handle<StandardMaterial>,
    vehicle_bike_mat: Handle<StandardMaterial>,
    vehicle_buggy_mat: Handle<StandardMaterial>,
    vehicle_humvee_mat: Handle<StandardMaterial>,
    vehicle_copter_mat: Handle<StandardMaterial>,
    wheel_dark_mat: Handle<StandardMaterial>,
    tier_materials: [Handle<StandardMaterial>; 4],
    ghost_valid: Handle<StandardMaterial>,
    ghost_invalid: Handle<StandardMaterial>,
    imported_models: std::collections::BTreeMap<String, (Handle<Mesh>, Handle<StandardMaterial>)>,
}

impl ToolAssets {
    fn tier_material(&self, tier: BuildingTier) -> Handle<StandardMaterial> {
        let index = match tier {
            BuildingTier::Twig => 0,
            BuildingTier::Wood => 1,
            BuildingTier::Stone => 2,
            BuildingTier::Metal => 3,
        };
        self.tier_materials[index].clone()
    }
}

#[derive(Resource, Default)]
struct FrameMonitor {
    recent_ms: VecDeque<f64>,
    display_elapsed: f64,
    benchmark: BenchmarkRecorder,
    focused_last_frame: bool,
    capture_resolution: [u32; 2],
}

fn resolve_asset_root() -> String {
    for candidate in ["assets", "../../assets", "../assets"] {
        let p = std::path::Path::new(candidate);
        if p.join("imported_map").exists() {
            if let Ok(canon) = std::fs::canonicalize(p) {
                return canon.to_string_lossy().to_string();
            }
            return candidate.to_string();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent();
        while let Some(parent) = cur {
            let candidate = parent.join("assets");
            if candidate.join("imported_map").exists() {
                return candidate.to_string_lossy().to_string();
            }
            cur = parent.parent();
        }
    }
    "assets".to_string()
}

pub fn build_app(options: ClientOptions, storage: Box<dyn StorageService>) -> App {
    let interface = Interface::new(storage.as_ref(), options.benchmark);
    let resolution = if options.benchmark {
        WindowResolution::new(1920, 1080).with_scale_factor_override(1.0)
    } else {
        WindowResolution::new(1280, 720)
    };
    let mut app = App::new();
    app.insert_resource(OnlineCapture::new(&options))
        .insert_resource(Online::new(&options))
        .insert_resource(interface)
        .insert_resource(PlatformStorage(storage))
        .init_resource::<Telemetry>()
        .init_resource::<FrameMonitor>()
        .insert_resource(ClearColor(Color::srgb(0.28, 0.56, 0.88)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.72, 0.84, 0.98),
            brightness: 320.0,
            ..default()
        })
        .insert_resource(bevy::winit::WinitSettings::continuous())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: resolve_asset_root(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Midnight Survival".into(),
                        resolution,
                        present_mode: if options.no_vsync {
                            PresentMode::Immediate
                        } else {
                            PresentMode::AutoVsync
                        },
                        resizable: !options.benchmark,
                        resize_constraints: WindowResizeConstraints {
                            min_width: 640.0,
                            min_height: 360.0,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        power_preference: PowerPreference::HighPerformance,
                        ..default()
                    })),
                    ..default()
                }),
        )
        .add_plugins((
            SharedSimulationPlugin,
            EguiPlugin::default(),
            OnlinePlugin,
            bevy::remote::RemotePlugin::default(),
            bevy::remote::http::RemoteHttpPlugin::default(),
        ))
        .insert_resource(RuntimeOptions(options))
        .init_resource::<AudioRuntimeState>()
        .init_resource::<CameraPerspective>()
        .init_resource::<SpatialPlacementGrid>()
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                register_adapter,
                attach_geometry,
                follow_camera,
                sync_survival_visuals,
                sync_third_person_tool,
                sync_phase5_world_visuals,
                sync_environment_atmosphere,
                stream_world_chunks,
                sync_audio_effects,
                measure_frames,
            )
                .chain(),
        )
        .add_systems(EguiPrimaryContextPass, draw_interface)
        .add_systems(Last, capture);
    app
}

fn build_bevy_mesh(rec: &shared::level::ImportedMeshRecord) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, rec.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, rec.normals.clone());
    let is_road = rec.name.to_ascii_lowercase().contains("road")
        || rec.texture.to_ascii_lowercase().contains("road");
    let uvs = if is_road {
        rec.uvs
            .iter()
            .map(|&[u, v]| [u, 1.0 - v])
            .collect::<Vec<_>>()
    } else {
        rec.uvs.clone()
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(rec.indices.clone()));
    mesh
}

fn hash_lattice(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ (z as u32).wrapping_mul(95_127_119);
    h = h.wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

fn grad3(hash: u32, dx: f32, dy: f32, dz: f32) -> f32 {
    match hash % 12 {
        0 => dx + dy,
        1 => -dx + dy,
        2 => dx - dy,
        3 => -dx - dy,
        4 => dx + dz,
        5 => -dx + dz,
        6 => dx - dz,
        7 => -dx - dz,
        8 => dy + dz,
        9 => -dy + dz,
        10 => dy - dz,
        _ => -dy - dz,
    }
}

fn perlin_fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn periodic_perlin3d(x: f32, y: f32, z: f32, period: i32) -> f32 {
    let p = period.max(1);
    let x0 = (x.floor() as i32).rem_euclid(p);
    let y0 = (y.floor() as i32).rem_euclid(p);
    let z0 = (z.floor() as i32).rem_euclid(p);

    let x1 = (x0 + 1) % p;
    let y1 = (y0 + 1) % p;
    let z1 = (z0 + 1) % p;

    let fx = x.fract();
    let fy = y.fract();
    let fz = z.fract();

    let u = perlin_fade(fx);
    let v = perlin_fade(fy);
    let w = perlin_fade(fz);

    let g000 = grad3(hash_lattice(x0, y0, z0), fx, fy, fz);
    let g100 = grad3(hash_lattice(x1, y0, z0), fx - 1.0, fy, fz);
    let g010 = grad3(hash_lattice(x0, y1, z0), fx, fy - 1.0, fz);
    let g110 = grad3(hash_lattice(x1, y1, z0), fx - 1.0, fy - 1.0, fz);
    let g001 = grad3(hash_lattice(x0, y0, z1), fx, fy, fz - 1.0);
    let g101 = grad3(hash_lattice(x1, y0, z1), fx - 1.0, fy, fz - 1.0);
    let g011 = grad3(hash_lattice(x0, y1, z1), fx, fy - 1.0, fz - 1.0);
    let g111 = grad3(hash_lattice(x1, y1, z1), fx - 1.0, fy - 1.0, fz - 1.0);

    let x00 = g000 + u * (g100 - g000);
    let x10 = g010 + u * (g110 - g010);
    let x01 = g001 + u * (g101 - g001);
    let x11 = g011 + u * (g111 - g011);

    let y0 = x00 + v * (x10 - x00);
    let y1 = x01 + v * (x11 - x01);

    y0 + w * (y1 - y0)
}

pub fn generate_cloud_3d_image() -> Image {
    const SIZE: usize = 64;
    let mut data = Vec::with_capacity(SIZE * SIZE * SIZE);

    for iz in 0..SIZE {
        for iy in 0..SIZE {
            for ix in 0..SIZE {
                let u = ix as f32 / SIZE as f32;
                let v = iy as f32 / SIZE as f32;
                let w = iz as f32 / SIZE as f32;

                // 3 octaves of periodic 3D Perlin noise seamlessly tiling across [0, 1)
                let n1 = periodic_perlin3d(u * 4.0, v * 4.0, w * 4.0, 4);
                let n2 = periodic_perlin3d(u * 8.0, v * 8.0, w * 8.0, 8);
                let n3 = periodic_perlin3d(u * 16.0, v * 16.0, w * 16.0, 16);

                let fbm = (0.55 * n1 + 0.30 * n2 + 0.15 * n3 + 1.0) * 0.5;

                // Vertical cumulus envelope: flat condensation level at base, billowy rounded tops
                let bottom_rise = (v / 0.18).clamp(0.0, 1.0);
                let top_falloff = ((1.0 - v) / 0.35).clamp(0.0, 1.0);
                let envelope = bottom_rise * top_falloff;

                // Density threshold creating distinct cumulus puffs with clear sky gaps
                let density = ((fbm * envelope - 0.38) / 0.62).clamp(0.0, 1.0);
                let byte_val = (density.powf(1.35) * 255.0).round() as u8;
                data.push(byte_val);
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: SIZE as u32,
        },
        TextureDimension::D3,
        data,
        TextureFormat::R8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

pub fn generate_cloud_canopy_texture() -> Image {
    const SIZE: usize = 512;
    let mut data = Vec::with_capacity(SIZE * SIZE * 4);

    let mut density_grid = vec![0.0f32; SIZE * SIZE];
    for iy in 0..SIZE {
        for ix in 0..SIZE {
            let u = ix as f32 / SIZE as f32;
            let v = iy as f32 / SIZE as f32;

            // 4 octaves of periodic Perlin noise seamlessly tiling across [0, 1)
            let n1 = periodic_perlin3d(u * 3.0, v * 3.0, 0.25, 3);
            let n2 = periodic_perlin3d(u * 6.0, v * 6.0, 0.50, 6);
            let n3 = periodic_perlin3d(u * 12.0, v * 12.0, 0.75, 12);
            let n4 = periodic_perlin3d(u * 24.0, v * 24.0, 1.25, 24);

            let fbm = (0.52 * n1 + 0.28 * n2 + 0.14 * n3 + 0.06 * n4 + 1.0) * 0.5;

            // Billowy cumulus puff curve with wide clear sky openings
            let coverage = 0.46;
            let density = ((fbm - coverage) / (1.0 - coverage)).clamp(0.0, 1.0);
            density_grid[iy * SIZE + ix] = density;
        }
    }

    // Compute 3D volumetric billow shading from surface gradients
    for iy in 0..SIZE {
        for ix in 0..SIZE {
            let d = density_grid[iy * SIZE + ix];
            if d <= 0.001 {
                data.push(220);
                data.push(230);
                data.push(245);
                data.push(0);
                continue;
            }

            let left = density_grid[iy * SIZE + (ix + SIZE - 1) % SIZE];
            let right = density_grid[iy * SIZE + (ix + 1) % SIZE];
            let down = density_grid[((iy + SIZE - 1) % SIZE) * SIZE + ix];
            let up = density_grid[((iy + 1) % SIZE) * SIZE + ix];

            let dx = (right - left) * 1.5;
            let dy = (up - down) * 1.5;

            // Sunlight from diagonal direction
            let slope = dx * 0.55 + dy * 0.55;
            let diffuse = (0.82 + slope * 0.28).clamp(0.68, 1.0);
            let ao = 0.88 + 0.12 * d;
            let shade = (diffuse * ao * 255.0).clamp(160.0, 255.0).round() as u8;

            let alpha = (d.powf(1.35) * 255.0).clamp(0.0, 255.0).round() as u8;

            data.push(shade);
            data.push(shade);
            data.push(((shade as f32 * 1.02).min(255.0)) as u8);
            data.push(alpha);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

pub fn generate_cirrus_texture() -> Image {
    const SIZE: usize = 512;
    let mut data = Vec::with_capacity(SIZE * SIZE * 4);

    for iy in 0..SIZE {
        for ix in 0..SIZE {
            let u = ix as f32 / SIZE as f32;
            let v = iy as f32 / SIZE as f32;

            let su = u * 4.0 + v * 2.0;
            let sv = v * 8.0 - u * 1.5;

            let n1 = periodic_perlin3d(su, sv, 0.1, 4);
            let n2 = periodic_perlin3d(su * 2.0, sv * 2.0, 0.4, 8);
            let n3 = periodic_perlin3d(su * 4.0, sv * 4.0, 0.8, 16);

            let fbm = (0.55 * n1 + 0.30 * n2 + 0.15 * n3 + 1.0) * 0.5;

            let coverage = 0.50;
            let density = ((fbm - coverage) / (1.0 - coverage)).clamp(0.0, 1.0);
            let alpha = (density.powf(1.8) * 135.0).round() as u8;

            data.push(250);
            data.push(252);
            data.push(255);
            data.push(alpha);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

pub fn generate_cloud_dome_texture() -> Image {
    generate_cloud_canopy_texture()
}

fn build_sky_dome_mesh(radius: f32) -> Mesh {
    let rings = 32;
    let sectors = 64;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for r in 0..=rings {
        let t = r as f32 / rings as f32;
        let phi = t * 1.62;
        let y = radius * phi.cos();
        let ring_r = radius * phi.sin();

        // Natural Rayleigh vertical gradient
        let cos_phi = phi.cos().clamp(0.0, 1.0);
        let grad = cos_phi.powf(0.55);
        let cr = 0.72 - grad * 0.54;
        let cg = 0.84 - grad * 0.40;
        let cb = 0.98 - grad * 0.12;

        for s in 0..=sectors {
            let theta = s as f32 / sectors as f32 * std::f32::consts::TAU;
            let x = ring_r * theta.sin();
            let z = ring_r * theta.cos();

            positions.push([x, y, z]);
            normals.push([-x / radius, -y / radius, -z / radius]);
            colors.push([cr, cg, cb, 1.0]);
        }
    }

    for r in 0..rings {
        for s in 0..sectors {
            let cur = r * (sectors + 1) + s;
            let next = cur + sectors + 1;

            indices.push(cur as u32);
            indices.push((next + 1) as u32);
            indices.push((cur + 1) as u32);

            indices.push(cur as u32);
            indices.push(next as u32);
            indices.push((next + 1) as u32);
        }
    }

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

fn build_cloud_canopy_mesh(radius: f32, altitude: f32) -> Mesh {
    let rings = 40;
    let sectors = 64;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let uv_scale = 0.00032;

    for r in 0..=rings {
        let t = r as f32 / rings as f32;
        let ring_r = radius * t;

        // Altitude: flat horizontal plane overhead across central 2200m radius,
        // then gently follows natural planetary curvature to stay high above terrain
        let horizon_dip = if t > 0.45 {
            let dt = (t - 0.45) / 0.55;
            dt * dt * 320.0
        } else {
            0.0
        };
        let y = altitude - horizon_dip;

        // Smooth cubic edge fade at horizon: 1.0 overhead -> 0.0 at far rim
        let alpha = if t < 0.48 {
            1.0
        } else {
            let fade = (1.0 - t) / 0.52;
            fade * fade * (3.0 - 2.0 * fade)
        };

        for s in 0..=sectors {
            let theta = s as f32 / sectors as f32 * std::f32::consts::TAU;
            let x = ring_r * theta.sin();
            let z = ring_r * theta.cos();

            positions.push([x, y, z]);
            normals.push([0.0, -1.0, 0.0]);
            // Planar world UV: completely eliminates polar pinching/spherical bowl warping!
            uvs.push([x * uv_scale + 0.5, z * uv_scale + 0.5]);
            colors.push([1.0, 1.0, 1.0, alpha]);
        }
    }

    for r in 0..rings {
        for s in 0..sectors {
            let cur = r * (sectors + 1) + s;
            let next = cur + sectors + 1;

            indices.push(cur as u32);
            indices.push((next + 1) as u32);
            indices.push((cur + 1) as u32);

            indices.push(cur as u32);
            indices.push(next as u32);
            indices.push((next + 1) as u32);
        }
    }

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

pub fn build_cloud_dome_mesh(radius: f32) -> Mesh {
    build_cloud_canopy_mesh(radius, 1350.0)
}

#[allow(clippy::too_many_arguments)]
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut physics: ResMut<shared::SimulationWorld>,
    mut survival: ResMut<SimulationSurvival>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut grid: ResMut<SpatialPlacementGrid>,
) {
    let rock_mesh = meshes.add(Sphere::new(0.11).mesh().uv(18, 12));
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.56, 0.58),
        perceptual_roughness: 0.9,
        ..default()
    });
    let unit_cube = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let ghost_valid = materials.add(StandardMaterial {
        base_color: Color::srgba(0.28, 0.85, 0.62, 0.42),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let ghost_invalid = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.32, 0.30, 0.42),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let tier_materials = [
        BuildingTier::Twig,
        BuildingTier::Wood,
        BuildingTier::Stone,
        BuildingTier::Metal,
    ]
    .map(|tier| {
        let [r, g, b] = tier.color();
        materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            perceptual_roughness: 0.88,
            ..default()
        })
    });

    let sky_zenith_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let sky_horizon_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.72, 0.84, 0.98, 0.88),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let cloud_canopy_texture = images.add(generate_cloud_canopy_texture());
    let cloud_canopy_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.95),
        base_color_texture: Some(cloud_canopy_texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let cirrus_canopy_texture = images.add(generate_cirrus_texture());
    let cirrus_canopy_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.70),
        base_color_texture: Some(cirrus_canopy_texture),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let sun_disc_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.97, 0.86),
        emissive: LinearRgba::rgb(4.0, 3.4, 2.2),
        unlit: true,
        ..default()
    });
    let sun_corona_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.88, 0.58, 0.34),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let power_on_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.96, 0.38),
        emissive: LinearRgba::rgb(0.6, 3.2, 0.8),
        unlit: true,
        ..default()
    });
    let power_off_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.94, 0.22, 0.20),
        emissive: LinearRgba::rgb(2.6, 0.25, 0.20),
        unlit: true,
        ..default()
    });
    let tracer_mesh = meshes.add(Cylinder::new(0.022, 1.0).mesh().resolution(10));
    let tracer_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.88, 0.38),
        emissive: LinearRgba::rgb(3.6, 2.6, 0.6),
        unlit: true,
        ..default()
    });

    let tool_assets = ToolAssets {
        rock: (rock_mesh.clone(), rock_mat.clone()),
        hatchet: (
            meshes.add(Cylinder::new(0.035, 0.44).mesh().resolution(12)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.68, 0.50, 0.32),
                perceptual_roughness: 0.85,
                ..default()
            }),
        ),
        pickaxe: (
            meshes.add(Cylinder::new(0.038, 0.46).mesh().resolution(12)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.56, 0.62, 0.68),
                perceptual_roughness: 0.8,
                ..default()
            }),
        ),
        spear: (
            meshes.add(Cuboid::new(0.04, 0.04, 0.95)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.72, 0.54, 0.34),
                perceptual_roughness: 0.85,
                ..default()
            }),
        ),
        bow: (
            meshes.add(Cuboid::new(0.04, 0.62, 0.14)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.64, 0.44, 0.24),
                perceptual_roughness: 0.82,
                ..default()
            }),
        ),
        revolver: (
            meshes.add(Cuboid::new(0.06, 0.14, 0.32)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.46, 0.50, 0.54),
                metallic: 0.75,
                perceptual_roughness: 0.35,
                ..default()
            }),
        ),
        rifle: (
            meshes.add(Cuboid::new(0.07, 0.16, 0.68)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.32, 0.36),
                metallic: 0.8,
                perceptual_roughness: 0.32,
                ..default()
            }),
        ),
        blueprint: (
            meshes.add(Cuboid::new(0.22, 0.16, 0.03)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.58, 0.88),
                perceptual_roughness: 0.6,
                ..default()
            }),
        ),
        hammer: (
            meshes.add(Cuboid::new(0.18, 0.38, 0.12)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.76, 0.56, 0.32),
                perceptual_roughness: 0.82,
                ..default()
            }),
        ),
        consumable: (
            meshes.add(Cylinder::new(0.065, 0.18).mesh().resolution(16)),
            materials.add(StandardMaterial {
                base_color: Color::srgb(0.38, 0.72, 0.68),
                perceptual_roughness: 0.7,
                ..default()
            }),
        ),
        unit_cube: unit_cube.clone(),
        sphere_mesh: meshes.add(Sphere::new(1.0).mesh().uv(20, 14)),
        cylinder_mesh: meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(18)),
        doorway_pillar: meshes.add(Cuboid::new(0.90, 2.50, 0.30)),
        doorway_lintel: meshes.add(Cuboid::new(1.20, 0.60, 0.30)),
        campfire_flame: materials.add(StandardMaterial {
            base_color: Color::srgb(0.98, 0.56, 0.16),
            emissive: LinearRgba::rgb(2.4, 0.9, 0.18),
            unlit: true,
            ..default()
        }),
        sky_zenith_material: sky_zenith_material.clone(),
        sky_horizon_material: sky_horizon_material.clone(),
        cloud_canopy_material: cloud_canopy_material.clone(),
        cirrus_canopy_material: cirrus_canopy_material.clone(),
        sun_disc_material: sun_disc_material.clone(),
        sun_corona_material: sun_corona_material.clone(),
        power_on_material: power_on_material.clone(),
        power_off_material: power_off_material.clone(),
        tracer_mesh,
        tracer_material,
        npc_zombie_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.28, 0.48, 0.26),
            perceptual_roughness: 0.9,
            ..default()
        }),
        npc_rad_zombie_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.46, 0.94, 0.22),
            emissive: LinearRgba::rgb(0.5, 1.8, 0.15),
            perceptual_roughness: 0.7,
            ..default()
        }),
        npc_scientist_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.62, 0.88),
            perceptual_roughness: 0.65,
            ..default()
        }),
        npc_bradley_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.32, 0.38, 0.24),
            metallic: 0.75,
            perceptual_roughness: 0.52,
            ..default()
        }),
        npc_heli_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.26, 0.30),
            metallic: 0.82,
            perceptual_roughness: 0.42,
            ..default()
        }),
        vehicle_bike_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.84, 0.26, 0.22),
            metallic: 0.65,
            perceptual_roughness: 0.45,
            ..default()
        }),
        vehicle_buggy_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.88, 0.64, 0.20),
            metallic: 0.55,
            perceptual_roughness: 0.55,
            ..default()
        }),
        vehicle_humvee_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.42, 0.46, 0.30),
            metallic: 0.70,
            perceptual_roughness: 0.58,
            ..default()
        }),
        vehicle_copter_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.74, 0.78, 0.82),
            metallic: 0.75,
            perceptual_roughness: 0.45,
            ..default()
        }),
        wheel_dark_mat: materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.15, 0.17),
            perceptual_roughness: 0.92,
            ..default()
        }),
        tier_materials,
        ghost_valid: ghost_valid.clone(),
        ghost_invalid,
        imported_models: std::collections::BTreeMap::new(),
    };
    let mut tool_assets = tool_assets;
    for rec in shared::level::load_imported_model_records() {
        let mesh_handle = meshes.add(build_bevy_mesh(&rec));
        let tex_handle: Handle<Image> =
            asset_server.load(format!("imported_map/textures/{}", rec.texture));
        let is_foliage = rec.name.starts_with("tree_")
            || rec.name.contains("grass")
            || rec.name.contains("fence");
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(tex_handle),
            alpha_mode: if is_foliage {
                AlphaMode::Mask(0.42)
            } else {
                AlphaMode::Opaque
            },
            cull_mode: None,
            double_sided: is_foliage,
            perceptual_roughness: 0.85,
            ..default()
        });
        tool_assets
            .imported_models
            .insert(rec.name.clone(), (mesh_handle, mat_handle));
    }

    let terrain_records = shared::level::load_imported_terrain_records();
    if !terrain_records.is_empty() {
        info!(
            "Loaded {} imported XAPK island sectors/roads (A-01..D-56) and {} 3D models",
            terrain_records.len(),
            tool_assets.imported_models.len()
        );
        physics.0.insert_imported_terrain(&terrain_records);

        // Remove test NPCs, enemies, bosses, vehicles, and demo resource nodes from the island world
        survival.0.npcs.clear();
        survival.0.vehicles.clear();
        survival.0.nodes.clear();
        let mut tex_mat_cache: std::collections::BTreeMap<String, Handle<StandardMaterial>> =
            std::collections::BTreeMap::new();
        for rec in &terrain_records {
            let lower_name = rec.name.to_ascii_lowercase();
            if lower_name.contains("jichang")
                || lower_name.contains("gangkou")
                || lower_name.contains("feiqichangfang")
                || lower_name.contains("maoyiqu")
                || lower_name.contains("shiyanshi")
                || lower_name.contains("object006")
            {
                continue;
            }
            let is_road = rec.name.to_ascii_lowercase().contains("road")
                || rec.texture.to_ascii_lowercase().contains("road");
            let tex_name = if !is_road {
                let upper = rec.name.to_ascii_uppercase();
                let sector_tex = format!("{upper}.png");
                if std::path::Path::new(&format!("assets/imported_map/textures/{sector_tex}"))
                    .exists()
                {
                    sector_tex
                } else {
                    // West chunks save as JUNSHIJIDI_TERRAIN_XX.png
                    rec.texture.clone()
                }
            } else {
                rec.texture.clone()
            };
            let mat = tex_mat_cache
                .entry(tex_name.clone())
                .or_insert_with(|| {
                    let tex: Handle<Image> = if is_road {
                        asset_server
                            .load_builder()
                            .with_settings(|settings: &mut bevy::image::ImageLoaderSettings| {
                                settings.sampler = bevy::image::ImageSampler::Descriptor(
                                    bevy::image::ImageSamplerDescriptor {
                                        address_mode_u: bevy::image::ImageAddressMode::ClampToEdge,
                                        address_mode_v: bevy::image::ImageAddressMode::Repeat,
                                        ..default()
                                    },
                                );
                            })
                            .load(format!("imported_map/textures/{}", tex_name))
                    } else {
                        asset_server.load(format!("imported_map/textures/{}", tex_name))
                    };
                    materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(tex),
                        alpha_mode: if is_road {
                            AlphaMode::Blend
                        } else {
                            AlphaMode::Opaque
                        },
                        depth_bias: if is_road { 80.0 } else { 0.0 },
                        perceptual_roughness: if is_road { 0.88 } else { 0.94 },
                        cull_mode: None,
                        ..default()
                    })
                })
                .clone();
            let mesh_handle = meshes.add(build_bevy_mesh(rec));
            commands.spawn((
                TerrainSectorChunk {
                    center: Vec3::from_array(rec.center),
                    is_road,
                },
                Mesh3d(mesh_handle),
                MeshMaterial3d(mat),
                Transform::default(),
            ));
        }

        // Ocean water plane surrounding the 6x6 km island (sea level ~0)
        let ocean_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.04, 0.28, 0.42, 0.88),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.06,
            metallic: 0.12,
            reflectance: 0.50,
            ..default()
        });
        commands.spawn((
            Mesh3d(unit_cube.clone()),
            MeshMaterial3d(ocean_mat),
            Transform::from_xyz(0.0, -0.4, 0.0).with_scale(Vec3::new(6000.0, 0.5, 6000.0)),
        ));

        // Partition scenic placements from sceneinfo/*.oc into spatial grid cells (180m x 180m)
        for plc in shared::level::load_imported_placements() {
            if plc.kind == 2 {
                // Exclude giant monolithic cliff slabs (stone_dayanshi_03/07) that blocked the central island fields
                continue;
            }
            // Exclude placements near central spawn to eliminate obstructions and demo artifacts
            let dx = plc.position[0] - 0.0;
            let dz = plc.position[2] - 12.0;
            if dx * dx + dz * dz < 35.0 * 35.0 {
                continue;
            }
            let cx = (plc.position[0] / CHUNK_CELL_SIZE).floor() as i32;
            let cz = (plc.position[2] / CHUNK_CELL_SIZE).floor() as i32;
            grid.cells.entry((cx, cz)).or_default().push(plc);
        }

        // Spawn initial active placement cells around player spawn
        let init_cx = 0;
        let init_cz = 0;
        grid.current_player_cell = Some((init_cx, init_cz));
        for cx in (init_cx - CHUNK_ACTIVE_RADIUS)..=(init_cx + CHUNK_ACTIVE_RADIUS) {
            for cz in (init_cz - CHUNK_ACTIVE_RADIUS)..=(init_cz + CHUNK_ACTIVE_RADIUS) {
                let key = (cx, cz);
                if let Some(placements) = grid.cells.get(&key) {
                    let mut cell_entities = Vec::with_capacity(placements.len());
                    for plc in placements {
                        let model_key = match plc.kind {
                            1 => "tree_jungle",
                            3 => "grass_clump",
                            4 => {
                                if tool_assets.imported_models.contains_key("rock_small") {
                                    "rock_small"
                                } else {
                                    "rock_node"
                                }
                            }
                            5 => "building_gas_station",
                            6 => "building_supermarket",
                            7 => "building_warehouse",
                            8 => "building_old_warehouse",
                            9 => "building_barracks",
                            10 => "building_garage",
                            11 => "building_pylon",
                            12 => "building_watchtower",
                            13 => "prop_fence",
                            14 => "prop_military_box",
                            _ => "tree_temperate",
                        };
                        if let Some((m_h, mat_h)) = tool_assets.imported_models.get(model_key) {
                            let pos_vec = Vec3::from_array(plc.position);
                            let e = commands
                                .spawn((
                                    ScenicPlacementMarker {
                                        position: pos_vec,
                                        scale: plc.scale,
                                    },
                                    Mesh3d(m_h.clone()),
                                    MeshMaterial3d(mat_h.clone()),
                                    Transform::from_translation(pos_vec)
                                        .with_rotation(Quat::from_array(plc.rotation))
                                        .with_scale(Vec3::splat(plc.scale)),
                                ))
                                .id();
                            cell_entities.push(e);
                        }
                    }
                    grid.active_entities.insert(key, cell_entities);
                }
            }
        }
    }

    if let Some((body_mesh, body_mat)) = tool_assets.imported_models.get("player_survivor") {
        commands
            .spawn((
                PlayerBodyVisual,
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(body_mat.clone()),
                Transform::from_xyz(0.0, 1.0, 12.0),
                Visibility::Hidden,
            ))
            .with_children(|parent| {
                parent.spawn((
                    ThirdPersonHeldToolVisual,
                    Mesh3d(rock_mesh.clone()),
                    MeshMaterial3d(rock_mat.clone()),
                    Transform::from_xyz(0.28, 0.82, 0.35)
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, -0.2, 0.1, -0.3))
                        .with_scale(Vec3::splat(0.85)),
                    Visibility::Inherited,
                ));
            });
    }

    let audio_assets = AudioAssets {
        gunshot_ak: asset_server.load("imported_audio/gunshot_ak.wav"),
        gunshot_shotgun: asset_server.load("imported_audio/gunshot_shotgun.wav"),
        gunshot_pistol: asset_server.load("imported_audio/gunshot_pistol.wav"),
        explosion: asset_server.load("imported_audio/explosion.wav"),
        reload_out: asset_server.load("imported_audio/reload_out.wav"),
        reload_in: asset_server.load("imported_audio/reload_in.wav"),
        reload_bolt: asset_server.load("imported_audio/reload_bolt.wav"),
        player_hit: asset_server.load("imported_audio/player_hit.wav"),
        footstep_grass: asset_server.load("imported_audio/footstep_grass.wav"),
        footstep_metal: asset_server.load("imported_audio/footstep_metal.wav"),
        footstep_water: asset_server.load("imported_audio/footstep_water.wav"),
        chop_wood: asset_server.load("imported_audio/chop_wood.wav"),
        chop_rock: asset_server.load("imported_audio/chop_rock.wav"),
        keycard_swipe: asset_server.load("imported_audio/keycard_swipe.wav"),
        fuse_switch: asset_server.load("imported_audio/fuse_switch.wav"),
        map_pin: asset_server.load("imported_audio/ui_map_biaoji.wav"),
        map_unpin: asset_server.load("imported_audio/ui_map_quxiao.wav"),
    };
    commands.insert_resource(audio_assets);

    let map_textures = MapTextures {
        overview: asset_server.load("imported_map/textures/map_overview.png"),
        marker: asset_server.load("imported_map/ui/Newbattle_badge_map_mark.png"),
        cargo_ship: asset_server.load("imported_map/ui/NewBattlePanel2_huolun.png"),
        vehicle: asset_server.load("imported_map/ui/BattlePane_zaiju_che.png"),
        loading_bg: asset_server.load("imported_map/ui/Loading_bg_1.png"),
        loading_bar_bg: asset_server.load("imported_map/ui/BattleLoadingPanel_loading1.png"),
        loading_bar_fill: asset_server.load("imported_map/ui/BattleLoadingPanel_loading2.png"),
    };
    commands.insert_resource(map_textures);

    commands.insert_resource(tool_assets.clone());

    commands.spawn((
        BuildingGhostVisual,
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(ghost_valid),
        Transform::default(),
        Visibility::Hidden,
    ));

    // Far Atmospheric Sky Dome (radius 4800m, far beyond island boundary)
    let sky_dome_mesh = meshes.add(build_sky_dome_mesh(4800.0));
    commands.spawn((
        SkyDomeVisual,
        Mesh3d(sky_dome_mesh),
        MeshMaterial3d(sky_zenith_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Layer 1: Flat Horizontal Cumulus Cloud Canopy at Y = 1350m (Planar Deck - NOT a sphere!)
    let cloud_canopy_mesh = meshes.add(build_cloud_canopy_mesh(4600.0, 1350.0));
    commands.spawn((
        CloudCanopyVisual,
        Mesh3d(cloud_canopy_mesh),
        MeshMaterial3d(cloud_canopy_material),
        Transform::from_xyz(0.0, 1350.0, 0.0),
    ));

    // Layer 2: High-Altitude Wispy Cirrus Streaks at Y = 1750m (Parallax Depth)
    let cirrus_canopy_mesh = meshes.add(build_cloud_canopy_mesh(4900.0, 1750.0));
    commands.spawn((
        CirrusCanopyVisual,
        Mesh3d(cirrus_canopy_mesh),
        MeshMaterial3d(cirrus_canopy_material),
        Transform::from_xyz(0.0, 1750.0, 0.0),
    ));

    let sun_sphere = meshes.add(Sphere::new(36.0).mesh().uv(32, 18));
    let corona_sphere = meshes.add(Sphere::new(88.0).mesh().uv(32, 18));
    commands
        .spawn((
            SunDiscVisual,
            Mesh3d(sun_sphere),
            MeshMaterial3d(sun_disc_material),
            Transform::from_xyz(320.0, 560.0, -360.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(corona_sphere),
                MeshMaterial3d(sun_corona_material),
                Transform::default(),
            ));
        });

    if terrain_records.is_empty() {
        // Fallback 4 Biomes visual surface pads (used only when imported XAPK terrain is absent)
        for (center, half_extents, biome) in [
            ([0.0, 0.015, -20.0], [29.6, 0.015, 10.0], Biome::Winter),
            ([-10.0, 0.015, 0.0], [19.6, 0.015, 10.0], Biome::Temperate),
            ([20.0, 0.018, 0.0], [9.6, 0.018, 10.0], Biome::Jungle),
            ([0.0, 0.015, 20.0], [29.6, 0.015, 10.0], Biome::Desert),
        ] {
            let [r, g, b] = biome.color();
            commands.spawn((
                Mesh3d(unit_cube.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(r, g, b),
                    perceptual_roughness: 0.95,
                    ..default()
                })),
                Transform::from_translation(Vec3::from_array(center))
                    .with_scale(Vec3::from_array(half_extents)),
            ));
        }
        let road_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.22, 0.24),
            perceptual_roughness: 0.92,
            ..default()
        });
        for (center, half_extents) in [
            ([0.0, 0.036, 0.0], [1.6, 0.004, 28.5]),
            ([0.0, 0.036, 2.0], [28.5, 0.004, 1.6]),
        ] {
            commands.spawn((
                Mesh3d(unit_cube.clone()),
                MeshMaterial3d(road_mat.clone()),
                Transform::from_translation(Vec3::from_array(center))
                    .with_scale(Vec3::from_array(half_extents)),
            ));
        }
    }

    // Rich Jungle Biome Environment (x > 10.0, z in -10.0..10.0) & Phase 4 Radtown Monument ([-22.0, 0.0, -10.5])
    spawn_jungle_and_radtown_scenery(
        &mut commands,
        &mut meshes,
        &mut materials,
        &unit_cube,
        &power_off_material,
        &tool_assets,
        !terrain_records.is_empty(),
    );

    commands
        .spawn((
            MainCamera,
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 80.0_f32.to_radians(),
                far: 3500.0,
                ..default()
            }),
            DistanceFog {
                color: Color::srgba(0.72, 0.84, 0.98, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.00018 },
                ..default()
            },
            Tonemapping::TonyMcMapface,
            Msaa::Sample4,
            Transform::from_xyz(0.0, 2.7, 12.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                HeldToolVisual,
                Mesh3d(rock_mesh),
                MeshMaterial3d(rock_mat),
                Transform::from_xyz(0.32, -0.26, -0.48),
            ));
        });
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
    let cascade_shadow_config: CascadeShadowConfig = CascadeShadowConfigBuilder {
        num_cascades: 4,
        minimum_distance: 0.2,
        maximum_distance: 400.0,
        first_cascade_far_bound: 12.0,
        overlap_proportion: 0.20,
    }
    .into();

    commands.spawn((
        SunLight,
        DirectionalLight {
            illuminance: 38_000.0,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.60,
            ..default()
        },
        cascade_shadow_config,
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_jungle_and_radtown_scenery(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    unit_cube: &Handle<Mesh>,
    power_off_material: &Handle<StandardMaterial>,
    tool_assets: &ToolAssets,
    has_imported_terrain: bool,
) {
    if has_imported_terrain {
        return;
    }

    let cylinder_mesh = meshes.add(Cylinder::new(1.0, 1.0).mesh().resolution(20));
    let sphere_mesh = meshes.add(Sphere::new(1.0).mesh().uv(24, 16));
    let cone_mesh = meshes.add(Cone::new(1.0, 1.0).mesh().resolution(20));

    // --- JUNGLE BIOME (East sector: x in 10.5..29.5, z in -9.5..9.5) ---
    let lagoon_water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.08, 0.62, 0.56, 0.88),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.12,
        metallic: 0.15,
        ..default()
    });
    let lagoon_bank_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.32, 0.16),
        perceptual_roughness: 0.95,
        ..default()
    });
    for (lx, lz, rx, rz) in [(17.5, -2.5, 3.4, 2.6), (23.2, 4.0, 3.0, 2.4)] {
        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),
            MeshMaterial3d(lagoon_bank_mat.clone()),
            Transform::from_xyz(lx, 0.036, lz).with_scale(Vec3::new(rx + 0.45, 0.02, rz + 0.45)),
        ));
        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),
            MeshMaterial3d(lagoon_water_mat.clone()),
            Transform::from_xyz(lx, 0.048, lz).with_scale(Vec3::new(rx, 0.02, rz)),
        ));
    }

    let jungle_trunk_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.22, 0.13),
        perceptual_roughness: 0.94,
        ..default()
    });
    let jungle_canopy_deep = materials.add(StandardMaterial {
        base_color: Color::srgb(0.07, 0.46, 0.18),
        perceptual_roughness: 0.86,
        ..default()
    });
    let jungle_canopy_bright = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.58, 0.24),
        perceptual_roughness: 0.84,
        ..default()
    });
    let vine_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.64, 0.26),
        perceptual_roughness: 0.9,
        ..default()
    });

    // Giant perimeter & canopy Jungle trees with wide umbrella domes and hanging lianas/vines
    for (tx, tz, height) in [
        (12.2, -8.4, 7.2),
        (18.8, -8.2, 7.8),
        (27.4, -8.0, 8.2),
        (28.0, -1.2, 7.6),
        (12.4, 4.4, 6.8),
        (21.0, 8.2, 7.5),
        (27.6, 7.8, 8.0),
        (16.2, 0.8, 6.9),
    ] {
        commands
            .spawn((Transform::from_xyz(tx, 0.0, tz), Visibility::Inherited))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(cylinder_mesh.clone()),
                    MeshMaterial3d(jungle_trunk_mat.clone()),
                    Transform::from_xyz(0.0, height * 0.5, 0.0)
                        .with_scale(Vec3::new(0.34, height, 0.34)),
                ));
                // Wide tropical umbrella canopy lobes
                parent.spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(jungle_canopy_deep.clone()),
                    Transform::from_xyz(0.0, height, 0.0).with_scale(Vec3::new(3.2, 1.35, 3.2)),
                ));
                parent.spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(jungle_canopy_bright.clone()),
                    Transform::from_xyz(1.1, height + 0.65, 0.6)
                        .with_scale(Vec3::new(2.3, 1.1, 2.3)),
                ));
                parent.spawn((
                    Mesh3d(sphere_mesh.clone()),
                    MeshMaterial3d(jungle_canopy_bright.clone()),
                    Transform::from_xyz(-1.2, height + 0.45, -0.7)
                        .with_scale(Vec3::new(2.4, 1.1, 2.4)),
                ));
                // Hanging jungle lianas / vines
                for (vx, vz, vlen) in [(-1.6, 0.8, 2.8), (1.5, -0.9, 3.1), (0.6, 1.7, 2.5)] {
                    parent.spawn((
                        Mesh3d(cylinder_mesh.clone()),
                        MeshMaterial3d(vine_mat.clone()),
                        Transform::from_xyz(vx, height - vlen * 0.5, vz)
                            .with_scale(Vec3::new(0.045, vlen, 0.045)),
                    ));
                }
            });
    }

    // Lush Jungle ferns & undergrowth bushes
    for (fx, fz, fs) in [
        (13.5, -4.5, 0.85),
        (15.8, -5.2, 0.95),
        (20.5, -4.8, 1.05),
        (24.8, -3.5, 0.90),
        (14.0, -0.8, 0.80),
        (21.2, 0.8, 1.00),
        (25.8, 1.8, 0.95),
        (18.2, 3.5, 0.88),
        (16.8, 6.8, 1.05),
        (24.2, 6.8, 0.92),
        (12.0, -2.5, 0.85),
        (26.5, -4.2, 1.10),
    ] {
        commands.spawn((
            Mesh3d(cone_mesh.clone()),
            MeshMaterial3d(jungle_canopy_bright.clone()),
            Transform::from_xyz(fx, 0.38 * fs, fz).with_scale(Vec3::new(
                0.85 * fs,
                0.75 * fs,
                0.85 * fs,
            )),
        ));
    }

    // --- PHASE 4: RADTOWN MONUMENT (RT) around [-22.0, 0.0, -10.5] ---
    let concrete_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.38, 0.41, 0.43),
        perceptual_roughness: 0.92,
        ..default()
    });
    let rust_metal_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.46, 0.28, 0.20),
        metallic: 0.65,
        perceptual_roughness: 0.68,
        ..default()
    });
    let hazard_yellow_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.72, 0.16),
        perceptual_roughness: 0.65,
        ..default()
    });
    let rad_slime_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.46, 0.94, 0.18),
        emissive: LinearRgba::rgb(0.8, 2.2, 0.2),
        unlit: true,
        ..default()
    });
    let mil_green_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.38, 0.24),
        perceptual_roughness: 0.75,
        ..default()
    });

    // Radtown Bunker walls & roof slab around [-23.2, 0.0, -11.5]
    for (cx, cy, cz, sx, sy, sz) in [
        (-23.2, 1.6, -13.4, 2.4, 1.6, 0.22),  // North wall
        (-25.4, 1.6, -11.5, 0.22, 1.6, 2.0),  // West wall
        (-21.0, 1.6, -12.3, 0.22, 1.6, 1.1),  // East wall back
        (-23.2, 3.25, -11.5, 2.5, 0.18, 2.1), // Bunker roof slab
    ] {
        commands.spawn((
            Mesh3d(unit_cube.clone()),
            MeshMaterial3d(concrete_mat.clone()),
            Transform::from_xyz(cx, cy, cz).with_scale(Vec3::new(sx, sy, sz)),
        ));
    }

    // Armored Bunker Door (slides open when green keycard reader is activated)
    commands.spawn((
        MonumentBunkerDoor,
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(rust_metal_mat.clone()),
        Transform::from_xyz(-22.2, 1.5, -9.7).with_scale(Vec3::new(1.1, 1.5, 0.14)),
    ));

    // Supply Crate (outside radiation core at RADTOWN_SUPPLY_CRATE_POS)
    if let Some((m_h, mat_h)) = tool_assets.imported_models.get("supply_crate") {
        commands.spawn((
            Mesh3d(m_h.clone()),
            MeshMaterial3d(mat_h.clone()),
            Transform::from_xyz(
                RADTOWN_SUPPLY_CRATE_POS[0],
                0.0,
                RADTOWN_SUPPLY_CRATE_POS[2],
            ),
        ));
    } else {
        commands.spawn((
            Mesh3d(unit_cube.clone()),
            MeshMaterial3d(hazard_yellow_mat.clone()),
            Transform::from_translation(Vec3::from_array(RADTOWN_SUPPLY_CRATE_POS))
                .with_scale(Vec3::new(0.55, 0.45, 0.55)),
        ));
    }

    // XAPK Monument Gas Station building alongside Radtown
    if let Some((m_h, mat_h)) = tool_assets.imported_models.get("monument_gas_station") {
        commands.spawn((
            Mesh3d(m_h.clone()),
            MeshMaterial3d(mat_h.clone()),
            Transform::from_xyz(-22.0, 0.0, -21.5),
        ));
    }

    // Industrial Recycler (at RADTOWN_RECYCLER_POS)
    commands.spawn((
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(hazard_yellow_mat.clone()),
        Transform::from_translation(Vec3::from_array(RADTOWN_RECYCLER_POS))
            .with_scale(Vec3::new(0.65, 0.65, 0.55)),
    ));
    commands.spawn((
        Mesh3d(cylinder_mesh.clone()),
        MeshMaterial3d(rust_metal_mat.clone()),
        Transform::from_xyz(
            RADTOWN_RECYCLER_POS[0],
            RADTOWN_RECYCLER_POS[1] + 0.55,
            RADTOWN_RECYCLER_POS[2],
        )
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(0.32, 1.1, 0.32)),
    ));

    // Electric Fuse Box & Generator Switch (at RADTOWN_FUSEBOX_POS) + Power Beacon
    commands.spawn((
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(rust_metal_mat.clone()),
        Transform::from_translation(Vec3::from_array(RADTOWN_FUSEBOX_POS))
            .with_scale(Vec3::new(0.32, 0.48, 0.20)),
    ));
    commands.spawn((
        MonumentPowerBeacon,
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(power_off_material.clone()),
        Transform::from_xyz(
            RADTOWN_FUSEBOX_POS[0],
            RADTOWN_FUSEBOX_POS[1] + 0.62,
            RADTOWN_FUSEBOX_POS[2],
        )
        .with_scale(Vec3::splat(0.16)),
    ));

    // Green Keycard Reader (at RADTOWN_READER_POS)
    commands.spawn((
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(mil_green_mat.clone()),
        Transform::from_translation(Vec3::from_array(RADTOWN_READER_POS))
            .with_scale(Vec3::new(0.28, 0.42, 0.18)),
    ));

    // Military Crate inside Bunker (at RADTOWN_MILITARY_CRATE_POS)
    if let Some((m_h, mat_h)) = tool_assets.imported_models.get("military_crate") {
        commands.spawn((
            Mesh3d(m_h.clone()),
            MeshMaterial3d(mat_h.clone()),
            Transform::from_xyz(
                RADTOWN_MILITARY_CRATE_POS[0],
                0.0,
                RADTOWN_MILITARY_CRATE_POS[2],
            ),
        ));
    } else {
        commands.spawn((
            Mesh3d(unit_cube.clone()),
            MeshMaterial3d(mil_green_mat),
            Transform::from_translation(Vec3::from_array(RADTOWN_MILITARY_CRATE_POS))
                .with_scale(Vec3::new(0.65, 0.42, 0.48)),
        ));
    }

    // Phase 5: Nuclear Silo Launch Terminal inside Bunker (at RADTOWN_NUCLEAR_BUTTON_POS)
    commands.spawn((
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(concrete_mat),
        Transform::from_xyz(
            RADTOWN_NUCLEAR_BUTTON_POS[0],
            0.58,
            RADTOWN_NUCLEAR_BUTTON_POS[2],
        )
        .with_scale(Vec3::new(0.36, 0.58, 0.28)),
    ));
    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(power_off_material.clone()),
        Transform::from_translation(Vec3::from_array(RADTOWN_NUCLEAR_BUTTON_POS))
            .with_scale(Vec3::splat(0.18)),
    ));

    // Phase 5: Cargo Ship patrolling the outer sea perimeter
    let (ship_pos, ship_yaw) = cargo_ship_transform(ServerTick(0));
    let imported_ship = tool_assets.imported_models.get("cargo_ship").cloned();
    commands
        .spawn((
            CargoShipVisual,
            Transform::from_translation(Vec3::from_array(ship_pos))
                .with_rotation(Quat::from_rotation_y(-ship_yaw)),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            if let Some((m_h, mat_h)) = imported_ship {
                parent.spawn((
                    Mesh3d(m_h),
                    MeshMaterial3d(mat_h),
                    Transform::from_xyz(0.0, 0.2, 0.0),
                ));
            } else {
                parent.spawn((
                    Mesh3d(unit_cube.clone()),
                    MeshMaterial3d(rust_metal_mat.clone()),
                    Transform::from_xyz(0.0, 1.4, 0.0).with_scale(Vec3::new(2.2, 1.4, 6.2)),
                ));
                parent.spawn((
                    Mesh3d(unit_cube.clone()),
                    MeshMaterial3d(hazard_yellow_mat.clone()),
                    Transform::from_xyz(0.0, 3.6, -2.6).with_scale(Vec3::new(1.6, 1.2, 1.4)),
                ));
            }
        });

    // Phase 5: Nuclear Mushroom Cloud & Fallout Crate at NUCLEAR_STRIKE_CENTER / FALLOUT_CRATE_POS
    let nuke_fire_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.42, 0.12, 0.92),
        emissive: LinearRgba::rgb(4.5, 1.4, 0.25),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    let nuke_smoke_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.32, 0.18, 0.14, 0.88),
        emissive: LinearRgba::rgb(1.4, 0.35, 0.08),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands
        .spawn((
            NuclearMushroomVisual,
            Transform::from_xyz(NUCLEAR_STRIKE_CENTER[0], 0.0, NUCLEAR_STRIKE_CENTER[2]),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            // Mushroom stem column
            parent.spawn((
                Mesh3d(cylinder_mesh.clone()),
                MeshMaterial3d(nuke_fire_mat.clone()),
                Transform::from_xyz(0.0, 14.0, 0.0).with_scale(Vec3::new(3.2, 28.0, 3.2)),
            ));
            // Fiery shockwave ring mid-column
            parent.spawn((
                Mesh3d(cylinder_mesh.clone()),
                MeshMaterial3d(nuke_fire_mat.clone()),
                Transform::from_xyz(0.0, 16.0, 0.0).with_scale(Vec3::new(9.5, 1.4, 9.5)),
            ));
            // Giant mushroom cap dome
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(nuke_smoke_mat),
                Transform::from_xyz(0.0, 28.0, 0.0).with_scale(Vec3::new(13.5, 7.5, 13.5)),
            ));
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(nuke_fire_mat),
                Transform::from_xyz(0.0, 25.0, 0.0).with_scale(Vec3::new(10.5, 4.5, 10.5)),
            ));
        });

    commands.spawn((
        FalloutCrateVisual,
        Mesh3d(unit_cube.clone()),
        MeshMaterial3d(rad_slime_mat.clone()),
        Transform::from_translation(Vec3::from_array(FALLOUT_CRATE_POS))
            .with_scale(Vec3::new(0.65, 0.50, 0.65)),
        Visibility::Hidden,
    ));

    // Radioactive hazard barrels & glowing green puddles around RADTOWN_CENTER
    let imported_barrel = tool_assets.imported_models.get("hazard_barrel").cloned();
    for (bx, bz) in [
        (RADTOWN_CENTER[0] + 2.4, RADTOWN_CENTER[2] + 1.8),
        (RADTOWN_CENTER[0] - 2.2, RADTOWN_CENTER[2] + 2.2),
    ] {
        if let Some((m_h, mat_h)) = &imported_barrel {
            commands.spawn((
                Mesh3d(m_h.clone()),
                MeshMaterial3d(mat_h.clone()),
                Transform::from_xyz(bx, 0.0, bz),
            ));
        } else {
            commands.spawn((
                Mesh3d(cylinder_mesh.clone()),
                MeshMaterial3d(hazard_yellow_mat.clone()),
                Transform::from_xyz(bx, 0.55, bz).with_scale(Vec3::new(0.38, 1.1, 0.38)),
            ));
        }
        commands.spawn((
            Mesh3d(cylinder_mesh.clone()),
            MeshMaterial3d(rad_slime_mat.clone()),
            Transform::from_xyz(bx + 0.35, 0.04, bz + 0.25).with_scale(Vec3::new(0.95, 0.02, 0.85)),
        ));
    }
}

fn attach_geometry(
    mut commands: Commands,
    tool_assets: Res<ToolAssets>,
    geometry: Query<(Entity, &LevelGeometry), Added<LevelGeometry>>,
    resource_nodes: Query<(Entity, &ResourceNodeMarker), Added<ResourceNodeMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !shared::level::load_imported_terrain_records().is_empty() {
        for (entity, _) in &geometry {
            commands.entity(entity).despawn();
        }
        for (entity, _) in &resource_nodes {
            commands.entity(entity).despawn();
        }
        return;
    }

    for (entity, LevelGeometry(shape)) in &geometry {
        let [r, g, b] = shape.color;
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::from_size(
                Vec3::from_array(shape.half_extents) * 2.0,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(r, g, b),
                perceptual_roughness: 0.95,
                ..default()
            })),
        ));
    }
    let trunk_cyl = meshes.add(Cylinder::new(0.28, 2.5).mesh().resolution(18));
    let cone_low = meshes.add(Cone::new(1.35, 1.9).mesh().resolution(20));
    let cone_mid = meshes.add(Cone::new(1.05, 1.6).mesh().resolution(20));
    let cone_top = meshes.add(Cone::new(0.72, 1.3).mesh().resolution(20));
    let sphere_canopy = meshes.add(Sphere::new(1.0).mesh().uv(24, 16));
    let rock_sphere = meshes.add(Sphere::new(0.85).mesh().uv(20, 14));

    for (entity, ResourceNodeMarker(node)) in &resource_nodes {
        let biome = Biome::at(node.position);
        let imported_key = match node.kind {
            ResourceKind::Tree => {
                if biome == Biome::Jungle {
                    "tree_jungle"
                } else {
                    "tree_temperate"
                }
            }
            ResourceKind::Rock => "rock_node",
        };
        let imported = tool_assets.imported_models.get(imported_key).cloned();
        commands
            .entity(entity)
            .insert(Visibility::Inherited)
            .with_children(|parent| {
                if let Some((m_h, mat_h)) = imported {
                    parent.spawn((Mesh3d(m_h), MeshMaterial3d(mat_h), Transform::default()));
                    return;
                }
                match node.kind {
                    ResourceKind::Tree => {
                        parent.spawn((
                            Mesh3d(trunk_cyl.clone()),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: if biome == Biome::Jungle {
                                    Color::srgb(0.35, 0.21, 0.12)
                                } else {
                                    Color::srgb(0.42, 0.28, 0.18)
                                },
                                perceptual_roughness: 0.94,
                                ..default()
                            })),
                            Transform::from_xyz(0.0, 1.25, 0.0),
                        ));
                        if biome == Biome::Jungle {
                            // Tropical Jungle tree: wide emerald umbrella canopy + hanging vines
                            let canopy_mat = materials.add(StandardMaterial {
                                base_color: Color::srgb(0.10, 0.54, 0.22),
                                perceptual_roughness: 0.85,
                                ..default()
                            });
                            parent.spawn((
                                Mesh3d(sphere_canopy.clone()),
                                MeshMaterial3d(canopy_mat.clone()),
                                Transform::from_xyz(0.0, 3.1, 0.0)
                                    .with_scale(Vec3::new(1.95, 0.95, 1.95)),
                            ));
                            parent.spawn((
                                Mesh3d(sphere_canopy.clone()),
                                MeshMaterial3d(canopy_mat.clone()),
                                Transform::from_xyz(0.55, 3.65, 0.35)
                                    .with_scale(Vec3::new(1.35, 0.80, 1.35)),
                            ));
                            for (vx, vz) in [(-1.1, 0.4), (1.0, -0.5)] {
                                parent.spawn((
                                    Mesh3d(trunk_cyl.clone()),
                                    MeshMaterial3d(canopy_mat.clone()),
                                    Transform::from_xyz(vx, 2.1, vz)
                                        .with_scale(Vec3::new(0.14, 0.65, 0.14)),
                                ));
                            }
                        } else {
                            let [r1, g1, b1] = match biome {
                                Biome::Winter => [0.58, 0.74, 0.78],
                                Biome::Desert => [0.36, 0.50, 0.24],
                                _ => [0.18, 0.44, 0.24],
                            };
                            parent.spawn((
                                Mesh3d(cone_low.clone()),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::srgb(r1, g1, b1),
                                    perceptual_roughness: 0.88,
                                    ..default()
                                })),
                                Transform::from_xyz(0.0, 2.35, 0.0),
                            ));
                            parent.spawn((
                                Mesh3d(cone_mid.clone()),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::srgb(
                                        (r1 * 1.08).min(1.0),
                                        (g1 * 1.08).min(1.0),
                                        (b1 * 1.08).min(1.0),
                                    ),
                                    perceptual_roughness: 0.88,
                                    ..default()
                                })),
                                Transform::from_xyz(0.0, 3.25, 0.0),
                            ));
                            parent.spawn((
                                Mesh3d(cone_top.clone()),
                                MeshMaterial3d(materials.add(StandardMaterial {
                                    base_color: Color::srgb(
                                        (r1 * 1.15).min(1.0),
                                        (g1 * 1.15).min(1.0),
                                        (b1 * 1.15).min(1.0),
                                    ),
                                    perceptual_roughness: 0.88,
                                    ..default()
                                })),
                                Transform::from_xyz(0.0, 4.05, 0.0),
                            ));
                        }
                    }
                    ResourceKind::Rock => {
                        parent.spawn((
                            Mesh3d(rock_sphere.clone()),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb(0.48, 0.53, 0.56),
                                perceptual_roughness: 0.92,
                                ..default()
                            })),
                            Transform::from_xyz(0.0, 0.55, 0.0)
                                .with_scale(Vec3::new(1.15, 0.82, 1.05)),
                        ));
                        parent.spawn((
                            Mesh3d(rock_sphere.clone()),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: Color::srgb(0.40, 0.46, 0.50),
                                perceptual_roughness: 0.92,
                                ..default()
                            })),
                            Transform::from_xyz(0.42, 0.38, 0.28)
                                .with_scale(Vec3::new(0.75, 0.62, 0.72)),
                        ));
                    }
                }
            });
    }
}

fn register_adapter(adapter: Option<Res<RenderAdapterInfo>>, mut telemetry: ResMut<Telemetry>) {
    if !telemetry.gpu.is_empty() {
        return;
    }
    if let Some(adapter) = adapter {
        telemetry.gpu = adapter.name.clone();
        telemetry.backend = format!("{:?}", adapter.backend);
        info!(
            "Renderer: {} | GPU: {} | {:?}",
            telemetry.backend, telemetry.gpu, adapter.device_type
        );
    }
}

pub(crate) fn compute_looked_object(
    player_pos: [f32; 3],
    yaw: f32,
    pitch: f32,
    physics: &SimulationWorld,
    survival: &SimulationSurvival,
    scenic_query: Option<&Query<(Entity, &ScenicPlacementMarker)>>,
) -> Option<LookedObjectInfo> {
    let eye = [player_pos[0], player_pos[1] + 0.6, player_pos[2]];
    let dir = shared::survival::look_direction(yaw, pitch);
    let dir_vec = Vec3::from_array(dir);

    let mut closest_dist = 250.0;
    let mut candidate: Option<LookedObjectInfo> = None;

    // 1. Check Resource Nodes (trees, rocks)
    for node in &survival.0.nodes {
        let to_node = Vec3::from_array(node.position) - Vec3::from_array(eye);
        let t = to_node.dot(dir_vec);
        if t > 0.4 && t < closest_dist {
            let perp_sq = (to_node - dir_vec * t).length_squared();
            let threshold = match node.kind {
                ResourceKind::Tree => 2.4 * 2.4,
                ResourceKind::Rock => 2.2 * 2.2,
            };
            if perp_sq < threshold {
                closest_dist = t;
                let (icon, label_ua) = match node.kind {
                    ResourceKind::Tree => ("🌲", "Дерево (Pine Tree)"),
                    ResourceKind::Rock => ("🪨", "Кам'яна жила (Stone Ore)"),
                };
                candidate = Some(LookedObjectInfo {
                    title: format!("{} {}", icon, label_ua),
                    detail: format!(
                        "Відстань: {:.1}м · Міцність: {}/{} [{}]",
                        t,
                        node.health,
                        node.max_health,
                        node.kind.item().label_en()
                    ),
                    distance: t,
                    hit_coords: node.position,
                    interact_prompt: Some("[ЛКМ / E] Збір ресурсів".to_string()),
                    target_kind: Some(LookedTargetKind::ResourceNode(node.id)),
                });
            }
        }
    }

    // 3. Check Placed Structures
    for st in &survival.0.structures {
        let to_st = Vec3::from_array(st.position) - Vec3::from_array(eye);
        let t = to_st.dot(dir_vec);
        if t > 0.4 && t < closest_dist {
            let perp_sq = (to_st - dir_vec * t).length_squared();
            if perp_sq < 2.2 * 2.2 {
                closest_dist = t;
                candidate = Some(LookedObjectInfo {
                    title: format!("🧱 {} [{}]", st.kind.label(), st.tier.label()),
                    detail: format!(
                        "Відстань: {:.1}м · Міцність: {:.0}/{:.0} HP",
                        t,
                        st.health,
                        st.tier.max_health()
                    ),
                    distance: t,
                    hit_coords: st.position,
                    interact_prompt: Some("[R] Повернути / [Киянка] Покращити".to_string()),
                    target_kind: Some(LookedTargetKind::Structure(st.id)),
                });
            }
        }
    }

    // 4. Check Vehicles
    for v in &survival.0.vehicles {
        let to_v = Vec3::from_array(v.position) - Vec3::from_array(eye);
        let t = to_v.dot(dir_vec);
        if t > 0.5 && t < closest_dist {
            let perp_sq = (to_v - dir_vec * t).length_squared();
            if perp_sq < 2.8 * 2.8 {
                closest_dist = t;
                candidate = Some(LookedObjectInfo {
                    title: format!("🚙 {}", v.kind.label()),
                    detail: format!(
                        "Відстань: {:.1}м · Паливо: {}L · Міцність: {}/{}",
                        t, v.fuel, v.health, v.max_health
                    ),
                    distance: t,
                    hit_coords: v.position,
                    interact_prompt: Some("[F] Сісти за кермо (Drive)".to_string()),
                    target_kind: Some(LookedTargetKind::Vehicle(v.id)),
                });
            }
        }
    }

    // 5. Check NPCs / Enemies
    for npc in &survival.0.npcs {
        if npc.health == 0 {
            continue;
        }
        let to_npc = Vec3::from_array(npc.position) - Vec3::from_array(eye);
        let t = to_npc.dot(dir_vec);
        if t > 0.5 && t < closest_dist {
            let perp_sq = (to_npc - dir_vec * t).length_squared();
            if perp_sq < 2.2 * 2.2 {
                closest_dist = t;
                candidate = Some(LookedObjectInfo {
                    title: format!("👾 {}", npc.kind.label()),
                    detail: format!(
                        "Ворог (Threat) · Відстань: {:.1}м · HP: {}/{}",
                        t, npc.health, npc.max_health
                    ),
                    distance: t,
                    hit_coords: npc.position,
                    interact_prompt: Some("[ЛКМ] Атакувати ціль".to_string()),
                    target_kind: Some(LookedTargetKind::Npc(npc.id)),
                });
            }
        }
    }

    // 6. Check Scenic Placements (imported trees, rocks, decorations)
    if let Some(scenic) = scenic_query {
        for (entity, marker) in scenic.iter() {
            let to_obj = marker.position - Vec3::from_array(eye);
            let t = to_obj.dot(dir_vec);
            if t > 0.4 && t < closest_dist {
                let perp_sq = (to_obj - dir_vec * t).length_squared();
                let radius = (marker.scale * 2.2).clamp(1.2, 5.5);
                if perp_sq < radius * radius {
                    closest_dist = t;
                    candidate = Some(LookedObjectInfo {
                        title: "🪨/🌲 Об'єкт сцени (Scenic Element)".to_string(),
                        detail: format!(
                            "Декорація карти · Відстань: {:.1}м · [X: {:.1}, Y: {:.1}, Z: {:.1}]",
                            t, marker.position.x, marker.position.y, marker.position.z
                        ),
                        distance: t,
                        hit_coords: marker.position.to_array(),
                        interact_prompt: Some("Декоративний об'єкт сцени".to_string()),
                        target_kind: Some(LookedTargetKind::Scenic(entity)),
                    });
                }
            }
        }
    }

    // 6. If no entity intercepted the ray, check terrain raycast
    if candidate.is_none()
        && let Some((hit_pos, _norm, dist)) = physics.0.cast_ray(eye, dir, 350.0, Some(PlayerId(0)))
    {
        let sector = shared::level::sector_label_at(hit_pos);
        let biome = if hit_pos[1] < 0.5 {
            "🌊 Прибережна зона / Океан (Coast Water)"
        } else if hit_pos[1] > 40.0 {
            "⛰️ Гірський хребет (Highland Ridge)"
        } else {
            "🌲 Рівнина / Хвойний ліс (Pine Forest)"
        };
        candidate = Some(LookedObjectInfo {
            title: format!("📍 Сектор {} ({})", sector, biome),
            detail: format!(
                "Відстань: {:.1}м · [X: {:.1}, Y: {:.1}, Z: {:.1}]",
                dist, hit_pos[0], hit_pos[1], hit_pos[2]
            ),
            distance: dist,
            hit_coords: hit_pos,
            interact_prompt: None,
            target_kind: Some(LookedTargetKind::Terrain(hit_pos)),
        });
    }

    candidate
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn follow_camera(
    player: Query<(&SimulationPose, &CharacterMotion), With<LocalPlayer>>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
    mut player_body: Query<
        (&mut Transform, &mut Visibility),
        (With<PlayerBodyVisual>, Without<MainCamera>),
    >,
    time: Res<Time<Fixed>>,
    input: Res<SimulationInput>,
    input_keys: Res<ButtonInput<KeyCode>>,
    mut perspective: ResMut<CameraPerspective>,
    options: Res<RuntimeOptions>,
    online: Res<Online>,
    mut telemetry: ResMut<Telemetry>,
    physics: Res<SimulationWorld>,
    survival: Res<SimulationSurvival>,
    scenic_query: Query<(Entity, &ScenicPlacementMarker)>,
) {
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };
    if input_keys.just_pressed(KeyCode::KeyV) || input_keys.just_pressed(KeyCode::F5) {
        *perspective = match *perspective {
            CameraPerspective::FirstPerson => CameraPerspective::ThirdPerson,
            CameraPerspective::ThirdPerson => CameraPerspective::FirstPerson,
        };
    }
    if options.0.benchmark {
        camera.translation = Vec3::new(0.0, 1.64, 12.0);
        camera.rotation = Quat::IDENTITY;
        telemetry.yaw = 0.0;
        return;
    }
    let Ok((pose, motion)) = player.single() else {
        return;
    };
    let current_pos = pose.previous.lerp(pose.current, time.overstep_fraction());
    let [yaw, pitch] = preview_look(motion.0.yaw, motion.0.pitch, input.0.pending_look());
    let (cam_pos, cam_rot) = match *perspective {
        CameraPerspective::FirstPerson => (
            current_pos + Vec3::Y * EYE_OFFSET + online.camera_offset,
            Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0),
        ),
        CameraPerspective::ThirdPerson => {
            let rot = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
            let pivot = current_pos + Vec3::new(0.0, 1.50, 0.0) + online.camera_offset;
            let offset = rot * Vec3::new(0.38, 0.12, 2.75);
            (pivot + offset, rot)
        }
    };
    camera.translation = cam_pos;
    camera.rotation = cam_rot;
    telemetry.yaw = yaw;
    telemetry.pitch = pitch;
    telemetry.player_position = pose.current.to_array();
    let velocity = (pose.current - pose.previous) * 60.0;
    telemetry.player_velocity = [velocity.x, motion.0.vertical_velocity, velocity.z];
    telemetry.looked_object = compute_looked_object(
        telemetry.player_position,
        telemetry.yaw,
        telemetry.pitch,
        &physics,
        &survival,
        Some(&scenic_query),
    );

    for (mut body_tf, mut body_vis) in &mut player_body {
        let desired_vis = match *perspective {
            CameraPerspective::FirstPerson => Visibility::Hidden,
            CameraPerspective::ThirdPerson => Visibility::Inherited,
        };
        body_vis.set_if_neq(desired_vis);

        let speed = Vec2::new(velocity.x, velocity.z).length();
        let elapsed = time.elapsed_secs();
        if speed > 0.15 {
            let walk_cycle = elapsed * (if speed > 4.5 { 13.0 } else { 8.5 });
            let stride_bounce = (walk_cycle).sin().abs() * 0.045 * (speed / 5.5).min(1.0);
            let sway_roll = (walk_cycle * 0.5).sin() * 0.045 * (speed / 5.5).min(1.0);
            let forward_lean = (speed / 7.0).clamp(0.0, 1.0) * 0.12;
            body_tf.translation = current_pos + Vec3::new(0.0, stride_bounce, 0.0);
            body_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, -forward_lean, sway_roll);
        } else if !motion.0.grounded {
            let air_pitch = (motion.0.vertical_velocity * 0.04).clamp(-0.18, 0.18);
            body_tf.translation = current_pos;
            body_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, air_pitch, 0.0);
        } else {
            let breath = (elapsed * 1.8).sin() * 0.006;
            body_tf.translation = current_pos + Vec3::new(0.0, breath, 0.0);
            body_tf.rotation = Quat::from_rotation_y(yaw);
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_survival_visuals(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut interface: ResMut<Interface>,
    survival: Res<SimulationSurvival>,
    online: Res<Online>,
    telemetry: Res<Telemetry>,
    perspective: Res<CameraPerspective>,
    tool_assets: Res<ToolAssets>,
    options: Res<RuntimeOptions>,
    mut nodes: Query<
        (&ResourceNodeMarker, &mut Visibility),
        (Without<HeldToolVisual>, Without<BuildingGhostVisual>),
    >,
    existing_structures: Query<(Entity, &StructureMarker)>,
    mut tracers: Query<(Entity, &mut BulletTracerVisual)>,
    mut power_beacon: Query<
        &mut MeshMaterial3d<StandardMaterial>,
        (
            With<MonumentPowerBeacon>,
            Without<BuildingGhostVisual>,
            Without<HeldToolVisual>,
        ),
    >,
    mut bunker_door: Query<
        &mut Transform,
        (
            With<MonumentBunkerDoor>,
            Without<BuildingGhostVisual>,
            Without<HeldToolVisual>,
        ),
    >,
    mut ghost_query: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<BuildingGhostVisual>, Without<HeldToolVisual>),
    >,
    mut held_tool: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (With<HeldToolVisual>, Without<BuildingGhostVisual>),
    >,
) {
    for (marker, mut visibility) in &mut nodes {
        let target = if marker.0.is_available() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != target {
            *visibility = target;
        }
    }

    let mut active_structure_ids = std::collections::BTreeSet::new();
    for (entity, marker) in &existing_structures {
        if let Some(current) = survival.0.structures.iter().find(|s| s.id == marker.0.id)
            && current.tier == marker.0.tier
            && current.kind == marker.0.kind
            && current.rotation == marker.0.rotation
        {
            active_structure_ids.insert(current.id);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for structure in &survival.0.structures {
        if !active_structure_ids.contains(&structure.id) {
            spawn_structure_entity(&mut commands, *structure, &tool_assets);
        }
    }

    // Update Radtown Monument power beacon & armored bunker door position
    if let Ok(mut beacon_mat) = power_beacon.single_mut() {
        let target = if survival.0.monument.is_powered() {
            &tool_assets.power_on_material
        } else {
            &tool_assets.power_off_material
        };
        if beacon_mat.0 != *target {
            beacon_mat.0 = target.clone();
        }
    }
    if let Ok(mut door_tf) = bunker_door.single_mut() {
        let target_x = if survival.0.monument.is_door_open() {
            -20.1
        } else {
            -22.2
        };
        door_tf.translation.x += (target_x - door_tf.translation.x) * 0.18;
    }

    // Spawn & decay 3D ballistic tracer streaks
    if let Some((start, end, duration)) = interface.last_shot_trace.take() {
        let a = Vec3::from_array(start);
        let b = Vec3::from_array(end);
        let diff = b - a;
        let len = diff.length();
        if len > 0.1 {
            let mid = (a + b) * 0.5;
            let rot = Quat::from_rotation_arc(Vec3::Y, diff / len);
            commands.spawn((
                BulletTracerVisual {
                    remaining_secs: duration,
                },
                Mesh3d(tool_assets.tracer_mesh.clone()),
                MeshMaterial3d(tool_assets.tracer_material.clone()),
                Transform::from_translation(mid)
                    .with_rotation(rot)
                    .with_scale(Vec3::new(1.0, len, 1.0)),
            ));
        }
    }
    for (entity, mut tracer) in &mut tracers {
        tracer.remaining_secs -= time.delta_secs();
        if tracer.remaining_secs <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    if interface.swing_timer > 0.0 {
        interface.swing_timer = (interface.swing_timer - time.delta_secs()).max(0.0);
    }
    if interface.recoil_timer > 0.0 {
        interface.recoil_timer = (interface.recoil_timer - time.delta_secs()).max(0.0);
    }

    let player_survival = survival.0.player(PlayerId(0));
    let active = player_survival.and_then(|p| p.inventory.active_item());

    if let Ok((mut ghost_tf, mut ghost_vis, mut ghost_mat)) = ghost_query.single_mut() {
        if !options.0.benchmark
            && interface.mode == UiMode::Playing
            && let Some(player) = player_survival
            && active.is_some_and(|s| s.kind == ItemKind::BuildingPlan)
        {
            let kind = player.selected_structure;
            let (pos, rot) = preview_socket_placement(
                telemetry.player_position,
                telemetry.yaw,
                telemetry.pitch,
                kind,
                player.build_rotation,
                &survival.0.structures,
            );
            let allowed = survival
                .0
                .has_building_privilege(online.local_player_id(), pos)
                && player.inventory.count_item(ItemKind::Wood) >= u32::from(kind.build_cost_wood());
            let rot_quat = Quat::from_rotation_y(-(rot as f32) * std::f32::consts::FRAC_PI_2);
            *ghost_tf = Transform::from_translation(Vec3::from_array(pos))
                .with_rotation(rot_quat)
                .with_scale(Vec3::from_array(kind.half_extents()));
            let target_ghost_mat = if allowed {
                &tool_assets.ghost_valid
            } else {
                &tool_assets.ghost_invalid
            };
            if ghost_mat.0 != *target_ghost_mat {
                ghost_mat.0 = target_ghost_mat.clone();
            }
            *ghost_vis = Visibility::Inherited;
        } else {
            *ghost_vis = Visibility::Hidden;
        }
    }

    let Ok((mut transform, mut visibility, mut mesh, mut material)) = held_tool.single_mut() else {
        return;
    };
    if options.0.benchmark || *perspective == CameraPerspective::ThirdPerson {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(stack) = active else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;
    let (target_mesh, target_mat) = match stack.kind {
        ItemKind::Rock | ItemKind::Stone | ItemKind::MetalFragments | ItemKind::Scrap => {
            &tool_assets.rock
        }
        ItemKind::StoneHatchet | ItemKind::Wood => tool_assets
            .imported_models
            .get("tool_hatchet")
            .unwrap_or(&tool_assets.hatchet),
        ItemKind::StonePickaxe => tool_assets
            .imported_models
            .get("tool_pickaxe")
            .unwrap_or(&tool_assets.pickaxe),
        ItemKind::WoodenSpear | ItemKind::Arrow => &tool_assets.spear,
        ItemKind::HuntingBow => &tool_assets.bow,
        ItemKind::TeleportGun | ItemKind::Revolver | ItemKind::PistolAmmo => tool_assets
            .imported_models
            .get("weapon_pistol")
            .unwrap_or(&tool_assets.revolver),
        ItemKind::AssaultRifle | ItemKind::RifleAmmo => tool_assets
            .imported_models
            .get("weapon_rifle")
            .unwrap_or(&tool_assets.rifle),
        ItemKind::BuildingPlan | ItemKind::GreenKeycard => &tool_assets.blueprint,
        ItemKind::C4Explosive => tool_assets
            .imported_models
            .get("item_c4")
            .unwrap_or(&tool_assets.hammer),
        ItemKind::Hammer | ItemKind::ElectricFuse => tool_assets
            .imported_models
            .get("tool_hammer")
            .unwrap_or(&tool_assets.hammer),
        ItemKind::Bandage
        | ItemKind::WaterBottle
        | ItemKind::Ration
        | ItemKind::AntiRadPills
        | ItemKind::LowGradeFuel => &tool_assets.consumable,
    };
    if mesh.0 != *target_mesh {
        mesh.0 = target_mesh.clone();
    }
    if material.0 != *target_mat {
        material.0 = target_mat.clone();
    }

    let elapsed = time.elapsed_secs();
    let speed = (telemetry.player_velocity[0] * telemetry.player_velocity[0]
        + telemetry.player_velocity[2] * telemetry.player_velocity[2])
        .sqrt();
    let (bob_x, bob_y) = if speed > 0.2 {
        let freq = if speed > 4.5 { 13.0 } else { 8.5 };
        let f = (speed / 5.5).min(1.0);
        (
            (elapsed * freq * 0.5).sin() * 0.012 * f,
            (elapsed * freq).sin().abs() * 0.014 * f,
        )
    } else {
        ((elapsed * 1.5).sin() * 0.003, (elapsed * 3.0).cos() * 0.003)
    };

    let swing_phase = if interface.swing_timer > 0.0 {
        (1.0 - interface.swing_timer / 0.28).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (swing_rot, swing_offset) = if swing_phase > 0.0 {
        if swing_phase < 0.25 {
            let t = swing_phase / 0.25;
            (
                Quat::from_euler(EulerRot::XYZ, t * 0.25, -t * 0.15, -t * 0.12),
                Vec3::new(t * 0.03, t * 0.04, t * 0.06),
            )
        } else if swing_phase < 0.70 {
            let t = (swing_phase - 0.25) / 0.45;
            (
                Quat::from_euler(
                    EulerRot::XYZ,
                    0.25 - t * 1.25,
                    -0.15 + t * 0.60,
                    -0.12 + t * 0.40,
                ),
                Vec3::new(0.03 - t * 0.18, 0.04 - t * 0.16, 0.06 - t * 0.22),
            )
        } else {
            let t = (swing_phase - 0.70) / 0.30;
            let inv = 1.0 - t;
            (
                Quat::from_euler(EulerRot::XYZ, -inv, 0.45 * inv, 0.28 * inv),
                Vec3::new(-0.15 * inv, -0.12 * inv, -0.16 * inv),
            )
        }
    } else {
        (Quat::IDENTITY, Vec3::ZERO)
    };

    let recoil_phase = if interface.recoil_timer > 0.0 {
        (interface.recoil_timer / 0.14).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let recoil_rot = Quat::from_euler(
        EulerRot::XYZ,
        recoil_phase * 0.35,
        -recoil_phase * 0.08,
        recoil_phase * 0.12,
    );
    let recoil_offset = Vec3::new(
        -recoil_phase * 0.02,
        recoil_phase * 0.03,
        recoil_phase * 0.08,
    );

    let base_pos = Vec3::new(0.32, -0.26, -0.48);
    let base_rot = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.08);

    transform.translation = base_pos + Vec3::new(bob_x, bob_y, 0.0) + swing_offset + recoil_offset;
    transform.rotation = base_rot * swing_rot * recoil_rot;
}

fn sync_third_person_tool(
    survival: Res<SimulationSurvival>,
    interface: Res<Interface>,
    tool_assets: Res<ToolAssets>,
    mut tp_tool: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
        ),
        With<ThirdPersonHeldToolVisual>,
    >,
) {
    let Ok((mut mesh, mut mat, mut tf)) = tp_tool.single_mut() else {
        return;
    };
    let player = survival.0.player(PlayerId(0));
    let active = player.and_then(|p| p.inventory.active_item());
    let Some(stack) = active else {
        tf.scale = Vec3::ZERO;
        return;
    };
    tf.scale = Vec3::splat(0.85);
    let (target_mesh, target_mat) = match stack.kind {
        ItemKind::Rock | ItemKind::Stone | ItemKind::MetalFragments | ItemKind::Scrap => {
            &tool_assets.rock
        }
        ItemKind::StoneHatchet | ItemKind::Wood => tool_assets
            .imported_models
            .get("tool_hatchet")
            .unwrap_or(&tool_assets.hatchet),
        ItemKind::StonePickaxe => tool_assets
            .imported_models
            .get("tool_pickaxe")
            .unwrap_or(&tool_assets.pickaxe),
        ItemKind::WoodenSpear | ItemKind::Arrow => &tool_assets.spear,
        ItemKind::HuntingBow => &tool_assets.bow,
        ItemKind::TeleportGun | ItemKind::Revolver | ItemKind::PistolAmmo => tool_assets
            .imported_models
            .get("weapon_pistol")
            .unwrap_or(&tool_assets.revolver),
        ItemKind::AssaultRifle | ItemKind::RifleAmmo => tool_assets
            .imported_models
            .get("weapon_rifle")
            .unwrap_or(&tool_assets.rifle),
        ItemKind::BuildingPlan | ItemKind::GreenKeycard => &tool_assets.blueprint,
        ItemKind::C4Explosive => tool_assets
            .imported_models
            .get("item_c4")
            .unwrap_or(&tool_assets.hammer),
        ItemKind::Hammer | ItemKind::ElectricFuse => tool_assets
            .imported_models
            .get("tool_hammer")
            .unwrap_or(&tool_assets.hammer),
        ItemKind::Bandage
        | ItemKind::WaterBottle
        | ItemKind::Ration
        | ItemKind::AntiRadPills
        | ItemKind::LowGradeFuel => &tool_assets.consumable,
    };
    if mesh.0 != *target_mesh {
        mesh.0 = target_mesh.clone();
    }
    if mat.0 != *target_mat {
        mat.0 = target_mat.clone();
    }
    let swing_phase = if interface.swing_timer > 0.0 {
        (1.0 - interface.swing_timer / 0.28).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let swing_sin = (swing_phase * std::f32::consts::PI).sin();
    tf.translation = Vec3::new(0.28, 0.82 - swing_sin * 0.12, 0.35 + swing_sin * 0.15);
    tf.rotation = Quat::from_euler(
        EulerRot::XYZ,
        -0.2 - swing_sin * 0.85,
        0.1 + swing_sin * 0.3,
        -0.3,
    );
}

fn spawn_structure_entity(
    commands: &mut Commands,
    structure: PlacedStructure,
    tool_assets: &ToolAssets,
) {
    let mat = tool_assets.tier_material(structure.tier);
    let pos = Vec3::from_array(structure.position);
    let rot_quat =
        Quat::from_rotation_y(-(structure.rotation as f32) * std::f32::consts::FRAC_PI_2);
    match structure.kind {
        StructureKind::Doorway => {
            commands
                .spawn((
                    StructureMarker(structure),
                    Transform::from_translation(pos).with_rotation(rot_quat),
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(tool_assets.doorway_pillar.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_xyz(-1.05, 0.0, 0.0),
                    ));
                    parent.spawn((
                        Mesh3d(tool_assets.doorway_pillar.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_xyz(1.05, 0.0, 0.0),
                    ));
                    parent.spawn((
                        Mesh3d(tool_assets.doorway_lintel.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(0.0, 0.95, 0.0),
                    ));
                });
        }
        StructureKind::Campfire => {
            commands
                .spawn((
                    StructureMarker(structure),
                    Mesh3d(tool_assets.unit_cube.clone()),
                    MeshMaterial3d(mat),
                    Transform::from_translation(pos)
                        .with_rotation(rot_quat)
                        .with_scale(Vec3::new(0.42, 0.10, 0.42)),
                    Visibility::Inherited,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Mesh3d(tool_assets.unit_cube.clone()),
                        MeshMaterial3d(tool_assets.campfire_flame.clone()),
                        Transform::from_xyz(0.0, 1.35, 0.0).with_scale(Vec3::new(0.45, 0.85, 0.45)),
                    ));
                });
        }
        _ => {
            commands.spawn((
                StructureMarker(structure),
                Mesh3d(tool_assets.unit_cube.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos)
                    .with_rotation(rot_quat)
                    .with_scale(Vec3::from_array(structure.kind.half_extents())),
                Visibility::Inherited,
            ));
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_phase5_world_visuals(
    mut commands: Commands,
    clock: Res<SimulationClock>,
    survival: Res<SimulationSurvival>,
    tool_assets: Res<ToolAssets>,
    mut existing_npcs: Query<
        (Entity, &NpcVisual, &mut Transform, &mut Visibility),
        (
            Without<VehicleVisual>,
            Without<CargoShipVisual>,
            Without<NuclearMushroomVisual>,
            Without<FalloutCrateVisual>,
        ),
    >,
    mut existing_vehicles: Query<
        (Entity, &VehicleVisual, &mut Transform),
        (
            Without<NpcVisual>,
            Without<CargoShipVisual>,
            Without<NuclearMushroomVisual>,
            Without<FalloutCrateVisual>,
        ),
    >,
    mut cargo_ship: Query<
        &mut Transform,
        (
            With<CargoShipVisual>,
            Without<NpcVisual>,
            Without<VehicleVisual>,
            Without<NuclearMushroomVisual>,
            Without<FalloutCrateVisual>,
        ),
    >,
    mut mushroom: Query<
        &mut Visibility,
        (
            With<NuclearMushroomVisual>,
            Without<NpcVisual>,
            Without<FalloutCrateVisual>,
        ),
    >,
    mut fallout_crate: Query<
        &mut Visibility,
        (
            With<FalloutCrateVisual>,
            Without<NpcVisual>,
            Without<NuclearMushroomVisual>,
        ),
    >,
) {
    let mut active_npc_ids = std::collections::BTreeSet::new();
    for (entity, marker, mut tf, mut vis) in &mut existing_npcs {
        if let Some(npc) = survival.0.npcs.iter().find(|n| n.id == marker.0) {
            active_npc_ids.insert(npc.id);
            if npc.is_alive() {
                *vis = Visibility::Inherited;
                tf.translation = Vec3::from_array(npc.position);
                tf.rotation = Quat::from_rotation_y(-npc.yaw);
            } else {
                *vis = Visibility::Hidden;
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
    for npc in &survival.0.npcs {
        if !active_npc_ids.contains(&npc.id) {
            spawn_npc_entity(&mut commands, *npc, &tool_assets);
        }
    }

    let mut active_vehicle_ids = std::collections::BTreeSet::new();
    for (entity, marker, mut tf) in &mut existing_vehicles {
        if let Some(v) = survival.0.vehicles.iter().find(|v| v.id == marker.0) {
            active_vehicle_ids.insert(v.id);
            tf.translation = Vec3::from_array(v.position);
            tf.rotation = Quat::from_rotation_y(-v.yaw);
        } else {
            commands.entity(entity).despawn();
        }
    }
    for v in &survival.0.vehicles {
        if !active_vehicle_ids.contains(&v.id) {
            spawn_vehicle_entity(&mut commands, *v, &tool_assets);
        }
    }

    if let Ok(mut ship_tf) = cargo_ship.single_mut() {
        let (pos, yaw) = cargo_ship_transform(ServerTick(clock.tick));
        ship_tf.translation = Vec3::from_array(pos);
        ship_tf.rotation = Quat::from_rotation_y(-yaw);
    }

    if let Ok(mut mush_vis) = mushroom.single_mut() {
        let show = matches!(
            survival.0.nuclear.phase,
            NuclearPhase::DetonationShockwave | NuclearPhase::Fallout
        );
        let target = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *mush_vis != target {
            *mush_vis = target;
        }
    }

    if let Ok(mut crate_vis) = fallout_crate.single_mut() {
        let show = survival.0.nuclear.phase == NuclearPhase::Fallout
            && !survival.0.nuclear.fallout_crate_looted;
        let target = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *crate_vis != target {
            *crate_vis = target;
        }
    }
}

fn spawn_npc_entity(commands: &mut Commands, npc: NpcEntity, tool_assets: &ToolAssets) {
    let pos = Vec3::from_array(npc.position);
    let rot = Quat::from_rotation_y(-npc.yaw);
    let vis = if npc.is_alive() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    match npc.kind {
        NpcKind::ZombieWalker | NpcKind::RadZombie | NpcKind::ScientistNpc => {
            let imported_key = match npc.kind {
                NpcKind::ScientistNpc => "npc_scientist",
                _ => "npc_zombie",
            };
            let imported = tool_assets.imported_models.get(imported_key).cloned();
            let body_mat = match npc.kind {
                NpcKind::ZombieWalker => tool_assets.npc_zombie_mat.clone(),
                NpcKind::RadZombie => tool_assets.npc_rad_zombie_mat.clone(),
                NpcKind::ScientistNpc => tool_assets.npc_scientist_mat.clone(),
                _ => tool_assets.npc_zombie_mat.clone(),
            };
            commands
                .spawn((
                    NpcVisual(npc.id),
                    Transform::from_translation(pos).with_rotation(rot),
                    vis,
                ))
                .with_children(|parent| {
                    if let Some((m_h, mat_h)) = imported {
                        let mat = if npc.kind == NpcKind::RadZombie {
                            body_mat
                        } else {
                            mat_h
                        };
                        parent.spawn((
                            Mesh3d(m_h),
                            MeshMaterial3d(mat),
                            Transform::from_xyz(0.0, -0.9, 0.0),
                        ));
                    } else {
                        parent.spawn((
                            Mesh3d(tool_assets.cylinder_mesh.clone()),
                            MeshMaterial3d(body_mat.clone()),
                            Transform::from_xyz(0.0, 0.0, 0.0)
                                .with_scale(Vec3::new(0.36, 1.25, 0.36)),
                        ));
                        parent.spawn((
                            Mesh3d(tool_assets.sphere_mesh.clone()),
                            MeshMaterial3d(body_mat),
                            Transform::from_xyz(0.0, 0.82, 0.0).with_scale(Vec3::splat(0.24)),
                        ));
                    }
                });
        }
        NpcKind::BradleyApc => {
            commands
                .spawn((
                    NpcVisual(npc.id),
                    Transform::from_translation(pos).with_rotation(rot),
                    vis,
                ))
                .with_children(|parent| {
                    // Armored hull
                    parent.spawn((
                        Mesh3d(tool_assets.unit_cube.clone()),
                        MeshMaterial3d(tool_assets.npc_bradley_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.15, 0.72, 1.85)),
                    ));
                    // Tank turret + cannon barrel
                    parent.spawn((
                        Mesh3d(tool_assets.unit_cube.clone()),
                        MeshMaterial3d(tool_assets.npc_bradley_mat.clone()),
                        Transform::from_xyz(0.0, 0.92, -0.15)
                            .with_scale(Vec3::new(0.68, 0.38, 0.85)),
                    ));
                    parent.spawn((
                        Mesh3d(tool_assets.cylinder_mesh.clone()),
                        MeshMaterial3d(tool_assets.wheel_dark_mat.clone()),
                        Transform::from_xyz(0.0, 0.96, 1.35)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(0.11, 1.6, 0.11)),
                    ));
                });
        }
        NpcKind::PatrolHeli => {
            commands
                .spawn((
                    NpcVisual(npc.id),
                    Transform::from_translation(pos).with_rotation(rot),
                    vis,
                ))
                .with_children(|parent| {
                    // Helicopter fuselage
                    parent.spawn((
                        Mesh3d(tool_assets.sphere_mesh.clone()),
                        MeshMaterial3d(tool_assets.npc_heli_mat.clone()),
                        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.3, 1.0, 2.6)),
                    ));
                    // Tail boom
                    parent.spawn((
                        Mesh3d(tool_assets.cylinder_mesh.clone()),
                        MeshMaterial3d(tool_assets.npc_heli_mat.clone()),
                        Transform::from_xyz(0.0, 0.15, -2.8)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                            .with_scale(Vec3::new(0.26, 3.2, 0.26)),
                    ));
                    // Main rotor disc
                    parent.spawn((
                        Mesh3d(tool_assets.cylinder_mesh.clone()),
                        MeshMaterial3d(tool_assets.wheel_dark_mat.clone()),
                        Transform::from_xyz(0.0, 1.15, 0.0).with_scale(Vec3::new(3.8, 0.05, 3.8)),
                    ));
                });
        }
    }
}

fn spawn_vehicle_entity(commands: &mut Commands, v: VehicleEntity, tool_assets: &ToolAssets) {
    let pos = Vec3::from_array(v.position);
    let rot = Quat::from_rotation_y(-v.yaw);
    let imported_key = match v.kind {
        VehicleKind::Bicycle => Some("vehicle_bike"),
        VehicleKind::DuneBuggy => Some("vehicle_buggy"),
        VehicleKind::ArmoredHumvee => Some("vehicle_humvee"),
        VehicleKind::Minicopter => None,
    };
    let imported = imported_key.and_then(|k| tool_assets.imported_models.get(k).cloned());
    let (body_mat, body_scale) = match v.kind {
        VehicleKind::Bicycle => (
            tool_assets.vehicle_bike_mat.clone(),
            Vec3::new(0.18, 0.42, 0.85),
        ),
        VehicleKind::DuneBuggy => (
            tool_assets.vehicle_buggy_mat.clone(),
            Vec3::new(0.85, 0.48, 1.45),
        ),
        VehicleKind::ArmoredHumvee => (
            tool_assets.vehicle_humvee_mat.clone(),
            Vec3::new(1.05, 0.72, 1.85),
        ),
        VehicleKind::Minicopter => (
            tool_assets.vehicle_copter_mat.clone(),
            Vec3::new(0.65, 0.58, 1.25),
        ),
    };
    commands
        .spawn((
            VehicleVisual(v.id),
            Transform::from_translation(pos).with_rotation(rot),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            if let Some((m_h, mat_h)) = imported {
                parent.spawn((
                    Mesh3d(m_h),
                    MeshMaterial3d(mat_h),
                    Transform::from_xyz(0.0, -0.35, 0.0),
                ));
            } else {
                parent.spawn((
                    Mesh3d(tool_assets.unit_cube.clone()),
                    MeshMaterial3d(body_mat),
                    Transform::from_xyz(0.0, 0.15, 0.0).with_scale(body_scale),
                ));
                if v.kind == VehicleKind::Minicopter {
                    parent.spawn((
                        Mesh3d(tool_assets.cylinder_mesh.clone()),
                        MeshMaterial3d(tool_assets.wheel_dark_mat.clone()),
                        Transform::from_xyz(0.0, 0.88, 0.0).with_scale(Vec3::new(2.4, 0.04, 2.4)),
                    ));
                }
            }
        });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_environment_atmosphere(
    options: Res<RuntimeOptions>,
    clock: Res<SimulationClock>,
    survival: Res<SimulationSurvival>,
    telemetry: Res<Telemetry>,
    interface: Res<Interface>,
    tool_assets: Res<ToolAssets>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera_fog: Query<&mut DistanceFog, With<MainCamera>>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<SunLight>>,
    mut sun_disc: Query<
        &mut Transform,
        (
            With<SunDiscVisual>,
            Without<SunLight>,
            Without<SkyDomeVisual>,
            Without<CloudCanopyVisual>,
            Without<CirrusCanopyVisual>,
        ),
    >,
    mut sky_dome: Query<
        &mut Transform,
        (
            With<SkyDomeVisual>,
            Without<SunLight>,
            Without<SunDiscVisual>,
            Without<CloudCanopyVisual>,
            Without<CirrusCanopyVisual>,
        ),
    >,
    mut cloud_canopy: Query<
        &mut Transform,
        (
            With<CloudCanopyVisual>,
            Without<SunLight>,
            Without<SunDiscVisual>,
            Without<SkyDomeVisual>,
            Without<CirrusCanopyVisual>,
        ),
    >,
    mut cirrus_canopy: Query<
        &mut Transform,
        (
            With<CirrusCanopyVisual>,
            Without<SunLight>,
            Without<SunDiscVisual>,
            Without<SkyDomeVisual>,
            Without<CloudCanopyVisual>,
        ),
    >,
) {
    if options.0.benchmark {
        return;
    }
    let env = survival
        .0
        .environment_at(ServerTick(clock.tick), telemetry.player_position);
    let sun_angle = (env.time_hours - 6.0) / 24.0 * std::f32::consts::TAU;
    let elevation = sun_angle.sin();
    let daylight = (elevation + 0.15).clamp(0.0, 1.0);
    let twilight = (1.0 - (elevation.abs() * 3.5)).clamp(0.0, 1.0) * (daylight * 1.4).min(1.0);

    let weather_dim = match env.weather {
        WeatherKind::Clear => 1.0,
        WeatherKind::Cloudy => 0.85,
        WeatherKind::RainStorm | WeatherKind::Blizzard => 0.56,
        WeatherKind::Fog | WeatherKind::Sandstorm => 0.68,
    };

    let player_pos = Vec3::from_array(telemetry.player_position);
    let sun_ray_dir = Vec3::new(
        sun_angle.cos() * 1150.0,
        (elevation.max(0.12) * 1050.0).max(120.0),
        -620.0,
    )
    .normalize();
    let sun_forward = -sun_ray_dir;

    if let Ok((mut light, mut transform)) = sun.single_mut() {
        light.illuminance = (1500.0 + daylight * 45_000.0) * weather_dim;
        let light_anchor = player_pos - sun_forward * 140.0;
        *transform = Transform::from_translation(light_anchor).looking_to(sun_forward, Vec3::Y);
    }

    if let Ok(mut sky_tf) = sky_dome.single_mut() {
        sky_tf.translation = Vec3::new(
            telemetry.player_position[0],
            0.0,
            telemetry.player_position[2],
        );
    }

    if let Ok(mut disc_tf) = sun_disc.single_mut() {
        disc_tf.translation = Vec3::new(
            telemetry.player_position[0] + sun_angle.cos() * 1150.0,
            (elevation * 1050.0).max(-180.0),
            telemetry.player_position[2] - 620.0,
        );
    }

    ambient.brightness = (120.0 + daylight * 260.0) * weather_dim;
    ambient.color = Color::srgb(0.72, 0.84, 0.98);

    // Realistic Rayleigh/Mie sky zenith & horizon colors
    let (mut zenith_rgb, mut horizon_rgb, mut fog_density): ([f32; 3], [f32; 3], f32) =
        match env.weather {
            WeatherKind::Sandstorm => (
                [
                    0.58 * daylight.max(0.22),
                    0.43 * daylight.max(0.22),
                    0.25 * daylight.max(0.22),
                ],
                [
                    0.76 * daylight.max(0.25),
                    0.58 * daylight.max(0.25),
                    0.34 * daylight.max(0.25),
                ],
                0.012,
            ),
            WeatherKind::Blizzard | WeatherKind::Fog => (
                [
                    0.52 * daylight.max(0.24),
                    0.58 * daylight.max(0.24),
                    0.66 * daylight.max(0.24),
                ],
                [
                    0.72 * daylight.max(0.26),
                    0.78 * daylight.max(0.26),
                    0.84 * daylight.max(0.26),
                ],
                0.010,
            ),
            WeatherKind::RainStorm => (
                [
                    0.22 * daylight.max(0.20),
                    0.29 * daylight.max(0.20),
                    0.38 * daylight.max(0.20),
                ],
                [
                    0.42 * daylight.max(0.22),
                    0.50 * daylight.max(0.22),
                    0.60 * daylight.max(0.22),
                ],
                0.0065,
            ),
            _ => (
                [
                    (0.03 + 0.14 * daylight + 0.24 * twilight) * weather_dim,
                    (0.05 + 0.40 * daylight + 0.08 * twilight) * weather_dim,
                    (0.12 + 0.74 * daylight - 0.10 * twilight) * weather_dim,
                ],
                [
                    (0.06 + 0.56 * daylight + 0.34 * twilight) * weather_dim,
                    (0.09 + 0.72 * daylight + 0.12 * twilight) * weather_dim,
                    (0.16 + 0.82 * daylight - 0.18 * twilight) * weather_dim,
                ],
                0.00018,
            ),
        };

    // Phase 5: Apocalyptic orange-red sky & fallout haze during Nuclear Strike detonation/fallout
    match survival.0.nuclear.phase {
        NuclearPhase::DetonationShockwave => {
            zenith_rgb = [0.78, 0.26, 0.10];
            horizon_rgb = [0.98, 0.44, 0.16];
            fog_density = 0.014;
        }
        NuclearPhase::Fallout => {
            zenith_rgb = [
                zenith_rgb[0] * 0.5 + 0.36,
                zenith_rgb[1] * 0.5 + 0.18,
                zenith_rgb[2] * 0.4 + 0.08,
            ];
            horizon_rgb = [
                horizon_rgb[0] * 0.5 + 0.44,
                horizon_rgb[1] * 0.5 + 0.24,
                horizon_rgb[2] * 0.4 + 0.10,
            ];
            fog_density = fog_density.max(0.0085);
        }
        _ => {}
    }

    clear_color.0 = Color::srgb(horizon_rgb[0], horizon_rgb[1], horizon_rgb[2]);
    if let Some(mut zenith_mat) = materials.get_mut(&tool_assets.sky_zenith_material) {
        let sky_mult = (0.15 + 0.85 * daylight * weather_dim).clamp(0.06, 1.0);
        zenith_mat.base_color = Color::srgb(
            (zenith_rgb[0] * sky_mult + twilight * 0.28).min(1.0),
            (zenith_rgb[1] * sky_mult + twilight * 0.08).min(1.0),
            (zenith_rgb[2] * sky_mult - twilight * 0.15).clamp(0.06, 1.0),
        );
    }
    if let Some(mut horizon_mat) = materials.get_mut(&tool_assets.sky_horizon_material) {
        horizon_mat.base_color = Color::srgba(horizon_rgb[0], horizon_rgb[1], horizon_rgb[2], 0.88);
    }
    if let Some(mut disc_mat) = materials.get_mut(&tool_assets.sun_disc_material) {
        disc_mat.base_color = Color::srgb(
            1.0,
            (0.96 - twilight * 0.28).clamp(0.4, 1.0),
            (0.84 - twilight * 0.45).clamp(0.2, 1.0),
        );
    }
    if let Some(mut corona_mat) = materials.get_mut(&tool_assets.sun_corona_material) {
        corona_mat.base_color = Color::srgba(
            1.0,
            (0.86 - twilight * 0.30).clamp(0.3, 1.0),
            (0.56 - twilight * 0.35).clamp(0.15, 1.0),
            0.34 * daylight * weather_dim,
        );
    }
    if let Ok(mut fog) = camera_fog.single_mut() {
        fog.color = Color::srgba(horizon_rgb[0], horizon_rgb[1], horizon_rgb[2], 1.0);
        fog.falloff = FogFalloff::ExponentialSquared {
            density: if interface.gm_fog_enabled {
                fog_density
            } else {
                0.000001
            },
        };
    }

    let px = telemetry.player_position[0];
    let pz = telemetry.player_position[2];

    if let Ok(mut canopy_tf) = cloud_canopy.single_mut() {
        canopy_tf.translation = Vec3::new(px, 1350.0, pz);
        canopy_tf.rotation = Quat::from_rotation_y(env.wind_offset[0] * 0.00008);
    }

    if let Ok(mut cirrus_tf) = cirrus_canopy.single_mut() {
        cirrus_tf.translation = Vec3::new(px, 1750.0, pz);
        cirrus_tf.rotation = Quat::from_rotation_y(env.wind_offset[0] * 0.00012 + 0.35);
    }

    let day_bright = (0.20 + 0.80 * daylight * weather_dim).clamp(0.08, 1.0);
    let cloud_r = (day_bright + twilight * 0.40).clamp(0.08, 1.0);
    let cloud_g = (day_bright + twilight * 0.12).clamp(0.08, 1.0);
    let cloud_b = (day_bright - twilight * 0.15 + (1.0 - daylight) * 0.08).clamp(0.10, 1.0);

    if let Some(mut cloud_mat) = materials.get_mut(&tool_assets.cloud_canopy_material) {
        cloud_mat.base_color = Color::srgba(cloud_r, cloud_g, cloud_b, 0.95);
    }
    if let Some(mut cirrus_mat) = materials.get_mut(&tool_assets.cirrus_canopy_material) {
        cirrus_mat.base_color = Color::srgba(
            cloud_r * 0.95 + 0.05,
            cloud_g * 0.95 + 0.05,
            cloud_b * 0.98 + 0.02,
            0.65 * daylight.max(0.15) * weather_dim,
        );
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn stream_world_chunks(
    mut commands: Commands,
    telemetry: Res<Telemetry>,
    mut grid: ResMut<SpatialPlacementGrid>,
    tool_assets: Res<ToolAssets>,
    mut terrain_chunks: Query<(&TerrainSectorChunk, &mut Visibility)>,
) {
    let px = telemetry.player_position[0];
    let pz = telemetry.player_position[2];

    // Distance LOD and culling for terrain sectors & roads
    let player_pos_2d = Vec2::new(px, pz);
    for (chunk, mut vis) in &mut terrain_chunks {
        let chunk_pos_2d = Vec2::new(chunk.center.x, chunk.center.z);
        let dist = player_pos_2d.distance(chunk_pos_2d);
        let max_dist = if chunk.is_road { 750.0 } else { 2400.0 };
        let new_vis = if dist <= max_dist {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != new_vis {
            *vis = new_vis;
        }
    }

    let player_cell_x = (px / CHUNK_CELL_SIZE).floor() as i32;
    let player_cell_z = (pz / CHUNK_CELL_SIZE).floor() as i32;
    let current_cell = (player_cell_x, player_cell_z);

    if grid.current_player_cell == Some(current_cell) {
        return;
    }
    grid.current_player_cell = Some(current_cell);

    // 1. Despawn distant cells beyond CHUNK_DESPAWN_RADIUS
    let mut to_despawn = Vec::new();
    for (&(cx, cz), entities) in &grid.active_entities {
        let dx = (cx - player_cell_x).abs();
        let dz = (cz - player_cell_z).abs();
        if dx > CHUNK_DESPAWN_RADIUS || dz > CHUNK_DESPAWN_RADIUS {
            for &e in entities {
                commands.entity(e).despawn();
            }
            to_despawn.push((cx, cz));
        }
    }
    for key in to_despawn {
        grid.active_entities.remove(&key);
    }

    // 2. Spawn newly entered cells within CHUNK_ACTIVE_RADIUS
    for cx in (player_cell_x - CHUNK_ACTIVE_RADIUS)..=(player_cell_x + CHUNK_ACTIVE_RADIUS) {
        for cz in (player_cell_z - CHUNK_ACTIVE_RADIUS)..=(player_cell_z + CHUNK_ACTIVE_RADIUS) {
            let key = (cx, cz);
            if grid.active_entities.contains_key(&key) {
                continue;
            }
            if let Some(placements) = grid.cells.get(&key) {
                let mut cell_entities = Vec::with_capacity(placements.len());
                for plc in placements {
                    let model_key = match plc.kind {
                        1 => "tree_jungle",
                        3 => "grass_clump",
                        4 => {
                            if tool_assets.imported_models.contains_key("rock_small") {
                                "rock_small"
                            } else {
                                "rock_node"
                            }
                        }
                        5 => "building_gas_station",
                        6 => "building_supermarket",
                        7 => "building_warehouse",
                        8 => "building_old_warehouse",
                        9 => "building_barracks",
                        10 => "building_garage",
                        11 => "building_pylon",
                        12 => "building_watchtower",
                        13 => "prop_fence",
                        14 => "prop_military_box",
                        _ => "tree_temperate",
                    };
                    if let Some((m_h, mat_h)) = tool_assets.imported_models.get(model_key) {
                        let pos_vec = Vec3::from_array(plc.position);
                        let e = commands
                            .spawn((
                                ScenicPlacementMarker {
                                    position: pos_vec,
                                    scale: plc.scale,
                                },
                                Mesh3d(m_h.clone()),
                                MeshMaterial3d(mat_h.clone()),
                                Transform::from_translation(pos_vec)
                                    .with_rotation(Quat::from_array(plc.rotation))
                                    .with_scale(Vec3::splat(plc.scale)),
                            ))
                            .id();
                        cell_entities.push(e);
                    }
                }
                grid.active_entities.insert(key, cell_entities);
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_audio_effects(
    mut commands: Commands,
    time: Res<Time<Real>>,
    interface: Res<Interface>,
    survival: Res<SimulationSurvival>,
    telemetry: Res<Telemetry>,
    audio: Option<Res<AudioAssets>>,
    mut audio_state: ResMut<AudioRuntimeState>,
    player_query: Query<(&SimulationPose, &CharacterMotion), With<LocalPlayer>>,
) {
    let Some(audio) = audio else {
        return;
    };
    let player = survival.0.player(PlayerId(0));
    let dt = time.delta_secs();

    // 1. Weapon Gunshots (when a shot tracer was triggered)
    if interface.last_shot_trace.is_some() && !audio_state.prev_shot_trace {
        let sound = if let Some(p) = player
            && let Some(active) = p.inventory.active_item()
        {
            match active.kind {
                ItemKind::TeleportGun => audio.keycard_swipe.clone(),
                ItemKind::AssaultRifle => audio.gunshot_ak.clone(),
                ItemKind::Revolver => audio.gunshot_pistol.clone(),
                _ => audio.gunshot_shotgun.clone(),
            }
        } else {
            audio.gunshot_ak.clone()
        };
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
    }
    audio_state.prev_shot_trace = interface.last_shot_trace.is_some();

    // 2. Harvesting Chops (when swinging axe/pickaxe/rock)
    if interface.swing_timer > 0.0
        && audio_state.prev_swing_timer == 0.0
        && let Some(p) = player
        && let Some(active) = p.inventory.active_item()
    {
        let chop_snd = match active.kind {
            ItemKind::StonePickaxe => Some(audio.chop_rock.clone()),
            ItemKind::StoneHatchet => Some(audio.chop_wood.clone()),
            ItemKind::Rock | ItemKind::Hammer => Some(audio.chop_rock.clone()),
            _ => None,
        };
        if let Some(snd) = chop_snd {
            commands.spawn((AudioPlayer::new(snd), PlaybackSettings::DESPAWN));
        }
    }
    audio_state.prev_swing_timer = interface.swing_timer;

    // 3. Footsteps based on velocity and surface
    if let Ok((pose, motion)) = player_query.single() {
        let dx = pose.current.x - pose.previous.x;
        let dz = pose.current.z - pose.previous.z;
        let horiz_dist = (dx * dx + dz * dz).sqrt();
        if motion.0.grounded && horiz_dist > 0.015 {
            audio_state.footstep_timer -= dt * (horiz_dist / 0.06).clamp(1.0, 2.5);
            if audio_state.footstep_timer <= 0.0 {
                audio_state.footstep_timer = 0.36;
                let is_water = telemetry.player_position[1] < 0.4;
                let step_sound = if is_water {
                    audio.footstep_water.clone()
                } else {
                    audio.footstep_grass.clone()
                };
                commands.spawn((AudioPlayer::new(step_sound), PlaybackSettings::DESPAWN));
            }
        } else {
            audio_state.footstep_timer = 0.1;
        }
    }

    // 4. Player damage / Hurt sound
    if let Some(p) = player {
        if audio_state.prev_health > 0.0 && p.vitals.health < audio_state.prev_health - 1.0 {
            commands.spawn((
                AudioPlayer::new(audio.player_hit.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
        audio_state.prev_health = p.vitals.health;
    }

    // 5. Monument Power Switch & Keycard Door
    let monument = &survival.0.monument;
    if monument.is_powered() && !audio_state.prev_monument_power {
        commands.spawn((
            AudioPlayer::new(audio.fuse_switch.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    audio_state.prev_monument_power = monument.is_powered();

    if monument.is_door_open() && !audio_state.prev_monument_door {
        commands.spawn((
            AudioPlayer::new(audio.keycard_swipe.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    audio_state.prev_monument_door = monument.is_door_open();

    // 6. Nuclear Explosion
    let nuke_active = survival.0.nuclear.is_active();
    if nuke_active && !audio_state.prev_nuke_active {
        commands.spawn((
            AudioPlayer::new(audio.explosion.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    audio_state.prev_nuke_active = nuke_active;
}

fn measure_frames(
    time: Res<Time<Real>>,
    options: Res<RuntimeOptions>,
    interface: Res<Interface>,
    mut monitor: ResMut<FrameMonitor>,
    mut telemetry: ResMut<Telemetry>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    let delta = time.delta_secs_f64();
    if delta <= 0.0 {
        return;
    }
    monitor.recent_ms.push_back(delta * 1000.0);
    if monitor.recent_ms.len() > 120 {
        monitor.recent_ms.pop_front();
    }
    monitor.display_elapsed += delta;
    if monitor.display_elapsed >= 0.25 {
        if let Some(stats) = FrameStatistics::from_samples(monitor.recent_ms.make_contiguous()) {
            telemetry.fps = stats.average_fps;
            telemetry.mean_ms = stats.mean_ms;
            telemetry.p95_ms = stats.p95_ms;
        }
        monitor.display_elapsed = 0.0;
    }
    if !options.0.benchmark
        || interface.mode != UiMode::Benchmark
        || monitor.benchmark.is_finished()
    {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let resolution = [window.physical_width(), window.physical_height()];
    if !window.focused || resolution.contains(&0) {
        monitor.focused_last_frame = false;
        monitor.benchmark.reset();
        telemetry.benchmark_status = "Очікування активного вікна; замір почнеться заново.".into();
        return;
    }
    if !monitor.focused_last_frame || monitor.capture_resolution != resolution {
        monitor.focused_last_frame = true;
        monitor.capture_resolution = resolution;
        monitor.benchmark.reset();
        telemetry.benchmark_status = "Прогрів: 10 с".into();
        return;
    }
    if let Some(statistics) = monitor.benchmark.observe(delta) {
        let target_conditions_met = !cfg!(debug_assertions)
            && resolution == [1920, 1080]
            && telemetry.gpu.contains("RTX 4060")
            && telemetry.backend == "Vulkan"
            && options.0.no_vsync;
        let target_achieved = target_conditions_met.then_some(statistics.average_fps >= 120.0);
        let report = BenchmarkReport {
            gpu: telemetry.gpu.clone(),
            backend: telemetry.backend.clone(),
            resolution,
            build_profile: if cfg!(debug_assertions) {
                "dev".into()
            } else {
                "release".into()
            },
            vsync: !options.0.no_vsync,
            warmup_seconds: WARMUP_SECONDS,
            statistics,
            target_fps: 120,
            target_conditions_met,
            target_achieved,
        };
        let result = serde_json::to_vec_pretty(&report)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                println!("{}", String::from_utf8_lossy(&bytes));
                if let Some(path) = &options.0.benchmark_report {
                    write_report(path, &bytes).map_err(|e| e.to_string())?;
                }
                Ok(())
            });
        if target_achieved == Some(false) {
            warn!("120 FPS target missed; retain this report for a separate optimization task.");
        }
        if !target_conditions_met {
            warn!(
                "Results recorded, but the agreed RTX 4060 / 1080p / release target conditions were not met."
            );
        }
        match result {
            Ok(()) => {
                exit.write(AppExit::Success);
            }
            Err(error) => {
                error!("Benchmark report: {error}");
                exit.write(AppExit::error());
            }
        }
    } else if monitor.benchmark.is_finished() {
        error!("Benchmark ended without valid frame samples.");
        exit.write(AppExit::error());
    } else {
        let elapsed = monitor.benchmark.elapsed();
        telemetry.benchmark_status = if elapsed < WARMUP_SECONDS {
            format!("Прогрів: {:.0} с", WARMUP_SECONDS - elapsed)
        } else {
            format!(
                "Замір: {:.0} / {:.0} с",
                elapsed - WARMUP_SECONDS,
                CAPTURE_SECONDS
            )
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volumetric_cloud_3d_density_texture_generation() {
        let image = generate_cloud_3d_image();
        assert_eq!(image.texture_descriptor.dimension, TextureDimension::D3);
        assert_eq!(image.texture_descriptor.size.width, 64);
        assert_eq!(image.texture_descriptor.size.height, 64);
        assert_eq!(image.texture_descriptor.size.depth_or_array_layers, 64);

        let data = image
            .data
            .as_ref()
            .expect("cloud texture data should exist");
        assert_eq!(data.len(), 64 * 64 * 64);

        // Verify non-zero cloud density exists
        let non_zero_count = data.iter().filter(|&&v| v > 0).count();
        assert!(non_zero_count > 1000);
        // Verify open sky / clear gaps also exist
        let zero_count = data.iter().filter(|&&v| v == 0).count();
        assert!(zero_count > 5000);
    }

    #[test]
    fn spatial_placement_grid_partitioning_and_streaming() {
        let mut grid = SpatialPlacementGrid::default();
        let dummy_placement = shared::level::ImportedPlacement {
            kind: 1,
            position: [200.0, 5.0, 350.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
        };
        let cx = (dummy_placement.position[0] / CHUNK_CELL_SIZE).floor() as i32;
        let cz = (dummy_placement.position[2] / CHUNK_CELL_SIZE).floor() as i32;
        grid.cells
            .entry((cx, cz))
            .or_default()
            .push(dummy_placement);

        assert!(grid.cells.contains_key(&(cx, cz)));
        assert_eq!(grid.cells.get(&(cx, cz)).unwrap().len(), 1);
        assert_eq!(cx, 1); // 200.0 / 180.0 = 1
        assert_eq!(cz, 1); // 350.0 / 180.0 = 1
    }

    #[test]
    fn celestial_cloud_canopy_generation() {
        let tex = generate_cloud_canopy_texture();
        assert_eq!(tex.texture_descriptor.dimension, TextureDimension::D2);
        assert_eq!(tex.texture_descriptor.size.width, 512);
        assert_eq!(tex.texture_descriptor.size.height, 512);
        let data = tex.data.as_ref().expect("cloud canopy texture data");
        assert_eq!(data.len(), 512 * 512 * 4);

        let cirrus = generate_cirrus_texture();
        assert_eq!(cirrus.texture_descriptor.dimension, TextureDimension::D2);
        let cirrus_data = cirrus.data.as_ref().expect("cirrus texture data");
        assert_eq!(cirrus_data.len(), 512 * 512 * 4);

        let mesh = build_cloud_canopy_mesh(4600.0, 1350.0);
        assert!(mesh.count_vertices() > 500);
    }
}
