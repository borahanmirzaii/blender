use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// GPU-ready vertex (32 bytes, matches WebGPU vertex buffer layout).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

/// Evaluated mesh — ready for GPU upload. Not an edit mesh.
/// In Blender terms, this is the result after modifier evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl Mesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertices: Vec::new(),
            indices: Vec::new(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        }
    }

    /// Create a unit cube centered at origin.
    pub fn cube() -> Self {
        let p = [
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
        ];
        let faces: &[(usize, usize, usize, usize, [f32; 3])] = &[
            (0, 3, 2, 1, [0.0, 0.0, -1.0]),
            (4, 5, 6, 7, [0.0, 0.0, 1.0]),
            (0, 4, 7, 3, [-1.0, 0.0, 0.0]),
            (1, 2, 6, 5, [1.0, 0.0, 0.0]),
            (3, 7, 6, 2, [0.0, 1.0, 0.0]),
            (0, 1, 5, 4, [0.0, -1.0, 0.0]),
        ];
        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        for &(a, b, c, d, n) in faces {
            let base = vertices.len() as u32;
            vertices.push(Vertex { position: p[a].into(), normal: n, uv: [0.0, 0.0] });
            vertices.push(Vertex { position: p[b].into(), normal: n, uv: [1.0, 0.0] });
            vertices.push(Vertex { position: p[c].into(), normal: n, uv: [1.0, 1.0] });
            vertices.push(Vertex { position: p[d].into(), normal: n, uv: [0.0, 1.0] });
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        Self {
            name: "Cube".into(),
            vertices,
            indices,
            bounds_min: Vec3::splat(-0.5),
            bounds_max: Vec3::splat(0.5),
        }
    }

    /// Create a UV sphere.
    pub fn sphere(segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            for seg in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * seg as f32 / segments as f32;
                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();
                let pos = Vec3::new(x * 0.5, y * 0.5, z * 0.5);
                let n = Vec3::new(x, y, z).normalize();
                vertices.push(Vertex {
                    position: pos.into(),
                    normal: n.into(),
                    uv: [seg as f32 / segments as f32, ring as f32 / rings as f32],
                });
            }
        }
        for ring in 0..rings {
            for seg in 0..segments {
                let cur = ring * (segments + 1) + seg;
                let next = cur + segments + 1;
                indices.extend_from_slice(&[cur, next, cur + 1, cur + 1, next, next + 1]);
            }
        }
        Self {
            name: "Sphere".into(),
            vertices,
            indices,
            bounds_min: Vec3::splat(-0.5),
            bounds_max: Vec3::splat(0.5),
        }
    }

    pub fn plane() -> Self {
        let n = [0.0, 1.0, 0.0];
        let vertices = vec![
            Vertex { position: [-0.5, 0.0, -0.5], normal: n, uv: [0.0, 0.0] },
            Vertex { position: [0.5, 0.0, -0.5], normal: n, uv: [1.0, 0.0] },
            Vertex { position: [0.5, 0.0, 0.5], normal: n, uv: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.0, 0.5], normal: n, uv: [0.0, 1.0] },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];
        Self {
            name: "Plane".into(),
            vertices,
            indices,
            bounds_min: Vec3::new(-0.5, 0.0, -0.5),
            bounds_max: Vec3::new(0.5, 0.0, 0.5),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn vertex_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn index_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.indices)
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_correct_counts() {
        let cube = Mesh::cube();
        assert_eq!(cube.vertex_count(), 24); // 6 faces * 4 verts
        assert_eq!(cube.triangle_count(), 12); // 6 faces * 2 tris
    }

    #[test]
    fn sphere_has_correct_topology() {
        let sphere = Mesh::sphere(16, 8);
        assert!(sphere.vertex_count() > 0);
        assert!(sphere.triangle_count() > 0);
        assert_eq!(sphere.indices.len() % 3, 0);
    }

    #[test]
    fn vertex_is_32_bytes() {
        assert_eq!(std::mem::size_of::<Vertex>(), 32);
    }

    #[test]
    fn mesh_serializes_roundtrip() {
        let cube = Mesh::cube();
        let json = serde_json::to_string(&cube).unwrap();
        let loaded: Mesh = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.vertex_count(), cube.vertex_count());
        assert_eq!(loaded.triangle_count(), cube.triangle_count());
    }
}
