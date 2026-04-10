use crate::camera::Camera;
use crate::id::DataBlockId;
use crate::light::Light;
use crate::transform::Transform;
use serde::{Deserialize, Serialize};

/// What data an object holds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectData {
    Mesh(usize),    // index into Scene::meshes
    Light(Light),
    Camera(Camera),
    Empty,
}

/// A scene object with transform and data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: DataBlockId,
    pub name: String,
    pub transform: Transform,
    pub data: ObjectData,
    pub material_index: Option<usize>,
    pub parent: Option<usize>,
    pub visible: bool,
}

impl Object {
    pub fn new(name: impl Into<String>, data: ObjectData) -> Self {
        Self {
            id: DataBlockId::new(),
            name: name.into(),
            transform: Transform::default(),
            data,
            material_index: None,
            parent: None,
            visible: true,
        }
    }

    pub fn with_transform(mut self, t: Transform) -> Self {
        self.transform = t;
        self
    }

    pub fn with_material(mut self, idx: usize) -> Self {
        self.material_index = Some(idx);
        self
    }

    pub fn is_mesh(&self) -> bool {
        matches!(self.data, ObjectData::Mesh(_))
    }

    pub fn mesh_index(&self) -> Option<usize> {
        if let ObjectData::Mesh(i) = self.data { Some(i) } else { None }
    }
}
