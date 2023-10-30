use std::sync::Arc;

use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::engine::Engine;

pub trait Runable {
    fn on_update(&self, event: Event<()>, window: &Arc<Window>, frame_info: FrameInfo) -> bool;
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
        let frame_info = FrameInfo { delta_time: 0.0 };

        let event_loop = self.event_loop.unwrap();
        self.event_loop = None;

        event_loop.set_control_flow(ControlFlow::Poll);

        if let Ok(_) = event_loop.run(move |event, elwt| {
            if !self
                .runable
                .on_update(event, self.engine.get_window(), frame_info)
            {
                elwt.exit();
            }
        }) {
            ();
        }
    }
}
