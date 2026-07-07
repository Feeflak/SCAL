use anyhow::Result;
use ffmpeg::software::scaling::{context::Context, flag::Flags};

use ffmpeg_next as ffmpeg;
use log::{debug, info};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::renderer::RenderingSettings;
use crate::sfx::{self, AudioEngine};

pub(crate) struct Encoder {
    output: ffmpeg::format::context::Output,
    encoder: ffmpeg::encoder::Video,
    stream_index: usize,

    width: u32,
    height: u32,
    frame_index: i64,
    scaler: Option<Context>,
    rgba_frame: ffmpeg::frame::Video,
    yuv_frame: ffmpeg::frame::Video,
    use_nv12: bool,

    total_encode_time: std::time::Duration,
    timing_memcpy: std::time::Duration,
    timing_convert: std::time::Duration,
    timing_nvenc: std::time::Duration,
    frame_count: u64,

    audio_engine: Option<AudioEngine>,
    audio_stream_index: Option<usize>,
    audio_encoder: Option<ffmpeg::encoder::Audio>,
}

impl Encoder {
    pub fn new(
        settings: EncodingSettings,
        mut output: ffmpeg::format::context::Output,
        width: u32,
        height: u32,
        fps: u32,
        audio_engine: Option<AudioEngine>,
    ) -> Result<Self> {
        ffmpeg::init()?;

        let is_nvenc = matches!(settings.codec_type, CodecType::H264Nvenc);
        let use_nv12 = matches!(settings.codec_type, CodecType::H264 | CodecType::H264Nvenc);

        let global_header = output
            .format()
            .flags()
            .contains(ffmpeg::format::Flags::GLOBAL_HEADER);

        let encoder_name: &str = match settings.codec_type {
            CodecType::H264Nvenc => "h264_nvenc",
            CodecType::H264 => "libx264",
            CodecType::PRORES => "prores",
        };
        let mut stream = output.add_stream(encoder_name)?;

        let mut context = ffmpeg::codec::context::Context::new().encoder().video()?;

        context.set_width(width);
        context.set_height(height);
        let pixel_format = if use_nv12 {
            ffmpeg::format::Pixel::NV12
        } else {
            ffmpeg::format::Pixel::YUV444P10LE
        };
        context.set_format(pixel_format);
        context.set_time_base((1, fps as i32));

        if global_header {
            context.set_flags(ffmpeg::codec::Flags::GLOBAL_HEADER);
        }

        let rgba_frame = ffmpeg::frame::Video::new(ffmpeg::format::Pixel::RGBA, width, height);
        let yuv_frame = ffmpeg::frame::Video::new(pixel_format, width, height);
        let scaler = if use_nv12 {
            None
        } else {
            Some(Context::get(
                ffmpeg::format::Pixel::RGBA,
                width,
                height,
                pixel_format,
                width,
                height,
                Flags::BILINEAR,
            )?)
        };
        let encoder = if is_nvenc {
            let opts: ffmpeg::Dictionary = [
                ("preset", "p1"),
                ("tune", "ll"),
                ("rc", "constqp"),
                ("qp", "23"),
            ]
            .iter()
            .collect();
            context.open_as_with("h264_nvenc", opts)?
        } else {
            context.open_as(encoder_name)?
        };

        stream.set_parameters(&encoder);
        stream.set_time_base((1, fps as i32));

        let stream_index = stream.index();

        let (audio_stream_index, audio_encoder) = setup_audio_stream(&mut output, &audio_engine)?;

        output.write_header()?;

        Ok(Self {
            scaler,
            rgba_frame,
            yuv_frame,
            use_nv12,
            output,
            encoder,
            stream_index,
            width,
            height,
            frame_index: 0,
            total_encode_time: std::time::Duration::ZERO,
            timing_memcpy: std::time::Duration::ZERO,
            timing_convert: std::time::Duration::ZERO,
            timing_nvenc: std::time::Duration::ZERO,
            frame_count: 0,
            audio_engine,
            audio_stream_index,
            audio_encoder,
        })
    }

    fn push_frame(&mut self, bytes: &[u8]) -> Result<()> {
        let t0 = std::time::Instant::now();

        self.frame_index += 1;

        let t_mem = std::time::Instant::now();
        if self.use_nv12 {
            let y_size = self.width as usize * self.height as usize;
            let expected = y_size + y_size / 2;
            assert_eq!(bytes.len(), expected);

            self.yuv_frame.set_pts(Some(self.frame_index - 1));

            let y_stride = self.yuv_frame.stride(0) as usize;
            let uv_stride = self.yuv_frame.stride(1) as usize;
            let row_bytes = self.width as usize;

            let dst_y = self.yuv_frame.data_mut(0);
            if y_stride == row_bytes {
                dst_y[..y_size].copy_from_slice(&bytes[..y_size]);
            } else {
                for y in 0..self.height as usize {
                    let src = y * row_bytes;
                    let dst = y * y_stride;
                    dst_y[dst..dst + row_bytes].copy_from_slice(&bytes[src..src + row_bytes]);
                }
            }

            let uv_size = y_size / 2;
            let dst_uv = self.yuv_frame.data_mut(1);
            if uv_stride == row_bytes {
                dst_uv[..uv_size].copy_from_slice(&bytes[y_size..y_size + uv_size]);
            } else {
                for y in 0..(self.height as usize / 2) {
                    let src = y_size + y * row_bytes;
                    let dst = y * uv_stride;
                    dst_uv[dst..dst + row_bytes].copy_from_slice(&bytes[src..src + row_bytes]);
                }
            }
        } else {
            let expected = self.width as usize * self.height as usize * 4;
            assert_eq!(bytes.len(), expected);

            self.rgba_frame.set_pts(Some(self.frame_index - 1));

            let row_bytes = self.width as usize * 4;
            let stride = self.rgba_frame.stride(0) as usize;
            let dst = self.rgba_frame.data_mut(0);
            if stride == row_bytes {
                dst[..bytes.len()].copy_from_slice(bytes);
            } else {
                for y in 0..self.height as usize {
                    let src_start = y * row_bytes;
                    let dst_start = y * stride;
                    dst[dst_start..dst_start + row_bytes]
                        .copy_from_slice(&bytes[src_start..src_start + row_bytes]);
                }
            }
        }
        self.timing_memcpy += t_mem.elapsed();

        let t_conv = std::time::Instant::now();
        if let Some(scaler) = &mut self.scaler {
            scaler.run(&self.rgba_frame, &mut self.yuv_frame)?;
        }
        self.timing_convert += t_conv.elapsed();

        let t_nv = std::time::Instant::now();
        let send_frame = if self.scaler.is_some() {
            &self.yuv_frame
        } else if self.use_nv12 {
            &self.yuv_frame
        } else {
            &self.rgba_frame
        };
        self.encoder.send_frame(send_frame)?;

        let mut packet = ffmpeg::Packet::empty();

        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.stream_index);
            packet.rescale_ts(
                self.encoder.time_base(),
                self.output.stream(self.stream_index).unwrap().time_base(),
            );
            packet.write_interleaved(&mut self.output)?;
        }
        self.timing_nvenc += t_nv.elapsed();

        self.total_encode_time += t0.elapsed();
        self.frame_count += 1;

        Ok(())
    }
    fn write_audio(&mut self) -> Result<()> {
        let Some(ref engine) = self.audio_engine else {
            return Ok(());
        };
        let Some(audio_encoder) = self.audio_encoder.as_mut() else {
            return Ok(());
        };
        let Some(audio_stream_index) = self.audio_stream_index else {
            return Ok(());
        };
        if engine.is_empty() {
            return Ok(());
        }

        let pcm = engine.mix()?;
        if pcm.is_empty() {
            return Ok(());
        }

        let total_samples = pcm.len() / 2;

        let frame_size = 1024usize;
        let mut offset = 0usize;
        let mut pts: i64 = 0;

        while offset < total_samples {
            let remaining = total_samples - offset;
            let samples_this_frame = remaining.min(frame_size);
            let mut frame = ffmpeg::frame::Audio::new(
                ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar),
                samples_this_frame,
                ffmpeg::ChannelLayout::STEREO,
            );
            frame.set_pts(Some(pts));

            for ch in 0..2 {
                let plane = frame.plane_mut::<f32>(ch);
                for i in 0..samples_this_frame {
                    plane[i] = pcm[(offset + i) * 2 + ch];
                }
            }

            audio_encoder.send_frame(&frame)?;

            let mut packet = ffmpeg::Packet::empty();
            while audio_encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(audio_stream_index);
                packet.rescale_ts(
                    audio_encoder.time_base(),
                    self.output.stream(audio_stream_index).unwrap().time_base(),
                );
                packet.write_interleaved(&mut self.output)?;
            }

            offset += samples_this_frame;
            pts += samples_this_frame as i64;
        }

        audio_encoder.send_eof()?;
        let mut packet = ffmpeg::Packet::empty();
        while audio_encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(audio_stream_index);
            packet.rescale_ts(
                audio_encoder.time_base(),
                self.output.stream(audio_stream_index).unwrap().time_base(),
            );
            packet.write_interleaved(&mut self.output)?;
        }

        debug!("audio encoding done, {} samples", total_samples);
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.write_audio()?;

        self.encoder.send_eof()?;
        let mut packet = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(self.stream_index);
            packet.rescale_ts(
                self.encoder.time_base(),
                self.output.stream(self.stream_index).unwrap().time_base(),
            );
            packet.write_interleaved(&mut self.output)?;
        }
        self.output.write_trailer()?;

        let fc = self.frame_count as f64;
        info!(
            "Encode total | total: {:.3}s  | avg: {:.1}ms  | frames: {}",
            self.total_encode_time.as_secs_f64(),
            self.total_encode_time.as_secs_f64() / fc * 1000.0,
            self.frame_count,
        );
        info!(
            "  memcpy     | total: {:.3}s  | avg: {:.1}ms",
            self.timing_memcpy.as_secs_f64(),
            self.timing_memcpy.as_secs_f64() / fc * 1000.0,
        );
        info!(
            "  convert    | total: {:.3}s  | avg: {:.1}ms",
            self.timing_convert.as_secs_f64(),
            self.timing_convert.as_secs_f64() / fc * 1000.0,
        );
        info!(
            "  nvenc api  | total: {:.3}s  | avg: {:.1}ms",
            self.timing_nvenc.as_secs_f64(),
            self.timing_nvenc.as_secs_f64() / fc * 1000.0,
        );

        Ok(())
    }
    async fn start_loop(
        &mut self,
        mut buffer_to_encode_rc: Receiver<EncoderComunication>,
        free_buffers_sd: Sender<usize>,
    ) {
        debug!("Start Encoding Loop");
        while let Some(communication) = buffer_to_encode_rc.recv().await {
            debug!("Received For Encoding");
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
        debug!("Finished Encoding");
        self.finish().expect("while finishing");
    }
}
fn setup_audio_stream(
    output: &mut ffmpeg::format::context::Output,
    audio_engine: &Option<AudioEngine>,
) -> Result<(Option<usize>, Option<ffmpeg::encoder::Audio>)> {
    let has_audio = audio_engine.as_ref().is_some_and(|e| !e.is_empty());
    if !has_audio {
        return Ok((None, None));
    }

    let mut audio_stream = output.add_stream("aac")?;

    let mut audio_ctx = ffmpeg::codec::context::Context::new().encoder().audio()?;
    audio_ctx.set_rate(sfx::OUTPUT_SAMPLE_RATE as i32);
    audio_ctx.set_format(ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Planar));
    audio_ctx.set_channel_layout(ffmpeg::ChannelLayout::default(2));
    audio_ctx.set_time_base((1, sfx::OUTPUT_SAMPLE_RATE as i32));

    let encoder = audio_ctx.open_as("aac")?;
    let stream_index = audio_stream.index();
    audio_stream.set_parameters(&encoder);
    audio_stream.set_time_base((1, sfx::OUTPUT_SAMPLE_RATE as i32));

    Ok((Some(stream_index), Some(encoder)))
}

pub fn start_encoding_task(
    encoding_settings: EncodingSettings,
    tokio_handle: &tokio::runtime::Handle,
    rendering_settings: RenderingSettings,
    mut encoder_rec: Receiver<EncoderComunication>,
    renderer_send: Sender<usize>,
    audio_engine: Option<AudioEngine>,
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
            audio_engine,
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
    H264Nvenc,
    PRORES,
}
pub enum EncoderComunication {
    Finish,
    FrameData { bytes: Vec<u8>, id: usize },
}
pub struct EncodingSettings {
    pub output_path: String,
    pub codec_type: CodecType,
}
