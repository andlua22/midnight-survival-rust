import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(1) @binding(0) var<uniform> tint: vec4<f32>;
@group(1) @binding(1) var atlas: texture_2d<f32>;
@group(1) @binding(2) var atlas_sampler: sampler;
@group(1) @binding(3) var coordinates: texture_2d<f32>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let dims = textureDimensions(coordinates);
    let p = clamp(vec2<i32>(in.uv * vec2<f32>(dims)), vec2<i32>(0), vec2<i32>(dims) - vec2<i32>(1));
    let mapping = textureLoad(coordinates, p, 0);
    if mapping.a < 0.5 { discard; }
    return textureSample(atlas, atlas_sampler, mapping.xy) * tint;
}
