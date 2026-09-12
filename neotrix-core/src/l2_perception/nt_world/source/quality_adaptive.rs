use super::types::*;

pub fn _suggest_quality(bandwidth_kbps: u64) -> Quality {
    if bandwidth_kbps > 1000 {
        Quality::Flac
    } else if bandwidth_kbps > 320 {
        Quality::High
    } else {
        Quality::Standard
    }
}

pub fn _suggest_quality_detailed(bandwidth_kbps: u64) -> (Quality, &'static str) {
    if bandwidth_kbps > 4000 {
        (Quality::Master, "master - full lossless")
    } else if bandwidth_kbps > 2000 {
        (Quality::Flac, "flac - lossless")
    } else if bandwidth_kbps > 1000 {
        (Quality::HiRes, "hi-res - high quality")
    } else if bandwidth_kbps > 320 {
        (Quality::High, "320k - high bitrate")
    } else if bandwidth_kbps > 192 {
        (Quality::Standard, "192k - standard")
    } else {
        (Quality::Low, "128k - low bandwidth")
    }
}

pub fn _select_best_quality(item: &MediaItem, bandwidth_kbps: u64) -> Quality {
    let suggested = _suggest_quality(bandwidth_kbps);
    item.qualities
        .iter()
        .find(|q| **q == suggested)
        .or_else(|| item.qualities.first())
        .copied()
        .unwrap_or(Quality::Standard)
}

pub fn _estimate_download_size(quality: Quality, duration_secs: u64) -> u64 {
    let bitrate_kbps = quality.bitrate() as u64;
    bitrate_kbps * duration_secs / 8
}
