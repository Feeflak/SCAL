use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

struct PlaybackAnchor {
    ts: cpal::StreamInstant,
    pos: usize,
}

pub struct AudioPlayer {
    buffer: Arc<Vec<f32>>,
    position: Arc<AtomicUsize>,
    paused: Arc<AtomicBool>,
    sample_rate: u32,
    _stream: cpal::Stream,
    play_frames: Arc<AtomicU64>,
    anchor: Arc<Mutex<Option<PlaybackAnchor>>>,
    latency_frames: Arc<AtomicU64>,
}

impl AudioPlayer {
    pub fn new(buffer: Vec<f32>, sample_rate: u32) -> Result<Self> {
        let buffer = Arc::new(buffer);
        let position = Arc::new(AtomicUsize::new(0));
        let paused = Arc::new(AtomicBool::new(true));
        let play_frames = Arc::new(AtomicU64::new(0));
        let anchor = Arc::new(Mutex::new(None::<PlaybackAnchor>));
        let latency_frames = Arc::new(AtomicU64::new(0));

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no audio output device found")?;

        let config = device
            .default_output_config()
            .context("no default audio output config")?
            .config();

        let buffer_clone = Arc::clone(&buffer);
        let position_clone = Arc::clone(&position);
        let paused_clone = Arc::clone(&paused);
        let play_frames_clone = Arc::clone(&play_frames);
        let anchor_clone = Arc::clone(&anchor);
        let latency_frames_clone = Arc::clone(&latency_frames);

        let stream = device
            .build_output_stream(
                &config,
                move |output: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    let paused = paused_clone.load(Ordering::Relaxed);
                    if paused {
                        output.fill(0.0);
                        return;
                    }

                    let mut pos = position_clone.load(Ordering::Relaxed);

                    // First unpaused callback: measure output‑buffer latency and
                    // pre‑roll the write cursor so audio hits the speaker at the
                    // correct wall‑clock time from the very first frame.
                    if latency_frames_clone.load(Ordering::Relaxed) == 0 {
                        let cb = info.timestamp().callback;
                        let pb = info.timestamp().playback;
                        if let Some(lat) = pb.duration_since(&cb) {
                            let lat_f = (lat.as_secs_f64() * sample_rate as f64) as u64;
                            latency_frames_clone.store(lat_f, Ordering::Release);
                            let skip = (lat_f as usize) * 2;
                            pos = pos.saturating_add(skip);
                            position_clone.store(pos, Ordering::Relaxed);
                        }
                    }

                    let to_copy = output.len().min(buffer_clone.len().saturating_sub(pos));
                    if to_copy > 0 {
                        output[..to_copy].copy_from_slice(&buffer_clone[pos..pos + to_copy]);
                        if to_copy < output.len() {
                            output[to_copy..].fill(0.0);
                        }
                        position_clone.store(pos + to_copy, Ordering::Relaxed);
                    } else {
                        output.fill(0.0);
                    }

                    // Track hardware playback position via cpal timestamps.
                    let ts = info.timestamp().playback;
                    let mut anchor = anchor_clone.lock().unwrap();
                    match anchor.as_ref() {
                        None => {
                            *anchor = Some(PlaybackAnchor { ts, pos });
                            play_frames_clone.store(pos as u64 / 2, Ordering::Release);
                        }
                        Some(anchor_data) => {
                            if let Some(delta) = ts.duration_since(&anchor_data.ts) {
                                let played = (delta.as_secs_f64() * sample_rate as f64) as u64;
                                play_frames_clone.store(
                                    (anchor_data.pos / 2) as u64 + played,
                                    Ordering::Release,
                                );
                            }
                        }
                    }
                },
                move |err| log::error!("audio playback error: {err}"),
                None,
            )
            .context("failed to build audio output stream")?;

        stream.play().context("failed to start audio stream")?;
        paused.store(true, Ordering::Relaxed);

        Ok(Self {
            buffer,
            position,
            paused,
            sample_rate,
            _stream: stream,
            play_frames,
            anchor,
            latency_frames,
        })
    }

    pub fn play(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn seek_to(&self, time_sec: f32) {
        let latency = self.latency_frames.load(Ordering::Acquire) as f32 / self.sample_rate as f32;
        let adjusted = time_sec + latency;
        let sample = (adjusted * self.sample_rate as f32) as usize * 2;
        self.position
            .store(sample.min(self.buffer.len()), Ordering::Relaxed);
        *self.anchor.lock().unwrap() = None;
        self.play_frames.store(
            (adjusted * self.sample_rate as f32) as u64,
            Ordering::Release,
        );
    }

    /// Returns the hardware playback position in seconds, using the `playback`
    /// timestamp from cpal to compensate for output‑buffer latency.
    pub fn current_position(&self) -> f32 {
        let frames = self.play_frames.load(Ordering::Acquire) as f32;
        frames / self.sample_rate as f32
    }

    pub fn total_duration(&self) -> f32 {
        if self.sample_rate == 0 || self.buffer.is_empty() {
            return 0.0;
        }
        (self.buffer.len() / 2) as f32 / self.sample_rate as f32
    }

    pub fn buffer(&self) -> &[f32] {
        &self.buffer
    }
}
