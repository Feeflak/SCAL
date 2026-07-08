use crate::types::Seconds;
use anyhow::{Context, Result};
use ffmpeg_next as ffmpeg;
use log::debug;

pub const OUTPUT_SAMPLE_RATE: u32 = 48000;
const MAX_AUDIO_DURATION: Seconds = 3600.0;

#[derive(Clone, Debug)]
pub struct ScheduledSound {
    pub path: String,
    pub volume: f32,
    pub pitch: f32,
    pub start_time: Seconds,
    pub seek_offset: Seconds,
    pub duration: Seconds,
}

pub struct AudioEngine {
    sounds: Vec<ScheduledSound>,
}

impl AudioEngine {
    pub fn new(sounds: Vec<ScheduledSound>) -> Self {
        Self { sounds }
    }

    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }

    pub fn mix(&self) -> Result<Vec<f32>> {
        if self.sounds.is_empty() {
            return Ok(vec![]);
        }

        let mut decoded_sounds: Vec<(Vec<f32>, Seconds)> = vec![];
        let mut total_duration = 0.0_f32;

        for sound in &self.sounds {
            debug!("mixing sound: {} at t={}", sound.path, sound.start_time);
            let samples = self.decode_and_transform(sound)?;
            if samples.is_empty() {
                debug!("  -> decoded empty, skipping");
                continue;
            }
            let actual_duration = (samples.len() / 2) as Seconds / OUTPUT_SAMPLE_RATE as Seconds;
            let end = sound.start_time + actual_duration;
            if end > total_duration {
                total_duration = end;
            }
            decoded_sounds.push((samples, sound.start_time.max(0.0)));
        }

        if total_duration <= 0.0 {
            return Ok(vec![]);
        }
        total_duration = total_duration.min(MAX_AUDIO_DURATION);
        let total_samples = (total_duration * OUTPUT_SAMPLE_RATE as f32).ceil() as usize;
        let mut mix_buffer = vec![0.0_f32; total_samples * 2];

        for (samples, start_time) in &decoded_sounds {
            let start_frame = (start_time * OUTPUT_SAMPLE_RATE as f32) as usize;
            if start_frame >= total_samples {
                continue;
            }
            let start_sample = start_frame * 2;
            for (i, &sample) in samples.iter().enumerate() {
                let mix_idx = start_sample + i;
                if mix_idx >= total_samples * 2 {
                    break;
                }
                mix_buffer[mix_idx] = (mix_buffer[mix_idx] + sample).clamp(-1.0, 1.0);
            }
        }

        Ok(mix_buffer)
    }

    fn decode_and_transform(&self, sound: &ScheduledSound) -> Result<Vec<f32>> {
        let mut ictx = ffmpeg::format::input(&sound.path)
            .with_context(|| format!("failed to open audio file: {}", sound.path))?;

        let input_stream = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .context("no audio stream found")?;

        let input_stream_index = input_stream.index();
        let codec = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
        let mut decoder = codec.decoder().audio().context("failed to create audio decoder")?;

        let input_layout = decoder.channel_layout();
        let file_sample_rate = decoder.rate();
        let file_format = decoder.format();

        let mut resampler = ffmpeg::software::resampling::Context::get(
            file_format,
            input_layout,
            file_sample_rate,
            ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed),
            ffmpeg::ChannelLayout::default(2),
            OUTPUT_SAMPLE_RATE,
        )?;

        let mut pcm_stereo: Vec<f32> = vec![];

        for (stream, packet) in ictx.packets() {
            if stream.index() != input_stream_index {
                continue;
            }
            decoder.send_packet(&packet)?;

            let mut decoded = ffmpeg::frame::Audio::empty();
            loop {
                match decoder.receive_frame(&mut decoded) {
                    Ok(()) => {
                        let mut converted = ffmpeg::frame::Audio::empty();
                        resampler.run(&decoded, &mut converted)?;

                        let n = converted.samples();
                        let data = converted.data(0);
                        let float_data = unsafe {
                            std::slice::from_raw_parts(
                                data.as_ptr() as *const f32,
                                n * 2,
                            )
                        };
                        pcm_stereo.extend_from_slice(float_data);
                    }
                    Err(ffmpeg::Error::Eof) => break,
                    Err(_) => break,
                }
            }
        }

        decoder.send_eof()?;
        let mut remaining = ffmpeg::frame::Audio::empty();
        loop {
            match decoder.receive_frame(&mut remaining) {
                Ok(()) => {
                    let mut converted = ffmpeg::frame::Audio::empty();
                    resampler.run(&remaining, &mut converted)?;

                    let n = converted.samples();
                    let data = converted.data(0);
                    let float_data = unsafe {
                        std::slice::from_raw_parts(
                            data.as_ptr() as *const f32,
                            n * 2,
                        )
                    };
                    pcm_stereo.extend_from_slice(float_data);
                }
                Err(ffmpeg::Error::Eof) => break,
                Err(_) => break,
            }
        }

        if pcm_stereo.is_empty() {
            return Ok(vec![]);
        }

        let total_frames = pcm_stereo.len() / 2;
        if sound.seek_offset > 0.0 {
            let seek_frames = (sound.seek_offset * OUTPUT_SAMPLE_RATE as f32) as usize;
            if seek_frames >= total_frames {
                return Ok(vec![]);
            }
            if sound.duration > 0.0 {
                let dur_frames = (sound.duration * OUTPUT_SAMPLE_RATE as f32) as usize;
                if seek_frames + dur_frames > total_frames {
                    return Ok(vec![]);
                }
            }
            let skip = seek_frames * 2;
            pcm_stereo.drain(0..skip);
        }

        let mut pcm_stereo =
            apply_pitch_and_volume(pcm_stereo, sound.pitch, sound.volume);

        if sound.duration > 0.0 {
            let target_samples = (sound.duration * OUTPUT_SAMPLE_RATE as f32) as usize * 2;
            if pcm_stereo.len() > target_samples {
                pcm_stereo.truncate(target_samples);
            }
        }

        Ok(pcm_stereo)
    }
}

fn apply_pitch_and_volume(
    samples: Vec<f32>,
    pitch: f32,
    volume: f32,
) -> Vec<f32> {
    if (pitch - 1.0).abs() < f32::EPSILON && (volume - 1.0).abs() < f32::EPSILON {
        return samples;
    }

    if (pitch - 1.0).abs() < f32::EPSILON {
        return samples.iter().map(|&s| s * volume).collect();
    }

    let input_len = samples.len() / 2;
    let ratio = 1.0 / pitch;
    let output_len = (input_len as f32 * ratio) as usize;
    let mut out = Vec::with_capacity(output_len * 2);

    for i in 0..output_len {
        let src_f = i as f32 * pitch;
        let src_i = src_f as usize;
        let frac = src_f - src_i as f32;

        if src_i + 1 >= input_len {
            break;
        }

        let si = src_i * 2;
        let l = samples[si] * (1.0 - frac) + samples[si + 2] * frac;
        let r = samples[si + 1] * (1.0 - frac) + samples[si + 3] * frac;
        out.push(l * volume);
        out.push(r * volume);
    }

    out
}
