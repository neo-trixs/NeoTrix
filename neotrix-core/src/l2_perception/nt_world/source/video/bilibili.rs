use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct BilibiliSource;

impl BilibiliSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for BilibiliSource {
    fn id(&self) -> &str { "bilibili" }
    fn name(&self) -> &str { "Bilibili" }
    fn media_type(&self) -> MediaType { MediaType::Video }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let q = query.to_string();
        Box::pin(async move {
            let url = format!("https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword={}&page={}", urlencoding::encode(&q), page);
            let resp = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "Mozilla/5.0")
                .header("Referer", "https://www.bilibili.com")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let results = json["data"]["result"].as_array().ok_or("No results")?;
            let data: Vec<MediaItem> = results.iter().filter_map(|v| {
                let bvid = v["bvid"].as_str()?.to_string();
                let title = v["title"].as_str()?.replace("<em class=\"keyword\">", "").replace("</em>", "");
                let author = v["author"].as_str()?.to_string();
                let duration = v["duration"].as_str()?.split(':').try_fold(0u64, |acc, s| {
                    let n: u64 = s.parse().map_err(|_| "")?;
                    Ok::<u64, ()>(acc * 60 + n)
                }).ok().map(|s| std::time::Duration::from_secs(s));
                let pic = v["pic"].as_str()?.strip_prefix("//").map(|p| format!("https://{}", p));
                Some(MediaItem {
                    id: bvid,
                    title,
                    artist: author,
                    album: String::new(),
                    duration,
                    cover_url: pic,
                    media_type: MediaType::Video,
                    qualities: vec![Quality::Standard],
                })
            }).collect();
            let total = json["data"]["numResults"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "bilibili".into(), page })
        })
    }
}
