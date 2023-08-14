struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coordinates: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tint: vec4<f32>,
}

struct InstanceInput {
    @location(2) pos: vec3<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) angle: f32,
    @location(5) color: vec4<f32>
};

struct Camera {
    view: mat4x4<f32>,
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var sample: sampler;

@group(1) @binding(0)
var<uniform> camera: Camera;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let coss = cos(instance.angle);
    let sinn = sin(instance.angle);
    let rotor = 
        mat3x3(
            coss, sinn, 0.,
            -sinn, coss, 0.,
            0., 0., 1.);

    out.tex_coords = model.texture_coordinates;
    out.tint = instance.color;
    out.clip_position = camera.view * vec4<f32>((instance.pos + rotor * model.position * vec3<f32>(instance.scale, 0.0)), 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sample, in.tex_coords) * in.tint;
}
