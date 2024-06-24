struct Uniforms {
    model_view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragUV: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.model_view_proj * vec4<f32>(position, 1.0);
    output.fragUV = uv;
    return output;
}

@fragment
fn fs_main(@location(0) fragUV: vec2<f32>) -> @location(0) vec4<f32> {
    // Parameters for Mandelbrot calculation
    let max_iter: u32 = 100u;
    let zoom: f32 = 1.5;
    let offset: vec2<f32> = vec2<f32>(-0.5, 0.0);

    // Map the UV coordinates to the complex plane
    let uv_transformed = uniforms.model_view_proj * vec4<f32>(fragUV.x * 3.5 - 2.5, fragUV.y * 2.0 - 1.0, 0.0, 1.0);
    let c: vec2<f32> = vec2<f32>(uv_transformed.x, uv_transformed.y) / zoom + offset;

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

    if iter == max_iter {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);  // Inside the Mandelbrot set
    } else {
        let t: f32 = f32(iter) / f32(max_iter);
        return vec4<f32>(t, t * 0.5, t * 0.25, 1.0);  // Gradient color for points outside
    }
}
