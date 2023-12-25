// Vertex shader

struct Vertex2DInput {
    @location(0) position: vec2<u32>,
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
    model: Vertex2DInput,
) -> VertexOutput {
    var scaled_x = ((f32(model.position[0]) / f32(window_dimensions[0])) * 2.0) - 1.0;
    var scaled_y = (((f32(model.position[1]) / f32(window_dimensions[1])) * 2.0) - 1.0) * -1.0;

    var out: VertexOutput;
    out.tex_coords = vec2<f32>(model.tex_coords[0], model.tex_coords[1]);
    out.clip_position = vec4<f32>(scaled_x, scaled_y, 0.0, 1.0);
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
