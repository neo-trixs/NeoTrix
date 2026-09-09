use crate::unified::layers::perception::nt_world::nt_world_media_source::types::*;
use crate::unified::layers::perception::nt_world::nt_world_media_source::engine::MediaSource;

pub struct QQMusicSource;

impl QQMusicSource {
    pub fn new() -> Self { Self }

    /// 获取歌词
    async fn get_lyrics(songmid: &str) -> Option<String> {
        let url = format!(
            "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg?songmid={}&format=json&nobase64=1",
            songmid
        );
        let resp = reqwest::get(&url).await.ok()?;
        let json: serde_json::Value = resp.json().await.ok()?;
        let lyric = json["lyric"].as_str()?.to_string();
        Some(lyric)
    }
}

impl MediaSource for QQMusicSource {
    fn id(&self) -> &str { "qqmusic" }
    fn name(&self) -> &str { "QQ音乐" }
    fn media_type(&self) -> MediaType { MediaType::Audio }

    fn search(&self, query: &str, page: u32) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<SearchResult, String>> + Send>> {
        let query = query.to_string();
        Box::pin(async move {
            let url = format!(
                "https://c.y.qq.com/soso/fcgi-bin/client_search_cp?w={}&p={}&n=30&format=json",
                urlencoding::encode(&query), page
            );
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let list = json["data"]["song"]["list"].as_array().ok_or("No songs")?;
            let data: Vec<MediaItem> = list.iter().filter_map(|s| {
                let id = s["songmid"].as_str()?.to_string();
                let title = s["songname"].as_str()?.to_string();
                let artist = s["singer"][0]["name"].as_str()?.unwrap_or("Unknown").to_string();
                let album = s["albumname"].as_str()?.unwrap_or("").to_string();
                let duration = s["interval"].as_i64().map(|s| std::time::Duration::from_secs(s as u64));
                let cover_url = s["albumid"].as_i64().map(|id| {
                    format!("https://y.gtimg.cn/music/photo_new/T002R300x300M000{}.jpg", id)
                });
                Some(MediaItem { id, title, artist, album, duration, cover_url, media_type: MediaType::Audio, qualities: vec![Quality::Flac, Quality::High, Quality::Standard] })
            }).collect();
            let total = json["data"]["song"]["totalnum"].as_u64().unwrap_or(0) as usize;
            Ok(SearchResult { data, total, source: "qqmusic".into(), page })
        })
    }

    fn play_url(&self, item: &MediaItem, quality: Quality) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ViewSource, String>> + Send>> {
        let id = item.id.clone();
        Box::pin(async move {
            // 使用 midurlinfo 获取播放地址
            let filename = match quality {
                Quality::Flac => format!("F000{}.flac", id),
                Quality::High => format!("M800{}.mp3", id),
                _ => format!("M500{}.mp3", id),
            };
            let url = format!(
                "https://c.y.qq.com/base/fcgi-bin/fcg_music_express_mobile3.fcg?songmid={}&filename={}",
                id, filename
            );
            let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            // 尝试从 midurlinfo 获取真实地址
            if let Some(purl) = json["data"]["items"][0]["purl"].as_str() {
                if !purl.is_empty() {
                    let full_url = format!("https://dl.stream.qqmusic.qq.com/{}", purl);
                    let format = if full_url.contains(".flac") { "flac" } else { "mp3" }.to_string();
                    return Ok(ViewSource { url: full_url, quality, format, bitrate: quality.bitrate(), size: 0, source: "qqmusic".into() });
                }
            }
            // VIP 歌曲降级: 返回试听链接
            let fallback_url = format!(
                "https://dl.stream.qqmusic.qq.com/M500{}.mp3?guid=0&vkey=test",
                id
            );
            Ok(ViewSource { url: fallback_url, quality, format: "mp3".into(), bitrate: 128, size: 0, source: "qqmusic".into() })
        })
    }
}
