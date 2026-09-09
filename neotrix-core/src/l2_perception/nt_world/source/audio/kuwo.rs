use crate::l2_perception::nt_world::source::types::*;
use crate::l2_perception::nt_world::source::engine::MediaSource;

pub struct KuwoSource;

impl KuwoSource {
    pub fn new() -> Self { Self }

    /// 获取 csrf token
    async fn get_csrf_token() -> String {
        let client = reqwest::Client::new();
        if let Ok(resp) = client.get("https://www.kuwo.cn/").send().await {
            if let Ok(cookies) = resp.cookies().collect::<Result<Vec<_>, _>>() {
                for cookie in cookies {
                    if cookie.name() == "kw_token" {
                        return cookie.value().to_string();
                    }
                }
            }
        }
        "HK4IEQVENDR".to_string() // 回退默认值
    }
}

impl MediaSource for KuwoSource {
    fn id(&self) -> &str { "kuwo" }
    fn name(&self) -> &str { "酷我音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let csrf = Self::get_csrf_token().await;
            let url = format!(
                "https://www.kuwo.cn/api/www/search/searchMusicBykeyWord?key={}&pn={}&rn=30",
                urlencoding::encode(&query), page
            );
            let resp = reqwest::Client::new()
                .get(&url)
                .header("csrf", &csrf)
                .header("Cookie", format!("kw_token={}", csrf))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["data"]["list"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["rid"].as_i64()?.to_string();
                let title = s["name"].as_str()?.to_string();
                let artist = s["artist"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["album"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                let cover_url = s["pic"].as_str().map(|s| s.to_string());
                Some(MediaItem { id, title, artist, album, duration, cover_url, media_type: MediaType::Audio, qualities: vec![Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["data"]["total"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "kuwo".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let (br, quality_str) = match quality {
                Quality::Flac24bit => (2000, "24bit"),
                Quality::Flac => (2000, "flac"),
                Quality::High => (320, "mp3"),
                _ => (192, "mp3"),
            };
            let url = format!(
                "https://www.kuwo.cn/api/v1/www/music/playUrl?mid={}&type={}&httpsStatus=1&reqId={}",
                id, quality_str, uuid::Uuid::new_v4()
            );
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let play_url = json["data"]["url"].as_str().unwrap_or(&url).to_string();
            let format = if play_url.contains(".flac") { "flac" } else { "mp3" }.to_string();
            Ok(ViewSource { url: play_url, quality, format, bitrate: br, size: 0, source: "kuwo".into() })
        })
    }
}
