mod camera;
mod cpu_picking;
pub mod demo;
mod material;
pub mod network;
mod node;
mod picking;
mod transform;

pub use camera::Camera;
pub use material::Material;
pub use node::SceneNode;
pub use picking::PickingState;
pub use transform::Transform;

use crate::graphics::geometry::Mesh;
use cpu_picking::{compute_node_aabb, ray_aabb_intersect, ray_mesh_intersect};
use glam::{Mat4, Vec3};

pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub nodes: Vec<SceneNode>,
    pub picking: PickingState,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
            nodes: Vec::new(),
            picking: PickingState::new(),
        }
    }

    pub fn update_node_position(&mut self, node_id: u32, new_position: Vec3) {
        if let Some(node) = self.nodes.get_mut(node_id as usize) {
            node.transform.position = new_position;
        }
    }

    /// Two-phase CPU-based raycast picking - returns closest hovered node
    ///
    /// Phase 1 (Broad): AABB intersection to cull non-hit meshes
    /// Phase 2 (Narrow): Triangle-ray intersection with backface culling for precise results
    ///
    /// Call this during input processing for immediate, precise results
    pub fn cpu_pick_ray(&self, ray_origin: Vec3, ray_dir: Vec3) -> Option<u32> {
        let mut closest_t = f32::INFINITY;
        let mut closest_node = None;

        for (idx, node) in self.nodes.iter().enumerate() {
            if !node.selectable {
                continue; // Skip non-selectable nodes
            }

            let mesh = &self.meshes[node.mesh_id];
            let transform_matrix = Mat4::from_cols_array_2d(&node.transform.to_matrix());

            // Phase 1: Broad-phase AABB test (fast cull)
            let aabb = compute_node_aabb(&mesh.vertices, transform_matrix);
            if ray_aabb_intersect(ray_origin, ray_dir, &aabb).is_none() {
                continue; // AABB miss - skip expensive triangle tests
            }

            // Phase 2: Narrow-phase triangle intersection with backface culling (precise)
            if let Some(t) = ray_mesh_intersect(
                ray_origin,
                ray_dir,
                &mesh.vertices,
                &mesh.indices,
                transform_matrix,
            ) {
                if t < closest_t {
                    closest_t = t;
                    closest_node = Some(idx as u32);
                }
            }
        }

        closest_node
    }
}
