use crate::neotrix::nt_act_social::auth::SocialAuth;
use crate::neotrix::nt_act_social::connector::{
    Platform, PlatformError, SocialSearchResult, SocialMediaPlatform, UserInfo, VideoInfo,
};
use crate::neotrix::nt_act_social::rate_limit::RateLimiter;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Deserialize;

#[derive(Deserialize)]
struct TwitterError {
    _title: Option<String>,
    _detail: Option<String>,
}

#[derive(Deserialize)]
struct TwitterUserMetrics {
    followers_count: Option<u64>,
}

#[derive(Deserialize)]
struct TwitterUserData {
    id: String,
    name: String,
    username: String,
    profile_image_url: Option<String>,
    public_metrics: Option<TwitterUserMetrics>,
}

#[derive(Deserialize)]
struct TweetMetrics {
    retweet_count: Option<u64>,
    reply_count: Option<u64>,
    like_count: Option<u64>,
    _quote_count: Option<u64>,
    impression_count: Option<u64>,
}

#[derive(Deserialize)]
struct TweetAttachments {
    media_keys: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct TweetData {
    id: String,
    text: String,
    created_at: Option<String>,
    public_metrics: Option<TweetMetrics>,
    attachments: Option<TweetAttachments>,
    author_id: Option<String>,
}

#[derive(Deserialize)]
struct TwitterMediaData {
    media_key: String,
    #[serde(rename = "type")]
    _media_type: Option<String>,
    url: Option<String>,
    duration_ms: Option<u64>,
}

#[derive(Deserialize)]
struct TwitterIncludes {
    users: Option<Vec<TwitterUserData>>,
    media: Option<Vec<TwitterMediaData>>,
}

#[derive(Deserialize)]
#[derive(Clone)]
struct TwitterMeta {
    result_count: Option<u32>,
    next_token: Option<String>,
}

#[derive(Deserialize)]
struct TwitterSearchResponse {
    data: Option<Vec<TweetData>>,
    includes: Option<TwitterIncludes>,
    meta: Option<TwitterMeta>,
}

#[derive(Deserialize)]
struct TwitterUserResponse {
    data: Option<TwitterUserData>,
}

#[derive(Deserialize)]
struct TwitterVideoResponse {
    data: Option<TweetData>,
    includes: Option<TwitterIncludes>,
}

pub struct TwitterConnector {
    client: reqwest::Client,
    auth: SocialAuth,
    rate_limiter: RateLimiter,
    bearer_token: Option<String>,
}

impl TwitterConnector {
    pub fn new(bearer_token: Option<String>) -> Self {
        TwitterConnector {
            client: reqwest::Client::new(),
            auth: SocialAuth::new(),
            rate_limiter: RateLimiter::new(),
            bearer_token,
        }
    }

    fn get_token(&self) -> Result<String, PlatformError> {
        if let Some(ref token) = self.bearer_token {
            return Ok(format!("Bearer {}", token));
        }
        if let Some(tokens) = self.auth.get_token("twitter") {
            return Ok(format!("Bearer {}", tokens.access_token));
        }
        Err(PlatformError::NotAuthenticated)
    }

    fn parse_tweet_time(s: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn build_video_info(
        tweet: &TweetData,
        includes: Option<&TwitterIncludes>,
        _username: Option<&str>,
    ) -> VideoInfo {
        let author = includes
            .and_then(|inc| inc.users.as_ref())
            .and_then(|users| {
                tweet.author_id.as_ref().and_then(|aid| {
                    users
                        .iter()
                        .find(|u| u.id == *aid)
                        .map(|u| u.username.clone())
                })
            });

        let (thumbnail_url, duration_secs) = includes
            .and_then(|inc| inc.media.as_ref())
            .and_then(|media| {
                tweet.attachments.as_ref().and_then(|att| {
                    att.media_keys.as_ref().and_then(|keys| {
                        for key in keys {
                            if let Some(m) = media.iter().find(|m| m.media_key == *key) {
                                let dur = m.duration_ms.map(|ms| ms / 1000);
                                return Some((m.url.clone(), dur));
                            }
                        }
                        None
                    })
                })
            })
            .unwrap_or((None, None));

        VideoInfo {
            id: tweet.id.clone(),
            title: tweet.text.chars().take(100).collect(),
            description: tweet.text.clone(),
            platform: Platform::Twitter,
            url: format!("https://twitter.com/i/web/status/{}", tweet.id),
            thumbnail_url,
            duration_secs,
            view_count: tweet
                .public_metrics
                .as_ref()
                .and_then(|m| m.impression_count),
            like_count: tweet.public_metrics.as_ref().and_then(|m| m.like_count),
            comment_count: tweet
                .public_metrics
                .as_ref()
                .and_then(|m| m.reply_count),
            author,
            published_at: tweet
                .created_at
                .as_deref()
                .and_then(TwitterConnector::parse_tweet_time),
        }
    }

    async fn call_api<T: for<'de> Deserialize<'de>>(
        &self,
        url: Url,
    ) -> Result<T, PlatformError> {
        let wait = self.rate_limiter.wait_if_needed("twitter");
        if wait > std::time::Duration::ZERO {
            tokio::time::sleep(wait).await;
        }

        let token = self.get_token()?;

        let response = self
            .client
            .get(url)
            .header("Authorization", &token)
            .send()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        self.rate_limiter.record_call("twitter");

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| PlatformError::Network(e.to_string()))?;

        if !status.is_success() {
            return if status.as_u16() == 429 {
                Err(PlatformError::RateLimited { retry_after: None })
            } else {
                Err(PlatformError::ApiError {
                    status: status.as_u16(),
                    message: body,
                })
            };
        }

        serde_json::from_str(&body)
            .map_err(|e| PlatformError::ParseError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl SocialMediaPlatform for TwitterConnector {
    fn platform(&self) -> Platform {
        Platform::Twitter
    }

    fn is_authenticated(&self) -> bool {
        self.bearer_token.is_some() || self.auth.is_token_valid("twitter")
    }

    async fn login(&mut self) -> Result<(), PlatformError> {
        if self.is_authenticated() {
            return Ok(());
        }
        Err(PlatformError::NotAuthenticated)
    }

    async fn search(&self, query: &str, max_results: u32) -> Result<SocialSearchResult, PlatformError> {
        let clamped = max_results.min(100);
        let url = Url::parse_with_params(
            "https://api.twitter.com/2/tweets/search/recent",
            &[
                ("query", query),
                ("max_results", &clamped.to_string()),
                (
                    "tweet.fields",
                    "created_at,public_metrics,attachments,author_id",
                ),
                ("media.fields", "url,duration_ms"),
                ("expansions", "attachments.media_keys,author_id"),
            ],
        )
        .map_err(|e| PlatformError::Network(e.to_string()))?;

        let resp: TwitterSearchResponse = self.call_api(url).await?;
        let includes = resp.includes.as_ref();
        let videos = resp
            .data
            .unwrap_or_default()
            .iter()
            .map(|t| TwitterConnector::build_video_info(t, includes, None))
            .collect();

        let next_token = resp.meta.clone().and_then(|m| m.next_token);
        let total_estimated = resp.meta.and_then(|m| m.result_count).map(u64::from);
        Ok(SocialSearchResult {
            videos,
            next_page_token: next_token,
            total_estimated,
        })
    }

    async fn get_video(&self, video_id: &str) -> Result<VideoInfo, PlatformError> {
        let url = Url::parse_with_params(
            &format!("https://api.twitter.com/2/tweets/{}", video_id),
            &[
                (
                    "tweet.fields",
                    "created_at,public_metrics,attachments,author_id",
                ),
                ("media.fields", "url,duration_ms"),
                ("expansions", "attachments.media_keys,author_id"),
            ],
        )
        .map_err(|e| PlatformError::Network(e.to_string()))?;

        let resp: TwitterVideoResponse = self.call_api(url).await?;
        let tweet = resp.data.ok_or_else(|| PlatformError::ApiError {
            status: 404,
            message: format!("tweet {} not found", video_id),
        })?;

        let includes = resp.includes.as_ref();
        Ok(TwitterConnector::build_video_info(&tweet, includes, None))
    }

    async fn trending(&self, max_results: u32) -> Result<Vec<VideoInfo>, PlatformError> {
        let result = self.search("video", max_results).await?;
        Ok(result.videos)
    }

    async fn get_me(&self) -> Result<UserInfo, PlatformError> {
        let url = Url::parse_with_params(
            "https://api.twitter.com/2/users/me",
            &[("user.fields", "id,name,username,profile_image_url,public_metrics")],
        )
        .map_err(|e| PlatformError::Network(e.to_string()))?;

        let resp: TwitterUserResponse = self.call_api(url).await?;
        let data = resp.data.ok_or_else(|| PlatformError::ApiError {
            status: 401,
            message: "unable to fetch authenticated user".into(),
        })?;

        Ok(UserInfo {
            platform_user_id: data.id,
            username: data.username,
            display_name: Some(data.name),
            avatar_url: data.profile_image_url,
            follower_count: data.public_metrics.and_then(|m| m.followers_count),
            platform: Platform::Twitter,
        })
    }

    async fn get_home_timeline(&self, max_results: u32) -> Result<Vec<crate::neotrix::nt_act_social::connector::TimelineTweet>, PlatformError> {
        let clamped = max_results.min(100);
        let url = Url::parse_with_params(
            "https://api.twitter.com/2/tweets/search/recent",
            &[
                ("query", "from:me OR from:following"), // requires following context; fallback
                ("max_results", &clamped.to_string()),
                ("tweet.fields", "created_at,public_metrics,author_id"),
                ("expansions", "author_id"),
                ("user.fields", "username"),
            ],
        ).map_err(|e| PlatformError::Network(e.to_string()))?;

        #[derive(Deserialize)]
        struct TimelineResponse {
            data: Option<Vec<TweetData>>,
            includes: Option<TwitterIncludes>,
        }

        let resp: TimelineResponse = self.call_api(url).await?;
        let users = resp.includes.as_ref().and_then(|i| i.users.as_ref());

        let tweets = resp.data.unwrap_or_default().iter().map(|t| {
            let username = users.and_then(|u| {
                t.author_id.as_ref().and_then(|aid| {
                    u.iter().find(|u2| u2.id == *aid).map(|u2| u2.username.clone())
                })
            });
            crate::neotrix::nt_act_social::connector::TimelineTweet {
                id: t.id.clone(),
                text: t.text.clone(),
                created_at: t.created_at.clone(),
                author_id: t.author_id.clone(),
                author_username: username,
                like_count: t.public_metrics.as_ref().and_then(|m| m.like_count),
                retweet_count: t.public_metrics.as_ref().and_then(|m| m.retweet_count),
                reply_count: t.public_metrics.as_ref().and_then(|m| m.reply_count),
                is_quote: false,
            }
        }).collect();

        Ok(tweets)
    }

    async fn get_user_tweets(&self, user_id: &str, max_results: u32) -> Result<Vec<crate::neotrix::nt_act_social::connector::TimelineTweet>, PlatformError> {
        let clamped = max_results.min(100);
        let fields = "created_at,public_metrics,author_id".to_string();
        let url = Url::parse_with_params(
            &format!("https://api.twitter.com/2/users/{}/tweets", user_id),
            &[
                ("max_results", &clamped.to_string()),
                ("tweet.fields", &fields),
            ],
        ).map_err(|e| PlatformError::Network(e.to_string()))?;

        #[derive(Deserialize)]
        struct UserTweetsResponse {
            data: Option<Vec<TweetData>>,
        }

        let resp: UserTweetsResponse = self.call_api(url).await?;

        let tweets = resp.data.unwrap_or_default().iter().map(|t| {
            crate::neotrix::nt_act_social::connector::TimelineTweet {
                id: t.id.clone(),
                text: t.text.clone(),
                created_at: t.created_at.clone(),
                author_id: t.author_id.clone(),
                author_username: None,
                like_count: t.public_metrics.as_ref().and_then(|m| m.like_count),
                retweet_count: t.public_metrics.as_ref().and_then(|m| m.retweet_count),
                reply_count: t.public_metrics.as_ref().and_then(|m| m.reply_count),
                is_quote: false,
            }
        }).collect();

        Ok(tweets)
    }

    async fn upload_video(
        &self,
        _title: &str,
        _description: &str,
        _file_path: &str,
    ) -> Result<VideoInfo, PlatformError> {
        Err(PlatformError::UploadFailed(
            "Twitter upload requires media/upload endpoint (chunked)".into(),
        ))
    }
}



