use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    device::Device,
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter, StandardMemoryAllocator,
    },
    pipeline::graphics::vertex_input,
    sync::Sharing,
};

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub in_position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub in_color: [f32; 3],
}

pub struct RenderObject {
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u32]>,
}

impl RenderObject {
    pub fn new(device: &Arc<Device>, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let buffer_info = BufferCreateInfo {
            sharing: Sharing::Exclusive, // TODO: handle sharing across different queues
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        };

        let allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            allocate_preference: MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let vertex_buffer =
            Buffer::from_iter(allocator.clone(), buffer_info, allocation_info, vertices)
                .expect("Failed to create vertex buffer");

        let buffer_info = BufferCreateInfo {
            sharing: Sharing::Exclusive, // TODO: handle sharing across different queues
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        };

        let allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            allocate_preference: MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let index_buffer =
            Buffer::from_iter(allocator.clone(), buffer_info, allocation_info, indices)
                .expect("Failed to create index buffer");

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub(crate) fn vectex_buffer(&self) -> &Subbuffer<[Vertex]> {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &Subbuffer<[u32]> {
        &self.index_buffer
    }
}
