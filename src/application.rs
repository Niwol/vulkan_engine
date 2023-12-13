use std::time::Instant;
use std::{borrow::BorrowMut, sync::Arc};

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use winit_input_helper::WinitInputHelper;

use crate::engine::Engine;

pub trait Runable {
    fn on_update(
        &mut self,
        input: &WinitInputHelper,
        window: &Arc<Window>,
        frame_info: &FrameInfo,
    ) -> bool;

    fn render(&mut self);
}

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

        let mut frame_info = FrameInfo { delta_time: 0.0 };

        let mut input = WinitInputHelper::new();

        if let Ok(_) = event_loop.run(move |event, elwt| {
            if input.update(&event) {
                let start = Instant::now();

                if input.close_requested() {
                    elwt.exit();
                    return;
                }

                if !self
                    .runable
                    .on_update(&input, self.engine.vulkan().window(), &frame_info)
                {
                    elwt.exit();
                    return;
                }

                self.runable.render();

                let delta_time = &mut frame_info.delta_time;
                *delta_time = start.elapsed().as_secs_f32();

                let mut window = self.engine.vulkan().window();
                let window = window.borrow_mut();
                window.set_title(
                    format!(
                        "Vulkan application -- {} ms  --  {} FPS",
                        (*delta_time * 10000.0) as i32,
                        (1.0 / *delta_time) as i32
                    )
                    .as_str(),
                );
            }
        }) {
            ();
        }
    }
}
