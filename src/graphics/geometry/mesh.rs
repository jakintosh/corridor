use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
            },
            // Back face
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
            },
            // Top face
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [0.0, 1.0, 0.0],
            },
            // Bottom face
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [0.0, -1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [0.0, -1.0, 0.0],
            },
            // Right face
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [1.0, 0.0, 0.0],
            },
            // Left face
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [-1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [-1.0, 0.0, 0.0],
            },
        ];

        let indices = vec![
            // Front face
            2, 1, 0, 3, 2, 0, // Back face
            6, 4, 5, 7, 4, 6, // Top face
            10, 9, 8, 11, 10, 8, // Bottom face
            14, 12, 13, 15, 12, 14, // Right face
            16, 17, 18, 16, 18, 19, // Left face
            21, 20, 22, 22, 20, 23,
        ];

        Self { vertices, indices }
    }

    pub fn quad() -> Self {
        let vertices = vec![
            Vertex {
                position: [-0.5, 0.0, -0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, -0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.0, 0.5],
                normal: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.0, 0.5],
                normal: [0.0, 1.0, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        Self { vertices, indices }
    }

    pub fn line_segment(width: f32) -> Self {
        // Create a thin quad (2 triangles) lying flat on the XZ plane
        // This renders as a thin line on the ground

        let vertices = vec![
            Vertex {
                position: [-0.5, 0.0, -width / 2.0],
                normal: [0.0, 1.0, 0.0],
            }, // Bottom left
            Vertex {
                position: [0.5, 0.0, -width / 2.0],
                normal: [0.0, 1.0, 0.0],
            }, // Bottom right
            Vertex {
                position: [0.5, 0.0, width / 2.0],
                normal: [0.0, 1.0, 0.0],
            }, // Top right
            Vertex {
                position: [-0.5, 0.0, width / 2.0],
                normal: [0.0, 1.0, 0.0],
            }, // Top left
        ];

        let indices = vec![
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ];

        Self { vertices, indices }
    }
}
