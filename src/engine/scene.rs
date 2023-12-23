use super::render_object::RenderObject;

pub struct Scene {
    render_objects: Vec<RenderObject>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            render_objects: Vec::new(),
        }
    }

    pub fn render_objects(&self) -> &Vec<RenderObject> {
        &self.render_objects
    }

    pub fn render_object_mut(&mut self) -> &mut Vec<RenderObject> {
        &mut self.render_objects
    }

    pub fn add_render_object(&mut self, render_object: RenderObject) {
        self.render_objects.push(render_object);
    }
}
