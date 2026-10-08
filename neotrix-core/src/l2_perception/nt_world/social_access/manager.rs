use super::feed::{FeedService, UniversalRecommender};
use super::traits::{
    Author, AuthState, FeedItem, FeedType, SocialAccessError, SocialAccessResult,
    SocialPlatform, SocialPlatformAdapter, SessionEntry, UnifiedPost,
};
use std::collections::HashMap;

pub struct SessionPool {
    sessions: HashMap<SocialPlatform, SessionEntry>,
}

impl Default for SessionPool {
    fn default() -> Self {
        Self::new()
    }
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
    recommender: UniversalRecommender,
    /// 上一次 `get_feed` 是否真的发生了排序（见 `last_fetch_was_ranked`）。
    last_ranked: bool,
}

impl Default for SocialAccessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialAccessManager {
    pub fn new() -> Self {
        Self {
            pool: SessionPool::new(),
            feed_service: FeedService::new(),
            recommender: UniversalRecommender::new(),
            last_ranked: false,
        }
    }

    /// 注册一个平台适配器。
    ///
    /// 2026-10-03 新增：`get_feed` 原先**无视** `feed_service` 直接
    /// 返回 `vec![]`，因此 `register_adapter` 注册的东西永远不会被读到 ——
    /// 注册路径与读取路径之间是断的。
    pub fn register_adapter(&mut self, adapter: std::sync::Arc<dyn SocialPlatformAdapter>) {
        self.feed_service.register_adapter(adapter);
    }

    /// 本次抓取是否真的发生了排序。
    ///
    /// 排序分数只在「适配器提供了预测」时才有意义（见
    /// [`RankOutcome`]）。本字段让调用方能区分「排过」与「顺序是任意的」，
    /// 而不是从一个恒 0 的分数里读出虚假结论。
    pub fn last_fetch_was_ranked(&self) -> bool {
        self.last_ranked
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

    /// 取某个平台的 feed。
    ///
    /// # ⛔ 2026-10-03：原实现是**桩**（无视 `feed_service` 直接 `vec![]`）
    ///
    /// 原签名还是 `&self`，而 `FeedService::get_recommended(&self, ...)`
    /// 恰好也是 `&self` —— 也就是说**它本来就能委托**，桩纯属未完成。
    ///
    /// ⚠️ `feed_type` 在原实现里被 `let _ = feed_type;` 丢弃。现按变体分派：
    /// `Following` 走 adapter 的 `get_following`（时间序，不参与排序），
    /// `Latest`/`Trending` 走 `get_ranked`。
    /// ⛔ 局限（据实记录，非静默降级）：`SocialPlatformAdapter` 没有独立的
    /// trending-推荐实现，故 `Trending` 目前复用推荐流。真正的 trending
    /// 需要 adapter 补 `get_trending` → FeedItem 的转换。
    pub fn get_feed(
        &mut self,
        platform: SocialPlatform,
        feed_type: FeedType,
        limit: usize,
    ) -> SocialAccessResult<FeedResponse> {
        let session = self.get_session(platform.clone())?.clone();
        let outcome = match feed_type {
            // ⛔ FeedType 只有 Latest/Trending/Following 三个变体（已核实）。
            //    Latest 与 Trending 都走 adapter 的 recommended 通道 ——
            //    `SocialPlatformAdapter` 没有独立的 trending 推荐实现。
            FeedType::Latest | FeedType::Trending => {
                self.feed_service.get_ranked(platform.clone(), &session, limit)?
            }
            FeedType::Following => {
                // Following 不走排序（它本身就是时间序），故记为「未排序」
                let r = self.feed_service.get_following(platform.clone(), &session, limit)?;
                let posts = r.items;
                self.last_ranked = false;
                return Ok(FeedResponse {
                    posts: posts
                        .into_iter()
                        .map(|i| Self::to_unified(&i, platform.clone(), false))
                        .collect(),
                });
            }
        };
        self.last_ranked = outcome.was_ranked();
        let posts = outcome
            .into_posts()
            .into_iter()
            .map(|i| Self::to_unified(&i, platform.clone(), self.last_ranked))
            .collect();
        Ok(FeedResponse { posts })
    }

    /// `FeedItem` → `UnifiedPost`。
    ///
    /// ⛔ `created_at` 取 Unix epoch(0) 表示「未知」——
    /// [`FeedItem`] 结构里没有时间字段（已核实），填 `SystemTime::now()`
    /// 会伪造出「刚刚发布」这个**错误的**语义。
    ///
    /// ⚠️ `ranked` 必须由调用方传入：`to_unified` 是关联函数（非方法），
    /// 读不到 `self.last_ranked`。传错会让 `platform_meta.ranked` 说谎。
    fn to_unified(
        item: &FeedItem,
        platform: SocialPlatform,
        ranked: bool,
    ) -> UnifiedPost {
        UnifiedPost {
            id: item.id.clone(),
            platform,
            author: Author {
                username: item.author.clone(),
                display_name: item.author.clone(),
                user_id: String::new(),
                avatar_url: None,
                followers: None,
                verified: false,
            },
            content: item.content.clone(),
            title: None,
            media: vec![],
            metrics: item.metrics.clone(),
            created_at: std::time::SystemTime::UNIX_EPOCH,
            url: String::new(),
            parent_id: None,
            platform_meta: serde_json::json!({
                "score": item.score,
                "ranked": ranked,
            }),
        }
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

#[derive(Debug)]
pub struct FeedResponse {
    pub posts: Vec<UnifiedPost>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_perception::nt_world::social_access::feed::PredictedAction;
    use crate::l2_perception::nt_world::social_access::traits::*;

    /// 假 adapter：返回带**可控预测**的帖子，用于验证排序真的发生。
    struct StubAdapter {
        /// 每条帖文的 like 预测概率；空 ⇒ 不提供任何预测。
        predictions: Vec<f64>,
    }

    impl SocialPlatformAdapter for StubAdapter {
        fn id(&self) -> String { "twitter".into() }
        fn name(&self) -> &'static str { "Stub" }
        fn auth_flow(&self) -> AuthFlow { AuthFlow::OAuth2PKCE }
        fn api_base_url(&self) -> &'static str { "https://x.com" }

        fn get_recommended(
            &self,
            _session: &SessionEntry,
            limit: usize,
        ) -> Result<FeedResult, SocialAccessError> {
            let predictions = self.predictions.clone();
            let items: Vec<FeedItem> = predictions
                .iter()
                .take(limit)
                .enumerate()
                .map(|(i, p)| {
                    let mut item = FeedItem::with_id(
                        &format!("{}", i),
                        "content",
                        if i == 0 { "alice" } else { "bob" },
                    );
                    if *p > 0.0 {
                        // 只在有预测时才填 —— 供「是否排序」的判定使用。
                        // 用 `with`（会 move self）不行，改用 set + Prediction。
                        if let Some(pred) = crate::l2_perception::nt_world::social_access::feed::Prediction::new(*p) {
                            item.predicted.set(PredictedAction::ShareViaCopyLink, pred);
                        }
                    }
                    item
                })
                .collect();
            let total = items.len();
            Ok(FeedResult { items, total })
        }

        fn get_following(
            &self,
            _session: &SessionEntry,
            limit: usize,
        ) -> Result<FeedResult, SocialAccessError> {
            let items: Vec<FeedItem> = (0..limit)
                .map(|i| FeedItem::with_id(&format!("f{}", i), "content", "alice"))
                .collect();
            let total = items.len();
            Ok(FeedResult { items, total })
        }

        fn get_trending(&self) -> Result<Vec<TrendingTopic>, SocialAccessError> {
            Ok(vec![])
        }
    }

    fn manager_with(predictions: Vec<f64>) -> SocialAccessManager {
        let mut m = SocialAccessManager::new();
        m.register_adapter(std::sync::Arc::new(StubAdapter { predictions }));
        m.init_platform(SocialPlatform::Twitter).expect("init_platform");
        m
    }

    // ── R-P79 回归：注册了 adapter 就必须读得到 ──────────────

    #[test]
    fn registered_adapter_is_actually_reachable() {
        // ⛔ 原实现在这里无条件返回 vec![]，且无视 feed_service ——
        //    register_adapter 注册的东西永远读不到。
        let mut m = manager_with(vec![0.1, 0.2]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Latest, 10)
            .expect("registered adapter must be reachable");
        assert_eq!(r.posts.len(), 2, "stub adapter yields 2 posts");
    }

    #[test]
    fn unranked_is_reported_when_adapter_gives_no_predictions() {
        // adapter 不给预测 ⇒ 分数恒 0 ⇒ 必须能观测到「未排序」
        let mut m = manager_with(vec![]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Latest, 10)
            .expect("empty adapter must not error");
        assert!(r.posts.is_empty());
        assert!(!m.last_fetch_was_ranked(), "no predictions means no ranking happened");
    }

    #[test]
    fn ranked_is_reported_when_predictions_exist() {
        let mut m = manager_with(vec![0.1, 0.5]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Latest, 10)
            .expect("must succeed");
        assert_eq!(r.posts.len(), 2);
        assert!(m.last_fetch_was_ranked(), "predictions present means ranking happened");
        // 排序真的按分数走了：20.0*0.5=10.0 > 20.0*0.1=2.0
        assert_eq!(r.posts[0].id, "1", "higher predicted share must rank first");
    }

    #[test]
    fn unified_post_carries_ranked_flag_in_meta() {
        // ranked 标志必须进 platform_meta，否则下游无法判断
        let mut m = manager_with(vec![0.1, 0.5]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Latest, 10)
            .expect("must succeed");
        assert_eq!(r.posts[0].platform_meta["ranked"], serde_json::json!(true));
    }

    #[test]
    fn following_feed_is_not_claimed_as_ranked() {
        // ⛔ Following 是时间序，不经排序 ⇒ 必须记为 false
        let mut m = manager_with(vec![0.5]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Following, 3)
            .expect("following must work");
        assert_eq!(r.posts.len(), 3);
        assert!(!m.last_fetch_was_ranked(), "following is chronological, not ranked");
    }

    #[test]
    fn created_at_is_epoch_not_now_when_time_unknown() {
        // ⛔ FeedItem 无时间字段 ⇒ 必须用 epoch(0) 表示「未知」，
        //    填 SystemTime::now() 会伪造「刚刚发布」。
        let mut m = manager_with(vec![0.1]);
        let r = m
            .get_feed(SocialPlatform::Twitter, FeedType::Latest, 1)
            .expect("must succeed");
        let d = r.posts[0].created_at.duration_since(std::time::UNIX_EPOCH);
        assert_eq!(d.expect("epoch is representable").as_secs(), 0);
    }

    #[test]
    fn unknown_platform_yields_error_not_empty_success() {
        let mut m = manager_with(vec![0.1]);
        let err = m
            .get_feed(SocialPlatform::Reddit, FeedType::Latest, 5)
            .expect_err("no adapter registered for reddit must error");
        assert!(matches!(err, SocialAccessError::AuthFailed { .. } | SocialAccessError::Platform(_)));
    }
}
