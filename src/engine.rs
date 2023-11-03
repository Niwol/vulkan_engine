use winit::event_loop::EventLoop;

use self::renderer::Renderer;

pub mod render_object;
pub mod renderer;
mod vulkan;

use vulkan::Vulkan;

pub struct Engine {
    vulkan: Vulkan,
}

impl Engine {
    pub(crate) fn new() -> (Self, EventLoop<()>) {
        let (vulkan, event_loop) = Vulkan::new();

        let engine = Self { vulkan };

        (engine, event_loop)
    }

    pub fn create_renderer(&self) -> Renderer {
        Renderer::new(self)
    }

    pub(crate) fn vulkan(&self) -> &Vulkan {
        &self.vulkan
    }
}
