use super::super::{
    build_headers, Author, EngagementMetrics, HttpPool, SessionEntry, SocialAccessError,
    SocialAccessResult, SocialPlatform, UnifiedPost,
};
use super::super::traits::{SocialPlatformAdapter, FeedResult, TrendingTopic, AuthFlow};

pub struct InstagramExtractor;

impl InstagramExtractor {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_user_posts(
        &self,
        session: &SessionEntry,
        username: &str,
        limit: usize,
    ) -> SocialAccessResult<Vec<UnifiedPost>> {
        let j: serde_json::Value = HttpPool::standard()
            .get(&format!(
                "https://i.instagram.com/api/v1/users/web_profile_info/?username={}",
                username
            ))
            .headers(build_headers(session))
            .send()
            .await
            .map_err(|e| SocialAccessError::Network(e.to_string()))?
            .json()
            .await
            .map_err(|e| SocialAccessError::Parse(e.to_string()))?;

        let u = &j["data"]["user"];
        let edges = u["edge_owner_to_timeline_media"]["edges"]
            .as_array()
            .ok_or_else(|| SocialAccessError::Parse("No edges".into()))?;

        let posts = edges
            .iter()
            .take(limit)
            .filter_map(|edge| {
                let n = &edge["node"];
                let author_u = u;
                Some(UnifiedPost {
                    id: n["id"].as_str()?.to_string(),
                    platform: SocialPlatform::Instagram,
                    author: Author {
                        username: author_u["username"].as_str().unwrap_or("").into(),
                        display_name: author_u["full_name"].as_str().unwrap_or("").into(),
                        user_id: author_u["id"].as_str().unwrap_or("").into(),
                        avatar_url: author_u["profile_pic_url"].as_str().map(|s| s.into()),
                        followers: author_u["edge_followed_by"]["count"].as_u64(),
                        verified: author_u["is_verified"].as_bool().unwrap_or(false),
                    },
                    content: n["edge_media_to_caption"]["edges"]
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|e| e["node"]["text"].as_str())
                        .unwrap_or("")
                        .into(),
                    title: None,
                    media: vec![],
                    metrics: EngagementMetrics {
                        likes: n["edge_liked_by"]["count"].as_u64().unwrap_or(0),
                        replies: n["edge_media_to_comment"]["count"].as_u64().unwrap_or(0),
                        views: n["video_view_count"].as_u64(),
                        ..Default::default()
                    },
                    created_at: std::time::SystemTime::now(),
                    url: format!(
                        "https://instagram.com/p/{}",
                        n["shortcode"].as_str().unwrap_or("")
                    ),
                    parent_id: None,
                    platform_meta: n.clone(),
                })
            })
            .collect();

        Ok(posts)
    }
}

impl SocialPlatformAdapter for InstagramExtractor {
    fn id(&self) -> String { "instagram".into() }
    fn name(&self) -> &'static str { "Instagram" }
    fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
    fn api_base_url(&self) -> &'static str { "https://instagram.com" }
    fn channel_name(&self) -> Option<&'static str> { Some("instagram") }

    fn get_recommended(&self, _session: &SessionEntry, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        Ok(FeedResult { items: vec![], total: 0 })
    }
    fn get_following(&self, _session: &SessionEntry, _limit: usize) -> Result<FeedResult, SocialAccessError> {
        Ok(FeedResult { items: vec![], total: 0 })
    }
    fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        Ok(vec![])
    }
}
