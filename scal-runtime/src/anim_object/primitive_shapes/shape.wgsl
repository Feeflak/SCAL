struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) params: vec2<f32>,
};


@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> transform:mat4x4<f32>;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var world_pos = transform * vec4<f32>(input.position, 0.0, 1.0);
    world_pos.z = 0.0;
    out.position = camera * world_pos;
    out.color = input.color;
    out.local_pos = input.position;
    out.params = input.uv;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // uv.y > 0.5 indicates a rectangle (non-SDF path)
    if (input.params.y > 0.5) {
        return vec4<f32>(input.color);
    }

    // Circle SDF with smoothstep anti-aliasing
    let dist = length(input.local_pos);
    let radius = input.params.x;
    let sdf = dist - radius;
    let fw = fwidth(sdf);
    let alpha = 1.0 - smoothstep(-fw, fw, sdf);

    return vec4<f32>(input.color.rgb, input.color.a * alpha);
}
