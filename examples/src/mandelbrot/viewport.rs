use bytemuck::{Pod,Zeroable};
use sursface::wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> sursface::wgpu::VertexBufferLayout<'a> {
        use std::mem;
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const fn uvs(top_left: (f32, f32), bottom_right: (f32, f32)) -> [[f32; 2]; 4] {
    [
        [top_left.0, top_left.1],
        [bottom_right.0, top_left.1],
        [bottom_right.0, bottom_right.1],
        [top_left.0, bottom_right.1],
    ]
}

pub const VERTICES: &[Vertex] = {
    let uvs = uvs((0.00, 1f32 / 3f32), (0.25 + 0.00, 2f32 / 3f32));
    
    &[
        Vertex { position: [-1.0, -1.0,  1.0], uv: uvs[0] },
        Vertex { position: [ 1.0, -1.0,  1.0], uv: uvs[1] },
        Vertex { position: [ 1.0,  1.0,  1.0], uv: uvs[2] },
        Vertex { position: [-1.0,  1.0,  1.0], uv: uvs[3] },
    ]
};

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,
];