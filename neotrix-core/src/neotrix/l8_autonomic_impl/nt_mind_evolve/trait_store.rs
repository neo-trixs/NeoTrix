use std::time::Instant;

#[derive(Debug, Clone)]
pub struct TraitStore {
    pub traits: Vec<(String, f64, Instant)>,
}

impl Default for TraitStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TraitStore {
    pub fn new() -> Self {
        Self {
            traits: Vec::new(),
        }
    }

    pub fn store(&mut self, trait_name: impl Into<String>, score: f64) {
        self.traits.push((trait_name.into(), score, Instant::now()));
    }

    pub fn get_top(&self, k: usize) -> Vec<(String, f64)> {
        let mut sorted: Vec<(String, f64)> = self.traits
            .iter()
            .map(|(name, score, _)| (name.clone(), *score))
            .collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(k).collect()
    }

    pub fn all(&self) -> &[(String, f64, Instant)] {
        &self.traits
    }

    pub fn len(&self) -> usize {
        self.traits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }

    pub fn clear(&mut self) {
        self.traits.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trait_store_new() {
        let store = TraitStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_trait_store_store_and_get_top() {
        let mut store = TraitStore::new();
        store.store("alpha", 0.5);
        store.store("beta", 0.9);
        store.store("gamma", 0.3);

        assert_eq!(store.len(), 3);

        let top2 = store.get_top(2);
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].0, "beta");
        assert!((top2[0].1 - 0.9).abs() < 1e-6);
        assert_eq!(top2[1].0, "alpha");
    }

    #[test]
    fn test_trait_store_get_top_returns_all_if_k_too_large() {
        let mut store = TraitStore::new();
        store.store("a", 0.5);
        store.store("b", 0.6);
        let top = store.get_top(10);
        assert_eq!(top.len(), 2);
    }

    #[test]
    fn test_trait_store_get_top_empty() {
        let store = TraitStore::new();
        let top = store.get_top(5);
        assert!(top.is_empty());
    }

    #[test]
    fn test_trait_store_clear() {
        let mut store = TraitStore::new();
        store.store("x", 0.8);
        assert!(!store.is_empty());
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_trait_store_all() {
        let mut store = TraitStore::new();
        store.store("a", 0.5);
        store.store("b", 0.6);
        assert_eq!(store.all().len(), 2);
    }

    #[test]
    fn test_trait_store_default() {
        let store = TraitStore::default();
        assert!(store.is_empty());
    }
}
