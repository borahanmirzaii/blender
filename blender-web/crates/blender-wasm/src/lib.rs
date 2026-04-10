use blender_core::camera::OrbitCamera;
use blender_core::material::Material;
use blender_core::mesh::Mesh;
use blender_core::object::{Object, ObjectData};
use blender_core::scene::Scene;
use glam::Vec3;
use wasm_bindgen::prelude::*;

/// Log to browser console.
macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into())
    };
}

// ── Scene Handle ────────────────────────────────────────────────────

/// Opaque scene handle exposed to JavaScript.
#[wasm_bindgen]
pub struct SceneHandle {
    scene: Scene,
    camera: OrbitCamera,
}

#[wasm_bindgen]
impl SceneHandle {
    /// Create Blender's default startup scene (cube + light + camera).
    #[wasm_bindgen(js_name = defaultScene)]
    pub fn default_scene() -> Self {
        console_log!("[blender-wasm] Creating default scene");
        Self {
            scene: Scene::default_scene(),
            camera: OrbitCamera::default(),
        }
    }

    /// Import a glTF/GLB file from raw bytes.
    #[wasm_bindgen(js_name = importGltf)]
    pub fn import_gltf(data: &[u8]) -> Result<SceneHandle, JsError> {
        let scene = blender_core::import::import_gltf(data)
            .map_err(|e| JsError::new(&e.to_string()))?;
        console_log!("[blender-wasm] Imported {} objects, {} meshes",
            scene.objects.len(), scene.meshes.len());
        Ok(Self { scene, camera: OrbitCamera::default() })
    }

    // ── Scene queries ──

    #[wasm_bindgen(js_name = objectCount)]
    pub fn object_count(&self) -> usize {
        self.scene.objects.len()
    }

    #[wasm_bindgen(js_name = meshCount)]
    pub fn mesh_count(&self) -> usize {
        self.scene.meshes.len()
    }

    #[wasm_bindgen(js_name = objectName)]
    pub fn object_name(&self, idx: usize) -> Result<String, JsError> {
        self.scene.objects.get(idx)
            .map(|o| o.name.clone())
            .ok_or_else(|| JsError::new("Object index out of bounds"))
    }

    #[wasm_bindgen(js_name = objectType)]
    pub fn object_type(&self, idx: usize) -> Result<String, JsError> {
        self.scene.objects.get(idx)
            .map(|o| match &o.data {
                ObjectData::Mesh(_) => "mesh".into(),
                ObjectData::Light(_) => "light".into(),
                ObjectData::Camera(_) => "camera".into(),
                ObjectData::Empty => "empty".into(),
            })
            .ok_or_else(|| JsError::new("Object index out of bounds"))
    }

    #[wasm_bindgen(js_name = objectVisible)]
    pub fn object_visible(&self, idx: usize) -> bool {
        self.scene.objects.get(idx).is_some_and(|o| o.visible)
    }

    /// Get the mesh index for a mesh object, or -1 if not a mesh.
    #[wasm_bindgen(js_name = objectMeshIndex)]
    pub fn object_mesh_index(&self, idx: usize) -> i32 {
        self.scene.objects.get(idx)
            .and_then(|o| o.mesh_index())
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    /// Get object's 4x4 transform matrix as flat f32 array (column-major).
    #[wasm_bindgen(js_name = objectMatrix)]
    pub fn object_matrix(&self, idx: usize) -> Result<Vec<f32>, JsError> {
        self.scene.objects.get(idx)
            .map(|o| o.transform.to_cols_array().to_vec())
            .ok_or_else(|| JsError::new("Object index out of bounds"))
    }

    // ── Mesh data for GPU upload ──

    /// Get vertex buffer as raw bytes (32 bytes per vertex: pos + normal + uv).
    #[wasm_bindgen(js_name = meshVertexData)]
    pub fn mesh_vertex_data(&self, mesh_idx: usize) -> Result<Vec<u8>, JsError> {
        self.scene.meshes.get(mesh_idx)
            .map(|m| m.vertex_bytes().to_vec())
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))
    }

    /// Get index buffer as raw bytes (4 bytes per index, u32).
    #[wasm_bindgen(js_name = meshIndexData)]
    pub fn mesh_index_data(&self, mesh_idx: usize) -> Result<Vec<u8>, JsError> {
        self.scene.meshes.get(mesh_idx)
            .map(|m| m.index_bytes().to_vec())
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))
    }

    #[wasm_bindgen(js_name = meshIndexCount)]
    pub fn mesh_index_count(&self, mesh_idx: usize) -> Result<u32, JsError> {
        self.scene.meshes.get(mesh_idx)
            .map(|m| m.indices.len() as u32)
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))
    }

    #[wasm_bindgen(js_name = meshVertexCount)]
    pub fn mesh_vertex_count(&self, mesh_idx: usize) -> Result<u32, JsError> {
        self.scene.meshes.get(mesh_idx)
            .map(|m| m.vertices.len() as u32)
            .ok_or_else(|| JsError::new("Mesh index out of bounds"))
    }

    // ── Material data ──

    /// Get material base color as [r, g, b, metallic, roughness].
    #[wasm_bindgen(js_name = materialData)]
    pub fn material_data(&self, idx: usize) -> Result<Vec<f32>, JsError> {
        self.scene.materials.get(idx)
            .map(|m| vec![
                m.base_color.x, m.base_color.y, m.base_color.z,
                m.metallic, m.roughness,
            ])
            .ok_or_else(|| JsError::new("Material index out of bounds"))
    }

    /// Get the material index for an object, or -1.
    #[wasm_bindgen(js_name = objectMaterialIndex)]
    pub fn object_material_index(&self, obj_idx: usize) -> i32 {
        self.scene.objects.get(obj_idx)
            .and_then(|o| o.material_index)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    // ── Mutations ──

    /// Add a primitive and return its object index.
    #[wasm_bindgen(js_name = addPrimitive)]
    pub fn add_primitive(&mut self, kind: &str, name: &str) -> Result<usize, JsError> {
        let mesh = match kind {
            "cube" => Mesh::cube(),
            "sphere" => Mesh::sphere(32, 16),
            "plane" => Mesh::plane(),
            _ => return Err(JsError::new(&format!("Unknown primitive: {kind}"))),
        };
        let mat_idx = if self.scene.materials.is_empty() {
            self.scene.add_material(Material::default())
        } else {
            0
        };
        let mesh_idx = self.scene.add_mesh(mesh);
        let obj = Object::new(name, ObjectData::Mesh(mesh_idx)).with_material(mat_idx);
        let idx = self.scene.add_object(obj);
        Ok(idx)
    }

    /// Set object position.
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, idx: usize, x: f32, y: f32, z: f32) -> Result<(), JsError> {
        self.scene.objects.get_mut(idx)
            .ok_or_else(|| JsError::new("Object index out of bounds"))?
            .transform.position = Vec3::new(x, y, z);
        Ok(())
    }

    /// Delete an object.
    #[wasm_bindgen(js_name = deleteObject)]
    pub fn delete_object(&mut self, idx: usize) -> Result<(), JsError> {
        if idx >= self.scene.objects.len() {
            return Err(JsError::new("Object index out of bounds"));
        }
        self.scene.objects.remove(idx);
        Ok(())
    }

    /// Export scene as JSON string.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsError> {
        self.scene.to_json().map_err(|e| JsError::new(&e.to_string()))
    }

    // ── Camera ──

    #[wasm_bindgen(js_name = cameraOrbit)]
    pub fn camera_orbit(&mut self, dx: f32, dy: f32) {
        self.camera.orbit(dx, dy);
    }

    #[wasm_bindgen(js_name = cameraPan)]
    pub fn camera_pan(&mut self, dx: f32, dy: f32) {
        self.camera.pan(dx, dy);
    }

    #[wasm_bindgen(js_name = cameraZoom)]
    pub fn camera_zoom(&mut self, delta: f32) {
        self.camera.zoom(delta);
    }

    /// Get view matrix as flat f32 array (column-major).
    #[wasm_bindgen(js_name = viewMatrix)]
    pub fn view_matrix(&self) -> Vec<f32> {
        self.camera.view_matrix().to_cols_array().to_vec()
    }

    /// Get projection matrix as flat f32 array (column-major).
    #[wasm_bindgen(js_name = projectionMatrix)]
    pub fn projection_matrix(&self, aspect: f32) -> Vec<f32> {
        self.camera.projection_matrix(aspect).to_cols_array().to_vec()
    }

    /// Get camera position as [x, y, z].
    #[wasm_bindgen(js_name = cameraPosition)]
    pub fn camera_position(&self) -> Vec<f32> {
        let p = self.camera.position();
        vec![p.x, p.y, p.z]
    }
}
