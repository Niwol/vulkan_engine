use crate::engine::Engine;

pub trait Runable {
    fn on_update(&self, frame_info: FrameInfo);
}

pub struct FrameInfo {
    pub delta_time: f32,
}

pub struct Application<T>
where
    T: Runable,
{
    runable: T,
    _engine: Engine,
}

impl<T> Application<T>
where
    T: Runable,
{
    pub fn create<B>(builder: B) -> Result<Self, String>
    where
        B: Fn(&Engine) -> T,
    {
        let engine = Engine::new();

        Ok(Self {
            runable: builder(&engine),
            _engine: engine,
        })
    }

    pub fn run(&self) {
        let frame_info = FrameInfo { delta_time: 0.0 };
        self.runable.on_update(frame_info);
    }
}
