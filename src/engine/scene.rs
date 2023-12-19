use crate::engine::mesh::Mesh;

pub struct Scene {
    meshes: Vec<Mesh>,
}

impl Scene {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn meshes(&self) -> &Vec<Mesh> {
        &self.meshes
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }
}
