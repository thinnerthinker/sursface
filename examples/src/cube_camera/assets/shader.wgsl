struct Uniforms {
    model_view_proj: mat4x4<f32>,
    camera_pan: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@group(1) @binding(0) var myTexture: texture_2d<f32>;
@group(1) @binding(1) var mySampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fragUV: vec2<f32>,
    @location(1) fragNormal: vec3<f32>,
    @location(2) fragPosition: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>, 
    @location(1) normal: vec3<f32>, 
    @location(2) uv: vec2<f32>
) -> VertexOutput {
    var output: VertexOutput;
    let worldPosition = uniforms.camera_pan * vec4<f32>(position, 1.0);
    output.position = uniforms.model_view_proj * worldPosition;
    output.fragUV = uv;
    output.fragNormal = normalize((uniforms.camera_pan * vec4<f32>(normal, 0.0)).xyz);
    output.fragPosition = worldPosition.xyz;
    return output;
}

@fragment
fn fs_main(
    @location(0) fragUV: vec2<f32>, 
    @location(1) fragNormal: vec3<f32>, 
    @location(2) fragPosition: vec3<f32>
) -> @location(0) vec4<f32> {
    // Hardcoded light direction and color
    let lightDir = normalize(vec3<f32>(0.5, 0.5, 0.5));
    let lightColor = vec3<f32>(1.0, 1.0, 1.0);
    let ambientColor = vec3<f32>(0.1, 0.1, 0.1);

    // Hardcoded camera position
    let viewPos = vec3<f32>(0.0, 0.0, 5.0);

    // Phong reflection model calculations
    let normal = normalize(fragNormal);
    let viewDir = normalize(viewPos - fragPosition);
    let reflectDir = reflect(-lightDir, normal);

    // Ambient component
    let ambient = ambientColor;

    // Diffuse component
    let diff = max(dot(normal, lightDir), 0.0);
    let diffuse = diff * lightColor;

    // Specular component
    let specularStrength = 0.5;
    let spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
    let specular = specularStrength * spec * lightColor;

    // Combine all components
    let lighting = ambient + diffuse + specular;

    // Texture color
    let texColor = textureSample(myTexture, mySampler, fragUV).rgb;

    // Apply sepia tone
    let sepia = vec3<f32>(
        texColor.r * 0.393 + texColor.g * 0.769 + texColor.b * 0.189,
        texColor.r * 0.349 + texColor.g * 0.686 + texColor.b * 0.168,
        texColor.r * 0.272 + texColor.g * 0.534 + texColor.b * 0.131
    );

    // Final color with sepia effect
    let finalColor = vec4<f32>(lighting * sepia, 1.0);
    return finalColor;
}