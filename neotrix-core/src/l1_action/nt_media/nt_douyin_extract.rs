//! 抖音视频页解析 — 统一媒体管道的抖音来源。
//!
//! 约束：抖音页/接口有反爬签名，HTTP 直抓通常只拿到 JSVM 壳；
//! 因此本模块只做纯解析（分享链规范化、视频 ID 提取、已落盘 HTML 解析），
//! 页面抓取由登录态浏览器采集器完成，下载走统一 [`download_to_file`]。
//!
//! 解析优先级：`__UNIVERSAL_DATA_FOR_REHYDRATION__` 内嵌 JSON
//! → `RENDER_DATA` → `og:video` meta（仅直链，无描述）。

use regex::Regex;
use serde_json::Value;

/// 抖音视频解析结果（对齐 [`crate::l1_action::nt_media::yt_extract::VideoInfo`] 字段口径）。
pub struct DouyinVideo {
    pub id: String,
    pub desc: String,
    pub nickname: String,
    pub sec_uid: String,
    pub play_url: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum DouyinError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("extraction failed: {0}")]
    Extraction(String),
    #[error("parse error: {0}")]
    Parse(String),
}

impl From<DouyinError> for neotrix_types::NtError {
    fn from(err: DouyinError) -> Self {
        match err {
            DouyinError::Network(e) => neotrix_types::NtError::Network(e.to_string()),
            DouyinError::Extraction(msg) => neotrix_types::NtError::OperationFailed(msg),
            DouyinError::Parse(msg) => neotrix_types::NtError::Serde(msg),
        }
    }
}

/// 分享链/视频页规范化为 `https://www.douyin.com/video/{id}`（纯函数）。
pub fn canonical_video_url(url: &str) -> Option<String> {
    extract_douyin_video_id(url)
        .map(|id| format!("https://www.douyin.com/video/{id}"))
}

/// 从任意抖音 URL 中提取纯数字视频 ID（`/video/{id}`）。
pub fn extract_douyin_video_id(url: &str) -> Option<String> {
    let marker = "/video/";
    let pos = url.find(marker)?;
    let rest = url.get(pos + marker.len()..)?;
    let id: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if id.is_empty() {
        return None;
    }
    Some(id)
}

fn extract_script_json<'a>(html: &'a str, id: &str) -> Option<&'a str> {
    let open = format!("<script id=\"{id}\"");
    let start = html.find(&open)?;
    let tag_end = html[start..].find('>')? + start + 1;
    let end = html[tag_end..].find("</script>")? + tag_end;
    Some(html[tag_end..end].trim())
}

fn extract_meta(html: &str, property: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"<meta[^>]+(?:property|name)=["']{}["'][^>]+content=["']([^"']+)["']"#,
        regex::escape(property)
    ))
    .ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// 递归寻找首个含 `play_addr.url_list` 的 aweme detail 对象。
fn find_detail(v: &Value) -> Option<&Value> {
    match v {
        Value::Object(map) => {
            if map
                .get("play_addr")
                .and_then(|p| p.get("url_list"))
                .and_then(|u| u.as_array())
                .is_some_and(|a| !a.is_empty())
            {
                return Some(v);
            }
            map.values().find_map(find_detail)
        }
        Value::Array(arr) => arr.iter().find_map(find_detail),
        _ => None,
    }
}

fn first_url(list: Option<&Value>) -> Option<String> {
    list.and_then(|u| u.as_array())
        .and_then(|a| a.iter().find_map(|x| x.as_str()))
        .map(String::from)
        .filter(|s| !s.is_empty())
}

fn detail_to_video(detail: &Value, fallback_id: &str) -> Option<DouyinVideo> {
    let play_url = first_url(
        detail.get("play_addr").and_then(|p| p.get("url_list")),
    )?;
    let author = detail.get("author");
    Some(DouyinVideo {
        id: detail
            .get("aweme_id")
            .and_then(|v| v.as_str())
            .unwrap_or(fallback_id)
            .to_string(),
        desc: detail
            .get("desc")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        nickname: author
            .and_then(|a| a.get("nickname"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        sec_uid: author
            .and_then(|a| a.get("sec_uid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        play_url,
        duration_ms: detail
            .get("video")
            .and_then(|v| v.get("duration"))
            .and_then(|v| v.as_u64()),
    })
}

/// 解析 aweme/detail API 响应 JSON → [`DouyinVideo`]（纯函数，可单测）。
///
/// `raw` 为 `{"aweme_detail": {...}}` 或裸 detail 对象。
pub fn parse_douyin_detail(raw: &str, fallback_id: &str) -> Result<DouyinVideo, DouyinError> {
    let v: Value = serde_json::from_str(raw).map_err(|e| DouyinError::Parse(e.to_string()))?;
    let detail = v.get("aweme_detail").unwrap_or(&v);
    if let Some(found) = find_detail(detail) {
        if let Some(video) = detail_to_video(found, fallback_id) {
            return Ok(video);
        }
    }
    detail_to_video(detail, fallback_id).ok_or_else(|| {
        DouyinError::Extraction("no play_addr.url_list in aweme detail".into())
    })
}

/// 解析已落盘的抖音视频页 HTML → [`DouyinVideo`]（纯函数，可单测）。
pub fn parse_douyin_page(html: &str, page_url: &str) -> Result<DouyinVideo, DouyinError> {
    let fallback_id = extract_douyin_video_id(page_url).unwrap_or_default();
    for script_id in ["__UNIVERSAL_DATA_FOR_REHYDRATION__", "RENDER_DATA"] {
        if let Some(raw) = extract_script_json(html, script_id) {
            let v: Value =
                serde_json::from_str(raw).map_err(|e| DouyinError::Parse(e.to_string()))?;
            if let Some(detail) = find_detail(&v) {
                if let Some(video) = detail_to_video(detail, &fallback_id) {
                    return Ok(video);
                }
            }
        }
    }
    if let Some(url) = extract_meta(html, "og:video") {
        return Ok(DouyinVideo {
            id: fallback_id,
            desc: extract_meta(html, "og:title").unwrap_or_default(),
            nickname: String::new(),
            sec_uid: String::new(),
            play_url: url,
            duration_ms: None,
        });
    }
    Err(DouyinError::Extraction(
        "no embedded aweme detail or og:video in douyin page".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_id() {
        assert_eq!(
            extract_douyin_video_id("https://www.douyin.com/video/7689231244575641958/"),
            Some("7689231244575641958".into())
        );
        assert_eq!(
            extract_douyin_video_id("https://www.douyin.com/video/123?modeFrom=search"),
            Some("123".into())
        );
        assert_eq!(extract_douyin_video_id("https://www.douyin.com/user/123"), None);
        assert_eq!(
            canonical_video_url("https://v.douyin.com/kLu0f4pFzlM/"),
            None
        );
    }

    const FIXTURE: &str = r#"<html><head>
<meta property="og:title" content="fallback desc"/>
<script id="__UNIVERSAL_DATA_FOR_REHYDRATION__" type="application/json">{"defaultData":{"detail":{"aweme_id":"7689231244575641958","desc":"确定性有层次","author":{"nickname":"蜗牛有点田","sec_uid":"SEC_abc"},"play_addr":{"url_list":["https://v3-web.douyinvod.com/video.mp4","https://v9-web.douyinvod.com/video.mp4"]},"video":{"duration":12345}}}}</script>
</head></html>"#;

    #[test]
    fn test_parse_embedded() {
        let v = parse_douyin_page(FIXTURE, "https://www.douyin.com/video/7689231244575641958/")
            .expect("fixture must parse");
        assert_eq!(v.id, "7689231244575641958");
        assert_eq!(v.desc, "确定性有层次");
        assert_eq!(v.nickname, "蜗牛有点田");
        assert_eq!(v.sec_uid, "SEC_abc");
        assert_eq!(v.play_url, "https://v3-web.douyinvod.com/video.mp4");
        assert_eq!(v.duration_ms, Some(12345));
    }

    #[test]
    fn test_parse_og_fallback() {
        let html = r#"<html><head><meta property="og:video" content="https://v3/x.mp4"/><meta property="og:title" content="t"/></head></html>"#;
        let v = parse_douyin_page(html, "https://www.douyin.com/video/99/").expect("og fallback");
        assert_eq!(v.play_url, "https://v3/x.mp4");
        assert_eq!(v.id, "99");
    }

    #[test]
    fn test_parse_empty_errors() {
        assert!(parse_douyin_page("<html></html>", "https://www.douyin.com/video/1/").is_err());
    }

    #[test]
    fn test_parse_detail_json() {
        let raw = r#"{"aweme_detail":{"aweme_id":"7689231244575641958","desc":"确定性有层次","author":{"nickname":"蜗牛有点田","sec_uid":"SEC_abc"},"play_addr":{"url_list":["https://v3-web.douyinvod.com/video.mp4"]},"video":{"duration":12345}}}"#;
        let v = parse_douyin_detail(raw, "7689231244575641958").expect("detail must parse");
        assert_eq!(v.play_url, "https://v3-web.douyinvod.com/video.mp4");
        assert_eq!(v.nickname, "蜗牛有点田");
        assert!(parse_douyin_detail(r#"{"aweme_detail":{}}"#, "1").is_err());
    }
}
