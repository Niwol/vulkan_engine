use glam::{Mat4, Vec3};

pub struct Camera {
    position: Vec3,
    front: Vec3,
    right: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, front: Vec3, up: Vec3) -> Self {
        let right = front.cross(up);

        Self {
            position,
            front,
            right,
            up,
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn front(&self) -> Vec3 {
        self.front
    }

    pub fn right(&self) -> Vec3 {
        self.right
    }

    pub fn up(&self) -> Vec3 {
        self.up
    }

    pub fn move_up(&mut self, amount: f32) {
        self.position += self.up * amount;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.position -= self.up * amount;
    }

    pub fn move_left(&mut self, amount: f32) {
        self.position -= self.right * amount;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.position += self.right * amount;
    }

    pub(crate) fn get_view(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.front, self.up)
    }
}
