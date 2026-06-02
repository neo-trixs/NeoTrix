use async_trait::async_trait;
use chrono::DateTime;
use serde::Deserialize;

use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;

const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Deserialize)]
struct BiliSearchItem {
    aid: i64,
    title: String,
    #[serde(default)]
    desc: Option<String>,
    #[serde(default)]
    pic: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    duration: Option<u64>,
    #[serde(default)]
    play: Option<u64>,
    #[serde(default)]
    video_review: Option<u64>,
    #[serde(default)]
    pubdate: Option<i64>,
}

#[derive(Deserialize)]
struct BiliSearchData {
    #[serde(default)]
    result: Option<Vec<BiliSearchItem>>,
}

#[derive(Deserialize)]
struct BiliSearchResponse {
    code: i64,
    #[serde(default)]
    data: Option<BiliSearchData>,
}

#[derive(Deserialize)]
struct BiliOwner {
    name: String,
    #[allow(dead_code)]
    face: String,
}

#[derive(Deserialize)]
struct BiliStat {
    view: i64,
    like: i64,
    danmaku: i64,
}

#[derive(Deserialize)]
struct BiliVideoData {
    aid: i64,
    title: String,
    desc: String,
    pic: String,
    owner: BiliOwner,
    stat: BiliStat,
    pubdate: i64,
    duration: u64,
}

#[derive(Deserialize)]
struct BiliVideoResponse {
    code: i64,
    #[serde(default)]
    data: Option<BiliVideoData>,
}

#[derive(Deserialize)]
struct BiliPopularItem {
    aid: i64,
    title: String,
    desc: String,
    pic: String,
    owner: BiliOwner,
    stat: BiliStat,
    pubdate: i64,
    duration: u64,
}

#[derive(Deserialize)]
struct BiliPopularData {
    list: Vec<BiliPopularItem>,
}

#[derive(Deserialize)]
struct BiliPopularResponse {
    code: i64,
    #[serde(default)]
    data: Option<BiliPopularData>,
}

#[derive(Deserialize)]
struct BiliNavData {
    mid: i64,
    uname: String,
    face: String,
}

#[derive(Deserialize)]
struct BiliNavResponse {
    code: i64,
    #[serde(default)]
    data: Option<BiliNavData>,
}

pub struct BilibiliConnector {
    client: reqwest::Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    cookies: Option<String>,
}

impl Default for BilibiliConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl BilibiliConnector {
    pub fn new() -> Self {
        BilibiliConnector {
            client: reqwest::Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            cookies: None,
        }
    }

    fn req(&self, url: &str) -> reqwest::RequestBuilder {
        let r = self
            .client
            .get(url)
            .header("Referer", "https://www.bilibili.com")
            .header("User-Agent", UA);
        if let Some(c) = &self.cookies {
            r.header("Cookie", format!("SESSDATA={}", c))
        } else {
            r
        }
    }

    async fn fetch(&self, builder: reqwest::RequestBuilder) -> Result<Vec<u8>, PlatformError> {
        let wait = self.rate_limiter.wait_if_needed("bilibili");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }
        self.rate_limiter.record_call("bilibili");
        let resp = builder
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(PlatformError::ApiError {
                status: status.as_u16(),
                message: String::from_utf8_lossy(&body).to_string(),
            });
        }
        Ok(body.to_vec())
    }
}

#[async_trait]
impl SocialMediaPlatform for BilibiliConnector {
    fn platform(&self) -> Platform {
        Platform::Bilibili
    }

    fn is_authenticated(&self) -> bool {
        self.cookies.is_some()
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        if let Some(t) = self.auth.get_token("bilibili") {
            self.cookies = Some(t.access_token);
            Ok(())
        } else {
            Err(PlatformError::NotAuthenticated)
        }
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SearchResult, PlatformError> {
        let ps = max_results.to_string();
        let builder = self
            .req("https://api.bilibili.com/x/web-interface/search/type")
            .query(&[
                ("search_type", "video"),
                ("keyword", query),
                ("page", "1"),
                ("page_size", ps.as_str()),
            ]);
        let body = self.fetch(builder).await?;
        let resp: BiliSearchResponse = serde_json::from_slice(&body)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("bilibili search error code: {}", resp.code),
            });
        }
        let items = resp.data.and_then(|d| d.result).unwrap_or_default();
        let videos = items
            .into_iter()
            .map(|item| VideoInfo {
                id: item.aid.to_string(),
                title: item.title,
                description: item.desc.unwrap_or_default(),
                platform: Platform::Bilibili,
                url: format!("https://www.bilibili.com/video/av{}", item.aid),
                thumbnail_url: item.pic,
                duration_secs: item.duration,
                view_count: item.play,
                like_count: None,
                comment_count: item.video_review,
                author: item.author,
                published_at: item.pubdate.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            })
            .collect();
        Ok(SearchResult {
            videos,
            next_page_token: None,
            total_estimated: None,
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let (pk, pv) = if video_id.starts_with("BV") || video_id.starts_with("bv") {
            ("bvid", video_id)
        } else {
            ("aid", video_id)
        };
        let builder = self
            .req("https://api.bilibili.com/x/web-interface/view")
            .query(&[(pk, pv)]);
        let body = self.fetch(builder).await?;
        let resp: BiliVideoResponse = serde_json::from_slice(&body)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("bilibili view error code: {}", resp.code),
            });
        }
        let data =
            resp.data
                .ok_or_else(|| PlatformError::ParseError("empty video data".to_string()))?;
        let url = if video_id.starts_with("BV") || video_id.starts_with("bv") {
            format!("https://www.bilibili.com/video/{}", video_id)
        } else {
            format!("https://www.bilibili.com/video/av{}", video_id)
        };
        Ok(VideoInfo {
            id: data.aid.to_string(),
            title: data.title,
            description: data.desc,
            platform: Platform::Bilibili,
            url,
            thumbnail_url: Some(data.pic),
            duration_secs: Some(data.duration),
            view_count: Some(data.stat.view as u64),
            like_count: Some(data.stat.like as u64),
            comment_count: Some(data.stat.danmaku as u64),
            author: Some(data.owner.name),
            published_at: DateTime::from_timestamp(data.pubdate, 0),
        })
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        let ps = max_results.to_string();
        let builder = self
            .req("https://api.bilibili.com/x/web-interface/popular")
            .query(&[("page", "1"), ("page_size", ps.as_str())]);
        let body = self.fetch(builder).await?;
        let resp: BiliPopularResponse = serde_json::from_slice(&body)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("bilibili popular error code: {}", resp.code),
            });
        }
        let items = resp.data.map(|d| d.list).unwrap_or_default();
        let videos = items
            .into_iter()
            .map(|item| VideoInfo {
                id: item.aid.to_string(),
                title: item.title,
                description: item.desc,
                platform: Platform::Bilibili,
                url: format!("https://www.bilibili.com/video/av{}", item.aid),
                thumbnail_url: Some(item.pic),
                duration_secs: Some(item.duration),
                view_count: Some(item.stat.view as u64),
                like_count: Some(item.stat.like as u64),
                comment_count: Some(item.stat.danmaku as u64),
                author: Some(item.owner.name),
                published_at: DateTime::from_timestamp(item.pubdate, 0),
            })
            .collect();
        Ok(videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        if !self.is_authenticated() {
            return Err(PlatformError::NotAuthenticated);
        }
        let body = self
            .fetch(self.req("https://api.bilibili.com/x/web-interface/nav"))
            .await?;
        let resp: BiliNavResponse = serde_json::from_slice(&body)
            .map_err(|e| PlatformError::ParseError(e.to_string()))?;
        if resp.code != 0 {
            return Err(PlatformError::ApiError {
                status: 200,
                message: format!("bilibili nav error code: {}", resp.code),
            });
        }
        let d = resp
            .data
            .ok_or_else(|| PlatformError::ParseError("empty nav data".to_string()))?;
        Ok(UserInfo {
            platform_user_id: d.mid.to_string(),
            username: d.uname.clone(),
            display_name: Some(d.uname),
            avatar_url: Some(d.face),
            follower_count: None,
            platform: Platform::Bilibili,
        })
    }

    async fn upload_video(
        &self,
        _title: &str,
        _description: &str,
        _file_path: &str,
    ) -> Result<VideoInfo, PlatformError> {
        Err(PlatformError::UploadFailed(
            "bilibili upload requires complex form submission".to_string(),
        ))
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
