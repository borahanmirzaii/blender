use glam::Vec3;
use serde::{Deserialize, Serialize};

/// PBR material matching glTF 2.0 metallic-roughness model.
/// Maps to WebGPU uniform buffer for shader consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub base_color: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: Vec3,
    pub emissive_strength: f32,
    pub alpha: f32,
    pub ior: f32,
    /// Index into texture array, None = no texture
    pub base_color_texture: Option<u32>,
    pub normal_texture: Option<u32>,
    pub metallic_roughness_texture: Option<u32>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            base_color: Vec3::new(0.8, 0.8, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            emissive: Vec3::ZERO,
            emissive_strength: 1.0,
            alpha: 1.0,
            ior: 1.45,
            base_color_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
        }
    }
}

impl Material {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a material from a simple color.
    pub fn from_color(name: impl Into<String>, r: f32, g: f32, b: f32) -> Self {
        Self {
            name: name.into(),
            base_color: Vec3::new(r, g, b),
            ..Default::default()
        }
    }

    /// Pack material data into a GPU-friendly uniform struct (16-byte aligned).
    pub fn to_gpu_uniforms(&self) -> MaterialUniforms {
        MaterialUniforms {
            base_color: [self.base_color.x, self.base_color.y, self.base_color.z, self.alpha],
            emissive: [self.emissive.x, self.emissive.y, self.emissive.z, self.emissive_strength],
            metallic: self.metallic,
            roughness: self.roughness,
            ior: self.ior,
            _padding: 0.0,
        }
    }
}

/// GPU uniform buffer layout for material data (64 bytes, 16-byte aligned).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MaterialUniforms {
    pub base_color: [f32; 4], // rgb + alpha
    pub emissive: [f32; 4],   // rgb + strength
    pub metallic: f32,
    pub roughness: f32,
    pub ior: f32,
    pub _padding: f32,
}
