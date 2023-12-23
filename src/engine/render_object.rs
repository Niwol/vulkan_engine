use super::{mesh::Mesh, transform::Transform};

pub struct RenderObject {
    mesh: Mesh,
    transform: Transform,
    // pipeline index
    // material index
    // uniforms
    // ...
}

impl RenderObject {
    pub fn new(mesh: Mesh) -> Self {
        Self {
            mesh,
            transform: Transform::new(),
        }
    }

    pub(crate) fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}
