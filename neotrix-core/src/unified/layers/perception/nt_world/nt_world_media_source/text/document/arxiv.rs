use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct ArxivSource;

impl ArxivSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for ArxivSource {
    fn id(&self) -> &str { "arxiv" }
    fn name(&self) -> &str { "arXiv" }
    fn media_type(&self) -> MediaType { MediaType::Document }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let start = (page - 1) * 20;
            let url = format!("http://export.arxiv.org/api/query?search_query=all:{}&start={}&max_results=20", urlencoding::encode(&q), start);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let body = resp.text().await.map_err(|e| e.to_string())?;
            let entries: Vec<MediaItem> = body.split("<entry>").skip(1).filter_map(|entry| {
                let id = entry.split("</id>").next()?.split("/abs/").last()?.to_string();
                let title = entry.split("<title>").nth(1)?.split("</title>").next()?.trim().replace('\n', " ");
                let author = entry.split("<name>").nth(1)?.split("</name>").next()?.to_string();
                let summary = entry.split("<summary>").nth(1)?.split("</summary>").next()?.trim().replace('\n', " ");
                let link = format!("https://arxiv.org/abs/{}", id);
                Some(MediaItem {
                    id,
                    title,
                    artist: author,
                    album: summary,
                    duration: None,
                    cover_url: Some(link),
                    media_type: MediaType::Document,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = entries.len();
            Ok(SearchResult { data: entries, total, source: "arxiv".into(), page })
        })
    }
}
