use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Storage directory: ~/Library/Application Support/neotrix/rkyv/
static RKYV_STORAGE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("neotrix")
        .join("rkyv")
});

/// A key-value store using rkyv zero-copy serialization.
///
/// Each value is stored as a separate file keyed by hash of `K`.
/// Uses `rkyv::to_bytes` for serialization and `rkyv::from_bytes` for
/// checked deserialization — no `unsafe` involved.
#[derive(Debug)]
pub struct RkyvStorage<K, V> {
    dir: PathBuf,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> RkyvStorage<K, V> {
    /// Creates or opens a storage directory under `~/.local/share/neotrix/rkyv/<subdir>`.
    pub fn new(subdir: &str) -> Self {
        let dir = RKYV_STORAGE_DIR.join(subdir);
        let _ = std::fs::create_dir_all(&dir);
        Self {
            dir,
            _phantom: PhantomData,
        }
    }

    fn key_path(&self, key: &K) -> PathBuf
    where
        K: Hash,
    {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        self.dir.join(format!("{:016x}.rkyv", hash))
    }

    /// Serialize `value` and write to disk under key `K`.
    pub fn store(&self, key: &K, value: &V) -> Result<(), String>
    where
        K: Hash,
        V: for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map_err(|e| format!("rkyv serialize: {}", e))?;
        let path = self.key_path(key);
        std::fs::write(&path, &bytes)
            .map_err(|e| format!("write rkyv store: {}", e))?;
        Ok(())
    }

    /// Load, validate, and deserialize a value for key `K`.
    pub fn load_deserialized(&self, key: &K) -> Result<V, String>
    where
        K: Hash + std::fmt::Debug,
        V: rkyv::Archive,
        V::Archived: rkyv::Deserialize<V, rkyv::rancor::Strategy<rkyv::de::Pool, rkyv::rancor::Error>>
            + for<'a> rkyv::bytecheck::CheckBytes<
                rkyv::api::high::HighValidator<'a, rkyv::rancor::Error>,
            >,
    {
        let path = self.key_path(key);
        if !path.exists() {
            return Err(format!("rkyv key not found: {:?}", key));
        }
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("read rkyv store: {}", e))?;
        let value = rkyv::from_bytes::<V, rkyv::rancor::Error>(&bytes)
            .map_err(|e| format!("rkyv deserialize: {}", e))?;
        Ok(value)
    }

    /// Check whether a key exists in storage.
    pub fn exists(&self, key: &K) -> bool
    where
        K: Hash,
    {
        self.key_path(key).exists()
    }

    /// Remove a key from storage.
    pub fn delete(&self, key: &K) -> Result<(), String>
    where
        K: Hash,
    {
        let path = self.key_path(key);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("delete rkyv store: {}", e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rkyv::{Archive, Deserialize, Serialize};

    #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
    #[rkyv(compare(PartialEq))]
    struct TestValue {
        x: u32,
        label: String,
    }

    #[test]
    fn test_store_and_load() {
        let store = RkyvStorage::<String, TestValue>::new("test_store");
        let val = TestValue { x: 42, label: "hello".into() };
        store.store(&"key1".to_string(), &val).expect("store should succeed");
        let loaded = store.load_deserialized(&"key1".to_string()).expect("load should succeed");
        assert_eq!(loaded, val);
    }

    #[test]
    fn test_key_not_found() {
        let store = RkyvStorage::<String, TestValue>::new("test_miss");
        let result = store.load_deserialized(&"nope".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_exists() {
        let store = RkyvStorage::<String, TestValue>::new("test_exists");
        let val = TestValue { x: 1, label: "a".into() };
        store.store(&"exist_key".to_string(), &val).expect("store should succeed");
        assert!(store.exists(&"exist_key".to_string()));
        assert!(!store.exists(&"missing".to_string()));
    }

    #[test]
    fn test_delete() {
        let store = RkyvStorage::<String, TestValue>::new("test_delete");
        let val = TestValue { x: 99, label: "delete_me".into() };
        store.store(&"del_key".to_string(), &val).expect("store should succeed");
        assert!(store.exists(&"del_key".to_string()));
        store.delete(&"del_key".to_string()).expect("delete should succeed");
        assert!(!store.exists(&"del_key".to_string()));
    }
}
