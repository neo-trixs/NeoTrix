//! MediaSource 桥接层 — 将现有 29 个媒体源适配到统一架构
//!
//! 自动将 `engine::MediaSource` 实现包装为 `unified::MediaSource`，
//! 无需修改任何现有源代码。

use super::engine;
use super::types::{MediaType, Quality};
use super::unified::{
    DataSource, MediaSource, SourceDomain, MediaSearchResult, MediaResult, PlaySource, Lyric,
};
use std::sync::Arc;

use super::text::document::arxiv::ArxivSource;
use super::text::document::semantic_scholar::SemanticScholarSource;
use super::text::book::openlibrary::OpenLibrarySource;
use super::text::book::annas_archive::AnnasArchiveSource;
use super::text::lyrics::lrclib::LrclibSource;
use super::text::lyrics::multi::MultiLyricSource;
use super::text::lyrics::genius::GeniusSource;
use super::text::social::tiktok::TikTokSource;
use super::text::social::instagram::InstagramSource;
use super::text::social::twitter::TwitterSource;
use super::text::social::ytdlp::YtdlpSource;

/// 将 engine::MediaSource 包装为 unified::MediaSource
pub struct MediaSourceBridge {
    id: String,
    name: String,
    domains: Vec<SourceDomain>,
    inner: Arc<dyn engine::MediaSource>,
}

impl MediaSourceBridge {
    pub fn new(source: Arc<dyn engine::MediaSource>) -> Self {
        let id = source.id().to_string();
        let name = source.name().to_string();
        let domains = Self::infer_domains(source.media_type());

        Self {
            id,
            name,
            domains,
            inner: source,
        }
    }

    /// 根据 MediaType 推断 Domain
    fn infer_domains(media_type: MediaType) -> Vec<SourceDomain> {
        match media_type {
            MediaType::Audio | MediaType::Video | MediaType::Image => {
                vec![SourceDomain::Media]
            }
            MediaType::Document | MediaType::Book => {
                vec![SourceDomain::Academic]
            }
            MediaType::Lyrics | MediaType::Feed => {
                vec![SourceDomain::Media, SourceDomain::Tech]
            }
            MediaType::Social => {
                vec![SourceDomain::Tech, SourceDomain::Intel]
            }
        }
    }

    fn parse_quality(s: &str) -> Quality {
        match s.to_lowercase().as_str() {
            "master" => Quality::Master,
            "atmos+" | "atmosplus" => Quality::AtmosPlus,
            "atmos" => Quality::Atmos,
            "hi-res" | "hires" => Quality::HiRes,
            "24bit" | "flac24" => Quality::Flac24bit,
            "flac" => Quality::Flac,
            "high" | "320" | "320k" => Quality::High,
            "standard" | "192" | "192k" => Quality::Standard,
            "low" | "128" | "128k" => Quality::Low,
            _ => Quality::Standard,
        }
    }
}

impl DataSource for MediaSourceBridge {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn domains(&self) -> Vec<SourceDomain> {
        self.domains.clone()
    }
}

impl MediaSource for MediaSourceBridge {
    fn search(
        &self,
        query: &str,
        page: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MediaSearchResult, String>> + Send>> {
        let inner = self.inner.clone();
        let q = query.to_string();
        Box::pin(async move {
            let result = inner.search(&q, page).await?;

            let items = result
                .data
                .into_iter()
                .map(|item| MediaResult {
                    id: item.id,
                    title: item.title,
                    artist: item.artist,
                    album: item.album,
                    duration_secs: item.duration.map(|d| d.as_secs()),
                    cover_url: item.cover_url,
                    qualities: item.qualities.iter().map(|q| q.label().to_string()).collect(),
                })
                .collect();

            Ok(MediaSearchResult {
                items,
                total: result.total,
                page: result.page,
            })
        })
    }

    fn play_url(
        &self,
        item_id: &str,
        quality: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PlaySource, String>> + Send>> {
        let inner = self.inner.clone();
        let id = item_id.to_string();
        let q = quality.to_string();
        Box::pin(async move {
            let item = super::types::MediaItem {
                id: id.clone(),
                title: String::new(),
                artist: String::new(),
                album: String::new(),
                duration: None,
                cover_url: None,
                media_type: inner.media_type(),
                qualities: vec![],
            };

            let quality_enum = Self::parse_quality(&q);

            let result = inner.play_url(&item, quality_enum).await?;

            Ok(PlaySource {
                url: result.url,
                quality: result.quality.label().to_string(),
                format: result.format,
                bitrate: result.bitrate,
                size: result.size,
            })
        })
    }

    fn lyric(
        &self,
        item_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Lyric, String>> + Send>> {
        let inner = self.inner.clone();
        let id = item_id.to_string();
        Box::pin(async move {
            let item = super::types::MediaItem {
                id: id.clone(),
                title: String::new(),
                artist: String::new(),
                album: String::new(),
                duration: None,
                cover_url: None,
                media_type: inner.media_type(),
                qualities: vec![],
            };

            let result = inner.lyric(&item).await?;

            let lines = result
                .lines
                .into_iter()
                .map(|l| super::unified::LyricLine {
                    timestamp_ms: l.timestamp.map(|t| t.as_millis() as u64),
                    text: l.text,
                })
                .collect();

            Ok(Lyric {
                title: result.title,
                artist: result.artist,
                lines,
            })
        })
    }
}

/// 从 engine::MediaSource 构建桥接器
pub fn bridge_media_source(source: Arc<dyn engine::MediaSource>) -> Arc<dyn MediaSource> {
    Arc::new(MediaSourceBridge::new(source))
}

/// 批量桥接所有媒体源
pub fn bridge_all_media_sources() -> Vec<Arc<dyn MediaSource>> {
    use super::audio::*;
    use super::video::*;

    let sources: Vec<Arc<dyn engine::MediaSource>> = vec![
        // 音频 (11)
        Arc::new(netease::NeteaseSource::new()),
        Arc::new(kuwo::KuwoSource::new()),
        Arc::new(kugou::KugouSource::new()),
        Arc::new(migu::MiguSource::new()),
        Arc::new(soundcloud::SoundCloudSource::new()),
        Arc::new(spotify::SpotifySource::new()),
        Arc::new(piped::PipedSource::new()),
        Arc::new(jiosaavn::JioSaavnSource::new()),
        Arc::new(deezer::DeezerSource::new()),
        Arc::new(bandcamp::BandcampSource::new()),
        Arc::new(qqmusic::QQMusicSource::new()),
        // 视频 (3)
        Arc::new(youtube::YouTubeSource::new()),
        Arc::new(bilibili::BilibiliSource::new()),
        Arc::new(vimeo::VimeoSource::new()),
        // 文档 (2)
        Arc::new(ArxivSource::new()),
        Arc::new(SemanticScholarSource::new()),
        // 图书 (2)
        Arc::new(OpenLibrarySource::new()),
        Arc::new(AnnasArchiveSource::new()),
        // 歌词 (3)
        Arc::new(LrclibSource::new()),
        Arc::new(MultiLyricSource::new()),
        Arc::new(GeniusSource::new()),
        // 社交 (4)
        Arc::new(TikTokSource::new()),
        Arc::new(InstagramSource::new()),
        Arc::new(TwitterSource::new()),
        Arc::new(YtdlpSource::new()),
    ];

    sources
        .into_iter()
        .map(|s| bridge_media_source(s) as Arc<dyn MediaSource>)
        .collect()
}
