use anyhow::{Context, Result};
use ffmpeg_next::{self as ffmpeg};

pub(crate) struct Encoder {
    output: ffmpeg::format::context::Output,
    stream_index: usize,
    frame_index: i64,
}

impl Encoder {
    pub fn new(settings: EncodingSettings, width: u32, height: u32, fps: u32) -> Result<Self> {
        ffmpeg::init()?;

        let codec = ffmpeg::encoder::find(codec_type).unwrap();

        let mut stream = output.add_stream(codec)?;

        {
            let mut video = codec.video()?;

            video.set_width(width);
            video.set_height(height);
            video.set_format(ffmpeg::format::Pixel::RGBA);

            video.set_time_base((1, fps as i32));
        }

        output.write_header()?;

        Ok(Self {
            stream_index: stream.index(),
            output,
            frame_index: 0,
        })
    }

    pub fn push_frame(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<()> {
        let mut frame = ffmpeg::util::frame::Video::new(ffmpeg::format::Pixel::RGBA, width, height);

        frame.data_mut(0).copy_from_slice(rgba);

        frame.set_pts(Some(self.frame_index));

        self.frame_index += 1;

        let stream = self.output.stream(self.stream_index);

        let mut packet = ffmpeg::Packet::empty();

        stream.codec().encoder().video()?.send_frame(&frame)?;

        while stream
            .codec()
            .encoder()
            .video()?
            .receive_packet(&mut packet)
            .is_ok()
        {
            packet.set_stream(self.stream_index);

            packet.write_interleaved(&mut self.output)?;
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.output.write_trailer()?;
        Ok(())
    }
}

pub struct EncodingSettings {
    pub output: ffmpeg_next::format::context::Output,
    pub codec_type: ffmpeg_next::codec::Id,
}
