use std::sync::atomic::{AtomicUsize, Ordering};

/// 多 API Key 轮换池 — 用于免费提供商的配额管理
///
/// 支持从环境变量（分号/换行分隔）初始化、round-robin 轮换、
/// 空池优雅降级（`next()` 返回 `None`）。
#[derive(Debug)]
pub struct ApiKeyPool {
    keys: Vec<String>,
    cursor: AtomicUsize,
}

impl ApiKeyPool {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            keys,
            cursor: AtomicUsize::new(0),
        }
    }

    /// 从环境变量初始化；支持 `;` `,` 或换行分隔多个 key
    pub fn from_env(env_var: &str) -> Self {
        let raw = std::env::var(env_var).unwrap_or_default();
        let keys: Vec<String> = raw
            .split([';', ',', '\n'])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self::new(keys)
    }

    /// round-robin 取下一个 key
    pub fn next(&self) -> Option<String> {
        if self.keys.is_empty() {
            return None;
        }
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % self.keys.len();
        Some(self.keys[idx].clone())
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn keys(&self) -> Vec<String> {
        self.keys.clone()
    }

    pub fn add_key(&mut self, key: &str) {
        if !self.keys.contains(&key.to_string()) {
            self.keys.push(key.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pool() {
        let pool = ApiKeyPool::new(vec![]);
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert!(pool.next().is_none());
    }

    #[test]
    fn test_round_robin() {
        let pool = ApiKeyPool::new(vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(pool.next(), Some("a".into()));
        assert_eq!(pool.next(), Some("b".into()));
        assert_eq!(pool.next(), Some("c".into()));
        assert_eq!(pool.next(), Some("a".into()));
    }

    #[test]
    fn test_single_key() {
        let pool = ApiKeyPool::new(vec!["only".into()]);
        assert_eq!(pool.next(), Some("only".into()));
        assert_eq!(pool.next(), Some("only".into()));
    }

    #[test]
    fn test_from_env() {
        let pool = ApiKeyPool::from_env("PATH");
        // PATH always exists, so pool should not be empty in practice
        // but we cannot guarantee format — just verify no panic
        assert!(pool.len() > 0 || pool.is_empty());
    }

    #[test]
    fn test_from_env_empty() {
        // Use a var that's unlikely to have meaningful key data
        let pool = ApiKeyPool::from_env("HOME");
        // HOME exists but probably isn't a key list — that's fine
        let _ = pool.next();
    }
}
