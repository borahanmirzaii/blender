use crate::blend_file::{self, BlendError};
use crate::fs::WasiFs;
use blender_core::scene::Scene;

/// A Blender project directory on the local filesystem.
/// Manages scene files, textures, and assets within a directory.
pub struct Project {
    fs: WasiFs,
    pub name: String,
}

impl Project {
    /// Open or create a project at the given directory path.
    pub fn open(path: &str) -> Result<Self, BlendError> {
        let fs = WasiFs::new(path);
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().into())
            .unwrap_or_else(|| "Untitled".into());

        // Create standard subdirectories
        let _ = fs.create_dir("textures");
        let _ = fs.create_dir("renders");
        let _ = fs.create_dir("exports");

        Ok(Self { fs, name })
    }

    /// Save a scene to the project directory as JSON.
    pub fn save_scene(&self, scene: &Scene, filename: &str) -> Result<(), BlendError> {
        let data = blend_file::export_scene_json(scene)?;
        self.fs
            .write_file(filename, &data)
            .map_err(|e| BlendError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    /// Load a scene from the project directory.
    pub fn load_scene(&self, filename: &str) -> Result<Scene, BlendError> {
        let data = self
            .fs
            .read_file(filename)
            .map_err(|e| BlendError::Io(std::io::Error::other(e.to_string())))?;
        blend_file::import_scene_json(&data)
    }

    /// List all scene files in the project.
    pub fn list_scenes(&self) -> Vec<String> {
        if let Ok(entries) = self.fs.list_dir(".") {
            entries
                .into_iter()
                .filter(|e| !e.is_dir && e.name.ends_with(".json"))
                .map(|e| e.name)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Save a texture/asset to the project.
    pub fn save_texture(&self, name: &str, data: &[u8]) -> Result<(), BlendError> {
        let path = format!("textures/{name}");
        self.fs
            .write_file(&path, data)
            .map_err(|e| BlendError::Io(std::io::Error::other(e.to_string())))?;
        Ok(())
    }
}
