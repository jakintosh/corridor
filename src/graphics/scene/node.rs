use super::Transform;

#[derive(Debug, Clone)]
pub struct SceneNode {
    pub mesh_id: usize,
    pub material_id: usize,
    pub transform: Transform,
    pub selectable: bool,
}

impl SceneNode {
    pub fn new(mesh_id: usize, material_id: usize, transform: Transform, selectable: bool) -> Self {
        Self {
            mesh_id,
            material_id,
            transform,
            selectable,
        }
    }
}
