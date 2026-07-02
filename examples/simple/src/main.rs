use std::sync::LazyLock;

use anyhow::Result;

use glam::{Vec2, vec2};
use log::{LevelFilter, info};
use scal::{
    anim_object::{
        AnimObject, Transform,
        primitive_shapes::Square,
        text::{
            Align, Text,
            code::{
                Code, Syntax,
                theme::{Base16, Theme},
            },
        },
        wait,
    },
    anim_op::AnimationCurve,
    projection::Camera,
    types::Color,
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
    let code = AnimObject::Code(
        Code {
            theme: THEME.to_owned(),
            source_code: "const t : String = 25;".to_string(),
            //             source_code: r#"
            //     pub fn new(anim: AnimOP) -> Result<Self> {
            //         Ok(Self {
            //             storage: vec![],
            //             anim_op: anim
            //                 .try_into()
            //                 .context("couldn't convert anim_op to animation")?,
            //             time: 0.0,
            //         })
            //     }
            //
            // "#
            //             .to_string(),
            syntax: Syntax::Rust,
            lines: vec![],
            dirty: true,
            font_family: "SF Pro Display Bold".to_string(),
            alignment: Align::Center,
            font_size: 255.,
        },
        Transform::new(vec![], CANVAS_SIZE / 2., 0., Vec2::ONE, 1.),
    );

    let text = AnimObject::Text(
        Text {
            font_family: "SF Pro Display Bold".to_string(),
            alignment: Align::Center,
            value: "const t : String = 25;".to_string(),
            color: Color::BLACK,
            font_size: 55.,
        },
        Transform::new(
            vec![],
            CANVAS_SIZE / 2. + vec2(100., 100.),
            0.,
            Vec2::ONE,
            1.,
        ),
    );
    let square = AnimObject::Square(
        Square {
            size: Vec2::ONE * 500.,
            corner_radius: 1.,
            color: Color::new(0., 0.2, 0.4, 1.),
        },
        Transform::new(vec![], CANVAS_SIZE / 2., 0., Vec2::ONE, 0.),
    );
    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            code.instantiate(),
            text.instantiate(),
            square.instantiate(),
            wait(1.0),
            (square
                .transform()
                .move_local(vec2(0.5, 0.5), 1., AnimationCurve::EaseOutBack)),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
