use std::sync::Arc;

use self::renderer::Renderer;

pub mod mesh;
pub mod render_object;
pub mod renderer;
pub mod input_handler;
pub mod scene;

use crate::vulkan_context::VulkanContext;

use anyhow::Result;
use winit::window::Window;

pub struct Engine {
    vulkan_context: Arc<VulkanContext>,
    renderer: Renderer,
}

impl Engine {
    pub(crate) fn new(vulkan_context: Arc<VulkanContext>, window: Arc<Window>) -> Result<Self> {
        let renderer = Renderer::new(Arc::clone(&vulkan_context), window)?;

        Ok(Self {
            vulkan_context,
            renderer,
        })
    }

    pub(crate) fn vulkan_context(&self) -> &VulkanContext {
        &self.vulkan_context
    }

    pub(crate) fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub(crate) fn suspend(&self) {}

    pub(crate) fn resume(&self, _window: Arc<Window>) {}
}
