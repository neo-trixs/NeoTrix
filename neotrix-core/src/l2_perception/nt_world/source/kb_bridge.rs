use crate::l2_perception::nt_world::source::types::*;

/// 搜索结果 → KB 节点存储
pub struct KbBridge;

impl KbBridge {
    /// 将搜索结果存入 KB
    pub fn store_search_result(result: &SearchResult) -> Vec<KbMediaNode> {
        result.data.iter().map(|item| {
            KbMediaNode {
                id: format!("media:{}:{}", result.source, item.id),
                title: item.title.clone(),
                artist: item.artist.clone(),
                album: item.album.clone(),
                source: result.source.clone(),
                media_type: format!("{:?}", item.media_type),
                qualities: item.qualities.iter().map(|q| format!("{:?}", q)).collect(),
            }
        }).collect()
    }
}

#[derive(Debug, Clone)]
pub struct KbMediaNode {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source: String,
    pub media_type: String,
    pub qualities: Vec<String>,
}
