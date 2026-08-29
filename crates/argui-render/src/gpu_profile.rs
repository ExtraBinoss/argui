use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use argui_paint::RenderObjectId;

use crate::{AdapterProfile, GpuFrameProfile, GpuPassProfile};

const MAX_PASSES: u32 = 128;
const QUERY_COUNT: u32 = MAX_PASSES * 2;
const READBACK_SLOTS: usize = 4;

#[derive(Clone)]
struct ReadbackSlot {
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    busy: Arc<AtomicBool>,
}

#[derive(Clone)]
struct PassMetadata {
    label: String,
    object: Option<RenderObjectId>,
    pixels: u64,
}

pub(crate) struct GpuProfiler {
    enabled: bool,
    query_set: Option<wgpu::QuerySet>,
    slots: Vec<ReadbackSlot>,
    next_slot: usize,
    next_frame: u64,
    timestamp_period: f32,
    completed: Arc<Mutex<VecDeque<GpuFrameProfile>>>,
    adapter: AdapterProfile,
}

pub(crate) struct GpuFrameCapture {
    frame: u64,
    query_set: wgpu::QuerySet,
    slot: ReadbackSlot,
    timestamp_period: f32,
    completed: Arc<Mutex<VecDeque<GpuFrameProfile>>>,
    passes: RefCell<Vec<PassMetadata>>,
    next_query: Cell<u32>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl GpuProfiler {
    pub fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        requested: bool,
    ) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();
        let features = adapter.features();
        let adapter = AdapterProfile {
            name: info.name,
            vendor: info.vendor,
            device: info.device,
            device_type: format!("{:?}", info.device_type),
            driver: info.driver,
            driver_info: info.driver_info,
            backend: format!("{:?}", info.backend),
            features: format!("{features:?}"),
            timestamp_queries: features.contains(wgpu::Features::TIMESTAMP_QUERY),
            max_texture_dimension_2d: limits.max_texture_dimension_2d,
            max_buffer_size: limits.max_buffer_size,
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
            max_bind_groups: limits.max_bind_groups,
        };
        let enabled = requested && device.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let query_set = enabled.then(|| {
            device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("argui-gpu-profile-queries"),
                ty: wgpu::QueryType::Timestamp,
                count: QUERY_COUNT,
            })
        });
        let byte_size = u64::from(QUERY_COUNT) * u64::from(wgpu::QUERY_SIZE);
        let slots = if enabled {
            (0..READBACK_SLOTS)
                .map(|_| ReadbackSlot {
                    resolve: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("argui-gpu-profile-resolve"),
                        size: byte_size,
                        usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    }),
                    readback: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("argui-gpu-profile-readback"),
                        size: byte_size,
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                        mapped_at_creation: false,
                    }),
                    busy: Arc::new(AtomicBool::new(false)),
                })
                .collect()
        } else {
            Vec::new()
        };
        Self {
            enabled,
            query_set,
            slots,
            next_slot: 0,
            next_frame: 0,
            timestamp_period: queue.get_timestamp_period(),
            completed: Arc::new(Mutex::new(VecDeque::with_capacity(READBACK_SLOTS * 2))),
            adapter,
        }
    }

    #[must_use]
    pub fn adapter(&self) -> &AdapterProfile {
        &self.adapter
    }

    pub fn begin_frame(&mut self, active: bool) -> Option<GpuFrameCapture> {
        if !self.enabled || !active {
            return None;
        }
        let slot_index = self.next_slot;
        self.next_slot = (self.next_slot + 1) % self.slots.len();
        let slot = self.slots[slot_index].clone();
        if slot.busy.swap(true, Ordering::AcqRel) {
            return None;
        }
        let frame = self.next_frame;
        self.next_frame += 1;
        Some(GpuFrameCapture {
            frame,
            query_set: self.query_set.as_ref().expect("enabled query set").clone(),
            slot,
            timestamp_period: self.timestamp_period,
            completed: Arc::clone(&self.completed),
            passes: RefCell::new(Vec::with_capacity(MAX_PASSES as usize)),
            next_query: Cell::new(0),
        })
    }

    pub fn take_latest(&self) -> Option<GpuFrameProfile> {
        self.completed.lock().ok()?.pop_back()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl GpuFrameCapture {
    pub fn timestamp_writes(
        &self,
        label: impl Into<String>,
        object: Option<RenderObjectId>,
        pixels: u64,
    ) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        let beginning = self.next_query.get();
        if beginning + 1 >= QUERY_COUNT {
            return None;
        }
        self.next_query.set(beginning + 2);
        self.passes.borrow_mut().push(PassMetadata {
            label: label.into(),
            object,
            pixels,
        });
        Some(wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(beginning),
            end_of_pass_write_index: Some(beginning + 1),
        })
    }

    pub fn finish(self, encoder: &mut wgpu::CommandEncoder) {
        let query_count = self.next_query.get();
        if query_count == 0 {
            self.slot.busy.store(false, Ordering::Release);
            return;
        }
        let byte_size = u64::from(query_count) * u64::from(wgpu::QUERY_SIZE);
        encoder.resolve_query_set(&self.query_set, 0..query_count, &self.slot.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.slot.resolve, 0, &self.slot.readback, 0, byte_size);
        let buffer = self.slot.readback.clone();
        let callback_buffer = buffer.clone();
        let busy = Arc::clone(&self.slot.busy);
        let completed = Arc::clone(&self.completed);
        let passes = self.passes.into_inner();
        let frame = self.frame;
        let period = self.timestamp_period;
        encoder.map_buffer_on_submit(&buffer, wgpu::MapMode::Read, 0..byte_size, move |result| {
            if result.is_ok() {
                let Ok(mapped) = callback_buffer.slice(0..byte_size).get_mapped_range() else {
                    callback_buffer.unmap();
                    busy.store(false, Ordering::Release);
                    return;
                };
                let timestamps = bytemuck::cast_slice::<u8, u64>(&mapped);
                let pairs = timestamps.as_chunks::<2>().0;
                let first = pairs
                    .iter()
                    .take(passes.len())
                    .map(|pair| pair[0])
                    .min()
                    .unwrap_or(0);
                let last = pairs
                    .iter()
                    .take(passes.len())
                    .map(|pair| pair[1])
                    .max()
                    .unwrap_or(first);
                let elapsed =
                    |ticks: u64| Duration::from_nanos((ticks as f64 * f64::from(period)) as u64);
                let profiles = passes
                    .into_iter()
                    .enumerate()
                    .map(|(index, metadata)| {
                        let start = timestamps[index * 2];
                        let end = timestamps[index * 2 + 1];
                        let duration = if end >= start {
                            elapsed(end - start)
                        } else {
                            Duration::ZERO
                        };
                        GpuPassProfile {
                            label: metadata.label,
                            start: elapsed(start.saturating_sub(first)),
                            duration,
                            pixels: metadata.pixels,
                            object: metadata.object,
                        }
                    })
                    .collect();
                drop(mapped);
                callback_buffer.unmap();
                if let Ok(mut queue) = completed.lock() {
                    if queue.len() == READBACK_SLOTS * 2 {
                        queue.pop_front();
                    }
                    queue.push_back(GpuFrameProfile {
                        frame,
                        total: elapsed(last.saturating_sub(first)),
                        passes: profiles,
                    });
                }
            }
            busy.store(false, Ordering::Release);
        });
    }
}
