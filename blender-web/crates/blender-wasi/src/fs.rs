use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Path not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Not a directory: {0}")]
    NotADirectory(String),
}

/// Entry in a directory listing.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// Filesystem abstraction that works through WASI.
/// In the browser, this is backed by the File System Access API
/// polyfilled through WASI fd_* calls.
/// On the server/edge, this uses the real filesystem via wasmtime.
pub struct WasiFs {
    root: PathBuf,
}

impl WasiFs {
    /// Create a new filesystem handle rooted at the given directory.
    /// WASI preopens must include this directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Read a file's contents.
    pub fn read_file(&self, relative_path: &str) -> Result<Vec<u8>, FsError> {
        let path = self.root.join(relative_path);
        std::fs::read(&path).map_err(FsError::Io)
    }

    /// Read a file as string.
    pub fn read_string(&self, relative_path: &str) -> Result<String, FsError> {
        let path = self.root.join(relative_path);
        std::fs::read_to_string(&path).map_err(FsError::Io)
    }

    /// Write data to a file, creating parent directories as needed.
    pub fn write_file(&self, relative_path: &str, data: &[u8]) -> Result<(), FsError> {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(FsError::Io)?;
        }
        std::fs::write(&path, data).map_err(FsError::Io)
    }

    /// List directory contents.
    pub fn list_dir(&self, relative_path: &str) -> Result<Vec<DirEntry>, FsError> {
        let path = self.root.join(relative_path);
        let mut entries = Vec::new();

        for entry in std::fs::read_dir(&path).map_err(FsError::Io)? {
            let entry = entry.map_err(FsError::Io)?;
            let metadata = entry.metadata().map_err(FsError::Io)?;
            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().into(),
                path: entry.path(),
                is_dir: metadata.is_dir(),
                size: metadata.len(),
            });
        }

        Ok(entries)
    }

    /// Check if a path exists.
    pub fn exists(&self, relative_path: &str) -> bool {
        self.root.join(relative_path).exists()
    }

    /// Create a directory (and parents).
    pub fn create_dir(&self, relative_path: &str) -> Result<(), FsError> {
        let path = self.root.join(relative_path);
        std::fs::create_dir_all(&path).map_err(FsError::Io)
    }

    /// Delete a file.
    pub fn delete_file(&self, relative_path: &str) -> Result<(), FsError> {
        let path = self.root.join(relative_path);
        std::fs::remove_file(&path).map_err(FsError::Io)
    }
}
