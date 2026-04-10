use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// PBR metallic-roughness material (matches glTF 2.0 model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub base_color: Vec3,
    pub alpha: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: Vec3,
    pub emissive_strength: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".into(),
            base_color: Vec3::new(0.8, 0.8, 0.8),
            alpha: 1.0,
            metallic: 0.0,
            roughness: 0.5,
            emissive: Vec3::ZERO,
            emissive_strength: 1.0,
        }
    }
}

impl Material {
    pub fn from_color(name: impl Into<String>, r: f32, g: f32, b: f32) -> Self {
        Self {
            name: name.into(),
            base_color: Vec3::new(r, g, b),
            ..Default::default()
        }
    }

    /// Pack for GPU uniform buffer (48 bytes, 16-byte aligned).
    pub fn to_gpu(&self) -> MaterialGpu {
        MaterialGpu {
            base_color: [self.base_color.x, self.base_color.y, self.base_color.z, self.alpha],
            emissive: [self.emissive.x, self.emissive.y, self.emissive.z, self.emissive_strength],
            metallic: self.metallic,
            roughness: self.roughness,
            _pad: [0.0; 2],
        }
    }
}

/// GPU uniform layout (48 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct MaterialGpu {
    pub base_color: [f32; 4],
    pub emissive: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub _pad: [f32; 2],
}
