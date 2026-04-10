use crate::transform::Transform;
use serde::{Deserialize, Serialize};

pub type ObjectId = u64;

/// The type of data an object holds, mirroring Blender's OB_* types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectData {
    /// Index into Scene::meshes
    Mesh(usize),
    /// Point light with color and power
    Light(LightData),
    /// Camera with projection settings
    Camera(CameraData),
    /// Empty / null object (used as parent or locator)
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightData {
    pub color: [f32; 3],
    pub power: f32,
    pub light_type: LightType,
    pub radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Point,
    Sun,
    Spot { angle: f32, blend: f32 },
    Area { size_x: f32, size_y: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraData {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection: Projection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Projection {
    Perspective,
    Orthographic { scale: f32 },
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            fov: 50.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,
            projection: Projection::Perspective,
        }
    }
}

/// A scene object with transform, data, and hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: ObjectId,
    pub name: String,
    pub transform: Transform,
    pub data: ObjectData,
    /// Index of parent object in Scene::objects, None = root
    pub parent: Option<usize>,
    /// Material slot indices into Scene::materials
    pub material_slots: Vec<usize>,
    pub visible: bool,
    pub selected: bool,
}

impl Object {
    pub fn new(id: ObjectId, name: impl Into<String>, data: ObjectData) -> Self {
        Self {
            id,
            name: name.into(),
            transform: Transform::default(),
            data,
            parent: None,
            material_slots: Vec::new(),
            visible: true,
            selected: false,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_material(mut self, material_index: usize) -> Self {
        self.material_slots.push(material_index);
        self
    }
}
