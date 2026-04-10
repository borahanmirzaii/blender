/// Render pipeline configuration types.
/// These describe the pipeline state without coupling to a specific GPU API,
/// so the same configuration can be used for both WebGPU (browser) and
/// wgpu (native/server).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub vertex_shader: String,
    pub fragment_shader: String,
    pub vertex_layout: VertexLayout,
    pub depth_test: bool,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
    pub topology: PrimitiveTopology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexLayout {
    pub stride: u32,
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub location: u32,
    pub offset: u32,
    pub format: VertexFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlendMode {
    Opaque,
    AlphaBlend,
    Additive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PrimitiveTopology {
    TriangleList,
    LineList,
    PointList,
}

impl PipelineConfig {
    /// Default mesh rendering pipeline.
    pub fn mesh_default() -> Self {
        Self {
            vertex_shader: "mesh".into(),
            fragment_shader: "mesh".into(),
            vertex_layout: VertexLayout {
                stride: 32, // 3*f32 pos + 3*f32 normal + 2*f32 uv
                attributes: vec![
                    VertexAttribute {
                        location: 0,
                        offset: 0,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        location: 1,
                        offset: 12,
                        format: VertexFormat::Float32x3,
                    },
                    VertexAttribute {
                        location: 2,
                        offset: 24,
                        format: VertexFormat::Float32x2,
                    },
                ],
            },
            depth_test: true,
            blend_mode: BlendMode::Opaque,
            cull_mode: CullMode::Back,
            topology: PrimitiveTopology::TriangleList,
        }
    }

    /// Wireframe overlay pipeline.
    pub fn wireframe() -> Self {
        Self {
            vertex_shader: "wireframe".into(),
            fragment_shader: "wireframe".into(),
            vertex_layout: VertexLayout {
                stride: 12,
                attributes: vec![VertexAttribute {
                    location: 0,
                    offset: 0,
                    format: VertexFormat::Float32x3,
                }],
            },
            depth_test: true,
            blend_mode: BlendMode::Opaque,
            cull_mode: CullMode::None,
            topology: PrimitiveTopology::LineList,
        }
    }
}
