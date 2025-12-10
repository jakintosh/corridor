use std::collections::HashMap;

use glam::Vec3;

use crate::graphics::geometry::Mesh;
use crate::graphics::scene::{Material, Scene, SceneNode, Transform};
use crate::model::{Network, TransportMode};

/// Convert a network into a 3D scene with hierarchical node pillars and dynamic edges
pub fn network_to_scene(network: &Network) -> Scene {
    let mut scene = Scene::new();

    // Add meshes (reusable geometry)
    scene.meshes.push(Mesh::cube()); // mesh_id = 0 (nodes and pillars)
    scene.meshes.push(Mesh::line_segment(0.125)); // mesh_id = 1 (edges)

    // Add materials with color coding BY MODE
    scene.materials.push(Material::from_rgb(0.5, 0.5, 0.5)); // Pillar - dark gray (material_id = 0)
    scene.materials.push(Material::from_rgb(0.0, 0.8, 0.0)); // Bike - green (material_id = 1)
    scene.materials.push(Material::from_rgb(0.0, 0.5, 1.0)); // Walk - blue (material_id = 2)
    scene.materials.push(Material::from_rgb(1.0, 0.0, 0.0)); // Transit - red (material_id = 3)
    scene.materials.push(Material::from_rgb(1.0, 0.8, 0.0)); // Car - orange (material_id = 4)

    // Group nodes by position (within 0.01 unit precision)
    let grouped_nodes = group_nodes_by_position(network);

    // Track mapping: (mode, node_index) -> scene_node_id for edge creation
    let mut node_to_scene_id: HashMap<(TransportMode, usize), u32> = HashMap::new();

    // Create pillars and child nodes
    for (_, mode_nodes) in &grouped_nodes {
        let avg_pos = calculate_average_position(network, mode_nodes);

        // Create parent pillar at base height (Y=0)
        let pillar_id = scene.nodes.len() as u32;
        scene.nodes.push(SceneNode::new(
            0, // cube mesh
            0, // pillar material (dark gray)
            Transform::new(
                [avg_pos[0], 0.0, avg_pos[1]], // Y=0 base
                [0.0, 0.0, 0.0],
                [0.5, 1.0, 0.5], // Tall thin box
            ),
            true, // selectable
        ));

        // Sort mode_nodes for consistent layer ordering (Car -> Bike -> Walk -> Transit)
        let mut sorted_modes = mode_nodes.clone();
        sorted_modes.sort_by_key(|(mode, _)| mode_sort_order(*mode));

        // Create child nodes for each mode, stacked vertically within pillar
        let layer_count = sorted_modes.len();
        for (layer_idx, (mode, node_idx)) in sorted_modes.iter().enumerate() {
            let y_offset = calculate_layer_y_offset(layer_idx, layer_count);

            let child_id = scene.nodes.len() as u32;
            scene.nodes.push(
                SceneNode::new(
                    0, // cube mesh
                    mode_material_id(*mode),
                    Transform::new(
                        [0.0, y_offset, 0.0], // Relative to parent (centered origin)
                        [0.0, 0.0, 0.0],
                        [0.15, 0.15, 0.15], // Keep existing size
                    ),
                    false, // NOT selectable - only pillar can be picked
                )
                .with_parent(pillar_id),
            );

            node_to_scene_id.insert((*mode, *node_idx), child_id);
        }
    }

    // Initialize edge node reference tracking (parallel to nodes array)
    scene.edge_node_refs = vec![None; scene.nodes.len()];

    // Create edges with endpoint tracking
    let modes = [
        TransportMode::Car,
        TransportMode::Bike,
        TransportMode::Walk,
        TransportMode::Transit,
    ];

    for mode in &modes {
        if let Some(graph) = network.graphs.get(mode) {
            for edge in &graph.edges {
                let from_scene_id = node_to_scene_id[&(*mode, edge.from_node)];
                let to_scene_id = node_to_scene_id[&(*mode, edge.to_node)];

                // Get world positions for initial edge transform
                let from_world = scene.compute_world_transform(from_scene_id);
                let to_world = scene.compute_world_transform(to_scene_id);

                let from_pos = extract_position_from_matrix(from_world);
                let to_pos = extract_position_from_matrix(to_world);

                scene.nodes.push(SceneNode::new(
                    1, // line mesh
                    mode_material_id(*mode),
                    edge_transform_from_positions(from_pos, to_pos),
                    false, // NOT selectable
                ));

                // Track which nodes this edge connects (parallel to nodes array)
                scene
                    .edge_node_refs
                    .push(Some((from_scene_id, to_scene_id)));
            }
        }
    }

    scene
}

/// Update all edges connected to a moved node
/// Call this after updating a node's position to keep edges synchronized
pub fn update_network_edges(scene: &mut Scene, moved_node_id: u32) {
    // Get all affected node IDs (parent + all children if parent moved)
    let affected_ids = if scene.nodes[moved_node_id as usize].parent_id.is_none() {
        // Parent moved - affects edges of all children
        let mut ids = vec![moved_node_id];
        ids.extend(scene.get_descendants(moved_node_id));
        ids
    } else {
        vec![moved_node_id]
    };

    // Update all edges connected to any affected node
    for (edge_idx, edge_ref) in scene.edge_node_refs.iter().enumerate() {
        if let Some((from_id, to_id)) = edge_ref {
            if affected_ids.contains(from_id) || affected_ids.contains(to_id) {
                // Recalculate edge transform based on new endpoint positions
                let from_world = scene.compute_world_transform(*from_id);
                let to_world = scene.compute_world_transform(*to_id);

                let from_pos = extract_position_from_matrix(from_world);
                let to_pos = extract_position_from_matrix(to_world);

                scene.nodes[edge_idx].transform = edge_transform_from_positions(from_pos, to_pos);
            }
        }
    }
}

/// Group nodes by position (0.01 unit precision)
/// Returns HashMap with quantized position as key and list of (mode, node_index) pairs
fn group_nodes_by_position(network: &Network) -> HashMap<[i32; 2], Vec<(TransportMode, usize)>> {
    let mut groups: HashMap<[i32; 2], Vec<(TransportMode, usize)>> = HashMap::new();

    for (mode, graph) in &network.graphs {
        for (node_idx, node) in graph.nodes.iter().enumerate() {
            // Quantize to 0.01 unit precision (multiply by 100, cast to i32)
            let quantized = [
                (node.position[0] * 100.0) as i32,
                (node.position[1] * 100.0) as i32,
            ];
            groups
                .entry(quantized)
                .or_insert_with(Vec::new)
                .push((*mode, node_idx));
        }
    }

    groups
}

/// Calculate average position of all nodes in a group
fn calculate_average_position(
    network: &Network,
    mode_nodes: &[(TransportMode, usize)],
) -> [f32; 2] {
    let mut sum = [0.0, 0.0];
    for (mode, idx) in mode_nodes {
        let pos = network.graphs[mode].nodes[*idx].position;
        sum[0] += pos[0];
        sum[1] += pos[1];
    }
    [
        sum[0] / mode_nodes.len() as f32,
        sum[1] / mode_nodes.len() as f32,
    ]
}

/// Calculate Y offset for a child node within its parent pillar
/// Pillar height is 1.0 with origin at center, so Y range is -0.5 to +0.5
fn calculate_layer_y_offset(layer_idx: usize, layer_count: usize) -> f32 {
    // Distribute children evenly with small margin on each end
    let margin = 0.1;
    let min_y = -0.5 + margin;
    let max_y = 0.5 - margin;

    if layer_count == 1 {
        return 0.0; // Single node at center
    }

    let spacing = (max_y - min_y) / (layer_count - 1) as f32;
    min_y + layer_idx as f32 * spacing
}

/// Extract position (translation) from a 4x4 transform matrix
fn extract_position_from_matrix(matrix: [[f32; 4]; 4]) -> Vec3 {
    Vec3::new(matrix[3][0], matrix[3][1], matrix[3][2])
}

/// Create edge transform from two 3D positions
/// Edge is centered at midpoint and stretched to span both endpoints
fn edge_transform_from_positions(from: Vec3, to: Vec3) -> Transform {
    let midpoint = (from + to) / 2.0;
    let diff = to - from;
    let length = diff.length();
    let angle = (-diff.z).atan2(diff.x);

    Transform::new(midpoint.to_array(), [0.0, angle, 0.0], [length, 1.0, 1.0])
}

/// Get consistent sort order for transport modes in pillar layers
fn mode_sort_order(mode: TransportMode) -> u8 {
    match mode {
        TransportMode::Car => 0,
        TransportMode::Bike => 1,
        TransportMode::Walk => 2,
        TransportMode::Transit => 3,
    }
}

/// Get material ID for a given transport mode
fn mode_material_id(mode: TransportMode) -> usize {
    match mode {
        TransportMode::Bike => 1,
        TransportMode::Walk => 2,
        TransportMode::Transit => 3,
        TransportMode::Car => 4,
    }
}
