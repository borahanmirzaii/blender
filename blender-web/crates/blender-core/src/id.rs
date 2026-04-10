use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global ID counter. Each data block gets a unique ID.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for any data block in the scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataBlockId(pub u64);

impl DataBlockId {
    pub fn new() -> Self {
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DataBlockId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type tag for data blocks — mirrors Blender's ID_* types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdType {
    Object,
    Mesh,
    Material,
    Camera,
    Light,
    Scene,
}
