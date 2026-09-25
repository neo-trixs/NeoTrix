//! Sniff segment: media sniffing + m3u8 playlist + sequential play cursor.

// ──────────────────────────────────────────────────────────────
// P22 media_sniff — 媒体流嗅探 + m3u8 管线 (res-downloader 吸收)
// MITM 嗅探分类 + 分片清单构建 + 顺序播放游标; 纯确定性, 无真实网络。
// ──────────────────────────────────────────────────────────────


/// 媒体类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    /// HLS (m3u8)。
    Hls,
    /// DASH (mpd)。
    Dash,
    /// 渐进式下载 (mp4/webm)。
    Progressive,
    /// 直播流。
    Live,
}

impl MediaKind {
    pub fn label(self) -> &'static str {
        match self {
            MediaKind::Hls => "hls",
            MediaKind::Dash => "dash",
            MediaKind::Progressive => "progressive",
            MediaKind::Live => "live",
        }
    }
}

/// MITM 嗅探到的媒体资源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _SniffedMedia {
    pub url: String,
    pub kind: MediaKind,
    pub headers: Vec<(String, String)>,
}

/// HTTP 响应快照 (嗅探输入, 本文件定义)。
#[derive(Debug, Clone)]
pub struct _HttpSniff {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl _HttpSniff {
    /// 小写化 Content-Type 头值 (无则空串)。
    pub fn content_type(&self) -> String {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.to_ascii_lowercase())
            .unwrap_or_default()
    }
}

/// 确定性媒体类型检测: m3u8 → Hls; mpd → Dash; mp4/webm → Progressive;
/// "live" token → Live; 其余 None。
fn detect_kind(_sniff: &_HttpSniff) -> Option<MediaKind> {
    let url = _sniff.url.to_ascii_lowercase();
    let ct = _sniff.content_type();
    let body = _sniff.body.to_ascii_lowercase();
    if url.contains("m3u8") || ct.contains("application/vnd.apple.mpegurl") {
        return Some(MediaKind::Hls);
    }
    if url.contains("mpd") || ct.contains("application/dash+xml") {
        return Some(MediaKind::Dash);
    }
    if ct.contains("video/mp4") || ct.contains("video/webm") {
        return Some(MediaKind::Progressive);
    }
    if url.contains("live") || body.contains("live") {
        return Some(MediaKind::Live);
    }
    None
}

/// 单个分片 (含 f64 时长, 不实现 Eq)。
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub uri: String,
    pub duration_s: f64,
    pub index: u32,
}

/// 播放清单。
#[derive(Debug, Clone)]
pub struct _MediaPlaylist {
    pub segments: Vec<Segment>,
    pub duration_total_s: f64,
}

/// 顺序播放游标 (循环)。
pub struct PipelineStatus {
    cursor: usize,
    cycle_len: usize,
    steps_in_cycle: usize,
    _cycles_completed: usize,
}

impl Default for PipelineStatus {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineStatus {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            cycle_len: 0,
            steps_in_cycle: 0,
            _cycles_completed: 0,
        }
    }

    /// 取下一分片 (循环播放); 空清单返回 None。
    pub fn next_segment(&mut self, playlist: &_MediaPlaylist) -> Option<Segment> {
        if playlist.segments.is_empty() {
            return None;
        }
        self.cycle_len = playlist.segments.len();
        let seg = playlist.segments[self.cursor].clone();
        self.cursor = (self.cursor + 1) % playlist.segments.len();
        self.steps_in_cycle = self.cursor;
        if self.cursor == 0 {
            self._cycles_completed += 1;
        }
        Some(seg)
    }

    /// 当前循环内进度 0..1 (空清单为 0)。
    pub fn progress(&self) -> f64 {
        if self.cycle_len == 0 {
            0.0
        } else {
            self.steps_in_cycle as f64 / self.cycle_len as f64
        }
    }

    pub fn _cycles_completed(&self) -> usize {
        self._cycles_completed
    }
}

/// 媒体嗅探器 — 记录嗅探历史 + 播放状态。
pub struct MediaSniffer {
    pub sniffed: Vec<_SniffedMedia>,
    pub status: PipelineStatus,
}

impl Default for MediaSniffer {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaSniffer {
    pub fn new() -> Self {
        Self {
            sniffed: Vec::new(),
            status: PipelineStatus::new(),
        }
    }

    /// 嗅探单条 HTTP 响应 → 命中则记录并返回; 未知返回 None。
    pub fn _sniff(&mut self, http_response: &_HttpSniff) -> Option<_SniffedMedia> {
        let kind = detect_kind(http_response)?;
        let media = _SniffedMedia {
            url: http_response.url.clone(),
            kind,
            headers: http_response.headers.clone(),
        };
        self.sniffed.push(media.clone());
        Some(media)
    }

    /// 生成 variant URL + N 个顺序分片 (index 0..segments)。
    pub fn _build_playlist(
        &mut self,
        master_uri: &str,
        variant: &str,
        segments: u32,
        seg_dur: f64,
    ) -> _MediaPlaylist {
        let base = format!("{}/{}", master_uri.trim_end_matches('/'), variant);
        let segs = (0..segments)
            .map(|i| Segment {
                uri: format!("{base}/seg_{i:04}.ts"),
                duration_s: seg_dur,
                index: i,
            })
            .collect();
        _MediaPlaylist {
            segments: segs,
            duration_total_s: segments as f64 * seg_dur,
        }
    }

    pub fn _sniffed_count(&self) -> usize {
        self.sniffed.len()
    }
}

/// SelfTest (T1): "nt_world_video_pipeline_media_sniff" — 嗅探/清单/游标自检。
impl crate::l0_substrate::nt_core_self_test::SelfTest for MediaSniffer {
    fn name(&self) -> &str {
        "nt_world_video_pipeline_media_sniff"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut s = MediaSniffer::new();
        let hls = _HttpSniff {
            url: "https://cdn.example.com/playlist.m3u8".into(),
            headers: vec![("content-type".into(), "application/vnd.apple.mpegurl".into())],
            body: String::new(),
        };
        match s._sniff(&hls) {
            Some(m) => {
                if m.kind != MediaKind::Hls {
                    failures.push("m3u8 _sniff must classify as Hls".into());
                }
            }
            None => failures.push("m3u8 _sniff returned None".into()),
        }
        let pl = s._build_playlist("https://cdn.example.com/master.m3u8", "720p", 3, 4.0);
        if pl.segments.len() != 3 {
            failures.push("playlist should have 3 segments".into());
        }
        match s.status.next_segment(&pl) {
            Some(first) => {
                if first.index != 0 {
                    failures.push("first segment index should be 0".into());
                }
            }
            None => failures.push("next_segment returned None for non-empty playlist".into()),
        }
        if s.status.progress() <= 0.0 {
            failures.push("progress should advance after next_segment".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod media_sniff_tests {
    use super::*;

    fn _sniff(url: &str, ct: &str, body: &str) -> _HttpSniff {
        _HttpSniff {
            url: url.to_string(),
            headers: vec![("content-type".into(), ct.into())],
            body: body.to_string(),
        }
    }

    #[test]
    fn media_sniff_hls_from_m3u8_url() {
        let mut s = MediaSniffer::new();
        let m = s._sniff(&_sniff("https://cdn.example.com/master.m3u8", "text/plain", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Hls);
        assert_eq!(s._sniffed_count(), 1);
    }

    #[test]
    fn media_sniff_hls_from_content_type() {
        let mut s = MediaSniffer::new();
        let m = s._sniff(&_sniff("https://cdn.example.com/stream", "application/vnd.apple.mpegurl", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Hls);
    }

    #[test]
    fn media_sniff_dash_from_mpd_url() {
        let mut s = MediaSniffer::new();
        let m = s._sniff(&_sniff("https://cdn.example.com/video.mpd", "application/dash+xml", "")).expect("sniffed");
        assert_eq!(m.kind, MediaKind::Dash);
    }

    #[test]
    fn media_sniff_progressive_from_video_content_type() {
        let mut s = MediaSniffer::new();
        let mp4 = s._sniff(&_sniff("https://cdn.example.com/movie.mp4", "video/mp4", "")).expect("sniffed");
        assert_eq!(mp4.kind, MediaKind::Progressive);
        let webm = s._sniff(&_sniff("https://cdn.example.com/clip", "video/webm", "")).expect("sniffed");
        assert_eq!(webm.kind, MediaKind::Progressive);
    }

    #[test]
    fn media_sniff_live_from_token() {
        let mut s = MediaSniffer::new();
        let url = s._sniff(&_sniff("https://cdn.example.com/live/room1", "text/html", "")).expect("sniffed");
        assert_eq!(url.kind, MediaKind::Live);
        let body = s._sniff(&_sniff("https://cdn.example.com/room", "text/html", "this is a live stream")).expect("sniffed");
        assert_eq!(body.kind, MediaKind::Live);
    }

    #[test]
    fn media_sniff_unknown_returns_none() {
        let mut s = MediaSniffer::new();
        assert!(s._sniff(&_sniff("https://cdn.example.com/other", "text/html", "static page")).is_none());
        assert_eq!(s._sniffed_count(), 0);
    }

    #[test]
    fn media_sniff_playlist_segments_count_and_indices() {
        let mut s = MediaSniffer::new();
        let pl = s._build_playlist("https://cdn.example.com/master.m3u8", "1080p", 5, 6.0);
        assert_eq!(pl.segments.len(), 5);
        assert!((pl.duration_total_s - 30.0).abs() < 1e-9);
        for (i, seg) in pl.segments.iter().enumerate() {
            assert_eq!(seg.index, i as u32);
            assert_eq!(seg.duration_s, 6.0);
            assert!(seg.uri.contains("1080p"));
            assert!(seg.uri.ends_with(&format!("seg_{i:04}.ts")));
        }
    }

    #[test]
    fn media_sniff_next_segment_cycles() {
        let mut s = MediaSniffer::new();
        let pl = s._build_playlist("https://cdn.example.com/master.m3u8", "720p", 3, 4.0);
        let got: Vec<u32> = (0..5)
            .filter_map(|_| s.status.next_segment(&pl).map(|seg| seg.index))
            .collect();
        assert_eq!(got, vec![0, 1, 2, 0, 1], "sequential cursor must cycle");
        assert_eq!(s.status._cycles_completed(), 1);
    }

    #[test]
    fn media_sniff_next_segment_empty_playlist_none() {
        let mut s = MediaSniffer::new();
        let pl = _MediaPlaylist {
            segments: vec![],
            duration_total_s: 0.0,
        };
        assert!(s.status.next_segment(&pl).is_none());
        assert_eq!(s.status.progress(), 0.0);
        assert_eq!(s.status._cycles_completed(), 0);
    }

    #[test]
    fn media_sniff_progress_tracks_cycle() {
        let mut s = MediaSniffer::new();
        let pl = s._build_playlist("https://cdn.example.com/master.m3u8", "720p", 4, 4.0);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.25).abs() < 1e-9);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.5).abs() < 1e-9);
        s.status.next_segment(&pl);
        s.status.next_segment(&pl);
        assert!((s.status.progress() - 0.0).abs() < 1e-9, "wrap must reset progress");
    }

    #[test]
    fn media_sniff_selftest_name_matches() {
        use crate::l0_substrate::nt_core_self_test::SelfTest;
        let s = MediaSniffer::new();
        assert_eq!(s.name(), "nt_world_video_pipeline_media_sniff");
        assert!(s.self_test().is_ok());
    }
}
