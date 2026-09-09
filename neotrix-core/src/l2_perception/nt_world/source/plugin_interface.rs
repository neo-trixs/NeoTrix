use super::types::*;
use super::engine::MediaSource;
use std::pin::Pin;
use std::future::Future;

/// 媒体源插件接口
pub trait MediaSourcePlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn search(&self, query: &str, page: u32) -> Pin<Box<dyn Future<Output = Result<SearchResult, String>> + Send>>;
    fn play_url(&self, item: &MediaItem, quality: Quality) -> Pin<Box<dyn Future<Output = Result<ViewSource, String>> + Send>>;
}
