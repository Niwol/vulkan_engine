use std::sync::Arc;
use std::time::Instant;

use winit::dpi::{LogicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowBuilder;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use anyhow::{Ok, Result};

use crate::engine::renderer::Renderer;
use crate::engine::Engine;
use crate::vulkan_context::VulkanContext;

pub trait Runable {
    fn on_update(&mut self, input: i32, window: &Arc<Window>, frame_info: &FrameInfo) -> bool;

    fn render(&mut self, renderer: &Renderer);
}

pub struct FrameInfo {
    pub delta_time: f32,
}

pub struct ApplicationInfo {
    pub window_title: String,
    pub window_size: [u32; 2],
    pub resizeable: bool,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        Self {
            window_title: String::from("Vulkan application"),
            window_size: [800, 600],
            resizeable: false,
        }
    }
}

pub struct Application<T>
where
    T: Runable,
{
    runable: T,
    _vulkan_context: Arc<VulkanContext>,
    engine: Engine,
    window: Arc<Window>,

    frame_info: FrameInfo,
    previous_frame_time: Instant,
}

impl<T> Application<T>
where
    T: 'static + Runable,
{
    pub fn run_application<B>(application_info: ApplicationInfo, builder: B) -> Result<()>
    where
        B: Fn(&Engine) -> T,
    {
        let event_loop = EventLoop::new()?;
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(application_info.window_title)
                .with_inner_size(Size::Logical(LogicalSize::from(
                    application_info.window_size,
                )))
                .with_resizable(application_info.resizeable)
                .build(&event_loop)?,
        );

        let vulkan_context = Arc::new(VulkanContext::new(&window)?);
        let engine = Engine::new(Arc::clone(&vulkan_context), Arc::clone(&window))?;
        let runable = builder(&engine);

        let mut app = Self {
            runable,
            _vulkan_context: vulkan_context,
            engine,
            window,

            frame_info: FrameInfo { delta_time: 0.0 },
            previous_frame_time: Instant::now(),
        };

        app.start(event_loop)?;

        Ok(())
    }

    fn start(&mut self, event_loop: EventLoop<()>) -> Result<()> {
        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run(move |event, window_target| {
            if let Err(error) = self.handle_event(&event, window_target) {
                panic!("Application error: {:#?}", error);
            }
        })?;

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: &Event<()>,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<()> {
        match event {
            Event::NewEvents(_) => {
                self.frame_info.delta_time =
                    Instant::elapsed(&self.previous_frame_time).as_secs_f32();
                
                self.previous_frame_time = Instant::now();
            }

            Event::WindowEvent { event, .. } => {
                self.handle_window_event(&event, window_target)?;
            }

            Event::Suspended => self.engine.suspend(),
            Event::Resumed => self.engine.resume(Arc::clone(&self.window)),

            Event::AboutToWait => {
                self.runable.on_update(0, &self.window, &self.frame_info);

                self.window.request_redraw();
            }

            _ => (),
        }

        Ok(())
    }

    fn handle_window_event(
        &mut self,
        window_event: &WindowEvent,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<()> {
        match window_event {
            WindowEvent::CloseRequested => {
                window_target.exit();
            }

            WindowEvent::RedrawRequested => self.runable.render(self.engine.renderer()),

            _ => (),
        }

        Ok(())
    }
}
