use glam::{Vec2, Vec3};
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryAllocatePreference, MemoryTypeFilter},
    pipeline::graphics::vertex_input,
    sync::Sharing,
};

use super::Engine;

pub mod primitives;

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub in_position: Vec3,

    #[format(R32G32B32_SFLOAT)]
    pub in_normal: Vec3,

    #[format(R32G32_SFLOAT)]
    pub in_texture_coord: Vec2,

    #[format(R32G32B32_SFLOAT)]
    pub in_color: Vec3,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            in_position: Vec3::ZERO,
            in_normal: Vec3::ZERO,
            in_texture_coord: Vec2::ZERO,
            in_color: Vec3::ZERO,
        }
    }
}

pub struct Mesh {
    vertex_buffer: Subbuffer<[Vertex]>,
    index_buffer: Subbuffer<[u32]>,
}

impl Mesh {
    pub fn new(engine: &Engine, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let allocator = engine.vulkan_context().standard_memory_allocator();

        let vertex_buffer_info = BufferCreateInfo {
            sharing: Sharing::Exclusive, // TODO: handle sharing across different queues
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        };

        let vertex_allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            allocate_preference: MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let vertex_buffer = Buffer::from_iter(
            allocator.clone(),
            vertex_buffer_info,
            vertex_allocation_info,
            vertices,
        )
        .expect("Failed to create vertex buffer");

        let index_buffer_info = BufferCreateInfo {
            sharing: Sharing::Exclusive, // TODO: handle sharing across different queues
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        };

        let index_allocation_info = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            allocate_preference: MemoryAllocatePreference::Unknown,
            ..Default::default()
        };

        let index_buffer = Buffer::from_iter(
            allocator.clone(),
            index_buffer_info,
            index_allocation_info,
            indices,
        )
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
