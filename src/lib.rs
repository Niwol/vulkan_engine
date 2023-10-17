pub trait Runable {
    fn on_update(&self, frame_info: FrameInfo);
}

pub struct FrameInfo {
    delta_time: f32,
}

pub struct Application<T>
where
    T: Runable,
{
    runable: T,
}

impl<T> Application<T>
where
    T: Runable,
{
    pub fn create<B>(builder: B) -> Result<Self, String>
    where
        B: Fn() -> T,
    {
        Ok(Self { runable: builder() })
    }

    pub fn run(&self) {
        let frame_info = FrameInfo { delta_time: 0.0 };
        self.runable.on_update(frame_info);
    }
}
