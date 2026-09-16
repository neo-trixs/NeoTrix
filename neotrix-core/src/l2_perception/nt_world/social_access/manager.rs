use super::feed::{FeedService, UniversalRecommender};
use super::traits::{
    AuthState, FeedType, SocialAccessError, SocialAccessResult,
    SocialPlatform, SessionEntry, UnifiedPost,
};
use std::collections::HashMap;

pub struct SessionPool {
    sessions: HashMap<SocialPlatform, SessionEntry>,
}

impl SessionPool {
    pub fn new() -> Self {
        Self { sessions: HashMap::new() }
    }

    pub fn insert(&mut self, platform: SocialPlatform, session: SessionEntry) {
        self.sessions.insert(platform, session);
    }

    pub fn get(&self, platform: &SocialPlatform) -> Option<&SessionEntry> {
        self.sessions.get(platform)
    }

    pub fn stats(&self) -> HashMap<SocialPlatform, (usize, usize)> {
        self.sessions
            .iter()
            .map(|(p, s)| {
                let authenticated = if matches!(s.state, AuthState::Authenticated) { 1 } else { 0 };
                let guest = if matches!(s.state, AuthState::Guest) { 1 } else { 0 };
                (p.clone(), (authenticated, guest))
            })
            .collect()
    }
}

pub struct SocialAccessManager {
    pool: SessionPool,
    feed_service: FeedService,
    _recommender: UniversalRecommender,
}

impl SocialAccessManager {
    pub fn new() -> Self {
        Self {
            pool: SessionPool::new(),
            feed_service: FeedService::new(),
            _recommender: UniversalRecommender::new(),
        }
    }

    pub fn init_platform(
        &mut self,
        platform: SocialPlatform,
    ) -> SocialAccessResult<SessionEntry> {
        let session = SessionEntry::guest(platform.clone());
        self.pool.insert(platform, session.clone());
        Ok(session)
    }

    pub fn init_all_guest(&mut self) -> Vec<SocialAccessResult<SessionEntry>> {
        let mut results = Vec::new();
        for platform in SocialPlatform::all() {
            results.push(self.init_platform(platform.clone()));
        }
        results
    }

    pub fn get_session(&self, platform: SocialPlatform) -> SocialAccessResult<&SessionEntry> {
        self.pool
            .get(&platform)
            .ok_or_else(|| SocialAccessError::AuthFailed {
                platform,
                reason: "no session for platform".into(),
            })
    }

    pub fn get_feed(
        &self,
        platform: SocialPlatform,
        feed_type: FeedType,
        _limit: usize,
    ) -> SocialAccessResult<FeedResponse> {
        let _ = feed_type;
        let _session = self.get_session(platform)?;
        let posts: Vec<UnifiedPost> = vec![];
        Ok(FeedResponse { posts })
    }

    pub fn get_feed_all(
        &mut self,
        feed_type: FeedType,
        limit_per_platform: usize,
    ) -> SocialAccessResult<Vec<UnifiedPost>> {
        let mut all = Vec::new();
        for platform in SocialPlatform::all() {
            match self.get_feed(platform.clone(), feed_type, limit_per_platform) {
                Ok(feed) => all.extend(feed.posts),
                Err(_) => continue,
            }
        }
        all.sort_by(|a, b| {
            let sa = a.metrics.likes + a.metrics.shares + a.metrics.replies;
            let sb = b.metrics.likes + b.metrics.shares + b.metrics.replies;
            sb.cmp(&sa)
        });
        Ok(all)
    }

    pub fn stats(&self) -> HashMap<SocialPlatform, (usize, usize)> {
        self.pool.stats()
    }
}

pub struct FeedResponse {
    pub posts: Vec<UnifiedPost>,
}
