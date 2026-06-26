use anyhow::{Context, Result, bail};
use wgpu::*;

pub(crate) struct Slot {
    pub buffer: Buffer,
    pub id: usize,
}
pub(crate) static mut SLOTS: Vec<Slot> = Vec::new();

pub(crate) fn init_buffers(slot_count: u32, size: usize, device: &wgpu::Device) -> Result<()> {
    unsafe {
        if SLOTS.len() != 0 {
            bail!(
                "BUFFERS were already initialized! they should be initialized only once to avoid any issues "
            )
        }
        SLOTS = (0..slot_count)
            .map(|id| Slot {
                id: id as usize,
                buffer: device.create_buffer(&BufferDescriptor {
                    label: Some("readback"),

                    size: size as u64,

                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,

                    mapped_at_creation: false,
                }),
            })
            .collect();
    }
    Ok(())
}
pub(crate) struct ReadbackRing {
    free_slots: tokio::sync::mpsc::Receiver<usize>,
}

impl ReadbackRing {
    pub fn new(free_buffers: tokio::sync::mpsc::Receiver<usize>) -> Self {
        Self {
            free_slots: free_buffers,
        }
    }

    pub async fn next(&mut self) -> Result<&Slot> {
        let id = self
            .free_slots
            .recv()
            .await
            .context("free_buffers channel was closed")?;
        unsafe { SLOTS.get(id).context("buffer index was out of bounds") }
    }
}
