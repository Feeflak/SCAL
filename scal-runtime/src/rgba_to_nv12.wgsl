@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> dst: array<u32>;

struct Params {
    width: u32,
    height: u32,
    y_stride: u32,
    _pad: u32,
}
@group(0) @binding(2) var<uniform> params: Params;

fn rgb_to_y(r: f32, g: f32, b: f32) -> u32 {
    return u32(clamp(0.2126 * r + 0.7152 * g + 0.0722 * b, 0.0, 1.0) * 255.0 + 0.5);
}

fn rgb_to_u(r: f32, g: f32, b: f32) -> u32 {
    return u32(clamp(-0.1146 * r - 0.3854 * g + 0.5000 * b + 0.5, 0.0, 1.0) * 255.0 + 0.5);
}

fn rgb_to_v(r: f32, g: f32, b: f32) -> u32 {
    return u32(clamp(0.5000 * r - 0.4542 * g - 0.0458 * b + 0.5, 0.0, 1.0) * 255.0 + 0.5);
}

// Each thread handles a 4x2 pixel block
@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let bx = id.x;  // block x index: 4 pixels per block
    let by = id.y;  // block y index: 2 rows per block
    let w = params.width;
    let h = params.height;
    let ys = params.y_stride;  // ceil(width / 4)

    let px = bx * 4u;
    let py = by * 2u;
    if px >= w || py >= h { return; }

    // Y plane: write packed u32 for row py
    var y0: u32 = 0u;
    var y1: u32 = 0u;
    for (var i = 0u; i < 4u; i++) {
        let cx = px + i;
        if cx < w {
            let rgb0 = textureLoad(src, vec2(cx, py), 0).rgb;
            y0 |= rgb_to_y(rgb0.r, rgb0.g, rgb0.b) << (i * 8u);
            if py + 1u < h {
                let rgb1 = textureLoad(src, vec2(cx, py + 1u), 0).rgb;
                y1 |= rgb_to_y(rgb1.r, rgb1.g, rgb1.b) << (i * 8u);
            }
        }
    }
    dst[py * ys + bx] = y0;
    if py + 1u < h {
        dst[(py + 1u) * ys + bx] = y1;
    }

    // UV plane: one pair per 2 rows
    // Pack U_left | V_left | U_right | V_right into one u32
    var uv: u32 = 0u;
    for (var block = 0u; block < 2u; block++) {
        let block_px = px + block * 2u;
        if block_px >= w { break; }

        var r_acc: f32 = 0.0;
        var g_acc: f32 = 0.0;
        var b_acc: f32 = 0.0;
        var count: u32 = 0u;

        for (var dy = 0u; dy < 2u; dy++) {
            for (var dx = 0u; dx < 2u; dx++) {
                let cx = block_px + dx;
                let cy = py + dy;
                if cx >= w || cy >= h { continue; }
                let rgb = textureLoad(src, vec2(cx, cy), 0).rgb;
                r_acc += rgb.r;
                g_acc += rgb.g;
                b_acc += rgb.b;
                count++;
            }
        }

        if count > 0u {
            let inv = 1.0 / f32(count);
            r_acc *= inv;
            g_acc *= inv;
            b_acc *= inv;
            uv |= rgb_to_u(r_acc, g_acc, b_acc) << (block * 16u);
            uv |= rgb_to_v(r_acc, g_acc, b_acc) << (block * 16u + 8u);
        }
    }

    let uv_offset = ys * h;
    dst[uv_offset + by * ys + bx] = uv;
}
