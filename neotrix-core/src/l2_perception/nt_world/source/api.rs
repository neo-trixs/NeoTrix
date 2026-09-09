use super::types::*;
use super::engine::MediaEngine;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SourceInfo { pub id: String, pub name: String, pub media_type: String, pub healthy: bool, pub priority: u32 }

pub struct MediaApi {
    engine: Arc<RwLock<MediaEngine>>,
}

pub fn media_api() -> MediaApi {
    MediaApi { engine: Arc::new(RwLock::new(super::build_default_engine())) }
}

impl MediaApi {
    pub async fn search(&self, query: &str, media_type: Option<&str>, page: u32) -> Result<SearchResult, String> {
        self.engine.read().await.search(query, media_type, page).await
    }
    pub async fn play_url(&self, item: &MediaItem, quality: Quality) -> Result<ViewSource, String> {
        self.engine.read().await.play_url(item, quality).await
    }
    pub async fn lyric(&self, item: &MediaItem) -> Result<Lyric, String> {
        self.engine.read().await.lyric(item).await
    }
    pub async fn sources(&self) -> Vec<SourceInfo> {
        self.engine.read().await.sources().iter().map(|(s, p, h)| SourceInfo {
            id: s.id().into(), name: s.name().into(),
            media_type: format!("{:?}", s.media_type()),
            healthy: *h, priority: *p,
        }).collect()
    }
    pub async fn mark_source_unhealthy(&self, id: &str) {
        self.engine.write().await.mark_unhealthy(id);
    }
}
