use crate::material::Material;
use crate::mesh::Mesh;
use crate::object::{Object, ObjectData, ObjectId};
use serde::{Deserialize, Serialize};

/// The root container for all scene data.
/// Mirrors Blender's Scene struct but designed for serialization and WASM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub objects: Vec<Object>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    next_id: ObjectId,
    /// Active camera object index
    pub active_camera: Option<usize>,
    /// Currently active (last selected) object index
    pub active_object: Option<usize>,
    /// Current frame for animation
    pub frame_current: i32,
    pub frame_start: i32,
    pub frame_end: i32,
    pub fps: f32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: "Scene".into(),
            objects: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            next_id: 1,
            active_camera: None,
            active_object: None,
            frame_current: 1,
            frame_start: 1,
            frame_end: 250,
            fps: 24.0,
        }
    }
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    fn alloc_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a mesh to the scene and return its index.
    pub fn add_mesh(&mut self, mesh: Mesh) -> usize {
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    /// Add a material and return its index.
    pub fn add_material(&mut self, material: Material) -> usize {
        self.materials.push(material);
        self.materials.len() - 1
    }

    /// Add an object referencing existing mesh/material data.
    pub fn add_object(&mut self, name: impl Into<String>, data: ObjectData) -> usize {
        let id = self.alloc_id();
        let obj = Object::new(id, name, data);
        self.objects.push(obj);
        self.objects.len() - 1
    }

    /// Create a default scene with a cube, light, and camera (like Blender's startup).
    pub fn default_scene() -> Self {
        let mut scene = Scene::new("Scene");

        // Default material
        let mat_idx = scene.add_material(Material::default());

        // Cube
        let mesh_idx = scene.add_mesh(Mesh::cube());
        let cube_idx = scene.add_object("Cube", ObjectData::Mesh(mesh_idx));
        scene.objects[cube_idx].material_slots.push(mat_idx);

        // Light
        use crate::object::{LightData, LightType};
        let light_data = LightData {
            color: [1.0, 1.0, 1.0],
            power: 1000.0,
            light_type: LightType::Point,
            radius: 0.25,
        };
        let light_idx = scene.add_object("Light", ObjectData::Light(light_data));
        scene.objects[light_idx].transform.position = glam::Vec3::new(4.0, 4.0, 3.0);

        // Camera
        use crate::object::CameraData;
        let cam_idx = scene.add_object("Camera", ObjectData::Camera(CameraData::default()));
        scene.objects[cam_idx].transform.position = glam::Vec3::new(7.0, -6.0, 5.0);
        scene.active_camera = Some(cam_idx);

        scene
    }

    /// Serialize scene to JSON for transport or storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize scene from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Find object by name.
    pub fn find_object(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name == name)
    }

    /// Get all root objects (no parent).
    pub fn root_objects(&self) -> Vec<usize> {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.parent.is_none())
            .map(|(i, _)| i)
            .collect()
    }

    /// Get children of an object.
    pub fn children_of(&self, parent_idx: usize) -> Vec<usize> {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.parent == Some(parent_idx))
            .map(|(i, _)| i)
            .collect()
    }
}
