use blender_core::material::Material;
use blender_core::mesh::Mesh;
use blender_core::object::ObjectData;
use blender_core::scene::Scene;
use blender_core::transform::Transform;
use glam::Vec3;
use wasm_bindgen::prelude::*;

/// Initialize the WASM module — call once on page load.
#[wasm_bindgen(start)]
pub fn init() {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("blender-wasm initialized");
}

/// Opaque handle to a Scene, exposed to JavaScript.
#[wasm_bindgen]
pub struct SceneHandle {
    scene: Scene,
}

#[wasm_bindgen]
impl SceneHandle {
    /// Create a new empty scene.
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        Self {
            scene: Scene::new(name),
        }
    }

    /// Create Blender's default startup scene (cube + light + camera).
    #[wasm_bindgen(js_name = defaultScene)]
    pub fn default_scene() -> Self {
        Self {
            scene: Scene::default_scene(),
        }
    }

    /// Serialize the entire scene to JSON for transport/storage.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsError> {
        self.scene.to_json().map_err(|e| JsError::new(&e.to_string()))
    }

    /// Load a scene from JSON.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<SceneHandle, JsError> {
        let scene = Scene::from_json(json).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { scene })
    }

    /// Get the number of objects in the scene.
    #[wasm_bindgen(js_name = objectCount)]
    pub fn object_count(&self) -> usize {
        self.scene.objects.len()
    }

    /// Get the number of meshes.
    #[wasm_bindgen(js_name = meshCount)]
    pub fn mesh_count(&self) -> usize {
        self.scene.meshes.len()
    }

    /// Add a primitive mesh (cube, sphere, etc.) and return its object index.
    #[wasm_bindgen(js_name = addPrimitive)]
    pub fn add_primitive(&mut self, primitive: &str, name: &str) -> Result<usize, JsError> {
        let mesh = match primitive {
            "cube" => Mesh::cube(),
            "sphere" => Mesh::uv_sphere(32, 16),
            "sphere_low" => Mesh::uv_sphere(16, 8),
            _ => return Err(JsError::new(&format!("Unknown primitive: {primitive}"))),
        };

        let mat_idx = self.scene.add_material(Material::default());
        let mesh_idx = self.scene.add_mesh(mesh);
        let obj_idx = self.scene.add_object(name, ObjectData::Mesh(mesh_idx));
        self.scene.objects[obj_idx].material_slots.push(mat_idx);
        Ok(obj_idx)
    }

    /// Set an object's position.
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, obj_idx: usize, x: f32, y: f32, z: f32) -> Result<(), JsError> {
        let obj = self
            .scene
            .objects
            .get_mut(obj_idx)
            .ok_or_else(|| JsError::new("Object index out of bounds"))?;
        obj.transform.position = Vec3::new(x, y, z);
        Ok(())
    }

    /// Set an object's scale.
    #[wasm_bindgen(js_name = setScale)]
    pub fn set_scale(&mut self, obj_idx: usize, x: f32, y: f32, z: f32) -> Result<(), JsError> {
        let obj = self
            .scene
            .objects
            .get_mut(obj_idx)
            .ok_or_else(|| JsError::new("Object index out of bounds"))?;
        obj.transform.scale = Vec3::new(x, y, z);
        Ok(())
    }

    /// Get vertex buffer data for a mesh (for GPU upload).
    #[wasm_bindgen(js_name = meshVertexData)]
    pub fn mesh_vertex_data(&self, mesh_idx: usize) -> Result<Vec<u8>, JsError> {
        let mesh = self
            .scene
            .meshes
            .get(mesh_idx)
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))?;
        Ok(mesh.vertex_bytes().to_vec())
    }

    /// Get index buffer data for a mesh.
    #[wasm_bindgen(js_name = meshIndexData)]
    pub fn mesh_index_data(&self, mesh_idx: usize) -> Result<Vec<u8>, JsError> {
        let mesh = self
            .scene
            .meshes
            .get(mesh_idx)
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))?;
        Ok(mesh.index_bytes().to_vec())
    }

    /// Get index count for a mesh (needed for draw calls).
    #[wasm_bindgen(js_name = meshIndexCount)]
    pub fn mesh_index_count(&self, mesh_idx: usize) -> Result<usize, JsError> {
        let mesh = self
            .scene
            .meshes
            .get(mesh_idx)
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))?;
        Ok(mesh.indices.len())
    }

    /// Get object transform as a 4x4 matrix (column-major, for GPU uniform).
    #[wasm_bindgen(js_name = objectMatrix)]
    pub fn object_matrix(&self, obj_idx: usize) -> Result<Vec<f32>, JsError> {
        let obj = self
            .scene
            .objects
            .get(obj_idx)
            .ok_or_else(|| JsError::new("Object index out of bounds"))?;
        let mat = obj.transform.to_matrix();
        Ok(mat.to_cols_array().to_vec())
    }

    /// Get an object's name.
    #[wasm_bindgen(js_name = objectName)]
    pub fn object_name(&self, obj_idx: usize) -> Result<String, JsError> {
        let obj = self
            .scene
            .objects
            .get(obj_idx)
            .ok_or_else(|| JsError::new("Object index out of bounds"))?;
        Ok(obj.name.clone())
    }

    /// Set a material's base color.
    #[wasm_bindgen(js_name = setMaterialColor)]
    pub fn set_material_color(
        &mut self,
        mat_idx: usize,
        r: f32,
        g: f32,
        b: f32,
    ) -> Result<(), JsError> {
        let mat = self
            .scene
            .materials
            .get_mut(mat_idx)
            .ok_or_else(|| JsError::new("Material index out of bounds"))?;
        mat.base_color = Vec3::new(r, g, b);
        Ok(())
    }

    /// Delete an object from the scene.
    #[wasm_bindgen(js_name = deleteObject)]
    pub fn delete_object(&mut self, obj_idx: usize) -> Result<(), JsError> {
        if obj_idx >= self.scene.objects.len() {
            return Err(JsError::new("Object index out of bounds"));
        }
        self.scene.objects.remove(obj_idx);
        // Fix parent references
        for obj in &mut self.scene.objects {
            if let Some(parent) = obj.parent {
                if parent == obj_idx {
                    obj.parent = None;
                } else if parent > obj_idx {
                    obj.parent = Some(parent - 1);
                }
            }
        }
        Ok(())
    }
}
