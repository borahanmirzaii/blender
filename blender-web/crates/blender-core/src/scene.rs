use crate::light::Light;
use crate::material::Material;
use crate::mesh::Mesh;
use crate::object::{Object, ObjectData};
use crate::transform::Transform;
use serde::{Deserialize, Serialize};

/// Root scene container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub objects: Vec<Object>,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub active_object: Option<usize>,
    pub active_camera: Option<usize>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            name: "Scene".into(),
            objects: Vec::new(),
            meshes: Vec::new(),
            materials: Vec::new(),
            active_object: None,
            active_camera: None,
        }
    }
}

impl Scene {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> usize {
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    pub fn add_material(&mut self, mat: Material) -> usize {
        self.materials.push(mat);
        self.materials.len() - 1
    }

    pub fn add_object(&mut self, obj: Object) -> usize {
        self.objects.push(obj);
        self.objects.len() - 1
    }

    /// Create Blender's default startup scene.
    pub fn default_scene() -> Self {
        let mut scene = Scene::new("Scene");

        let mat = scene.add_material(Material::default());
        let mesh = scene.add_mesh(Mesh::cube());
        let cube = scene.add_object(
            Object::new("Cube", ObjectData::Mesh(mesh)).with_material(mat),
        );

        let light = Light { power: 1000.0, ..Default::default() };
        scene.add_object(
            Object::new("Light", ObjectData::Light(light))
                .with_transform(Transform::from_position(4.0, 3.0, 4.0)),
        );

        scene.add_object(
            Object::new("Camera", ObjectData::Camera(Default::default()))
                .with_transform(Transform::from_position(7.0, 5.0, -6.0)),
        );

        scene.active_object = Some(cube);
        scene.active_camera = Some(2);
        scene
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn find_object(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scene_has_three_objects() {
        let scene = Scene::default_scene();
        assert_eq!(scene.objects.len(), 3);
        assert_eq!(scene.meshes.len(), 1);
        assert_eq!(scene.materials.len(), 1);
    }

    #[test]
    fn scene_json_roundtrip() {
        let scene = Scene::default_scene();
        let json = scene.to_json().unwrap();
        let loaded = Scene::from_json(&json).unwrap();
        assert_eq!(loaded.objects.len(), 3);
        assert_eq!(loaded.meshes[0].name, "Cube");
    }
}
