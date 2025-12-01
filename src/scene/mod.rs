mod camera;
mod material;
mod node;
mod transform;
pub mod demo;

pub use camera::Camera;
pub use material::Material;
pub use node::SceneNode;
pub use transform::Transform;

use crate::geometry::Mesh;

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub nodes: Vec<SceneNode>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
            nodes: Vec::new(),
        }
    }
}
