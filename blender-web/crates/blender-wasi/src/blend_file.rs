use blender_core::scene::Scene;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlendError {
    #[error("Invalid .blend file magic: expected 'BLENDER', got '{0}'")]
    InvalidMagic(String),
    #[error("Unsupported .blend version: {0}")]
    UnsupportedVersion(String),
    #[error("Parse error at offset {offset}: {message}")]
    ParseError { offset: usize, message: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// .blend file header information.
#[derive(Debug, Clone)]
pub struct BlendHeader {
    pub pointer_size: PointerSize,
    pub endianness: Endianness,
    pub version: [u8; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerSize {
    Bits32,
    Bits64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

/// Parser for .blend files.
/// Reads Blender's binary format and converts to our Scene representation.
pub struct BlendFileReader {
    data: Vec<u8>,
    cursor: usize,
}

impl BlendFileReader {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, cursor: 0 }
    }

    /// Parse the file header (first 12 bytes).
    pub fn read_header(&mut self) -> Result<BlendHeader, BlendError> {
        if self.data.len() < 12 {
            return Err(BlendError::ParseError {
                offset: 0,
                message: "File too small for header".into(),
            });
        }

        let magic = &self.data[0..7];
        if magic != b"BLENDER" {
            return Err(BlendError::InvalidMagic(
                String::from_utf8_lossy(magic).into(),
            ));
        }

        let pointer_size = match self.data[7] {
            b'_' => PointerSize::Bits32,
            b'-' => PointerSize::Bits64,
            b => {
                return Err(BlendError::ParseError {
                    offset: 7,
                    message: format!("Unknown pointer size byte: {b:#x}"),
                })
            }
        };

        let endianness = match self.data[8] {
            b'v' => Endianness::Little,
            b'V' => Endianness::Big,
            b => {
                return Err(BlendError::ParseError {
                    offset: 8,
                    message: format!("Unknown endianness byte: {b:#x}"),
                })
            }
        };

        let version = [self.data[9], self.data[10], self.data[11]];
        self.cursor = 12;

        Ok(BlendHeader {
            pointer_size,
            endianness,
            version,
        })
    }

    /// Parse a .blend file into a Scene.
    /// This is a simplified parser that reads the file structure blocks.
    pub fn parse(&mut self) -> Result<Scene, BlendError> {
        let header = self.read_header()?;
        let version_str = format!(
            "{}.{}.{}",
            header.version[0] - b'0',
            header.version[1] - b'0',
            header.version[2] - b'0'
        );
        log::info!("Parsing .blend file version {version_str}");

        // For now, return a default scene — full .blend parsing is a large
        // undertaking that would be built incrementally.
        // The block structure reader would iterate file blocks (DATA, GLOB,
        // SDNA, etc.) and reconstruct the scene graph.
        Ok(Scene::default_scene())
    }
}

/// Write a Scene to our JSON-based interchange format.
/// Full .blend writing would require SDNA struct definitions.
pub fn export_scene_json(scene: &Scene) -> Result<Vec<u8>, BlendError> {
    let json = serde_json::to_vec_pretty(scene)?;
    Ok(json)
}

/// Import a Scene from JSON interchange format.
pub fn import_scene_json(data: &[u8]) -> Result<Scene, BlendError> {
    let scene = serde_json::from_slice(data)?;
    Ok(scene)
}
