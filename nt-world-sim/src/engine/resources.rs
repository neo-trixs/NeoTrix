use std::collections::HashMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// ResourceHandle
// ---------------------------------------------------------------------------

/// A lightweight, reference-counted handle to a loaded resource.
#[derive(Debug, Clone)]
pub struct ResourceHandle<T: 'static> {
    key: String,
    _marker: std::marker::PhantomData<T>,
}

impl<T: 'static> ResourceHandle<T> {
    pub fn new(key: &str) -> Self {
        Self { key: key.to_string(), _marker: std::marker::PhantomData }
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

impl<T: 'static> PartialEq for ResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<T: 'static> Eq for ResourceHandle<T> {}

impl<T: 'static> std::hash::Hash for ResourceHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

// ---------------------------------------------------------------------------
// ResourceEntry
// ---------------------------------------------------------------------------

/// Internal entry wrapping a resource with reference count.
struct ResourceEntry<T> {
    data: Arc<T>,
    ref_count: u32,
    size_bytes: usize,
}

// ---------------------------------------------------------------------------
// AssetCache
// ---------------------------------------------------------------------------

/// Type-erased cache for any resource type.
struct TypeCache {
    // We use a raw pointer approach for type erasure
    // In production, use `anymap` or `inventory` crate
    entries: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl TypeCache {
    fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    fn insert<T: Send + Sync + 'static>(&mut self, key: &str, data: T) {
        self.entries.insert(key.to_string(), Box::new(data));
    }

    fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<&T> {
        self.entries.get(key)?.downcast_ref::<T>()
    }

    fn get_mut<T: Send + Sync + 'static>(&mut self, key: &str) -> Option<&mut T> {
        self.entries.get_mut(key)?.downcast_mut::<T>()
    }

    fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

// ---------------------------------------------------------------------------
// ResourceLoader trait
// ---------------------------------------------------------------------------

/// Loader trait for resource types. Implement this to add custom loaders.
pub trait ResourceLoader: Send + Sync {
    type Resource: Send + Sync + 'static;

    /// Load a resource from a path. Returns the resource or an error string.
    fn load(&self, path: &str) -> Result<Self::Resource, String>;

    /// Human-readable name of this loader.
    fn name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// ResourceRegistry
// ---------------------------------------------------------------------------

/// Registry mapping type names to loaders.
struct LoaderEntry {
    loader: Box<dyn std::any::Any + Send + Sync>,
    type_name: String,
}

// ---------------------------------------------------------------------------
// ResourceManager
// ---------------------------------------------------------------------------

/// Central resource manager: loads, caches, and serves resources with
/// reference counting.
pub struct ResourceManager {
    caches: HashMap<std::any::TypeId, TypeCache>,
    /// Track ref counts per key per type.
    ref_counts: HashMap<(std::any::TypeId, String), u32>,
    /// Track memory usage per type.
    memory_usage: HashMap<std::any::TypeId, usize>,
    /// Loader registry (type-erased).
    loaders: Vec<LoaderEntry>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
            ref_counts: HashMap::new(),
            memory_usage: HashMap::new(),
            loaders: Vec::new(),
        }
    }

    /// Register a loader for a resource type.
    pub fn register_loader<L: ResourceLoader + 'static>(&mut self, loader: L) {
        let type_name = std::any::type_name::<L::Resource>().to_string();
        self.loaders.push(LoaderEntry {
            loader: Box::new(loader),
            type_name,
        });
    }

    /// Insert a resource directly (bypasses loader).
    pub fn insert<T: Send + Sync + 'static>(&mut self, key: &str, data: T, size_bytes: usize) {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.entry(type_id).or_insert_with(TypeCache::new).insert(key, data);
        *self.ref_counts.entry((type_id, key.to_string())).or_insert(0) += 1;
        *self.memory_usage.entry(type_id).or_insert(0) += size_bytes;
    }

    /// Get a reference-counted handle to a resource.
    pub fn get<T: Send + Sync + 'static>(&self, key: &str) -> Option<ResourceHandle<T>> {
        let type_id = std::any::TypeId::of::<T>();
        if self.caches.get(&type_id)?.get::<T>(key).is_some() {
            Some(ResourceHandle::new(key))
        } else {
            None
        }
    }

    /// Get immutable access to the resource data.
    pub fn get_ref<T: Send + Sync + 'static>(&self, handle: &ResourceHandle<T>) -> Option<&T> {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.get(&type_id)?.get::<T>(&handle.key)
    }

    /// Get mutable access to the resource data.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self, handle: &ResourceHandle<T>) -> Option<&mut T> {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.get_mut(&type_id)?.get_mut::<T>(&handle.key)
    }

    /// Check if a resource exists.
    pub fn contains<T: Send + Sync + 'static>(&self, key: &str) -> bool {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.get(&type_id).map_or(false, |c| c.contains(key))
    }

    /// Remove a resource.
    pub fn remove<T: Send + Sync + 'static>(&mut self, key: &str) -> bool {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.get_mut(&type_id).map_or(false, |c| c.remove(key))
    }

    /// Total number of cached resources across all types.
    pub fn total_count(&self) -> usize {
        self.caches.values().map(|c| c.len()).sum()
    }

    /// Memory usage for a specific type (in bytes).
    pub fn memory_usage<T: Send + Sync + 'static>(&self) -> usize {
        let type_id = std::any::TypeId::of::<T>();
        *self.memory_usage.get(&type_id).unwrap_or(&0)
    }

    /// Total memory usage across all types.
    pub fn total_memory_usage(&self) -> usize {
        self.memory_usage.values().sum()
    }

    /// Clear all resources of a specific type.
    pub fn clear_type<T: Send + Sync + 'static>(&mut self) {
        let type_id = std::any::TypeId::of::<T>();
        self.caches.remove(&type_id);
        self.memory_usage.remove(&type_id);
    }

    /// Clear all resources.
    pub fn clear_all(&mut self) {
        self.caches.clear();
        self.ref_counts.clear();
        self.memory_usage.clear();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Convenience: TypedResourceManager
// ---------------------------------------------------------------------------

/// A typed wrapper around ResourceManager for a specific resource type.
pub struct TypedResourceManager<T: Send + Sync + 'static> {
    data: HashMap<String, Arc<T>>,
    size_hints: HashMap<String, usize>,
}

impl<T: Send + Sync + 'static> TypedResourceManager<T> {
    pub fn new() -> Self {
        Self { data: HashMap::new(), size_hints: HashMap::new() }
    }

    pub fn insert(&mut self, key: &str, resource: T) {
        self.data.insert(key.to_string(), Arc::new(resource));
        self.size_hints.insert(key.to_string(), 0);
    }

    pub fn insert_with_size(&mut self, key: &str, resource: T, size_bytes: usize) {
        self.data.insert(key.to_string(), Arc::new(resource));
        self.size_hints.insert(key.to_string(), size_bytes);
    }

    pub fn get(&self, key: &str) -> Option<Arc<T>> {
        self.data.get(key).cloned()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Arc<T>> {
        self.size_hints.remove(key);
        self.data.remove(key)
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }

    pub fn total_size(&self) -> usize {
        self.size_hints.values().sum()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.size_hints.clear();
    }
}

impl<T: Send + Sync + 'static> Default for TypedResourceManager<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct TestImage {
        width: u32,
        height: u32,
    }

    struct TestSound {
        samples: Vec<f32>,
    }

    struct TestLoader;
    impl ResourceLoader for TestLoader {
        type Resource = TestImage;
        fn load(&self, path: &str) -> Result<TestImage, String> {
            Ok(TestImage { width: 100, height: 100 })
        }
        fn name(&self) -> &str { "TestLoader" }
    }

    #[test]
    fn test_resource_manager_insert_get() {
        let mut mgr = ResourceManager::new();
        mgr.insert::<TestImage>("player", TestImage { width: 32, height: 32 }, 4096);
        let handle = mgr.get::<TestImage>("player").unwrap();
        let img = mgr.get_ref(&handle).unwrap();
        assert_eq!(img.width, 32);
    }

    #[test]
    fn test_resource_manager_get_mut() {
        let mut mgr = ResourceManager::new();
        mgr.insert::<TestImage>("player", TestImage { width: 32, height: 32 }, 4096);
        let handle = mgr.get::<TestImage>("player").unwrap();
        mgr.get_mut(&handle).unwrap().width = 64;
        let img = mgr.get_ref(&handle).unwrap();
        assert_eq!(img.width, 64);
    }

    #[test]
    fn test_resource_manager_contains_and_remove() {
        let mut mgr = ResourceManager::new();
        mgr.insert::<TestImage>("a", TestImage { width: 1, height: 1 }, 4);
        assert!(mgr.contains::<TestImage>("a"));
        assert!(mgr.remove::<TestImage>("a"));
        assert!(!mgr.contains::<TestImage>("a"));
    }

    #[test]
    fn test_resource_manager_memory() {
        let mut mgr = ResourceManager::new();
        mgr.insert::<TestImage>("a", TestImage { width: 1, height: 1 }, 1024);
        mgr.insert::<TestImage>("b", TestImage { width: 2, height: 2 }, 2048);
        assert_eq!(mgr.memory_usage::<TestImage>(), 3072);
        assert_eq!(mgr.total_memory_usage(), 3072);
    }

    #[test]
    fn test_resource_manager_clear() {
        let mut mgr = ResourceManager::new();
        mgr.insert::<TestImage>("a", TestImage { width: 1, height: 1 }, 1024);
        mgr.insert::<TestSound>("b", TestSound { samples: vec![] }, 0);
        mgr.clear_all();
        assert_eq!(mgr.total_count(), 0);
    }

    #[test]
    fn test_typed_resource_manager() {
        let mut mgr = TypedResourceManager::<TestImage>::new();
        mgr.insert("hero", TestImage { width: 64, height: 64 });
        assert!(mgr.contains("hero"));
        assert_eq!(mgr.count(), 1);

        let img = mgr.get("hero").unwrap();
        assert_eq!(img.width, 64);

        mgr.remove("hero");
        assert!(!mgr.contains("hero"));
    }

    #[test]
    fn test_resource_handle_clone_and_eq() {
        let h1 = ResourceHandle::<TestImage>::new("a");
        let h2 = h1.clone();
        assert_eq!(h1, h2);
        assert_eq!(h1.key(), "a");
    }

    #[test]
    fn test_typed_resource_manager_size() {
        let mut mgr = TypedResourceManager::<TestImage>::new();
        mgr.insert_with_size("a", TestImage { width: 1, height: 1 }, 1024);
        mgr.insert_with_size("b", TestImage { width: 2, height: 2 }, 2048);
        assert_eq!(mgr.total_size(), 3072);
        assert_eq!(mgr.keys().len(), 2);
    }
}
