import bevy_render::view::View;
import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;

struct CloudSettings {
    sun: vec4<f32>,
    wind: vec4<f32>,
    layer: vec4<f32>,
    sky_zenith: vec4<f32>,
    sky_horizon: vec4<f32>,
    light_color: vec4<f32>,
    quality: vec4<u32>,
}
@group(0) @binding(0) var<uniform> view: View;
@group(0) @binding(1) var<uniform> clouds: CloudSettings;
@if(MULTISAMPLED)
@group(0) @binding(2) var scene_depth: texture_depth_multisampled_2d;
@else
@group(0) @binding(2) var scene_depth: texture_depth_2d;
@group(0) @binding(3) var density: texture_3d<f32>;
@group(0) @binding(4) var linear_sampler: sampler;
@group(0) @binding(5) var scene_color: texture_2d<f32>;
@group(0) @binding(6) var cloud_color: texture_2d<f32>;
@group(0) @binding(7) var screen_sampler: sampler;

fn ndc(uv: vec2<f32>) -> vec2<f32> {
    return uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
}
fn ray_direction(uv: vec2<f32>) -> vec3<f32> {
    let near = view.view_from_clip * vec4<f32>(ndc(uv), 1.0, 1.0);
    return normalize((view.world_from_view * vec4<f32>(near.xyz / near.w, 0.0)).xyz);
}
fn depth_at(uv: vec2<f32>) -> f32 {
    let size = textureDimensions(scene_depth);
    let pixel = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2<i32>(0), vec2<i32>(size) - vec2<i32>(1));
    @if(MULTISAMPLED) {
        // Reverse Z: the nearest sample safely occludes volume at silhouette edges.
        var depth = 0.0;
        for (var sample = 0; sample < i32(textureNumSamples(scene_depth)); sample += 1) {
            depth = max(depth, textureLoad(scene_depth, pixel, sample));
        }
        return depth;
    }
    @else {
        return textureLoad(scene_depth, pixel, 0);
    }
}
fn opaque_distance(uv: vec2<f32>) -> f32 {
    let depth = depth_at(uv);
    if depth <= 0.0 { return 18000.0; }
    let world = view.world_from_clip * vec4<f32>(ndc(uv), depth, 1.0);
    return distance(world.xyz / world.w, view.world_position);
}
fn layer_interval(origin: vec3<f32>, direction: vec3<f32>, limit: f32) -> vec2<f32> {
    if abs(direction.y) < 0.00001 {
        if origin.y < clouds.layer.x || origin.y > clouds.layer.y { return vec2<f32>(0.0); }
        return vec2<f32>(0.0, limit);
    }
    let a = (clouds.layer.x - origin.y) / direction.y;
    let b = (clouds.layer.y - origin.y) / direction.y;
    return vec2<f32>(max(min(a, b), 0.0), min(max(a, b), limit));
}
fn sample_density(position: vec3<f32>) -> f32 {
    let height = (position.y - clouds.layer.x) / (clouds.layer.y - clouds.layer.x);
    if height <= 0.0 || height >= 1.0 { return 0.0; }
    let uvw = vec3<f32>((position.x - clouds.wind.x) / 2300.0, height, (position.z - clouds.wind.z) / 2300.0);
    let envelope = smoothstep(0.0, 0.12, height) * (1.0 - smoothstep(0.65, 1.0, height));
    return textureSampleLevel(density, linear_sampler, uvw, 0.0).r * envelope * clouds.layer.w;
}
fn sunlight_transmittance(position: vec3<f32>) -> f32 {
    if clouds.sun.w <= 0.0 { return 0.0; }
    let sun_ray = clouds.sun.xyz;
    let interval = layer_interval(position, sun_ray, 1500.0);
    let count = clamp(clouds.quality.y, 1u, 8u);
    let step = max(interval.y, 0.0) / f32(count);
    var optical_depth = 0.0;
    for (var i = 0u; i < count; i += 1u) {
        optical_depth += sample_density(position + sun_ray * (f32(i) + 0.5) * step) * step * clouds.layer.z;
    }
    return exp(-optical_depth);
}

@fragment
fn raymarch(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if clouds.quality.z == 0u { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    let direction = ray_direction(in.uv);
    let interval = layer_interval(view.world_position, direction, opaque_distance(in.uv));
    if interval.y <= interval.x { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    let count = clamp(clouds.quality.x, 12u, 48u);
    let step = (interval.y - interval.x) / f32(count);
    let cosine = dot(direction, clouds.sun.xyz);
    let g = 0.55;
    let phase = (1.0 - g * g) / pow(max(1.0 + g * g - 2.0 * g * cosine, 0.02), 1.5);
    let ambient = mix(vec3<f32>(0.008, 0.014, 0.025), vec3<f32>(0.35, 0.43, 0.55), clouds.sun.w);
    var transmittance = 1.0;
    var scattered = vec3<f32>(0.0);
    for (var i = 0u; i < count; i += 1u) {
        let position = view.world_position + direction * (interval.x + (f32(i) + 0.5) * step);
        let rho = sample_density(position);
        if rho > 0.001 {
            let attenuation = exp(-rho * step * clouds.layer.z);
            let direct = clouds.light_color.rgb * clouds.sun.w * sunlight_transmittance(position) * (0.35 + phase * 0.25);
            scattered += transmittance * (1.0 - attenuation) * (ambient + direct);
            transmittance *= attenuation;
            if transmittance < 0.015 { break; }
        }
    }
    // Fade at the distance limit instead of drawing a sharp horizon curtain.
    let horizon = 1.0 - smoothstep(14000.0, 18000.0, interval.x);
    return vec4<f32>(scattered * horizon, mix(1.0, transmittance, horizon));
}

fn sky(direction: vec3<f32>) -> vec3<f32> {
    let elevation = pow(clamp(direction.y, 0.0, 1.0), 0.55);
    var color = mix(clouds.sky_horizon.rgb, clouds.sky_zenith.rgb, elevation);
    let alignment = dot(direction, clouds.sun.xyz);
    let disc = smoothstep(cos(0.012), cos(0.009), alignment);
    let corona = pow(max(alignment, 0.0), 600.0);
    color += clouds.light_color.rgb * clouds.sun.w * (disc * 12.0 + corona * 0.5);
    return color;
}

@fragment
fn composite(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let direction = ray_direction(in.uv);
    var background = textureSampleLevel(scene_color, screen_sampler, in.uv, 0.0).rgb;
    if depth_at(in.uv) <= 0.0 { background = sky(direction); }
    let interval = layer_interval(view.world_position, direction, opaque_distance(in.uv));
    // Full-resolution depth rejection prevents half-resolution cloud samples
    // bleeding onto foreground terrain, buildings and foliage silhouettes.
    if clouds.quality.z == 0u || interval.y <= interval.x { return vec4<f32>(background, 1.0); }
    let uv = clamp(in.uv, vec2<f32>(0.0001), vec2<f32>(0.9999));
    let volume = textureSampleLevel(cloud_color, screen_sampler, uv, 0.0);
    return vec4<f32>(background * volume.a + volume.rgb, 1.0);
}
