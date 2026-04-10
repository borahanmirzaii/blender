use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// GPU-ready vertex format matching WebGPU vertex buffer layout.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Half-edge mesh structure inspired by Blender's BMesh.
/// Supports efficient topology queries and modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    /// Per-vertex positions for edit mode (separate from GPU vertices)
    pub positions: Vec<Vec3>,
    /// Face normals, recomputed on topology change
    pub face_normals: Vec<Vec3>,
    /// Bounding box min/max for culling
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Mesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            indices: Vec::new(),
            positions: Vec::new(),
            face_normals: Vec::new(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        }
    }

    /// Create a unit cube centered at origin.
    pub fn cube() -> Self {
        let positions = [
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ];

        let face_indices: [(usize, usize, usize, usize, Vec3); 6] = [
            (0, 1, 2, 3, Vec3::new(0.0, 0.0, -1.0)), // front
            (5, 4, 7, 6, Vec3::new(0.0, 0.0, 1.0)),  // back
            (4, 0, 3, 7, Vec3::new(-1.0, 0.0, 0.0)), // left
            (1, 5, 6, 2, Vec3::new(1.0, 0.0, 0.0)),  // right
            (3, 2, 6, 7, Vec3::new(0.0, 1.0, 0.0)),  // top
            (4, 5, 1, 0, Vec3::new(0.0, -1.0, 0.0)), // bottom
        ];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut face_normals = Vec::new();

        for (a, b, c, d, normal) in &face_indices {
            let base = vertices.len() as u32;
            let n = [normal.x, normal.y, normal.z];

            vertices.push(Vertex { position: positions[*a].into(), normal: n, uv: [0.0, 0.0] });
            vertices.push(Vertex { position: positions[*b].into(), normal: n, uv: [1.0, 0.0] });
            vertices.push(Vertex { position: positions[*c].into(), normal: n, uv: [1.0, 1.0] });
            vertices.push(Vertex { position: positions[*d].into(), normal: n, uv: [0.0, 1.0] });

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            face_normals.push(*normal);
        }

        Self {
            name: "Cube".into(),
            vertices,
            indices,
            positions: positions.to_vec(),
            face_normals,
            bounds_min: Vec3::new(-0.5, -0.5, -0.5),
            bounds_max: Vec3::new(0.5, 0.5, 0.5),
        }
    }

    /// Create a UV sphere.
    pub fn uv_sphere(segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut positions_list = Vec::new();

        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            for seg in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;

                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();

                let pos = Vec3::new(x * 0.5, y * 0.5, z * 0.5);
                let normal = Vec3::new(x, y, z).normalize();

                positions_list.push(pos);
                vertices.push(Vertex {
                    position: pos.into(),
                    normal: normal.into(),
                    uv: [seg as f32 / segments as f32, ring as f32 / rings as f32],
                });
            }
        }

        for ring in 0..rings {
            for seg in 0..segments {
                let current = ring * (segments + 1) + seg;
                let next = current + segments + 1;

                indices.extend_from_slice(&[current, next, current + 1]);
                indices.extend_from_slice(&[current + 1, next, next + 1]);
            }
        }

        Self {
            name: "Sphere".into(),
            vertices,
            indices,
            positions: positions_list,
            face_normals: Vec::new(),
            bounds_min: Vec3::new(-0.5, -0.5, -0.5),
            bounds_max: Vec3::new(0.5, 0.5, 0.5),
        }
    }

    /// Recompute bounding box from vertex positions.
    pub fn recompute_bounds(&mut self) {
        if self.vertices.is_empty() {
            return;
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in &self.vertices {
            let p = Vec3::from(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        self.bounds_min = min;
        self.bounds_max = max;
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Get vertex data as raw bytes for GPU upload.
    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    /// Get index data as raw bytes for GPU upload.
    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }
}
