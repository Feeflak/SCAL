pub mod mesh;

use anyhow::Context;
use glam::Vec2;

use crate::anim_object::image::StretchMode;
use crate::types::*;

#[derive(Clone, Debug)]
pub struct Svg {
    pub path: String,
    pub size: Vec2,
    pub tint: Color,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: Option<f32>,
    pub stretch: StretchMode,
}

fn color_to_hex(c: Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        (c.r.clamp(0.0, 1.0) * 255.0) as u8,
        (c.g.clamp(0.0, 1.0) * 255.0) as u8,
        (c.b.clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn insert_after_svg_opening(svg: &str, fragment: &str) -> String {
    let insert_at = svg.find("<svg").or_else(|| svg.find("<SVG")).map(|start| {
        let after_tag = &svg[start..];
        after_tag.find('>').map(|gt| start + gt + 1).unwrap_or(0)
    }).unwrap_or(0);
    let mut result = String::with_capacity(svg.len() + fragment.len());
    result.push_str(&svg[..insert_at]);
    result.push_str(fragment);
    result.push_str(&svg[insert_at..]);
    result
}

pub fn load_svg_bind_group(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    svg: &Svg,
) -> Result<wgpu::BindGroup, anyhow::Error> {
    let svg_data = std::fs::read(&svg.path)?;

    let raw = String::from_utf8(svg_data)?;

    let mut style = String::new();
    if let Some(c) = svg.fill {
        style.push_str(&format!("fill:{}!important;", color_to_hex(c)));
    }
    if let Some(c) = svg.stroke {
        style.push_str(&format!("stroke:{}!important;", color_to_hex(c)));
    }
    if let Some(w) = svg.stroke_width {
        style.push_str(&format!("stroke-width:{}!important;", w));
    }
    let modified = if style.is_empty() {
        raw
    } else {
        insert_after_svg_opening(&raw, &format!("<style>*{{{style}}}</style>"))
    };

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(modified.as_bytes(), &opt)
        .with_context(|| format!("while parsing the modified svg: {:?}", modified))?;

    let svg_size = tree.size();
    let svg_w = svg_size.width() as f32;
    let svg_h = svg_size.height() as f32;

    let (render_w, render_h) = match svg.stretch {
        StretchMode::Fill => (svg.size.x as u32, svg.size.y as u32),
        StretchMode::Fit => {
            let img_aspect = svg_w / svg_h;
            let quad_aspect = svg.size.x / svg.size.y;
            let (w, h) = if img_aspect > quad_aspect {
                (svg.size.x as u32, (svg.size.x / img_aspect) as u32)
            } else {
                ((svg.size.y * img_aspect) as u32, svg.size.y as u32)
            };
            (w.max(1), h.max(1))
        }
    };

    let mut pixmap = tiny_skia::Pixmap::new(render_w, render_h)
        .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap {}x{}", render_w, render_h))?;

    let scale_x = render_w as f32 / svg_w;
    let scale_y = render_h as f32 / svg_h;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    let pixels = pixmap.take();

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("svg_tex_{}", svg.path)),
        size: wgpu::Extent3d {
            width: render_w,
            height: render_h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(render_w * 4),
            rows_per_image: Some(render_h),
        },
        wgpu::Extent3d {
            width: render_w,
            height: render_h,
            depth_or_array_layers: 1,
        },
    );

    let texture_view = texture.create_view(&Default::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("svg_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("svg_loader_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("svg_bg_{}", svg.path)),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    Ok(bind_group)
}
