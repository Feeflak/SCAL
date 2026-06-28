use anyhow::{Context, Result};

use log::info;
use tokio::runtime::Handle;

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();
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
    scal::run_loop(&handle, encoding_settings, rendering_settings, 300).await?;
    info!("Hello, world!");
    Ok(())
}
