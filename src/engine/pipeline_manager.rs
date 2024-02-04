use std::sync::Arc;

use vulkano::{
    descriptor_set::layout::DescriptorSetLayout,
    pipeline::{GraphicsPipeline, PipelineLayout},
    render_pass::RenderPass,
};

use anyhow::Result;

use crate::vulkan_context::VulkanContext;

mod shader_loader;

pub struct VulkanPipeline {
    pub pipeline: Arc<GraphicsPipeline>,
    pub layout: Arc<PipelineLayout>,
}

pub struct PipelineManager {
    normal_pipeline: VulkanPipeline,
    depth_pipeline: VulkanPipeline,
    _mesh_view_pipeine: VulkanPipeline,
    material_pipeline: VulkanPipeline,
}

impl PipelineManager {
    pub const MATERIAL_BINDING: u32 = 0;

    pub fn new(
        vulkan_context: &Arc<VulkanContext>,
        render_pass: &Arc<RenderPass>,
        material_set_layout: Arc<DescriptorSetLayout>,
    ) -> Result<Self> {
        let device = vulkan_context.device();

        let normal_pipeline = shader_loader::load_normal(device, render_pass)?;
        let depth_pipeline = shader_loader::load_depth(device, render_pass)?;
        let mesh_view_pipeine = shader_loader::load_mesh_view(device, render_pass)?;

        let material_pipeline =
            shader_loader::load_material_simple(device, render_pass, material_set_layout)?;

        Ok(Self {
            normal_pipeline,
            depth_pipeline,
            _mesh_view_pipeine: mesh_view_pipeine,
            material_pipeline,
        })
    }

    pub fn normal_pipeline(&self) -> &VulkanPipeline {
        &self.normal_pipeline
    }

    pub fn depth_pipeline(&self) -> &VulkanPipeline {
        &self.depth_pipeline
    }

    pub fn _mesh_view_pipeine(&self) -> &VulkanPipeline {
        &self._mesh_view_pipeine
    }

    pub fn material_pipeline(&self) -> &VulkanPipeline {
        &self.material_pipeline
    }
}
