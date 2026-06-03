use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssetKind {
    Texture,
    Mesh,
    Audio,
}

impl fmt::Display for AssetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetKind::Texture => f.write_str("texture"),
            AssetKind::Mesh => f.write_str("mesh"),
            AssetKind::Audio => f.write_str("audio"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssetPath {
    pub kind: AssetKind,
    pub path: PathBuf,
}

impl AssetPath {
    pub fn new(kind: AssetKind, path: impl Into<PathBuf>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }

    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}

#[derive(Debug, Clone)]
pub struct AssetMetadata {
    pub size_bytes: u64,
    pub extension: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedAsset<T> {
    pub kind: AssetKind,
    pub handle: AssetHandle,
    pub data: T,
    pub metadata: AssetMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetHandle {
    pub id: u32,
    pub generation: u32,
}

impl AssetHandle {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}
