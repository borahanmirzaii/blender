/// WGSL shaders for the WebGPU viewport renderer.
/// These mirror a simplified version of Blender's EEVEE shading model.

/// Basic PBR vertex/fragment shader in WGSL.
pub const MESH_SHADER_WGSL: &str = r#"
struct Uniforms {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad0: f32,
}

struct MaterialUniforms {
    base_color: vec4<f32>,  // rgb + alpha
    emissive: vec4<f32>,    // rgb + strength
    metallic: f32,
    roughness: f32,
    ior: f32,
    _pad: f32,
}

struct LightData {
    position: vec3<f32>,
    _pad0: f32,
    color: vec3<f32>,
    power: f32,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var<uniform> material: MaterialUniforms;
@group(2) @binding(0) var<uniform> light: LightData;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.clip_position = uniforms.projection * uniforms.view * world_pos;

    // Transform normal (using upper-left 3x3 of model matrix)
    let normal_mat = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz,
    );
    out.world_normal = normalize(normal_mat * in.normal);
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(uniforms.camera_pos - in.world_pos);
    let L = normalize(light.position - in.world_pos);
    let H = normalize(V + L);

    // Simplified PBR: Lambertian diffuse + Blinn-Phong specular
    let NdotL = max(dot(N, L), 0.0);
    let NdotH = max(dot(N, H), 0.0);

    let diffuse = material.base_color.rgb * NdotL;

    let roughness = max(material.roughness, 0.04);
    let spec_power = 2.0 / (roughness * roughness) - 2.0;
    let specular = vec3<f32>(pow(NdotH, spec_power)) * mix(vec3<f32>(0.04), material.base_color.rgb, material.metallic);

    let ambient = material.base_color.rgb * 0.1;
    let emissive = material.emissive.rgb * material.emissive.w;

    let light_intensity = light.color * light.power / max(dot(light.position - in.world_pos, light.position - in.world_pos), 1.0);
    let color = ambient + (diffuse + specular) * light_intensity + emissive;

    // Tone mapping (Reinhard)
    let mapped = color / (color + vec3<f32>(1.0));

    return vec4<f32>(mapped, material.base_color.a);
}
"#;

/// Grid shader for the 3D viewport floor grid.
pub const GRID_SHADER_WGSL: &str = r#"
struct Uniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // Full-screen quad vertices
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );

    var out: VertexOutput;
    let pos = positions[idx];
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);

    // Reconstruct world position on the Y=0 plane
    let inv_vp = transpose(uniforms.projection * uniforms.view);
    let world = inv_vp * vec4<f32>(pos, 0.0, 1.0);
    out.world_pos = world.xyz / world.w;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let grid_size = 1.0;
    let grid = abs(fract(in.world_pos.xz / grid_size - 0.5) - 0.5);
    let line = min(grid.x, grid.y);
    let alpha = 1.0 - smoothstep(0.0, 0.02, line);

    // Fade with distance
    let dist = length(in.world_pos.xz);
    let fade = 1.0 - smoothstep(10.0, 50.0, dist);

    return vec4<f32>(0.5, 0.5, 0.5, alpha * fade * 0.5);
}
"#;
