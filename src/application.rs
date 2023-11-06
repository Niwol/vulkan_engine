use std::time::Instant;
use std::{borrow::BorrowMut, sync::Arc};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::engine::Engine;

pub trait Runable {
    fn on_update(&mut self, event: Event<()>, window: &Arc<Window>, frame_info: FrameInfo) -> bool;
}

#[derive(Clone, Copy)]
pub struct FrameInfo {
    pub delta_time: f32,
}

pub struct Application<T>
where
    T: Runable,
{
    runable: T,
    engine: Engine,

    event_loop: Option<EventLoop<()>>,
}

impl<T> Application<T>
where
    T: 'static + Runable,
{
    pub fn create<B>(builder: B) -> Result<Self, String>
    where
        B: Fn(&Engine) -> T,
    {
        let (engine, event_loop) = Engine::new();

        Ok(Self {
            event_loop: Some(event_loop),
            runable: builder(&engine),
            engine,
        })
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.unwrap();
        self.event_loop = None;

        event_loop.set_control_flow(ControlFlow::Poll);

        let mut delta_time = 0.0;

        if let Ok(_) = event_loop.run(move |event, elwt| {
            let start = Instant::now();

            if let Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } = event
            {
                elwt.exit();
            }

            if !self.runable.on_update(
                event,
                self.engine.vulkan().window(),
                FrameInfo { delta_time },
            ) {
                elwt.exit();
            }

            delta_time = start.elapsed().as_secs_f32();

            let mut window = self.engine.vulkan().window();
            let window = window.borrow_mut();
            window.set_title(
                format!(
                    "Vulkan application -- {} ms  --  {} FPS",
                    (delta_time * 10000.0) as i32,
                    (1.0 / delta_time) as i32
                )
                .as_str(),
            );
        }) {
            ();
        }
    }
}
