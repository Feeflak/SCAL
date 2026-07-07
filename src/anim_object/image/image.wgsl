struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> transform: mat4x4<f32>;

@group(2) @binding(0)
var tex: texture_2d<f32>;

@group(2) @binding(1)
var samp: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var world_pos = transform * vec4<f32>(input.position, 0.0, 1.0);
    world_pos.z = 0.0;
    out.position = camera * world_pos;
    out.color = input.color;
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, input.uv) * input.color;
}
