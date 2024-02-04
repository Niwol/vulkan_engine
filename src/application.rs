use std::sync::Arc;
use std::time::Instant;

use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoopWindowTarget;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use anyhow::{Ok, Result};

use crate::engine::input_handler::InputHandler;
use crate::engine::Engine;
use crate::vulkan_context::VulkanContext;

pub trait Runable {
    fn new(engine: &mut Engine) -> Self;
    fn on_update(
        &mut self,
        engine: &mut Engine,
        input: &InputHandler,
        frame_info: &FrameInfo,
    ) -> bool;
}

pub struct FrameInfo {
    pub delta_time: f32,
}

pub struct ApplicationInfo {
    pub window_title: String,
    pub window_size: [u32; 2],
    pub resizeable: bool,
    pub exit_on_escape: bool,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        Self {
            window_title: String::from("Vulkan application"),
            window_size: [800, 600],
            resizeable: false,
            exit_on_escape: false,
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

    input_handler: InputHandler,
    exit_on_escape: bool,
}

impl<T> Application<T>
where
    T: Runable,
{
    pub fn run_application(application_info: ApplicationInfo) -> Result<()> {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(application_info.window_title)
                .with_inner_size(Size::Logical(LogicalSize::from(
                    application_info.window_size,
                )))
                .with_resizable(application_info.resizeable)
                .build(&event_loop)
                .expect("Failed to build window"),
        );

        let vulkan_context = Arc::new(VulkanContext::new(&window)?);
        let mut engine = Engine::new(Arc::clone(&vulkan_context), Arc::clone(&window))?;
        let runable = T::new(&mut engine);

        let mut app = Self {
            runable,
            _vulkan_context: vulkan_context,
            engine,
            window,

            frame_info: FrameInfo { delta_time: 0.0 },
            previous_frame_time: Instant::now(),

            input_handler: InputHandler::new(),
            exit_on_escape: application_info.exit_on_escape,
        };

        app.start(event_loop)?;

        Ok(())
    }

    fn start(&mut self, event_loop: EventLoop<()>) -> Result<()> {
        event_loop.set_control_flow(ControlFlow::Poll);

        // TODO: Handle web applications, see EventLoop::run
        event_loop
            .run(move |event, window_target| {
                if let Err(error) = self.handle_event(event, window_target) {
                    panic!("Application error: {}", error);
                }
            })
            .expect("An event loop error occured");

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: Event<()>,
        window_target: &EventLoopWindowTarget<()>,
    ) -> Result<()> {
        match &event {
            Event::NewEvents(_) => {
                self.frame_info.delta_time =
                    Instant::elapsed(&self.previous_frame_time).as_secs_f32();

                self.previous_frame_time = Instant::now();

                self.input_handler.step();
            }

            Event::WindowEvent { event, .. } => {
                self.handle_window_event(event, window_target)?;
            }

            Event::Suspended => self.engine.suspend(),
            Event::Resumed => self.engine.resume(Arc::clone(&self.window)),

            Event::AboutToWait => {
                if !self
                    .runable
                    .on_update(&mut self.engine, &self.input_handler, &self.frame_info)
                {
                    window_target.exit();
                }

                self.window.request_redraw();
            }

            _ => (),
        }

        self.input_handler.update(&event);

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

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if self.exit_on_escape {
                    window_target.exit();
                }
            }

            WindowEvent::Resized(new_size) => {
                self.engine.handle_window_resized(*new_size)?;
            }

            WindowEvent::RedrawRequested => self.engine.render_frame(),

            _ => (),
        }

        Ok(())
    }
}
