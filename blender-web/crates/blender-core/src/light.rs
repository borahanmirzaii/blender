use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Sun,
    Spot { angle: f32, blend: f32 },
    Area { size_x: f32, size_y: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub color: [f32; 3],
    pub power: f32,
    pub light_type: LightType,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0],
            power: 100.0,
            light_type: LightType::Point,
        }
    }
}

/// GPU uniform for a single light (32 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightGpu {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub color: [f32; 3],
    pub power: f32,
}
