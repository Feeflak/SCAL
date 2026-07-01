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

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let screen_size = vec2<f32>(1920.0, 1080.0);

    let pos = input.position / screen_size * 2.0 - 1.0;

    out.position = vec4<f32>(
        pos.x,
        -pos.y,
        0.0,
        1.0
    );

    out.color = input.color;
    out.uv = input.uv;

    return out;
}



@group(0)
@binding(0)
var atlas:texture_2d<f32>;

@group(0)
@binding(1)
var sampler0:sampler;



@fragment
fn fs_main(input: VertexOutput)
    -> @location(0) vec4<f32>
{
    let alpha =
        textureSample(
            atlas,
            sampler0,
            input.uv
        ).r;

    return vec4<f32>(
        input.color.xyz,
        alpha * input.color.w
    );
}
