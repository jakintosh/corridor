use super::Transform;

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub mesh_id: usize,
    pub material_id: usize,
    pub transform: Transform,
    pub selectable: bool,
    pub parent_id: Option<u32>,
}

impl SceneNode {
    pub fn new(mesh_id: usize, material_id: usize, transform: Transform, selectable: bool) -> Self {
        Self {
            mesh_id,
            material_id,
            transform,
            selectable,
            parent_id: None,
        }
    }

    pub fn with_parent(mut self, parent_id: u32) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}
