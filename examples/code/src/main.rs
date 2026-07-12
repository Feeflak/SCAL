use glam::{Vec2, vec2};
use scal_core::{CodeAnimationStyle::TypeWriter, prelude::*};

const SOURCE: &str = r"
const THEME: LazyLock<Theme> = LazyLock::new(|| {
    Theme::from_base16(Base16 {
        colors: [
            0x11121d.into(),
            0x1A1B2A.into(),
        ],
    })
});
";

const NEW_LINES: &str = r"
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
";

const WINDOW: Vec2 = vec2(1920., 1080.);
#[scal_ipc::main]
fn main() -> Project {
    let theme = Theme::from_base16(Base16::from_hex([
        0x11121d, 0x1A1B2A, 0x212234, 0x282c34, 0x4a5057, 0xa0a8cd, 0xa0a8cd, 0xa0a8cd, 0xee6d85,
        0xf6955b, 0xd7a65f, 0x95c561, 0x38a89d, 0x7199ee, 0xa485dd, 0x773440,
    ]));

    let code = code()
        .source(SOURCE)
        .font_family("SF Pro Display")
        .font_size(20.)
        .syntax(Syntax::Rust)
        .theme(theme)
        .pos(WINDOW / 2.)
        .build();

    Project {
        scene_settings: SceneSettings {
            background_color: Color::new(0.8, 0.8, 0.8, 0.),
            camera: Camera::new(WINDOW, Vec2::ZERO, 1.),
            default_theme: Theme::default(),
        },
        timeline: timeline![
            code.instantiate(),
            code.add_lines()
                .str(NEW_LINES)
                .over(5.s())
                .style(TypeWriter),
            code.highlight()
                .lines(3..6)
                .color(Color::new(1.0, 1.0, 0.0, 0.3))
                .over(1.s())
                .ease(Ease::InOutCubic),
            wait(500.ms()),
            code.add_lines()
                .from_line(5)
                .str("let x = 1;")
                .over(1.s())
                .style(TypeWriter),
        ],
    }
}
