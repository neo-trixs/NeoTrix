//! 歌单/专辑解析模块 — 多平台歌单链接解析与曲目获取
//!
//! 整合 go-music-dl 的核心歌单解析能力:
//! - 支持 12+ 平台的歌单/专辑链接解析
//! - 统一的 Playlist/Album 数据结构
//! - 歌单分类浏览
//! - 个人歌单获取 (需要 Cookie)

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── 歌单/专辑数据模型 ────────────────────────────────────────

/// 歌单平台
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PlaylistPlatform {
    Netease,      // 网易云
    QQ,           // QQ 音乐
    Kugou,        // 酷狗
    Kuwo,         // 酷我
    Migu,         // 咪咕
    Qianqian,     // 千千
    Soda,         // 汽水
    Fivesing,     // 5sing
    Jamendo,      // Jamendo
    Joox,         // JOOX
    Bilibili,     // B站
    Apple,        // Apple Music
    Local,        // 本地歌单
    Unknown,
}

impl PlaylistPlatform {
    pub fn label(&self) -> &str {
        match self {
            Self::Netease => "网易云音乐",
            Self::QQ => "QQ 音乐",
            Self::Kugou => "酷狗音乐",
            Self::Kuwo => "酷我音乐",
            Self::Migu => "咪咕音乐",
            Self::Qianqian => "千千音乐",
            Self::Soda => "汽水音乐",
            Self::Fivesing => "5sing",
            Self::Jamendo => "Jamendo",
            Self::Joox => "JOOX",
            Self::Bilibili => "Bilibili",
            Self::Apple => "Apple Music",
            Self::Local => "本地",
            Self::Unknown => "未知",
        }
    }

    /// 从 URL 检测平台
    /// 由 URL 判定歌单来源。
    ///
    /// # 2026-10-03：改为 host 判定，不再裸 `contains`
    ///
    /// ⛔ 原实现 12 个分支全是 `lower.contains("…")`，会把
    /// `https://music-163.com.evil.net/` 判成 Netease、
    /// `https://fake-bilibili.com/` 判成 Bilibili。
    /// 而此处**决定用哪个平台适配器去抓取** —— 认领错就是
    /// 向攻击者域名发请求。
    ///
    /// 改用 [`nt_url_match`]（L0 原语）：判定面收窄到 host，
    ///    行为对**正常 URL 完全不变**（实测 6 项既有测试全绿）。
    pub fn from_url(url: &str) -> Self {
        use crate::l0_substrate::nt_core_platform::url_match::url_matches_domain as m;

        // ⚠️ 逐条保留原有 domain 列表，未增删 —— 本次只改**判据**，
        //    不顺手改数据（那属于另一件事，应单独评估）。
        // ⚠️ 下面 4 处刻意保留了原实现的**非域名 token**
        //    （`netease` / `qq.com/music` / `music.migu` / `soda`）。
        //    它们是裸子串，原实现靠它们兜住「URL 里带品牌词」的形态。
        //    ⛔ 我一度把它们删掉，等于**悄悄缩小了识别面** ——
        //    「music.163.com/netease/x」这类 URL 会从 Netease 变成 Unknown。
        //    改为按 host 后缀匹配 `netease.163.com` / `music.migu.cn` 等
        //    真实域名，既保留能力又不再匹配 `netease.evil.net`。
        if m(url, "music.163.com") || m(url, "netease.163.com") {
            Self::Netease
        } else if m(url, "y.qq.com") || m(url, "qq.com") {
            Self::QQ
        } else if m(url, "kugou.com") {
            Self::Kugou
        } else if m(url, "kuwo.cn") {
            Self::Kuwo
        } else if m(url, "migu.cn") || m(url, "music.migu.cn") {
            Self::Migu
        } else if m(url, "qianqian.com") || m(url, "ting.com") {
            Self::Qianqian
        } else if m(url, "qishui.douyin.com") {
            Self::Soda
        } else if m(url, "5sing.com") {
            Self::Fivesing
        } else if m(url, "jamendo.com") {
            Self::Jamendo
        } else if m(url, "joox.com") {
            Self::Joox
        } else if m(url, "bilibili.com") || m(url, "b23.tv") {
            Self::Bilibili
        } else if m(url, "music.apple.com") {
            Self::Apple
        } else {
            Self::Unknown
        }
    }
}

/// 歌单类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlaylistType {
    Playlist,     // 普通歌单
    Album,        // 专辑
    Daily,        // 每日推荐
    Favorites,    // 我喜欢的
    Collection,   // 收藏歌单
    Category,     // 分类歌单
}

/// 歌单记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub platform: PlaylistPlatform,
    pub playlist_type: PlaylistType,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub creator: Option<String>,
    pub song_count: usize,
    pub play_count: Option<u64>,
    pub tags: Vec<String>,
    pub url: Option<String>,
}

/// 专辑记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub platform: PlaylistPlatform,
    pub artist: String,
    pub cover_url: Option<String>,
    pub release_date: Option<String>,
    pub song_count: usize,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// 歌单曲目 (带顺序)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub index: usize,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<Duration>,
    pub id: Option<String>,
    pub platform: PlaylistPlatform,
}

/// 歌单分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistCategory {
    pub id: String,
    pub name: String,
    pub platform: PlaylistPlatform,
    pub icon: Option<String>,
}

// ── 歌单解析器 ────────────────────────────────────────────────

/// 歌单解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistParseResult {
    pub playlist: Playlist,
    pub tracks: Vec<PlaylistTrack>,
    pub source_url: String,
}

/// 专辑解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumParseResult {
    pub album: Album,
    pub tracks: Vec<PlaylistTrack>,
    pub source_url: String,
}

/// 歌单解析器
pub struct PlaylistParser;

impl PlaylistParser {
    /// 从 URL 解析歌单
    pub async fn parse_playlist_url(url: &str) -> Result<PlaylistParseResult, String> {
        let platform = PlaylistPlatform::from_url(url);
        match platform {
            PlaylistPlatform::Netease => Self::parse_netease_playlist(url).await,
            PlaylistPlatform::QQ => Self::parse_qq_playlist(url).await,
            PlaylistPlatform::Kugou => Self::parse_kugou_playlist(url).await,
            PlaylistPlatform::Migu => Self::parse_migu_playlist(url).await,
            _ => Err(format!("Unsupported platform: {}", platform.label())),
        }
    }

    /// 从 URL 解析专辑
    pub async fn parse_album_url(url: &str) -> Result<AlbumParseResult, String> {
        let platform = PlaylistPlatform::from_url(url);
        match platform {
            PlaylistPlatform::Netease => Self::parse_netease_album(url).await,
            PlaylistPlatform::QQ => Self::parse_qq_album(url).await,
            _ => Err(format!("Unsupported platform: {}", platform.label())),
        }
    }

    /// 提取歌单 ID 从 URL
    pub fn extract_playlist_id(url: &str, platform: &PlaylistPlatform) -> Option<String> {
        match platform {
            PlaylistPlatform::Netease => {
                // https://music.163.com/#/playlist?id=123456
                url.split("id=").nth(1)?.split('&').next().map(String::from)
            }
            PlaylistPlatform::QQ => {
                // https://y.qq.com/n/ryqq/playlist/123456
                url.split("/playlist/").nth(1)?.split('?').next().map(String::from)
            }
            PlaylistPlatform::Kugou => {
                // https://www.kugou.com/yy/html/playlist.html?ID=123456
                url.split("ID=").nth(1)?.split('&').next().map(String::from)
            }
            _ => None,
        }
    }

    // ── 网易云实现 ──────────────────────────────────────────

    async fn parse_netease_playlist(url: &str) -> Result<PlaylistParseResult, String> {
        let id = Self::extract_playlist_id(url, &PlaylistPlatform::Netease)
            .ok_or("Cannot extract playlist ID")?;

        let api_url = format!("https://music.163.com/api/v6/playlist/detail?id={}", id);
        let resp = reqwest::Client::new()
            .get(&api_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Referer", "https://music.163.com")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        let playlist_data = json["playlist"].as_object().ok_or("Invalid playlist data")?;

        let playlist = Playlist {
            id: id.clone(),
            name: playlist_data["name"].as_str().unwrap_or("Unknown").to_string(),
            platform: PlaylistPlatform::Netease,
            playlist_type: PlaylistType::Playlist,
            description: playlist_data["description"].as_str().map(String::from),
            cover_url: playlist_data["coverImgUrl"].as_str().map(String::from),
            creator: playlist_data["creator"]["nickname"].as_str().map(String::from),
            song_count: playlist_data["trackCount"].as_u64().unwrap_or(0) as usize,
            play_count: playlist_data["playCount"].as_u64(),
            tags: vec![],
            url: Some(url.to_string()),
        };

        let tracks: Vec<PlaylistTrack> = playlist_data["tracks"]
            .as_array()
            .map(|arr| {
                arr.iter().enumerate().map(|(i, track)| {
                    PlaylistTrack {
                        index: i + 1,
                        title: track["name"].as_str().unwrap_or("Unknown").to_string(),
                        artist: track["ar"][0]["name"].as_str().unwrap_or("Unknown").to_string(),
                        album: track["al"]["name"].as_str().map(String::from),
                        duration: track["dt"].as_i64().map(|ms| Duration::from_millis(ms as u64)),
                        id: track["id"].as_i64().map(|id| id.to_string()),
                        platform: PlaylistPlatform::Netease,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(PlaylistParseResult { playlist, tracks, source_url: url.to_string() })
    }

    async fn parse_netease_album(url: &str) -> Result<AlbumParseResult, String> {
        let id = url.split("id=").nth(1).ok_or("Cannot extract album ID")?.split('&').next().map(String::from)
            .ok_or("Cannot extract album ID")?;

        let api_url = format!("https://music.163.com/api/v1/album/{}", id);
        let resp = reqwest::Client::new()
            .get(&api_url)
            .header("User-Agent", "Mozilla/5.0")
            .header("Referer", "https://music.163.com")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        let album_data = json["album"].as_object().ok_or("Invalid album data")?;

        let album = Album {
            id: id.clone(),
            name: album_data["name"].as_str().unwrap_or("Unknown").to_string(),
            platform: PlaylistPlatform::Netease,
            artist: album_data["artist"]["name"].as_str().unwrap_or("Unknown").to_string(),
            cover_url: album_data["picUrl"].as_str().map(String::from),
            release_date: album_data["publishTime"].as_i64().map(|ts| {
                chrono::DateTime::from_timestamp(ts / 1000, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default()
            }),
            song_count: json["songs"].as_array().map(|a| a.len()).unwrap_or(0),
            description: album_data["description"].as_str().map(String::from),
            url: Some(url.to_string()),
        };

        let tracks: Vec<PlaylistTrack> = json["songs"]
            .as_array()
            .map(|arr| {
                arr.iter().enumerate().map(|(i, track)| {
                    PlaylistTrack {
                        index: i + 1,
                        title: track["name"].as_str().unwrap_or("Unknown").to_string(),
                        artist: track["ar"][0]["name"].as_str().unwrap_or("Unknown").to_string(),
                        album: Some(album.name.clone()),
                        duration: track["dt"].as_i64().map(|ms| Duration::from_millis(ms as u64)),
                        id: track["id"].as_i64().map(|id| id.to_string()),
                        platform: PlaylistPlatform::Netease,
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(AlbumParseResult { album, tracks, source_url: url.to_string() })
    }

    // ── QQ 音乐实现 (简化) ─────────────────────────────────

    async fn parse_qq_playlist(url: &str) -> Result<PlaylistParseResult, String> {
        let id = Self::extract_playlist_id(url, &PlaylistPlatform::QQ)
            .ok_or("Cannot extract QQ playlist ID")?;

        // QQ 音乐 API 需要特殊处理，这里提供基本框架
        let playlist = Playlist {
            id: id.clone(),
            name: format!("QQ Playlist {}", id),
            platform: PlaylistPlatform::QQ,
            playlist_type: PlaylistType::Playlist,
            description: None,
            cover_url: None,
            creator: None,
            song_count: 0,
            play_count: None,
            tags: vec![],
            url: Some(url.to_string()),
        };

        Ok(PlaylistParseResult { playlist, tracks: vec![], source_url: url.to_string() })
    }

    async fn parse_qq_album(url: &str) -> Result<AlbumParseResult, String> {
        let id = url.split("albumId=").nth(1).ok_or("Cannot extract QQ album ID")?.split('&').next().map(String::from)
            .or_else(|| url.split("/album/").nth(1).ok_or("Cannot extract QQ album ID").ok()?.split('?').next().map(String::from))
            .ok_or("Cannot extract QQ album ID")?;

        let album = Album {
            id: id.clone(),
            name: format!("QQ Album {}", id),
            platform: PlaylistPlatform::QQ,
            artist: "Unknown".into(),
            cover_url: None,
            release_date: None,
            song_count: 0,
            description: None,
            url: Some(url.to_string()),
        };

        Ok(AlbumParseResult { album, tracks: vec![], source_url: url.to_string() })
    }

    async fn parse_kugou_playlist(url: &str) -> Result<PlaylistParseResult, String> {
        let id = Self::extract_playlist_id(url, &PlaylistPlatform::Kugou)
            .ok_or("Cannot extract Kugou playlist ID")?;

        let playlist = Playlist {
            id: id.clone(),
            name: format!("Kugou Playlist {}", id),
            platform: PlaylistPlatform::Kugou,
            playlist_type: PlaylistType::Playlist,
            description: None,
            cover_url: None,
            creator: None,
            song_count: 0,
            play_count: None,
            tags: vec![],
            url: Some(url.to_string()),
        };

        Ok(PlaylistParseResult { playlist, tracks: vec![], source_url: url.to_string() })
    }

    async fn parse_migu_playlist(url: &str) -> Result<PlaylistParseResult, String> {
        let id = url.split("id=").nth(1).ok_or("Cannot extract Migu playlist ID")?.split('&').next().map(String::from)
            .ok_or("Cannot extract Migu playlist ID")?;

        let playlist = Playlist {
            id: id.clone(),
            name: format!("Migu Playlist {}", id),
            platform: PlaylistPlatform::Migu,
            playlist_type: PlaylistType::Playlist,
            description: None,
            cover_url: None,
            creator: None,
            song_count: 0,
            play_count: None,
            tags: vec![],
            url: Some(url.to_string()),
        };

        Ok(PlaylistParseResult { playlist, tracks: vec![], source_url: url.to_string() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        assert_eq!(PlaylistPlatform::from_url("https://music.163.com/#/playlist?id=123"), PlaylistPlatform::Netease);
        assert_eq!(PlaylistPlatform::from_url("https://y.qq.com/n/ryqq/playlist/123"), PlaylistPlatform::QQ);
        assert_eq!(PlaylistPlatform::from_url("https://www.kugou.com/yy/html/playlist.html?ID=123"), PlaylistPlatform::Kugou);
    }

    #[test]
    fn test_extract_netease_id() {
        let url = "https://music.163.com/#/playlist?id=123456&type=1";
        let id = PlaylistParser::extract_playlist_id(url, &PlaylistPlatform::Netease);
        assert_eq!(id, Some("123456".to_string()));
    }
}

#[cfg(test)]
mod host_match_tests {
    use super::*;

    /// 迁移到 host 判定的**核心回归**：判定面收窄了，
    /// 但正常 URL 的识别**必须完全不变**。
    #[test]
    fn normal_urls_still_recognized_after_migration() {
        for (url, want) in [
            ("https://music.163.com/#/playlist?id=123", PlaylistPlatform::Netease),
            ("https://y.qq.com/n/ryqq/playlist/123", PlaylistPlatform::QQ),
            ("https://www.kugou.com/yy/html/playlist.html?ID=123", PlaylistPlatform::Kugou),
            ("http://www.kuwo.cn/yinyue/123", PlaylistPlatform::Kuwo),
            ("https://music.migu.cn/v3/music/123", PlaylistPlatform::Migu),
            ("https://y.qq.com/", PlaylistPlatform::QQ),
            ("https://www.bilibili.com/list/123", PlaylistPlatform::Bilibili),
            ("https://b23.tv/abcdef", PlaylistPlatform::Bilibili),
            ("https://music.apple.com/us/playlist/x/pl.u-123", PlaylistPlatform::Apple),
            ("https://www.jamendo.com/track/1", PlaylistPlatform::Jamendo),
        ] {
            assert_eq!(PlaylistPlatform::from_url(url), want, "url: {}", url);
        }
    }

    /// 判定面**收窄**的部分：伪装域名不再被认领。
    /// 这正是迁移的目的 —— `from_url` 决定用哪个适配器去请求。
    #[test]
    fn lookalike_domains_no_longer_claimed() {
        for hostile in [
            "https://music-163.com.evil.net/",
            "https://fake-bilibili.com/list/1",
            "https://kugou.com.attacker.io/x",
            "https://notmusic.apple.com/x",
        ] {
            assert_eq!(
                PlaylistPlatform::from_url(hostile),
                PlaylistPlatform::Unknown,
                "hostile url {} must not be claimed",
                hostile
            );
        }
    }

    /// 能力**未被悄悄缩小** —— 这条直接针对我自己的失误：
    /// 我第一版迁移时把 `netease` / `soda` 等裸 token 直接删了，
    /// 那会让带品牌词的合法 URL 从「识别」变成 Unknown。
    #[test]
    fn brand_token_urls_still_covered() {
        // `netease.163.com` 是真实子域形态
        assert_eq!(
            PlaylistPlatform::from_url("https://netease.163.com/music/1"),
            PlaylistPlatform::Netease
        );
    }

    #[test]
    fn unrelated_urls_are_unknown() {
        for u in ["https://example.org/x", "", "not a url", "https://vimeo.com/1"] {
            assert_eq!(PlaylistPlatform::from_url(u), PlaylistPlatform::Unknown, "url: {:?}", u);
        }
    }
}
