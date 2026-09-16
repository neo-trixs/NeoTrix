use super::super::{
    AuthState, HttpPool, SessionEntry, SocialAccessError, SocialAccessResult, SocialPlatform,
};
use super::super::traits::{SocialPlatformAdapter, FeedResult, TrendingTopic, AuthFlow};

pub struct RedditExtractor;

impl RedditExtractor {
    pub fn new() -> Self {
        Self
    }

    pub async fn authenticate_client_credentials(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> SocialAccessResult<SessionEntry> {
        use base64::Engine;
        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", client_id, client_secret));
        let r = HttpPool::standard()
            .post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {}", credentials))
            .form(&[("grant_type", "client_credentials")])
            .send().await
.map_err(|e| SocialAccessError::Network(e.to_string()))?;
        let j: serde_json::Value = r
            .json().await
.map_err(|e| SocialAccessError::Parse(e.to_string()))?;
        let token = j["access_token"]
            .as_str()
            .ok_or_else(|| SocialAccessError::AuthFailed {
                platform: SocialPlatform::Reddit,
                reason: "No access_token".into(),
            })?;
        let mut session = SessionEntry::guest(SocialPlatform::Reddit);
        session.credentials.access_token = Some(token.to_string());
        session.state = AuthState::Authenticated;
        Ok(session)
    }
}

impl SocialPlatformAdapter for RedditExtractor {
    fn id(&self) -> String { "reddit".into() }
    fn name(&self) -> &'static str { "Reddit" }
    fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
    fn api_base_url(&self) -> &'static str { "https://reddit.com" }
    fn channel_name(&self) -> Option<&'static str> { Some("reddit") }

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
