struct UIVertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct Uniforms {
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    color: vec4<f32>,
};

@vertex
fn vs_main(in: UIVertex) -> VertexOutput {
    var out: VertexOutput;

    let clip = vec2<f32>(
        2.0 * in.position.x / uniforms.resolution.x - 1.0,
        -(2.0 * in.position.y / uniforms.resolution.y - 1.0),
    );

    out.position = vec4<f32>(clip, 0.0, 1.0);
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
