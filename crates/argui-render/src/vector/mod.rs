mod mesh;
mod pipeline;

use std::collections::HashMap;

use argui_paint::{DisplayCommand, DisplayList, VectorAsset, VectorId};
use bytemuck::{Pod, Zeroable};
use pipeline::{VectorInstance, VectorPipeline};
use wgpu::util::DeviceExt;

use crate::RendererError;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuVertex {
    from: [f32; 2],
    to: [f32; 2],
    color_from: [f32; 4],
    color_to: [f32; 4],
    barycentric: [f32; 3],
    boundary: [f32; 3],
}

struct Mesh {
    vertices: wgpu::Buffer,
    vertex_count: u32,
    size: [f32; 2],
}

pub(crate) struct VectorGpu {
    pipeline: VectorPipeline,
    meshes: HashMap<VectorId, Mesh>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl VectorGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            pipeline: VectorPipeline::new(device, format),
            meshes: HashMap::new(),
        }
    }

    pub fn register(&mut self, device: &wgpu::Device, asset: &VectorAsset) {
        let vertices = mesh::expand(asset);
        let vertex_count = vertices.len() as u32;
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("argui-vector-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.meshes.insert(
            asset.id,
            Mesh {
                vertices,
                vertex_count,
                size: [asset.size.width, asset.size.height],
            },
        );
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale: f32,
    ) -> Result<(), RendererError> {
        let mut instances = Vec::new();
        let mut clips = Vec::new();
        for primitive in display_list.commands().iter().filter_map(|command| {
            if let DisplayCommand::Vector(vector) = command {
                Some(vector)
            } else {
                None
            }
        }) {
            let Some(mesh) = self.meshes.get(&primitive.vector) else {
                return Err(RendererError::MissingVector(primitive.vector.0));
            };
            instances.push(VectorInstance::new(primitive, mesh.size, scale, &mut clips));
        }
        self.pipeline.write(device, queue, &instances, &clips);
        Ok(())
    }

    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        id: VectorId,
        range: std::ops::Range<u32>,
        viewport: u32,
    ) {
        if let Some(mesh) = self.meshes.get(&id) {
            self.pipeline.draw(pass, mesh, range, viewport);
        }
    }
}
