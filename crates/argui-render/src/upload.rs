use std::mem::size_of;

use bytemuck::Pod;

#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) fn write_changed<T: Pod + Copy>(
    queue: &wgpu::Queue,
    buffer: &wgpu::Buffer,
    values: &[T],
    previous: &mut Vec<T>,
    reallocated: bool,
) {
    if values.is_empty() {
        previous.clear();
        return;
    }
    let current: &[u8] = bytemuck::cast_slice(values);
    let old: &[u8] = bytemuck::cast_slice(previous.as_slice());
    let stride = size_of::<T>();
    let (start, end) = if reallocated || values.len() != previous.len() {
        (0, values.len())
    } else {
        let first = current
            .chunks_exact(stride)
            .zip(old.chunks_exact(stride))
            .position(|(current, old)| current != old);
        let Some(first) = first else {
            return;
        };
        let suffix = current
            .chunks_exact(stride)
            .rev()
            .zip(old.chunks_exact(stride).rev())
            .take_while(|(current, old)| current == old)
            .count();
        (first, values.len() - suffix)
    };
    queue.write_buffer(
        buffer,
        (start * stride) as u64,
        bytemuck::cast_slice(&values[start..end]),
    );
    previous.clear();
    previous.extend_from_slice(values);
}
