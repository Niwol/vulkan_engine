pub(crate) mod material_manager;
pub mod simple_material;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Simple,
    BlinnPhong,
    GLTF2,
}

pub trait Material {
    fn material_type(&self) -> MaterialType;
    fn shader_data(&self) -> Vec<u8>;
}
