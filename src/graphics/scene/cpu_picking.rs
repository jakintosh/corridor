use crate::graphics::geometry::Vertex;
use glam::{Mat4, Vec3};

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

/// Compute axis-aligned bounding box for a mesh in world space
pub fn compute_node_aabb(vertices: &[Vertex], transform_matrix: Mat4) -> AABB {
    if vertices.is_empty() {
        return AABB {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        };
    }

    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for vertex in vertices {
        let pos = Vec3::from_array(vertex.position);
        let world_pos = transform_matrix.transform_point3(pos);
        min = min.min(world_pos);
        max = max.max(world_pos);
    }

    AABB { min, max }
}

/// Fast ray-AABB intersection using slab method
/// Returns distance along ray if hit, None if miss
pub fn ray_aabb_intersect(ray_origin: Vec3, ray_dir: Vec3, aabb: &AABB) -> Option<f32> {
    let inv_dir = Vec3::new(
        if ray_dir.x.abs() < 1e-7 {
            1e7
        } else {
            1.0 / ray_dir.x
        },
        if ray_dir.y.abs() < 1e-7 {
            1e7
        } else {
            1.0 / ray_dir.y
        },
        if ray_dir.z.abs() < 1e-7 {
            1e7
        } else {
            1.0 / ray_dir.z
        },
    );

    let t1 = (aabb.min - ray_origin) * inv_dir;
    let t2 = (aabb.max - ray_origin) * inv_dir;

    let tmin = t1.min(t2).max_element();
    let tmax = t1.max(t2).min_element();

    // Check if ray intersects and is in front of camera
    if tmax >= tmin && tmax >= 0.0 {
        Some(tmin.max(0.0))
    } else {
        None
    }
}

/// Möller-Trumbore ray-triangle intersection algorithm with backface culling
/// Returns distance along ray if hit, None if miss
///
/// References:
/// - https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
/// - Fast, Minimum Storage Ray/Triangle Intersection (1997)
pub fn ray_triangle_intersect(
    ray_origin: Vec3,
    ray_dir: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    const EPSILON: f32 = 0.0000001;

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray_dir.cross(edge2);
    let a = edge1.dot(h);

    // Ray is parallel to triangle
    if a > -EPSILON && a < EPSILON {
        return None;
    }

    // Backface culling - skip triangles facing away from camera
    let normal = edge1.cross(edge2);
    if normal.dot(ray_dir) < 0.0 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(h);

    // Intersection outside triangle
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray_dir.dot(q);

    // Intersection outside triangle
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // Compute t to find intersection point
    let t = f * edge2.dot(q);

    // Ray intersection (t > EPSILON means in front of ray origin)
    if t > EPSILON { Some(t) } else { None }
}

/// Test ray against all triangles in a mesh
/// Returns closest hit distance, or None if no hit
pub fn ray_mesh_intersect(
    ray_origin: Vec3,
    ray_dir: Vec3,
    vertices: &[Vertex],
    indices: &[u32],
    transform_matrix: Mat4,
) -> Option<f32> {
    let mut closest_t = f32::INFINITY;

    // Process triangles (every 3 indices)
    for triangle in indices.chunks(3) {
        if triangle.len() != 3 {
            continue;
        }

        // Get triangle vertices in local space
        let v0_local = Vec3::from_array(vertices[triangle[0] as usize].position);
        let v1_local = Vec3::from_array(vertices[triangle[1] as usize].position);
        let v2_local = Vec3::from_array(vertices[triangle[2] as usize].position);

        // Transform to world space
        let v0 = transform_matrix.transform_point3(v0_local);
        let v1 = transform_matrix.transform_point3(v1_local);
        let v2 = transform_matrix.transform_point3(v2_local);

        if let Some(t) = ray_triangle_intersect(ray_origin, ray_dir, v0, v1, v2) {
            if t < closest_t {
                closest_t = t;
            }
        }
    }

    if closest_t < f32::INFINITY {
        Some(closest_t)
    } else {
        None
    }
}
