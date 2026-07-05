use std::sync::LazyLock;

use anyhow::Result;

use glam::{Vec2, vec2, vec3};
use log::{LevelFilter, info};
use scal::{
    anim_object::text::code::{
        CodeAnimationStyle, Syntax,
        theme::{Base16, Theme},
    },
    prelude::*,
    projection::Camera,
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Info;
const THEME: LazyLock<Theme> = LazyLock::new(|| {
    Theme::from_base16(Base16 {
        colors: [
            0x11121d.into(),
            0x1A1B2A.into(),
            0x212234.into(),
            0x282c34.into(),
            0x4a5057.into(),
            0xa0a8cd.into(),
            0xa0a8cd.into(),
            0xa0a8cd.into(),
            0xee6d85.into(),
            0xf6955b.into(),
            0xd7a65f.into(),
            0x95c561.into(),
            0x38a89d.into(),
            0x7199ee.into(),
            0xa485dd.into(),
            0x773440.into(),
        ],
    })
});
pub const CANVAS_SIZE: Vec2 = vec2(1920., 1080.);
#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = colog::default_builder();
    builder.filter_level(LEVEL_FILTER);
    builder.init();
    let handle = Handle::current();

    let encoding_settings = scal::encoder::EncodingSettings {
        output_path: "test.mov".to_string(),
        codec_type: scal::encoder::CodecType::PRORES,
    };
    let rendering_settings = scal::renderer::RenderingSettings {
        camera: Camera::new(CANVAS_SIZE, Vec2::ZERO, 1.),
        background_color: Color::new(0.8, 0.8, 0.8, 0.),
        buffer_count: 3,
        width: 1920,
        height: 1080,
        fps: 60,
    };
    let curve = AnimationCurve::Linear;
    let code = code(
        transform(CANVAS_SIZE.extend(0.) / 2. - vec3(500., 500., 0.)),
        r#"
const THEME: LazyLock<Theme> = LazyLock::new(|| {
    Theme::from_base16(Base16 {
        colors: [
            0x11121d.into(),
            0x1A1B2A.into(),
        ],
    })
});
        "#
        .to_string(),
        THEME.to_owned(),
        "SF Pro Display Bold".to_string(),
        scal::anim_object::text::Align::Center,
        10.,
        Syntax::Rust,
        vec![],
    );
    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            code.instantiate(),
            code.add_lines(
                r#"
fn copy_texture_to_buffer(
    encoder_send: Sender<encoder::EncoderComunication>,
    queue: &wgpu::Queue,
    settings: RenderingSettings,
    device: &Device,
    texture: &Texture,
    slot: &readback::Slot,
) -> Result<()> {
    let id = slot.id;
    let mut cmd = device.create_command_encoder(&Default::default());
    cmd.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &slot.buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(settings.width * BYTES_PER_PIXEL),
                rows_per_image: Some(settings.height),
            },
        },
        wgpu::Extent3d {
            width: settings.width,
            height: settings.height,
            depth_or_array_layers: 1,
        },
    );
    Ok(())
}
                "#
                .into(),
                1,
                curve.clone(),
                5.0,
                CodeAnimationStyle::TypeWriter,
            ),
            wait(0.5),
            code.add_lines(
                "let x = 1;".into(),
                1,
                curve.clone(),
                1.0,
                CodeAnimationStyle::TypeWriter,
            ),
            // code.add_lines(
            //     "let x = 1;".into(),
            //     1,
            //     curve.clone(),
            //     1.0,
            //     CodeAnimationStyle::TypeWriter,
            // ),
            // wait(1.5),
            // code.modify_line(
            //     2,
            //     "new line text".into(),
            //     curve.clone(),
            //     0.5,
            //     CodeAnimationStyle::TypeWriter,
            // ),
            // wait(1.5),
            // code.remove_lines(1..4, curve.clone(), 0.8, CodeAnimationStyle::Fold),
            // wait(1.5),
            // code.transform
            //     .position_to(Vec2::ZERO, 1., AnimationCurve::Linear),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
