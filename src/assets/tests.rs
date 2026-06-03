#[cfg(test)]
mod tests {
    use crate::assets::{AssetKind, AssetManager, AssetPath};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_LOCK: OnceLock<()> = OnceLock::new();

    fn unique_temp_root() -> PathBuf {
        let mut root = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        root.push(format!("rustyengine-asset-test-{ts}"));
        let _ = std::fs::create_dir_all(&root);
        root
    }

    #[tokio::test]
    async fn test_asset_manager_load_texture_from_disk() {
        let root = unique_temp_root();
        let sample_path = root.join("sample_player_ship_texture.png");
        fs::write(&sample_path, b"player_texture_bytes").expect("failed to write sample texture file");

        let manager = AssetManager::init(root);
        let loaded = manager
            .load_texture(AssetPath::new(AssetKind::Texture, sample_path.clone()))
            .await
            .expect("load_texture should succeed");

        assert_eq!(loaded.data, b"player_texture_bytes");
        assert_eq!(loaded.kind, AssetKind::Texture);
        assert_eq!(loaded.handle.generation, 1);

        let cached = manager
            .load_texture(AssetPath::new(AssetKind::Texture, sample_path))
            .await
            .expect("cached load should succeed");

        assert_eq!(cached.handle.id, loaded.handle.id);
        assert_eq!(cached.handle.generation, loaded.handle.generation);
        assert_eq!(cached.data, loaded.data);
    }

    #[tokio::test]
    async fn test_asset_manager_loading_fails_on_missing() {
        let manager = AssetManager::init(std::env::temp_dir());
        let missing = AssetPath::new(AssetKind::Texture, PathBuf::from("does_not_exist.png"));
        let err = manager.load_texture(missing).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_asset_manager_cache_hit_returns_cached_handle_without_reread() {
        let root = unique_temp_root();
        let path = root.join("cached.obj");
        fs::write(&path, b"mesh_data").expect("failed to write sample mesh file");

        let manager = AssetManager::init(root);
        let loaded = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path.clone()))
            .await
            .expect("load_mesh should succeed");

        let cached = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path))
            .await
            .expect("cached load should succeed");

        assert_eq!(cached.handle.id, loaded.handle.id);
        assert_eq!(cached.data, loaded.data);
    }

    #[tokio::test]
    async fn test_asset_manager_lru_eviction_removes_oldest_entry_when_limit_exceeded() {
        let _ = TEST_LOCK.get_or_init(|| {
            let _ = fs::create_dir_all("assets/state_capture");
            ();
        });

        let root = PathBuf::from("assets/state_capture");
        let _ = fs::create_dir_all(&root);

        let manager = AssetManager::init(root.clone());
        manager.set_max_cache_entries(2);

        let path_a = root.join("cache_test_a.obj");
        let path_b = root.join("cache_test_b.obj");
        let path_c = root.join("cache_test_c.obj");

        fs::write(&path_a, b"a").expect("failed to write cache_test_a.obj");
        fs::write(&path_b, b"b").expect("failed to write cache_test_b.obj");
        fs::write(&path_c, b"c").expect("failed to write cache_test_c.obj");

        let loaded_a = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path_a.clone()))
            .await
            .expect("load mesh a");
        let _ = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path_b.clone()))
            .await
            .expect("load mesh b");
        let _ = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path_c.clone()))
            .await
            .expect("load mesh c");

        let reloaded_a = manager
            .load_mesh(AssetPath::new(AssetKind::Mesh, path_a))
            .await
            .expect("reload should succeed even after eviction");

        assert_eq!(reloaded_a.data, loaded_a.data);
        assert_ne!(reloaded_a.handle.id, loaded_a.handle.id, "expected new handle after LRU eviction, not stale cache entry");
    }
}
