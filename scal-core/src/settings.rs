use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RenderingSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub buffer_count: u32,
    pub text_resolution_multiplier: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodingSettings {
    pub output_path: String,
    pub codec_type: CodecType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CodecType {
    H264,
    H264Nvenc,
    PRORES,
}
