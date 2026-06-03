use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use tokio::io::AsyncReadExt;

use crate::assets::types::{AssetHandle, AssetKind, AssetMetadata, AssetPath, LoadedAsset};

pub type CacheKey = String;

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("file read failed: {0}")]
    ReadError(String),
    #[error("asset not found: {0}")]
    NotFound(String),
    #[error("unsupported kind: {0}")]
    UnsupportedKind(String),
}

pub struct CacheEntry<T> {
    pub handle: AssetHandle,
    pub loaded_at: Instant,
    pub data: T,
}

pub struct AssetManager {
    root: PathBuf,
    textures: Mutex<HashMap<CacheKey, CacheEntry<Vec<u8>>>>,
    meshes: Mutex<HashMap<CacheKey, CacheEntry<Vec<u8>>>>,
    audio: Mutex<HashMap<CacheKey, CacheEntry<Vec<u8>>>>,
    access_order: Mutex<Vec<CacheKey>>,
    next_id: std::sync::atomic::AtomicU32,
    current_generation: std::sync::atomic::AtomicU32,
    max_cache_entries: std::sync::atomic::AtomicUsize,
}

impl AssetManager {
    pub fn init(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            textures: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            audio: Mutex::new(HashMap::new()),
            access_order: Mutex::new(Vec::new()),
            next_id: std::sync::atomic::AtomicU32::new(0),
            current_generation: std::sync::atomic::AtomicU32::new(1),
            max_cache_entries: std::sync::atomic::AtomicUsize::new(256),
        }
    }

    fn next_handle(&self) -> AssetHandle {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let generation = self
            .current_generation
            .load(std::sync::atomic::Ordering::Relaxed);
        AssetHandle::new(id, generation)
    }

    fn record_access(&self, key: &CacheKey) {
        if let Ok(mut order) = self.access_order.lock() {
            order.retain(|k| k != key);
            order.push(key.clone());
        }
    }

    fn pooled_cache_for_kind(&self, kind: AssetKind) -> &Mutex<HashMap<CacheKey, CacheEntry<Vec<u8>>>> {
        match kind {
            AssetKind::Texture => &self.textures,
            AssetKind::Mesh => &self.meshes,
            AssetKind::Audio => &self.audio,
        }
    }

    fn evict_lru_for_kind(&self, kind: AssetKind) {
        let max = self.max_cache_entries.load(std::sync::atomic::Ordering::Relaxed);
        let mut best: Option<CacheKey> = None;

        {
            if let Ok(cache) = self.pooled_cache_for_kind(kind).lock() {
                if cache.len() < max {
                    return;
                }
            }
        }

        if let Ok(order) = self.access_order.lock() {
            for key in order.iter() {
                if let Ok(cache) = self.pooled_cache_for_kind(kind).lock() {
                    if cache.contains_key(key) {
                        best = Some(key.clone());
                        break;
                    }
                }
            }
        }

        if let Some(key) = best {
            if let Ok(mut cache) = self.pooled_cache_for_kind(kind).lock() {
                if cache.remove(&key).is_some() {
                    self.current_generation
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
            if let Ok(mut order) = self.access_order.lock() {
                order.retain(|k| k != &key);
            }
        }
    }

    async fn read_asset_bytes(&self, path: &AssetPath) -> Result<Vec<u8>, AssetError> {
        let mut candidate = self.root.join(path.path.clone());
        if !candidate.exists() {
            candidate = PathBuf::from(&path.path);
        }
        if !candidate.exists() {
            return Err(AssetError::NotFound(path.file_name().unwrap_or("?").to_string()));
        }

        let mut file = match tokio::fs::File::open(candidate).await {
            Ok(f) => f,
            Err(err) => return Err(AssetError::ReadError(err.to_string())),
        };
        let mut buf = Vec::new();
        if let Err(err) = file.read_to_end(&mut buf).await {
            return Err(AssetError::ReadError(err.to_string()));
        }
        Ok(buf)
    }

    fn cache_key_for_path(&self, kind: AssetKind, path: &AssetPath) -> CacheKey {
        let candidate = if path.path.is_absolute() {
            PathBuf::from(&path.path)
        } else {
            self.root.join(&path.path)
        };
        let canonical = candidate
            .canonicalize()
            .unwrap_or(candidate);
        format!("{kind}:{canonical}", canonical = canonical.display())
    }

    async fn load_into_cache_typed(
        &self,
        kind: AssetKind,
        path: AssetPath,
    ) -> Result<LoadedAsset<Vec<u8>>, AssetError> {
        let key = self.cache_key_for_path(kind, &path);

        {
            let Ok(cache) = self.pooled_cache_for_kind(kind).lock() else { return Err(AssetError::ReadError("lock poisoned".into())); };
            if let Some(entry) = cache.get(&key) {
                self.record_access(&key);
                let metadata = AssetMetadata {
                    size_bytes: entry.data.len() as u64,
                    extension: path.extension().map(|v| v.to_string()),
                };
                return Ok(LoadedAsset {
                    kind,
                    handle: entry.handle,
                    data: entry.data.clone(),
                    metadata,
                });
            }
        }

        let bytes = self.read_asset_bytes(&path).await?;
        let handle = self.next_handle();
        let metadata = AssetMetadata {
            size_bytes: bytes.len() as u64,
            extension: path.extension().map(|v| v.to_string()),
        };

        self.evict_lru_for_kind(kind);
        if let Ok(mut cache) = self.pooled_cache_for_kind(kind).lock() {
            cache.insert(
                key.clone(),
                CacheEntry {
                    handle,
                    loaded_at: Instant::now(),
                    data: bytes.clone(),
                },
            );
        }
        self.record_access(&key);

        Ok(LoadedAsset {
            kind,
            handle,
            data: bytes,
            metadata,
        })
    }

    pub async fn load_texture(&self, path: impl Into<AssetPath>) -> Result<LoadedAsset<Vec<u8>>, AssetError> {
        self.load_into_cache_typed(AssetKind::Texture, path.into()).await
    }

    pub async fn load_mesh(&self, path: impl Into<AssetPath>) -> Result<LoadedAsset<Vec<u8>>, AssetError> {
        self.load_into_cache_typed(AssetKind::Mesh, path.into()).await
    }

    pub async fn load_audio(&self, path: impl Into<AssetPath>) -> Result<LoadedAsset<Vec<u8>>, AssetError> {
        self.load_into_cache_typed(AssetKind::Audio, path.into()).await
    }

    pub fn set_max_cache_entries(&self, max_cache_entries: usize) {
        self.max_cache_entries
            .store(max_cache_entries, std::sync::atomic::Ordering::Relaxed);
    }
}
