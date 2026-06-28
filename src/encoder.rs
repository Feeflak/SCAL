use anyhow::Result;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

use ffmpeg_next as ffmpeg;
use log::{info, trace};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::renderer::RenderingSettings;

pub(crate) struct Encoder {
    output: ffmpeg::format::context::Output,
    encoder: ffmpeg::encoder::Video,
    stream_index: usize,

    width: u32,
    height: u32,
    frame_index: i64,
    scaler: Context,
}

impl Encoder {
    pub fn new(
        settings: EncodingSettings,
        mut output: ffmpeg::format::context::Output,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<Self> {
        ffmpeg::init()?;

        let codec = ffmpeg::encoder::find(settings.codec_type.into())
            .ok_or(ffmpeg::Error::EncoderNotFound)?;

        let global_header = output
            .format()
            .flags()
            .contains(ffmpeg::format::Flags::GLOBAL_HEADER);
        let mut stream = output.add_stream(codec)?;

        let mut context = ffmpeg::codec::context::Context::new().encoder().video()?;

        context.set_width(width);
        context.set_height(height);
        match settings.codec_type {
            CodecType::H264 => context.set_format(ffmpeg::format::Pixel::YUV420P),
            CodecType::PRORES => context.set_format(ffmpeg::format::Pixel::YUV444P10LE),
        }
        context.set_time_base((1, fps as i32));

        if global_header {
            context.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }

        let scaler = Context::get(
            ffmpeg::format::Pixel::RGBA,
            width,
            height,
            context.format(),
            width,
            height,
            Flags::BILINEAR,
        )?;
        let encoder = context.open_as(codec)?;

        stream.set_parameters(&encoder);
        stream.set_time_base((1, fps as i32));

        let stream_index = stream.index();

        output.write_header()?;

        Ok(Self {
            scaler,
            output: output,
            encoder,
            stream_index,
            width,
            height,
            frame_index: 0,
        })
    }

    fn push_frame(&mut self, bytes: &[u8]) -> Result<()> {
        assert_eq!(bytes.len(), self.width as usize * self.height as usize * 4);
        trace!("push_frame 1");

        let mut rgba =
            ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGBA, self.width, self.height);

        // Ensure frame is writable / properly allocated
        rgba.set_pts(Some(self.frame_index));
        self.frame_index += 1;

        let stride = rgba.stride(0);
        let dst = rgba.data_mut(0);

        let row_bytes = self.width as usize * 4;

        // Copy row-by-row to respect FFmpeg alignment
        for y in 0..self.height as usize {
            let src_start = y * row_bytes;
            let src_end = src_start + row_bytes;

            let dst_start = y * stride;
            let dst_end = dst_start + row_bytes;

            dst[dst_start..dst_end].copy_from_slice(&bytes[src_start..src_end]);
        }

        trace!("push_frame 2");

        let mut frame = ffmpeg::frame::Video::empty();
        self.scaler.run(&rgba, &mut frame)?;

        frame.set_pts(Some(self.frame_index - 1));

        self.encoder.send_frame(&frame)?;

        trace!("push_frame 3");

        let mut packet = ffmpeg::Packet::empty();

        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.stream_index);
            packet.rescale_ts(
                self.encoder.time_base(),
                self.output.stream(self.stream_index).unwrap().time_base(),
            );
            packet.write_interleaved(&mut self.output)?;
        }

        Ok(())
    }
    fn finish(&mut self) -> Result<()> {
        self.encoder.send_eof()?;
        self.output.write_trailer()?;

        Ok(())
    }
    async fn start_loop(
        &mut self,
        mut buffer_to_encode_rc: Receiver<EncoderComunication>,
        free_buffers_sd: Sender<usize>,
    ) {
        trace!("Start Encoding Loop");
        while let Some(communication) = buffer_to_encode_rc.recv().await {
            trace!("Received For Encoding");
            match communication {
                EncoderComunication::Finish => {
                    break;
                }
                EncoderComunication::FrameData { bytes, id } => {
                    self.push_frame(&bytes)
                        .expect("while pushing a new frame in the encoding loop");
                    free_buffers_sd
                        .try_send(id)
                        .expect("while sending free frame index in the encoding loop");
                }
            }
        }
        trace!("Finished Encoding");
        self.finish().expect("while finishing");
    }
}
pub fn start_encoding_task(
    encoding_settings: EncodingSettings,
    tokio_handle: &tokio::runtime::Handle,
    rendering_settings: RenderingSettings,
    mut encoder_rec: Receiver<EncoderComunication>,
    renderer_send: Sender<usize>,
) -> Result<()> {
    tokio_handle.spawn_blocking(move || {
        let output =
            ffmpeg::format::output(&encoding_settings.output_path).expect("invalid output path");

        let mut encoder = Encoder::new(
            encoding_settings,
            output,
            rendering_settings.width,
            rendering_settings.height,
            rendering_settings.fps,
        )
        .unwrap();

        // Need a blocking receiver instead of tokio Receiver
        while let Some(msg) = encoder_rec.blocking_recv() {
            match msg {
                EncoderComunication::Finish => break,

                EncoderComunication::FrameData { bytes, id } => {
                    encoder.push_frame(&bytes).expect("encoding frame");

                    renderer_send
                        .blocking_send(id)
                        .expect("sending free buffer");
                }
            }
        }

        encoder.finish().expect("finishing encoder");
        info!("encoding finished!");
    });

    Ok(())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    H264,
    PRORES,
}

impl From<CodecType> for ffmpeg_next::codec::Id {
    fn from(value: CodecType) -> Self {
        match value {
            CodecType::H264 => ffmpeg_next::codec::Id::H264,
            CodecType::PRORES => ffmpeg_next::codec::Id::PRORES,
        }
    }
}
pub enum EncoderComunication {
    Finish,
    FrameData { bytes: Vec<u8>, id: usize },
}
pub struct EncodingSettings {
    pub output_path: String,
    pub codec_type: CodecType,
}
