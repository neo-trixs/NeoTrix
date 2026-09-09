use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;
use crate::l2_perception::nt_world::source::crypto;

pub struct NeteaseSource;

impl NeteaseSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for NeteaseSource {
    fn id(&self) -> &str { "netease" }
    fn name(&self) -> &str { "网易云音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let params = format!(
                r#"{{"s":"{}","type":1,"limit":30,"offset":{}}}"#,
                query,
                (page - 1) * 30
            );
            let encrypted = crypto::netease_weapi_encrypt(&params);
            let client = reqwest::Client::new();
            let resp = client
                .post("https://music.163.com/weapi/search/get")
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Referer", "https://music.163.com")
                .form(&[("params", encrypted.as_str())])
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let songs = json["result"]["songs"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = songs.iter().filter_map(|s| {
                let id = s["id"].as_i64()?.to_string();
                let title = s["name"].as_str()?.to_string();
                let artist = s["artists"][0]["name"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album"]["name"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_i64().map(|ms| std::time::Duration::from_millis(ms as u64));
                Some(MediaItem { id, title, artist, album, duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["result"]["songCount"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "netease".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let params = format!(
                r#"{{"ids":"[{}]","br":{}}}"#,
                id,
                quality.bitrate() * 1000
            );
            let encrypted = crypto::netease_eapi_encrypt("/api/song/enhance/player/url", &params);
            let client = reqwest::Client::new();
            let resp = client
                .post(format!(
                    "https://music.163.com/api/song/enhance/player/url?eapi={}",
                    urlencoding::encode(&encrypted)
                ))
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Referer", "https://music.163.com")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let url = json["data"][0]["url"].as_str().unwrap_or(&format!(
                "https://music.163.com/song/media/outer/url?id={}",
                id
            )).to_string();
            let format = if url.contains(".flac") { "flac" } else { "mp3" }.to_string();
            Ok(ViewSource { url, quality, format, bitrate: quality.bitrate(), size: 0, source: "netease".into() })
        })
    }
}
