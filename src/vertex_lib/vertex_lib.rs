use std::mem;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
// basically struct will derive Pod Vertex & [u8] Zeroable std::mem::zeroed() Vertex
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    // x, y and z vertices coordinates
    position: [f32; 3],
    // color vertices
    color: [f32; 3],
}

// vertex as copy
// vertices coordinate counter clockwise
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

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
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}
