struct Uniforms {
    translation: vec2<f32>,
    cursor_pos: vec2<f32>,
    scale: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragUV: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(position, 1.0);
    output.fragUV = uv;
    return output;
}

@fragment
fn fs_main(@location(0) fragUV: vec2<f32>) -> @location(0) vec4<f32> {
    let max_iter: u32 = 100u;

    // Calculate the coordinates in the complex plane
    let centered_uv = fragUV * 2.0 - vec2<f32>(1.0, 1.0); // Convert [0,1] range to [-1,1]
    
    // Adjust coordinates based on translation and scale, without affecting rotation
    let c = uniforms.translation + vec2<f32>(
        centered_uv.x * uniforms.scale,
        centered_uv.y * uniforms.scale
    );

    var z: vec2<f32> = vec2<f32>(0.0, 0.0);
    var iter: u32 = 0u;

    // Iterate to determine if the point is in the Mandelbrot set
    for (var i: u32 = 0u; i < max_iter; i = i + 1u) {
        if (dot(z, z) > 4.0) {
            break;
        }
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );
        iter = iter + 1u;
    }

    if (iter == max_iter) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Inside the Mandelbrot set
    } else {
        let t: f32 = f32(iter) / f32(max_iter);
        return vec4<f32>(t * 0.15, t * 0.35, t * 0.25, 1.0); // Gradient color for points outside
    }
}
