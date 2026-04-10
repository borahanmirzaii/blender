use serde::{Deserialize, Serialize};

/// Modifier stack, mirroring Blender's modifier system.
/// Each modifier transforms mesh data non-destructively.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modifier {
    Subdivision {
        levels_viewport: u32,
        levels_render: u32,
        use_creases: bool,
    },
    Mirror {
        axis_x: bool,
        axis_y: bool,
        axis_z: bool,
        merge_threshold: f32,
    },
    Array {
        count: u32,
        offset: [f32; 3],
        use_relative_offset: bool,
    },
    Solidify {
        thickness: f32,
        offset: f32,
    },
    Bevel {
        width: f32,
        segments: u32,
    },
    Boolean {
        operation: BooleanOp,
        target_object: String,
    },
    Decimate {
        ratio: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BooleanOp {
    Union,
    Intersect,
    Difference,
}

/// A stack of modifiers applied to an object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModifierStack {
    pub modifiers: Vec<ModifierEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierEntry {
    pub name: String,
    pub modifier: Modifier,
    pub enabled_viewport: bool,
    pub enabled_render: bool,
}

impl ModifierStack {
    pub fn add(&mut self, name: impl Into<String>, modifier: Modifier) {
        self.modifiers.push(ModifierEntry {
            name: name.into(),
            modifier,
            enabled_viewport: true,
            enabled_render: true,
        });
    }

    pub fn remove(&mut self, index: usize) -> Option<ModifierEntry> {
        if index < self.modifiers.len() {
            Some(self.modifiers.remove(index))
        } else {
            None
        }
    }

    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.modifiers.len() {
            self.modifiers.swap(index, index - 1);
        }
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.modifiers.len() {
            self.modifiers.swap(index, index + 1);
        }
    }
}
