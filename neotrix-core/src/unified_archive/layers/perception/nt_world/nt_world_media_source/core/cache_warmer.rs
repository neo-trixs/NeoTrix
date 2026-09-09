use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;
use crate::unified::layers::perception::nt_world::nt_world_media_source::multi_cache::MultiLevelCache;
use std::sync::Arc;

pub struct CacheWarmer;

impl CacheWarmer {
    pub async fn warm(
        queries: &[String],
        sources: &[Arc<dyn MediaSource>],
        cache: &mut MultiLevelCache,
    ) -> WarmResult {
        let mut success = 0usize;
        let mut failed = 0usize;

        for query in queries {
            if cache.contains(query) {
                success += 1;
                continue;
            }

            let mut got_result = false;
            for source in sources {
                match source.search(query, 1).await {
                    Ok(result) => {
                        cache.set(query.clone(), result);
                        got_result = true;
                        success += 1;
                        break;
                    }
                    Err(_) => continue,
                }
            }
            if !got_result {
                failed += 1;
            }
        }

        WarmResult { success, failed }
    }

    pub async fn warm_concurrent(
        queries: Vec<String>,
        sources: Arc<Vec<Arc<dyn MediaSource>>>,
        cache: Arc<tokio::sync::RwLock<MultiLevelCache>>,
    ) -> WarmResult {
        let mut handles = Vec::with_capacity(queries.len());

        for query in queries {
            let sources = sources.clone();
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                {
                    let c = cache.read().await;
                    if c.contains(&query) {
                        return true;
                    }
                }

                for source in sources.iter() {
                    if let Ok(result) = source.search(&query, 1).await {
                        let mut c = cache.write().await;
                        c.set(query, result);
                        return true;
                    }
                }
                false
            }));
        }

        let mut success = 0usize;
        let mut failed = 0usize;
        for handle in handles {
            match handle.await {
                Ok(true) => success += 1,
                Ok(false) => failed += 1,
                Err(_) => failed += 1,
            }
        }

        WarmResult { success, failed }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WarmResult {
    pub success: usize,
    pub failed: usize,
}

impl WarmResult {
    pub fn total(&self) -> usize {
        self.success + self.failed
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            self.success as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warm_result_stats() {
        let r = WarmResult { success: 8, failed: 2 };
        assert_eq!(r.total(), 10);
        assert!((r.hit_rate() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_warm_result_empty() {
        let r = WarmResult::default();
        assert_eq!(r.total(), 0);
        assert_eq!(r.hit_rate(), 0.0);
    }
}
