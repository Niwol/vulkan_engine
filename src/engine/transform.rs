use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn translate(&mut self, translation: Vec3) -> &mut Self {
        self.translation += translation;
        self
    }

    pub fn rotate(&mut self, _axis: Vec3, _angle: f32) -> &mut Self {
        todo!();
    }

    pub fn scale(&mut self, scale: Vec3) -> &mut Self {
        self.scale *= scale;
        self
    }
}
