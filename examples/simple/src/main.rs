use std::sync::LazyLock;

use anyhow::Result;

use glam::{Vec2, Vec3, Vec3Swizzles, vec2, vec3};
use log::{LevelFilter, info};
use scal::{
    anim_object::{
        AnimObject, Transform,
        image::{Image as Img, StretchMode},
        primitive_shapes::{Circle, Polygon, Rectangle},
        text::{
            Align, Text,
            code::{
                Code, Syntax,
                theme::{Base16, Theme},
            },
        },
        wait,
    },
    anim_op::{AnimationCurve, all},
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
            font_size: 55.,
        },
        Transform::new(None, CANVAS_SIZE.extend(0.) / 2., 0., Vec2::ONE),
    );

    let rect = AnimObject::Square(
        Rectangle {
            size: vec2(600., 400.),
            corner_radius: 40.,
            color: Color::new(0., 0.2, 0.4, 1.),
        },
        Transform::new(None, vec3(400., 540., 0.), 0., Vec2::ONE),
    );

    let circle = AnimObject::Circle(
        Circle {
            radius: 200.,
            color: Color::new(0.8, 0.2, 0.2, 1.),
        },
        Transform::new(None, vec3(1200., 500., 0.), 0., Vec2::ONE),
    );

    let hex = AnimObject::Polygon(
        Polygon {
            radius: 180.,
            sides: 6,
            color: Color::new(0.2, 0.7, 0.3, 1.),
        },
        Transform::new(None, vec3(800., 300., 0.), 0., Vec2::ONE),
    );

    let triangle = AnimObject::Polygon(
        Polygon {
            radius: 150.,
            sides: 3,
            color: Color::new(0.9, 0.6, 0.1, 1.),
        },
        Transform::new(None, vec3(1600., 700., 0.), 0., Vec2::ONE),
    );

    let text = AnimObject::Text(
        Text {
            font_family: "SF Pro Display Bold".to_string(),
            alignment: Align::Center,
            value: "const t : String = 25;".to_string(),
            color: Color::BLACK,
            font_size: 55.,
        },
        Transform::new(Some(&rect), Vec3::ZERO, 0., Vec2::ONE),
    );
    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            code.instantiate(),
            text.instantiate(),
            rect.instantiate(),
            circle.instantiate(),
            hex.instantiate(),
            triangle.instantiate(),
            wait(1.0),
            all(vec![
                triangle
                    .transform()
                    .move_local(vec2(350., 800.), 1., AnimationCurve::EaseOutBack),
                rect.transform()
                    .move_local(vec2(0.5, 0.5), 1., AnimationCurve::EaseOutBack),
            ]),
            (rect
                .transform()
                .move_local(CANVAS_SIZE / 2., 1., AnimationCurve::EaseInOutBack)),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
