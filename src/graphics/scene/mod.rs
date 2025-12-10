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
    pub edge_node_refs: Vec<Option<(u32, u32)>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            materials: Vec::new(),
            nodes: Vec::new(),
            picking: PickingState::new(),
            edge_node_refs: Vec::new(),
        }
    }

    pub fn update_node_position(&mut self, node_id: u32, new_position: Vec3) {
        if let Some(node) = self.nodes.get_mut(node_id as usize) {
            node.transform.position = new_position;
        }
    }

    /// Compute the world transform for a node by traversing its parent chain
    /// Returns the combined world matrix
    pub fn compute_world_transform(&self, node_id: u32) -> [[f32; 4]; 4] {
        let node = &self.nodes[node_id as usize];

        match node.parent_id {
            None => {
                // Root node - local transform is world transform
                node.transform.to_matrix()
            }
            Some(parent_id) => {
                // Has parent - compute parent's world transform first (recursive)
                let parent_world = self.compute_world_transform(parent_id);
                let parent_matrix = Mat4::from_cols_array_2d(&parent_world);
                node.transform.combine_with_parent(parent_matrix)
            }
        }
    }

    /// Get all direct children of a node
    pub fn get_children(&self, parent_id: u32) -> Vec<u32> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if node.parent_id == Some(parent_id) {
                    Some(idx as u32)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all descendants (recursive) of a node
    pub fn get_descendants(&self, parent_id: u32) -> Vec<u32> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![parent_id];

        while let Some(current) = to_visit.pop() {
            let children = self.get_children(current);
            descendants.extend(&children);
            to_visit.extend(&children);
        }

        descendants
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
            let node_id = idx as u32;
            let world_matrix = self.compute_world_transform(node_id);
            let transform_matrix = Mat4::from_cols_array_2d(&world_matrix);

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
