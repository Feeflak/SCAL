use anyhow::Result;

use log::{LevelFilter, info};
use scal::{
    anim_object::{
        AnimObject, Transform,
        primitive_shapes::Square,
        text::{Align, Text},
        wait,
    },
    anim_op::AnimationCurve,
    types::{Color, Vec2},
};
use tokio::runtime::Handle;

const LEVEL_FILTER: LevelFilter = LevelFilter::Debug;
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
        background_color: Color::new(0.8, 0.8, 0.8, 0.),
        buffer_count: 3,
        width: 1920,
        height: 1080,
        fps: 60,
    };
    let text = AnimObject::Text(
        Text {
            font_family: "SF Pro Display Bold".to_string(),
            alignment: Align::Center,
            value: "Texting to you LOL".to_string(),
            color: Color::WHITE,
            font_size: 55.,
        },
        Transform::new(vec![], Vec2::new(0.5, 0.5), 0., 1., 1.),
    );
    let square = AnimObject::Square(
        Square {
            size: Vec2::new(0.8, 0.8),
            corner_radius: 1.,
            color: Color::new(0., 0.2, 0.4, 1.),
        },
        Transform::new(vec![], Vec2::new(0.0, 0.0), 0., 1., 0.),
    );
    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![
            text.instantiate(),
            square.instantiate(),
            wait(1.0),
            (square
                .transform()
                .move_local(Vec2::new(0.5, 0.5), 1., AnimationCurve::EaseOutBack)),
        ],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
