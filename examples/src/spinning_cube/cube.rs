use bytemuck::{Pod,Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

const fn face_uvs(top_left: (f32, f32), bottom_right: (f32, f32)) -> [[f32; 2]; 4] {
    [
        [top_left.0, top_left.1],
        [bottom_right.0, top_left.1],
        [bottom_right.0, bottom_right.1],
        [top_left.0, bottom_right.1],
    ]
}

pub const VERTICES: &[Vertex] = {
    let uvs = [
        face_uvs((0.00, 1f32 / 3f32), (0.25 + 0.00, 2f32 / 3f32)), // 6
        face_uvs((0.50, 1f32 / 3f32), (0.25 + 0.50, 2f32 / 3f32)), // 1
        face_uvs((0.25, 1f32 / 3f32), (0.25 + 0.25, 2f32 / 3f32)), // 4
        face_uvs((0.75, 1f32 / 3f32), (0.25 + 0.75, 2f32 / 3f32)), // 3
        face_uvs((0.50, 0f32 / 3f32), (0.25 + 0.50, 1f32 / 3f32)), // 2
        face_uvs((0.50, 2f32 / 3f32), (0.25 + 0.50, 3f32 / 3f32)), // 5
    ];
    
    &[
        // 6
        Vertex { position: [-1.0, -1.0,  1.0], uv: uvs[0][0] },
        Vertex { position: [ 1.0, -1.0,  1.0], uv: uvs[0][1] },
        Vertex { position: [ 1.0,  1.0,  1.0], uv: uvs[0][2] },
        Vertex { position: [-1.0,  1.0,  1.0], uv: uvs[0][3] },

        // 1
        Vertex { position: [-1.0, -1.0, -1.0], uv: uvs[1][0] },
        Vertex { position: [ 1.0, -1.0, -1.0], uv: uvs[1][1] },
        Vertex { position: [ 1.0,  1.0, -1.0], uv: uvs[1][2] },
        Vertex { position: [-1.0,  1.0, -1.0], uv: uvs[1][3] },

        // 4
        Vertex { position: [-1.0, -1.0, -1.0], uv: uvs[2][0] },
        Vertex { position: [-1.0,  1.0, -1.0], uv: uvs[2][1] },
        Vertex { position: [-1.0,  1.0,  1.0], uv: uvs[2][2] },
        Vertex { position: [-1.0, -1.0,  1.0], uv: uvs[2][3] },

        // 3
        Vertex { position: [ 1.0, -1.0, -1.0], uv: uvs[3][0] },
        Vertex { position: [ 1.0,  1.0, -1.0], uv: uvs[3][1] },
        Vertex { position: [ 1.0,  1.0,  1.0], uv: uvs[3][2] },
        Vertex { position: [ 1.0, -1.0,  1.0], uv: uvs[3][3] },

        // 2
        Vertex { position: [-1.0, -1.0, -1.0], uv: uvs[4][0] },
        Vertex { position: [ 1.0, -1.0, -1.0], uv: uvs[4][1] },
        Vertex { position: [ 1.0, -1.0,  1.0], uv: uvs[4][2] },
        Vertex { position: [-1.0, -1.0,  1.0], uv: uvs[4][3] },

        // 5
        Vertex { position: [-1.0,  1.0, -1.0], uv: uvs[5][0] },
        Vertex { position: [ 1.0,  1.0, -1.0], uv: uvs[5][1] },
        Vertex { position: [ 1.0,  1.0,  1.0], uv: uvs[5][2] },
        Vertex { position: [-1.0,  1.0,  1.0], uv: uvs[5][3] },
    ]
};

pub const INDICES: &[u16] = &[
    // 6
    0, 1, 2, 2, 3, 0,
    // 1
    4, 6, 5, 6, 4, 7,
    // 4
    8, 10, 9, 10, 8, 11,
    // 3
    12, 13, 14, 14, 15, 12,
    // 2
    16, 17, 18, 18, 19, 16,
    // 5
    20, 22, 21, 22, 20, 23,
];