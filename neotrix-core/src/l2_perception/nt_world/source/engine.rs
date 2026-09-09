use super::types::*;
use std::sync::Arc;
use std::collections::HashMap;

pub trait MediaSource: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn media_type(&self) -> MediaType;
    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>>;
    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let _ = (item, quality);
        Box::pin(async { Err("Not implemented".into()) })
    }
    fn lyric(&self, item: &MediaItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Lyric, String>> + Send>> {
        let _ = item;
        Box::pin(async { Err("Not implemented".into()) })
    }
}

struct SourceEntry {
    source: Arc<dyn MediaSource>,
    priority: u32,
    healthy: bool,
}

pub struct MediaEngine {
    sources: Vec<SourceEntry>,
    cache: HashMap<String, SearchResult>,
}

impl MediaEngine {
    pub fn new() -> Self { Self { sources: Vec::new(), cache: HashMap::new() } }
    pub fn add_source(&mut self, source: Arc<dyn MediaSource>, priority: u32) {
        self.sources.push(SourceEntry { source, priority, healthy: true });
        self.sources.sort_by_key(|s| s.priority);
    }
    pub fn sources(&self) -> Vec<(&dyn MediaSource, u32, bool)> {
        self.sources.iter().map(|s| (s.source.as_ref(), s.priority, s.healthy)).collect()
    }
    pub async fn search(&self, query: &str, media_type: Option<&str>, page: u32) -> Result<SearchResult, String> {
        let cache_key = format!("{}:{:?}:{}", query, media_type, page);
        if let Some(cached) = self.cache.get(&cache_key) { return Ok(cached.clone()); }
        for entry in &self.sources {
            if !entry.healthy { continue; }
            if let Some(mt) = media_type {
                let mt_str = format!("{:?}", entry.source.media_type()).to_lowercase();
                if mt_str != mt.to_lowercase() { continue; }
            }
            match entry.source.search(query, page).await {
                Ok(result) => { /* would cache here */ return Ok(result); }
                Err(_) => continue,
            }
        }
        Err("No source available".into())
    }
    pub async fn play_url(&self, item: &MediaItem, quality: Quality) -> Result<ViewSource, String> {
        for q in quality.fallback_chain() {
            for entry in &self.sources {
                if !entry.healthy { continue; }
                if entry.source.media_type() != item.media_type { continue; }
                if let Ok(source) = entry.source.play_url(item, q).await { return Ok(source); }
            }
        }
        Err("No source available for any quality".into())
    }
    pub async fn lyric(&self, item: &MediaItem) -> Result<Lyric, String> {
        for entry in &self.sources {
            if !entry.healthy { continue; }
            if let Ok(lyric) = entry.source.lyric(item).await { return Ok(lyric); }
        }
        Err("No lyrics available".into())
    }
    pub fn mark_unhealthy(&mut self, source_id: &str) {
        if let Some(entry) = self.sources.iter_mut().find(|s| s.source.id() == source_id) {
            entry.healthy = false;
        }
    }
    pub fn mark_healthy(&mut self, source_id: &str) {
        if let Some(entry) = self.sources.iter_mut().find(|s| s.source.id() == source_id) {
            entry.healthy = true;
        }
    }
}
