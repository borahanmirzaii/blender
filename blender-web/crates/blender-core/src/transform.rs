use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// 3D transform with position, rotation, scale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            ..Default::default()
        }
    }

    /// Column-major 4x4 matrix for GPU upload.
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Column-major f32 array for wasm-bindgen export.
    pub fn to_cols_array(&self) -> [f32; 16] {
        self.to_matrix().to_cols_array()
    }
}
