use crate::engine::{mesh::Mesh, transform::Transform};

pub struct MeshComponent {
    pub mesh: Mesh,
    pub model: Transform,
    pub material: u64,
}
