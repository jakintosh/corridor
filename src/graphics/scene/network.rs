use std::collections::HashSet;

use glam::Vec3;

use crate::graphics::geometry::Mesh;
use crate::graphics::scene::{Material, Scene, SceneNode, Transform};
use crate::model::{Network, TransportMode};

pub fn network_to_scene(network: &Network) -> Scene {
    let mut scene = Scene::new();

    // Add meshes (reusable geometry)
    scene.meshes.push(Mesh::cube()); // mesh_id = 0 (nodes)
    scene.meshes.push(Mesh::line_segment(0.05)); // mesh_id = 1 (edges)

    // Add materials with color coding BY MODE
    scene.materials.push(Material::from_rgb(0.8, 0.8, 0.8)); // Nodes - gray (material_id = 0)
    scene.materials.push(Material::from_rgb(0.0, 0.8, 0.0)); // Bike - green (material_id = 1)
    scene.materials.push(Material::from_rgb(0.0, 0.5, 1.0)); // Walk - blue (material_id = 2)
    scene.materials.push(Material::from_rgb(1.0, 0.0, 0.0)); // Transit - red (material_id = 3)
    scene.materials.push(Material::from_rgb(1.0, 0.8, 0.0)); // Car - orange (material_id = 4)

    // Render all mode graphs
    // Use different Y heights to prevent z-fighting when edges overlap
    // Keep close to node Y=0.1 so edges connect to nodes
    let mode_heights = [
        (TransportMode::Car, 0.1001),
        (TransportMode::Bike, 0.1002),
        (TransportMode::Walk, 0.1003),
        (TransportMode::Transit, 0.1004),
    ];

    // Render edges for each mode
    for (mode, height) in &mode_heights {
        if let Some(graph) = network.graphs.get(mode) {
            for edge in &graph.edges {
                let from = &graph.nodes[edge.from_node];
                let to = &graph.nodes[edge.to_node];

                scene.nodes.push(SceneNode::new(
                    1, // line mesh
                    mode_material_id(*mode),
                    edge_transform(from.position, to.position, *height),
                ));
            }
        }
    }

    // Render nodes from all graphs (deduplicated by position)
    for graph in network.graphs.values() {
        for node in &graph.nodes {
            let mat = mode_material_id(graph.mode);
            scene.nodes.push(SceneNode::new(
                0,   // cube mesh
                mat, // gray material
                Transform::new(
                    [node.position[0], 0.1, node.position[1]],
                    [0.0, 0.0, 0.0],
                    [0.15, 0.15, 0.15], // Small cube
                ),
            ));
        }
    }

    scene
}

fn mode_material_id(mode: TransportMode) -> usize {
    match mode {
        TransportMode::Bike => 1,
        TransportMode::Walk => 2,
        TransportMode::Transit => 3,
        TransportMode::Car => 4,
    }
}

fn edge_transform(from: [f32; 2], to: [f32; 2], height: f32) -> Transform {
    let from_3d = Vec3::new(from[0], height, from[1]);
    let to_3d = Vec3::new(to[0], height, to[1]);

    let midpoint = (from_3d + to_3d) / 2.0;
    let diff = to_3d - from_3d;
    let length = diff.length();
    let angle = (-diff.z).atan2(diff.x);

    Transform::new(midpoint.to_array(), [0.0, angle, 0.0], [length, 1.0, 1.0])
}
