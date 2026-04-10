use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

/// Per-object uniforms uploaded to the GPU each frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ObjectUniforms {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _pad0: f32,
}

impl ObjectUniforms {
    pub fn new(model: Mat4, view: Mat4, projection: Mat4, camera_pos: Vec3) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            view: view.to_cols_array_2d(),
            projection: projection.to_cols_array_2d(),
            camera_pos: camera_pos.into(),
            _pad0: 0.0,
        }
    }
}

/// Per-material uniforms (matches WGSL MaterialUniforms struct).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialGpuData {
    pub base_color: [f32; 4],
    pub emissive: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ior: f32,
    pub _pad: f32,
}

/// Light uniform data.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightGpuData {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub color: [f32; 3],
    pub power: f32,
}
