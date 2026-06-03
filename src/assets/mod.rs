pub mod manager;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use manager::AssetError;
pub use manager::AssetManager;
pub use types::{AssetHandle, AssetKind, AssetMetadata, AssetPath, LoadedAsset};
