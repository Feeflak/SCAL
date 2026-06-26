use anyhow::{Context, Result};
async fn copy_data() {}
pub(crate) async fn draw_frame() {}

pub struct RenderingSettings {
    pub tokio_handle: tokio::runtime::Handle,

    pub buffer_count: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}
