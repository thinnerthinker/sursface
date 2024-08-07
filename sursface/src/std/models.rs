use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct VertexPositionUv {
    position: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct VertexPositionNormalUv {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

pub const fn quad_uvs(top_left: (f32, f32), bottom_right: (f32, f32)) -> [[f32; 2]; 4] {
    [
        [top_left.0, top_left.1],
        [bottom_right.0, top_left.1],
        [bottom_right.0, bottom_right.1],
        [top_left.0, bottom_right.1],
    ]
}

pub const fn quad(
    pos1: [f32; 3],
    pos2: [f32; 3],
    pos3: [f32; 3],
    pos4: [f32; 3],
    normal: [f32; 3],
    uvs: [[f32; 2]; 4],
) -> [VertexPositionNormalUv; 6] {
    [
        VertexPositionNormalUv {
            position: pos1,
            normal,
            uv: uvs[0],
        },
        VertexPositionNormalUv {
            position: pos2,
            normal,
            uv: uvs[1],
        },
        VertexPositionNormalUv {
            position: pos3,
            normal,
            uv: uvs[2],
        },
        VertexPositionNormalUv {
            position: pos3,
            normal,
            uv: uvs[2],
        },
        VertexPositionNormalUv {
            position: pos4,
            normal,
            uv: uvs[3],
        },
        VertexPositionNormalUv {
            position: pos1,
            normal,
            uv: uvs[0],
        },
    ]
}

pub const fn quad_no_normal(
    pos1: [f32; 3],
    pos2: [f32; 3],
    pos3: [f32; 3],
    pos4: [f32; 3],
    uvs: [[f32; 2]; 4],
) -> [VertexPositionUv; 6] {
    [
        VertexPositionUv {
            position: pos1,
            uv: uvs[0],
        },
        VertexPositionUv {
            position: pos2,
            uv: uvs[1],
        },
        VertexPositionUv {
            position: pos3,
            uv: uvs[2],
        },
        VertexPositionUv {
            position: pos3,
            uv: uvs[2],
        },
        VertexPositionUv {
            position: pos4,
            uv: uvs[3],
        },
        VertexPositionUv {
            position: pos1,
            uv: uvs[0],
        },
    ]
}

pub const fn cube(uvs: &[[[f32; 2]; 4]; 6]) -> [VertexPositionNormalUv; 36] {  
    let front = quad(
        [-1.0, -1.0,  1.0],
        [ 1.0, -1.0,  1.0],
        [ 1.0,  1.0,  1.0],
        [-1.0,  1.0,  1.0], 
        [ 0.0,  0.0,  1.0], 
        uvs[0]
    );
    let back = quad(
        [-1.0, -1.0,  -1.0],
        [-1.0,  1.0,  -1.0],
        [ 1.0,  1.0,  -1.0],
        [ 1.0, -1.0,  -1.0], 
        [ 0.0,  0.0, -1.0], 
        uvs[1]
    );
    let left = quad(
        [-1.0, -1.0,  -1.0],
        [-1.0, -1.0,   1.0],
        [-1.0,  1.0,   1.0],
        [-1.0,  1.0,  -1.0], 
        [-1.0, 0.0,  0.0], 
        uvs[2]
    );
    let right = quad(
        [ 1.0, -1.0, -1.0],
        [ 1.0,  1.0, -1.0],
        [ 1.0,  1.0,  1.0],
        [ 1.0, -1.0,  1.0],
        [ 1.0,  0.0,  0.0], 
        uvs[3]
    );
    let bottom = quad(
        [-1.0, -1.0, -1.0],
        [ 1.0, -1.0, -1.0],
        [ 1.0, -1.0,  1.0],
        [-1.0, -1.0,  1.0],
        [ 0.0, -1.0,  0.0], 
        uvs[4]
    );
    let top = quad(
        [-1.0,  1.0, -1.0],
        [-1.0,  1.0,  1.0],
        [ 1.0,  1.0,  1.0],
        [ 1.0,  1.0, -1.0],
        [ 0.0,  1.0,  0.0], 
        uvs[5]
    );

    [
        front[0], front[1], front[2], front[3], front[4], front[5],
        back[0], back[1], back[2], back[3], back[4], back[5],
        left[0], left[1], left[2], left[3], left[4], left[5],
        right[0], right[1], right[2], right[3], right[4], right[5],
        bottom[0], bottom[1], bottom[2], bottom[3], bottom[4], bottom[5],
        top[0], top[1], top[2], top[3], top[4], top[5],
    ]
}
