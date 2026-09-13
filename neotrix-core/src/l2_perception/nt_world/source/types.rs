use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    Audio,
    Video,
    Image,
    Document,
    Book,
    Lyrics,
    Social,
    Feed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    Master,
    AtmosPlus,
    Atmos,
    HiRes,
    Flac24bit,
    Flac,
    High,
    Standard,
    Low,
}

impl Quality {
    pub fn bitrate(&self) -> u32 {
        match self {
            Quality::Master => 9000,
            Quality::AtmosPlus => 4000,
            Quality::Atmos => 3000,
            Quality::HiRes => 2400,
            Quality::Flac24bit => 2000,
            Quality::Flac => 1411,
            Quality::High => 320,
            Quality::Standard => 192,
            Quality::Low => 128,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Quality::Master => "Master",
            Quality::AtmosPlus => "Atmos+",
            Quality::Atmos => "Atmos",
            Quality::HiRes => "Hi-Res",
            Quality::Flac24bit => "24bit",
            Quality::Flac => "FLAC",
            Quality::High => "320k",
            Quality::Standard => "192k",
            Quality::Low => "128k",
        }
    }

    pub fn fallback_chain(&self) -> Vec<Quality> {
        match self {
            Quality::Master => vec![Quality::Master, Quality::AtmosPlus, Quality::Atmos, Quality::HiRes, Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::AtmosPlus => vec![Quality::AtmosPlus, Quality::Atmos, Quality::HiRes, Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::Atmos => vec![Quality::Atmos, Quality::HiRes, Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::HiRes => vec![Quality::HiRes, Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::Flac24bit => vec![Quality::Flac24bit, Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::Flac => vec![Quality::Flac, Quality::High, Quality::Standard, Quality::Low],
            Quality::High => vec![Quality::High, Quality::Standard, Quality::Low],
            Quality::Standard => vec![Quality::Standard, Quality::Low],
            Quality::Low => vec![Quality::Low],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<Duration>,
    pub cover_url: Option<String>,
    pub media_type: MediaType,
    pub qualities: Vec<Quality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaSearchResult {
    pub data: Vec<MediaItem>,
    pub total: usize,
    pub source: String,
    pub page: u32,
}

pub type SearchResult = MediaSearchResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSource {
    pub url: String,
    pub quality: Quality,
    pub format: String,
    pub bitrate: u32,
    pub size: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub timestamp: Option<Duration>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lyric {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub lines: Vec<LyricLine>,
    pub source: String,
}
