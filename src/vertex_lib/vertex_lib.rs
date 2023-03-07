use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
// basically struct will derive Pod Vertex & [u8] Zeroable std::mem::zeroed() Vertex
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    // x, y and z vertices coordinates
    position: [f32; 3],
    tex_coordinates: [f32; 2],
}

// <----- CHANGES ----->
// vertex as copy
// vertices coordinate counter clockwise
// Removing duplicate vertexs
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coordinates: [9.4131759, 0.99240386],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coordinates: [0.0048659444, 0.56958647],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coordinates: [0.28081453, 0.05060294],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coordinates: [0.85967, 0.1526709],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coordinates: [0.9414737, 0.7347359],
    }, // E
];

// Create matric for shortest path (adjencency) in between the polygonated vertex
pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

impl Vertex {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress, // vertex wide. 24 bytes per vertex
            step_mode: VertexStepMode::Vertex, // telling pipeline each element of the array representing per-vertex data
            // 1:1 mapping with vertex struct attributes
            attributes: &[
                VertexAttribute {
                    offset: 0,                       // bytes start for vertex
                    shader_location: 0, // location to store shader attributes. @location(0) x: vec<32> position Vertex @location(1) x:vec<32> color
                    format: VertexFormat::Float32x3, // shape of attribute
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}
