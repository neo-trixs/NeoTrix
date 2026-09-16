use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct OsintConfig {
    pub max_concurrent: usize,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub cache_ttl_secs: u64,
}

impl Default for OsintConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            timeout_ms: 5000,
            retry_count: 3,
            cache_ttl_secs: 3600,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsintQuery {
    pub target: String,
    pub query_type: QueryType,
    pub params: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    Whois,
    Dns,
    Subdomains,
    Emails,
    SocialMedia,
    Technology,
    Screenshot,
}

#[derive(Debug, Clone)]
pub struct OsintResult {
    pub query: OsintQuery,
    pub data: HashMap<String, String>,
    pub source: String,
    pub confidence: f64,
    pub cached: bool,
}

pub struct AsyncOsintEngine {
    config: OsintConfig,
    semaphore: Arc<Semaphore>,
    cache: HashMap<String, (OsintResult, Instant)>,
}

impl AsyncOsintEngine {
    pub fn new(config: OsintConfig) -> Self {
        Self {
            config: config.clone(),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent)),
            cache: HashMap::new(),
        }
    }

    pub async fn query(&self, query: &OsintQuery) -> Result<OsintResult, String> {
        let cache_key = format!("{}:{:?}", query.target, query.query_type);
        if let Some((cached, time)) = self.cache.get(&cache_key) {
            if time.elapsed().as_secs() < self.config.cache_ttl_secs {
                return Ok(OsintResult {
                    cached: true,
                    ..cached.clone()
                });
            }
        }

        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| "Semaphore closed".to_string())?;

        let mut data = HashMap::new();
        data.insert("target".into(), query.target.clone());
        data.insert("type".into(), format!("{:?}", query.query_type));

        Ok(OsintResult {
            query: query.clone(),
            data,
            source: "async_osint".into(),
            confidence: 0.8,
            cached: false,
        })
    }

    pub async fn batch_query(&self, queries: &[OsintQuery]) -> Vec<Result<OsintResult, String>> {
        let mut handles = Vec::new();
        for q in queries {
            let engine = AsyncOsintEngine {
                config: self.config.clone(),
                semaphore: self.semaphore.clone(),
                cache: HashMap::new(),
            };
            let q = q.clone();
            handles.push(tokio::spawn(async move { engine.query(&q).await }));
        }

        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.unwrap_or(Err("Task failed".into())));
        }
        results
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    pub fn config(&self) -> &OsintConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query() {
        let e = AsyncOsintEngine::new(OsintConfig::default());
        let q = OsintQuery {
            target: "example.com".into(),
            query_type: QueryType::Whois,
            params: HashMap::new(),
        };
        let r = e.query(&q).await.unwrap();
        assert!(!r.data.is_empty());
    }

    #[tokio::test]
    async fn test_batch() {
        let e = AsyncOsintEngine::new(OsintConfig::default());
        let queries = vec![
            OsintQuery {
                target: "a.com".into(),
                query_type: QueryType::Dns,
                params: HashMap::new(),
            },
            OsintQuery {
                target: "b.com".into(),
                query_type: QueryType::Subdomains,
                params: HashMap::new(),
            },
        ];
        let results = e.batch_query(&queries).await;
        assert_eq!(results.len(), 2);
    }
}
