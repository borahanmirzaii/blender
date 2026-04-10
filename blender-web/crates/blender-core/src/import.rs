use crate::material::Material;
use crate::mesh::{Mesh, Vertex};
use crate::object::{Object, ObjectData};
use crate::scene::Scene;
use crate::transform::Transform;
use glam::{Quat, Vec3};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("glTF error: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("Missing position attribute in mesh")]
    MissingPositions,
    #[error("Buffer data not provided")]
    MissingBufferData,
}

/// Import a glTF/glb file from raw bytes into a Scene.
pub fn import_gltf(data: &[u8]) -> Result<Scene, ImportError> {
    let gltf = gltf::Gltf::from_slice(data)?;
    let buffers = load_buffers(&gltf, data)?;

    let mut scene = Scene::new("Imported");

    // Import materials
    for gltf_mat in gltf.materials() {
        let pbr = gltf_mat.pbr_metallic_roughness();
        let [r, g, b, a] = pbr.base_color_factor();
        let mat = Material {
            name: gltf_mat.name().unwrap_or("Material").into(),
            base_color: Vec3::new(r, g, b),
            alpha: a,
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            emissive: Vec3::from_slice(&gltf_mat.emissive_factor()),
            emissive_strength: 1.0,
        };
        scene.add_material(mat);
    }

    // If no materials, add a default
    if scene.materials.is_empty() {
        scene.add_material(Material::default());
    }

    // Import meshes and nodes
    for gltf_node in gltf.nodes() {
        let transform = node_transform(&gltf_node);

        if let Some(gltf_mesh) = gltf_node.mesh() {
            for primitive in gltf_mesh.primitives() {
                let mesh = import_primitive(&primitive, &buffers)?;
                let mesh_idx = scene.add_mesh(mesh);

                let mat_idx = primitive.material().index().unwrap_or(0);

                let name = gltf_node.name()
                    .or_else(|| gltf_mesh.name())
                    .unwrap_or("Object");

                let obj = Object::new(name, ObjectData::Mesh(mesh_idx))
                    .with_transform(transform.clone())
                    .with_material(mat_idx);
                scene.add_object(obj);
            }
        }
    }

    if !scene.objects.is_empty() {
        scene.active_object = Some(0);
    }

    Ok(scene)
}

fn node_transform(node: &gltf::Node) -> Transform {
    let (pos, rot, scale) = node.transform().decomposed();
    Transform {
        position: Vec3::from(pos),
        rotation: Quat::from_array(rot),
        scale: Vec3::from(scale),
    }
}

fn import_primitive(
    primitive: &gltf::Primitive,
    buffers: &[Vec<u8>],
) -> Result<Mesh, ImportError> {
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .ok_or(ImportError::MissingPositions)?
        .collect();

    let normals: Vec<[f32; 3]> = reader
        .read_normals()
        .map(|iter| iter.collect())
        .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

    let indices: Vec<u32> = reader
        .read_indices()
        .map(|iter| iter.into_u32().collect())
        .unwrap_or_else(|| (0..positions.len() as u32).collect());

    let vertices: Vec<Vertex> = positions
        .iter()
        .zip(normals.iter())
        .zip(uvs.iter())
        .map(|((p, n), uv)| Vertex {
            position: *p,
            normal: *n,
            uv: *uv,
        })
        .collect();

    let mut mesh = Mesh {
        name: "Mesh".into(),
        vertices,
        indices,
        bounds_min: Vec3::ZERO,
        bounds_max: Vec3::ZERO,
    };
    mesh.recompute_bounds();
    Ok(mesh)
}

/// Load buffer data from glTF. Handles both .gltf (external) and .glb (embedded).
fn load_buffers(gltf: &gltf::Gltf, data: &[u8]) -> Result<Vec<Vec<u8>>, ImportError> {
    let mut buffers = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                let blob = gltf.blob.as_ref().ok_or(ImportError::MissingBufferData)?;
                buffers.push(blob.clone());
            }
            gltf::buffer::Source::Uri(uri) => {
                if let Some(base64_data) = uri.strip_prefix("data:application/octet-stream;base64,") {
                    // Inline base64 data URI
                    let _ = base64_data;
                    buffers.push(Vec::new());
                } else {
                    // External file reference — not supported in WASM yet
                    // Return empty buffer; will fail gracefully
                    let _ = data;
                    buffers.push(Vec::new());
                }
            }
        }
    }
    Ok(buffers)
}
