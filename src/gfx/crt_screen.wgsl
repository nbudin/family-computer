// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(2)
var<uniform> window_dimensions: vec2<u32>;
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var aspect: f32 = f32(window_dimensions[0]) / f32(window_dimensions[1]);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    if aspect >= 1.0 {
        out.clip_position = vec4<f32>(model.position[0] / aspect, model.position[1], model.position[2], 1.0);
    } else {
        out.clip_position = vec4<f32>(model.position[0], model.position[1] * aspect, model.position[2], 1.0);
    }
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
