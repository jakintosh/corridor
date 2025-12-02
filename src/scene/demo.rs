use super::{Material, Scene, SceneNode, Transform};
use crate::geometry::Mesh;

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
    ));

    // Red cube at (-2, 0.5, 0) - will rotate on Y axis
    scene.nodes.push(SceneNode::new(
        0,
        0,
        Transform::new([-2.0, 0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
    ));

    // Blue cube at (2, 0.5, 0) - will rotate on X axis
    scene.nodes.push(SceneNode::new(
        0,
        1,
        Transform::new([2.0, 0.5, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
    ));

    // Small green cube at (0, 1, 2) - will rotate on multiple axes
    scene.nodes.push(SceneNode::new(
        0,
        2,
        Transform::new([0.0, 1.0, 2.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
    ));

    // Additional cubes showing instancing
    scene.nodes.push(SceneNode::new(
        0,
        0,
        Transform::new([-4.0, 0.3, -3.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
    ));

    scene.nodes.push(SceneNode::new(
        0,
        1,
        Transform::new([4.0, 0.3, -3.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
    ));

    scene.nodes.push(SceneNode::new(
        0,
        2,
        Transform::new([0.0, 0.3, -4.0], [0.0, 0.0, 0.0], [0.6, 0.6, 0.6]),
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
        ));
    }

    scene
}
