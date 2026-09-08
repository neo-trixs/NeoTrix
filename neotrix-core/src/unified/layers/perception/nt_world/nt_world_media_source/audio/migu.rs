use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct MiguSource;

impl MiguSource {
    pub fn new() -> Self { Self }
}

impl MediaSource for MiguSource {
    fn id(&self) -> &str { "migu" }
    fn name(&self) -> &str { "咪咕音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!("https://m.music.migu.cn/migu/remoting/scr_search_tag?keyword={}&type=2&pgc={}&rows=30", urlencoding::encode(&query), page);
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["musics"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["id"].as_str()?.to_string();
                let title = s["songName"].as_str()?.to_string();
                let artist = s["singerName"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["albumName"].as_str()?.unwrap_or("").to_string();
                let duration = s["duration"].as_str().and_then(|d| d.parse::<u64>().ok()).map(|s| std::time::Duration::from_secs(s));
                Some(MediaItem { id, title, artist, album, duration, cover_url: None, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["pgt"].as_u64().unwrap_or(0) as usize * 30;
            Ok(SearchResult { data, total, source: "migu".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            let br = match quality {
                Quality::Flac => 2000,
                Quality::High => 320,
                _ => 128,
            };
            let url = format!("https://app.c.nf.migu.cn/MIGUM2.0/v1.0/content/sub/listenSong?toneFlag={}&songId={}", "HQ", id);
            Ok(ViewSource { url, quality, format: "mp3".into(), bitrate: br, size: 0, source: "migu".into() })
        })
    }
}
