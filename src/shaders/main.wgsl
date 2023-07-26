struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tint: vec3<f32>,
}

struct InstanceInput {
    @location(2) pos: vec2<f32>,
};

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var sample: sampler;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.texture_coordinates;
    out.tint = vec3<f32>(1.0, 1.0, 1.0);
    out.clip_position = vec4<f32>(vec3<f32>(instance.pos, 0.0) + model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sample, in.tex_coords) * vec4<f32>(in.tint, 1.0);
}
