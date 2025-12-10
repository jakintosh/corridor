use super::{Material, Scene, SceneNode, Transform};
use crate::graphics::geometry::Mesh;

pub fn create_demo_scene() -> Scene {
    let mut scene = Scene::new();

    // Create meshes
    let cube_mesh = Mesh::cube();
    let quad_mesh = Mesh::quad();
    let line_mesh = Mesh::line_segment(0.05);

    scene.meshes.push(cube_mesh); // mesh_id = 0
    scene.meshes.push(quad_mesh); // mesh_id = 1
    scene.meshes.push(line_mesh); // mesh_id = 2

    // Create materials
    scene.materials.push(Material::from_rgb(1.0, 0.0, 0.0)); // Red, material_id = 0
    scene.materials.push(Material::from_rgb(0.0, 0.0, 1.0)); // Blue, material_id = 1
    scene.materials.push(Material::from_rgb(0.0, 1.0, 0.0)); // Green, material_id = 2
    scene.materials.push(Material::from_rgb(0.5, 0.5, 0.5)); // Gray, material_id = 3
    scene.materials.push(Material::from_rgb(0.0, 0.0, 0.0)); // Gray, material_id = 4

    // Ground plane (large gray quad)
    scene.nodes.push(SceneNode::new(
        1,
        3,
        Transform::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [10.0, 1.0, 10.0]),
        true,
    ));

    // Red cube at (-2, 0.5, 0) - will rotate on Y axis
    scene.nodes.push(SceneNode::new(
        0,
        0,
        Transform::new([-2.0, 0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        true,
    ));

    // Blue cube at (2, 0.5, 0) - will rotate on X axis
    scene.nodes.push(SceneNode::new(
        0,
        1,
        Transform::new([2.0, 0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        true,
    ));

    // Small green cube at (0, 1, 2) - will rotate on multiple axes
    scene.nodes.push(SceneNode::new(
        0,
        2,
        Transform::new([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
        true,
    ));

    // Additional cubes showing instancing
    scene.nodes.push(SceneNode::new(
        0,
        0,
        Transform::new([-4.0, 0.3, -3.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
        true,
    ));

    scene.nodes.push(SceneNode::new(
        0,
        1,
        Transform::new([4.0, 0.3, -3.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
        true,
    ));

    scene.nodes.push(SceneNode::new(
        0,
        2,
        Transform::new([0.0, 0.3, -4.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
        true,
    ));

    // Line segments forming a simple grid on the ground
    // Lines along X axis
    for i in -2..=2 {
        scene.nodes.push(SceneNode::new(
            2,
            4,
            Transform::new(
                [0.0, 0.01, i as f32 * 2.0],
                [0.0, 0.0, 0.0],
                [8.0, 1.0, 1.0],
            ),
            true,
        ));
    }

    // Lines along Z axis
    for i in -2..=2 {
        scene.nodes.push(SceneNode::new(
            2,
            4,
            Transform::new(
                [i as f32 * 2.0, 0.01, 0.0],
                [0.0, std::f32::consts::FRAC_PI_2, 0.0],
                [8.0, 1.0, 1.0],
            ),
            true,
        ));
    }

    // Parent/child hierarchy demonstration
    // Create yellow material for parent
    scene.materials.push(Material::from_rgb(1.0, 1.0, 0.0)); // Yellow, material_id = 5

    // Parent cube (yellow)
    let parent_idx = scene.nodes.len() as u32;
    scene.nodes.push(SceneNode::new(
        0,
        5,
        Transform::new([0.0, 1.5, -2.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
        true,
    ));

    // Create cyan material for first child
    scene.materials.push(Material::from_rgb(0.0, 1.0, 1.0)); // Cyan, material_id = 6

    // Child cube 1 (cyan) - offset to the right of parent (local space)
    scene.nodes.push(
        SceneNode::new(
            0,
            6,
            Transform::new([1.5, 0.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
            true,
        )
        .with_parent(parent_idx)
    );

    // Create magenta material for second child
    scene.materials.push(Material::from_rgb(1.0, 0.0, 1.0)); // Magenta, material_id = 7

    // Child cube 2 (magenta) - offset to the left of parent (local space)
    scene.nodes.push(
        SceneNode::new(
            0,
            7,
            Transform::new([-1.5, 0.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
            true,
        )
        .with_parent(parent_idx)
    );

    // Create white material for grandchild
    scene.materials.push(Material::from_rgb(1.0, 1.0, 1.0)); // White, material_id = 8

    // Grandchild cube (white) - child of cyan cube
    let cyan_idx = (parent_idx + 1) as u32;
    scene.nodes.push(
        SceneNode::new(
            0,
            8,
            Transform::new([0.0, 1.0, 0.0], [0.0, 0.0, 0.0], [0.3, 0.3, 0.3]),
            true,
        )
        .with_parent(cyan_idx)
    );

    scene
}
