import bevy_pbr::render::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing, calculate_tbn_mikktspace},
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var splat_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var splat_sampler: sampler;

@group(constants::MATERIAL_BIND_GROUP) @binding(2) var layer0_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var layer0_sampler: sampler;

@group(constants::MATERIAL_BIND_GROUP) @binding(4) var layer1_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var layer1_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(6) var layer2_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(7) var layer2_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(8) var layer3_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(9) var layer3_sampler: sampler;

struct TerrainUniform {
    tiling: vec4<f32>,
    smoothness: vec4<f32>,
    layer_defaults: array<vec4<f32>, 4>,
    normal_defaults: array<vec4<f32>, 4>,
    control_default: vec4<f32>,
    layer_present: vec4<u32>,
    normal_present: vec4<u32>,
    layer_count: u32,
    control_present: u32,
    normal_enabled: u32,
};
@group(constants::MATERIAL_BIND_GROUP) @binding(10) var<uniform> terrain: TerrainUniform;
@group(constants::MATERIAL_BIND_GROUP) @binding(11) var normal0_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(12) var normal0_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(13) var normal1_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(14) var normal1_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(15) var normal2_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(16) var normal2_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(17) var normal3_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(18) var normal3_sampler: sampler;

// Mesh UVs and PNGs have a top-left origin. Reconstruct the compiled shader's
// UV0 * _SplatN_S before converting the sampled image coordinate. This also
// preserves non-integral source tiling without introducing a vertical offset.
fn layer_uv(uv: vec2<f32>, scale: f32) -> vec2<f32> {
    return vec2<f32>(uv.x * scale, 1.0 - (1.0 - uv.y) * scale);
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let control_sample = textureSample(splat_texture, splat_sampler, in.uv);
    let control = select(terrain.control_default, control_sample, terrain.control_present != 0u);
    let l0 = select(terrain.layer_defaults[0], textureSample(layer0_texture, layer0_sampler, layer_uv(in.uv, terrain.tiling.x)), terrain.layer_present.x != 0u);
    let l1 = select(terrain.layer_defaults[1], textureSample(layer1_texture, layer1_sampler, layer_uv(in.uv, terrain.tiling.y)), terrain.layer_present.y != 0u);
    let l2 = select(terrain.layer_defaults[2], textureSample(layer2_texture, layer2_sampler, layer_uv(in.uv, terrain.tiling.z)), terrain.layer_present.z != 0u);
    let l3 = select(terrain.layer_defaults[3], textureSample(layer3_texture, layer3_sampler, layer_uv(in.uv, terrain.tiling.w)), terrain.layer_present.w != 0u);

    // Verified from complete APK GLES3 programs: raw weights, no normalization,
    // alpha retained in the three-layer residual, interpolation towards L0.
    let sum = dot(control, vec4<f32>(1.0));
    let residual = clamp(1.0 - sum, 0.0, 1.0);
    let color_weights = control * vec4<f32>(1.0, 1.0, 1.0, select(0.0, 1.0, terrain.layer_count == 4u));
    let blended = l0.rgb * color_weights.r + l1.rgb * color_weights.g + l2.rgb * color_weights.b + l3.rgb * color_weights.a;
    let albedo = blended + residual * (l0.rgb - blended);
    let smoothness = dot(color_weights, vec4<f32>(l0.a, l1.a, l2.a, l3.a) * terrain.smoothness);

    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);
    pbr_input.material.base_color = vec4<f32>(albedo, sum);
    pbr_input.material.perceptual_roughness = clamp(1.0 - smoothness, 0.0, 1.0);
    pbr_input.material.reflectance = vec3<f32>(0.5);
    pbr_input.material.flags |= STANDARD_MATERIAL_FLAGS_FOG_ENABLED_BIT;

@if(VERTEX_TANGENTS) {
    if terrain.normal_enabled != 0u {
        let n0 = select(terrain.normal_defaults[0], textureSample(normal0_texture, normal0_sampler, layer_uv(in.uv, terrain.tiling.x)), terrain.normal_present.x != 0u);
        let n1 = select(terrain.normal_defaults[1], textureSample(normal1_texture, normal1_sampler, layer_uv(in.uv, terrain.tiling.y)), terrain.normal_present.y != 0u);
        let n2 = select(terrain.normal_defaults[2], textureSample(normal2_texture, normal2_sampler, layer_uv(in.uv, terrain.tiling.z)), terrain.normal_present.z != 0u);
        let n3 = select(terrain.normal_defaults[3], textureSample(normal3_texture, normal3_sampler, layer_uv(in.uv, terrain.tiling.w)), terrain.normal_present.w != 0u);
        let blended_normal = (n0.rgb * 2.0 - 1.0) * color_weights.r
            + (n1.rgb * 2.0 - 1.0) * color_weights.g
            + (n2.rgb * 2.0 - 1.0) * color_weights.b
            + (n3.rgb * 2.0 - 1.0) * color_weights.a;
        // The shipped normal-map programs interpolate towards ENCODED N0,
        // not its decoded [-1,1] value. Preserve that unusual source behavior.
        var tangent_normal = blended_normal + residual * (n0.rgb - blended_normal);
        tangent_normal.y = -tangent_normal.y;
        let tbn = calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
        let world_normal = tbn * tangent_normal;
        pbr_input.N = world_normal * inverseSqrt(max(dot(world_normal, world_normal), 1.17549435e-38));
    }
}

    // Lighting, shadows, mip selection and post-processing remain Bevy's.
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
