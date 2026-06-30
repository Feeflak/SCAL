use anyhow::{Context, Result};

use log::{LevelFilter, info};
use scal::anim_object::{AnimObject, Transform, primitive_shapes::Square, wait};
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
        buffer_count: 3,
        width: 1920,
        height: 1080,
        fps: 60,
    };
    let square = AnimObject::Square(
        Square {
            size: (0.8, 0.8),
            corner_radius: 1.,
            color: (1., 1., 1., 1.),
        },
        Transform::new(vec![], (0.0, 0.0), 0., 1., 0.),
    );
    scal::run_loop(
        &handle,
        encoding_settings,
        rendering_settings,
        vec![wait(0.1), square.instantiate(), wait(1.0)],
    )
    .await?;
    info!("Hello, world!");
    Ok(())
}
