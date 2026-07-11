struct UIVertex {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct Uniforms {
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var time_texture: texture_2d<f32>;

@group(1) @binding(1)
var time_sampler: sampler;

struct VertexOutput {
    @builtin(position)
    position: vec4<f32>,

    @location(0)
    color: vec4<f32>,

    @location(1)
    uv: vec2<f32>,
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
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // uv.x < 0.0 marks a solid-color quad (no texture)
    if (in.uv.x < 0.0) {
        return in.color;
    }

    let alpha = textureSample(time_texture, time_sampler, in.uv).r;
    let out_alpha = alpha * in.color.a;
    return vec4<f32>(in.color.rgb, out_alpha);
}
