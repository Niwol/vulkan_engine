use glam::Vec3;

use super::{Material, MaterialType};

pub struct SimpleMaterial {
    pub color: Vec3,
}

impl SimpleMaterial {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: Vec3::new(r, g, b),
        }
    }
}

impl Material for SimpleMaterial {
    fn material_type(&self) -> MaterialType {
        MaterialType::Simple
    }

    fn shader_data(&self) -> Vec<u8> {
        self.color
            .to_array()
            .into_iter()
            .map(|x| x.to_bits().to_ne_bytes())
            .flatten()
            .collect()
    }
}
