use std::sync::Arc;

use self::{ecs::Scene, renderer::Renderer};

pub mod ecs;
pub mod input_handler;
pub mod material;
pub mod mesh;
pub mod renderer;
pub mod transform;

mod pipeline_manager;

use crate::vulkan_context::VulkanContext;

use anyhow::{Ok, Result};
use winit::{dpi::PhysicalSize, window::Window};

pub struct Engine {
    vulkan_context: Arc<VulkanContext>,
    renderer: Renderer,
    scene: Scene,
}

impl Engine {
    pub(crate) fn new(vulkan_context: Arc<VulkanContext>, window: Arc<Window>) -> Result<Self> {
        let scene = Scene::new(Arc::clone(&vulkan_context));
        let renderer = Renderer::new(
            Arc::clone(&vulkan_context),
            window,
            scene.material_manager(),
        )?;

        Ok(Self {
            vulkan_context,
            renderer,
            scene,
        })
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub(crate) fn vulkan_context(&self) -> &VulkanContext {
        &self.vulkan_context
    }

    pub(crate) fn handle_window_resized(&mut self, new_size: PhysicalSize<u32>) -> Result<()> {
        self.renderer.resize(new_size)?;
        Ok(())
    }

    pub(crate) fn suspend(&self) {}

    pub(crate) fn resume(&self, _window: Arc<Window>) {}

    pub(crate) fn render_frame(&mut self) {
        debug_assert!(self.scene.camera().is_some());
        let _ = self.renderer.render_scene(&self.scene);
    }
}
