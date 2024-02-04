use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::{
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter},
    shader::ShaderStages,
    sync::Sharing,
};

use crate::{engine::pipeline_manager::PipelineManager, vulkan_context::VulkanContext};

use super::{Material, MaterialType};

struct MaterialBuffer {
    _material: Box<dyn Material>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    _buffer: Subbuffer<[u8]>,
}

pub struct MaterialManager {
    next_id: u64,
    materials: Vec<MaterialBuffer>,
    material_set_layout: Arc<DescriptorSetLayout>,
}

impl MaterialManager {
    pub fn new(device: Arc<Device>) -> Self {
        let material_set_layout = {
            let set_info = DescriptorSetLayoutCreateInfo {
                bindings: [(
                    PipelineManager::MATERIAL_BINDING,
                    DescriptorSetLayoutBinding {
                        descriptor_count: 1,
                        stages: ShaderStages::FRAGMENT,
                        ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
                    },
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            };

            DescriptorSetLayout::new(Arc::clone(&device), set_info)
                .expect("Failed to create descriptor set layout")
        };

        Self {
            next_id: 0,
            materials: Vec::new(),
            material_set_layout,
        }
    }

    pub fn new_material<T: Material + 'static>(
        &mut self,
        material: T,
        vulkan_context: Arc<VulkanContext>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let descriptor_allocator = vulkan_context.standard_descripor_set_allocator();
        let buffer_allocator = Arc::clone(vulkan_context.standard_memory_allocator());

        let buffer = Buffer::from_iter(
            buffer_allocator,
            BufferCreateInfo {
                sharing: Sharing::Exclusive,
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            material.shader_data(),
        )
        .expect("Failed to allocate buffer");

        let descriptor_set = PersistentDescriptorSet::new(
            descriptor_allocator.as_ref(),
            Arc::clone(&self.material_set_layout),
            vec![WriteDescriptorSet::buffer(
                PipelineManager::MATERIAL_BINDING,
                buffer.clone(),
            )],
            Vec::new(),
        )
        .expect("Failed to create persistant descriptor set");

        self.materials.push(MaterialBuffer {
            _material: Box::new(material),
            descriptor_set,
            _buffer: buffer,
        });

        id
    }

    pub fn _material_type(&self, id: u64) -> Option<MaterialType> {
        self.materials
            .get(id as usize)
            .map(|material| material._material.material_type())
    }

    pub fn _material<SimpleMaterial>(_id: u64) -> Option<SimpleMaterial> {
        None
    }

    pub fn descriptor_set(&self, material_id: u64) -> &Arc<PersistentDescriptorSet> {
        &self.materials[material_id as usize].descriptor_set
    }

    pub fn material_set_layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.material_set_layout
    }
}
