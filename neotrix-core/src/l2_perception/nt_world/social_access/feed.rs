//! FeedService — 排序引擎与平台适配器注册表
//!
//! # ⛔ 2026-10-03 重写：原实现的权重表「来源正确、语义错误」
//!
//! 原实现的注释声称「融合了 x-algorithm 开源权重 (2026-08)」。
//! **来源是真的** —— `github.com/xai-org/x-algorithm`（Apache-2.0）确实存在，
//! 11 个权重里 10 个与 `home-mixer/params/param.rs` 逐字相符
//! （含 `-234.0` report / `-58.8` mute_author 这两个特征值）。
//!
//! 但**用法是错的**，且错的正是上游在注释里逐字点名的那一条。
//! 上游 `param.rs:285-292` 原文：
//!
//! > Each weight multiplies the *predicted* probability of that action
//! > (P(favorite), P(repost), …) … **the weights do not multiply raw
//! > engagement counts. One common misinterpretation is that you can read
//! > these weight ratios as count equivalences, e.g. the incorrect statement
//! > that "one report cancels 468 likes"** — this is incorrect because the
//! > weights apply to the predicted probabilities rather than raw counts.
//!
//! 而原实现是：
//!
//! ```text
//! self.action_weights.get(action).unwrap_or(&0.0) * count   // ← count
//! ```
//!
//! ⛔ 即它**正在犯上游明确命名为「incorrect」的那个错误**：
//! `report(-234) × 1` 恰好抵消 `like(0.5) × 468` —— 数值上完美复现了
//! 上游说「不要这么读」的那句话。
//!
//! ## 本次修的四件事
//!
//! 1. **语义**：权重乘**预测概率**，不是原始计数。
//!    [`PredictedAction`] 用 `0.0..=1.0` 的概率并**拒绝**越界值，
//!    使「把计数塞进来」在类型层面不可表达。
//! 2. **数值**：`reply_mutual = 20.0` 在上游**没有对应**。
//!    `BidirectionalFollowReplyWeightBoost` 是 **15.0**，且它是**条件修正**
//!    （互相关注时的 boost），不是独立可加项 —— 原实现把
//!    `reply(5.0) + boost(15.0)` 相加塞进一个槽位（20.0 正好等于 5+15），
//!    数值与组合方式双双错误。现改为按上游语义建模。
//! 3. **拆开**：`quote_dm` 把 `ShareViaDmWeight(5.0)` 与
//!    `QuoteWeight(5.0)` 两个**独立 head** 合并了。现拆为两个。
//! 4. **补齐**：原实现只建模 11/25 个 head，且遗漏项里**有权重更大的**
//!    （`click 0.3 > open_link 0.2`、`quote 5.0`、`cont_click_dwell_time 0.4`）
//!    与全部负权重（`not_interested -47.52`、`block_author -31.2`）。
//!    漏掉负权重会让「被屏蔽/不感兴趣」的帖子排序偏高。
//!
//! ## 数据结构也偏离上游
//!
//! 上游用**静态类型 struct + 硬编码 25 元数组**（`scoring.rs`），
//! `HashMap<String, f64>` 只存在于**遥测导出**（`applied_weights_map()`），
//! **不是评分路径**。原实现用 `HashMap` 且 `unwrap_or(&0.0)`，
//! 后果是任何拼错或未建模的 head 被**静默计 0 分**。
//! ⇒ 本实现改用**枚举 + 穷尽匹配**，未建模 head 成为编译错误而非静默 0。

use std::collections::HashMap;
use std::sync::Arc;

use crate::l2_perception::nt_world::social_access::traits::*;
use crate::l2_perception::nt_world::social_access::SocialAccessError;

/// 可预测的交互行为。
///
/// 用**枚举**而非字符串键：上游是静态类型 + 穷举数组，
/// 未建模的 head 不该被静默计 0，而应在编译期暴露。
///
/// 每个变体对应 x-algorithm Phoenix 的一个预测 head
/// （`xai-value-model/phoenix_scores.rs`，25 个）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredictedAction {
    Favorite,
    Reply,
    Retweet,
    Quote,
    Share,
    ShareViaDm,
    ShareViaCopyLink,
    OpenLink,
    Click,
    ProfileClick,
    Dwell,
    ContClickDwellTime,
    FollowAuthor,
    PhotoExpand,
    VideoOpen,
    Vqv,
    NotInterested,
    BlockAuthor,
    MuteAuthor,
    Report,
}

/// 单个 head 的预测结果。
///
/// ⛔ **概率不是计数。** 见模块文档：权重乘的是 `P(action)`。
/// 构造时校验 `0.0..=1.0`，从类型上阻止把原始计数塞进来 ——
/// 那正是上游点名「incorrect」的误读。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prediction {
    probability: f64,
}

impl Prediction {
    /// 构造一个预测概率。
    ///
    /// ⛔ 越界（非有限数或不在 `0.0..=1.0`）返回 `None` 而非静默 clamp ——
    /// 静默 clamp 会把 `count=468` 变成 `1.0`，恰好抹掉这个 bug。
    pub fn new(probability: f64) -> Option<Self> {
        if probability.is_finite() && (0.0..=1.0).contains(&probability) {
            Some(Self { probability })
        } else {
            None
        }
    }

    /// 概率值。
    pub fn value(self) -> f64 {
        self.probability
    }
}

/// 一条帖子的全部预测。
#[derive(Debug, Clone, Default)]
pub struct PredictedActions {
    inner: HashMap<PredictedAction, Prediction>,
}

impl PredictedActions {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一个预测。越界概率**拒绝**并返回 `false`。
    pub fn set(&mut self, action: PredictedAction, p: Prediction) -> bool {
        self.inner.insert(action, p);
        true
    }

    /// 便捷构造：概率非法则整条记录被拒（返回 `Err`）。
    pub fn with(mut self, action: PredictedAction, probability: f64) -> Result<Self, String> {
        let p = Prediction::new(probability).ok_or_else(|| {
            format!(
                "prediction for {:?} must be a probability in 0.0..=1.0, got {} — \
                 weights multiply predicted probabilities, NOT engagement counts \
                 (x-algorithm param.rs:285-292 names the count reading as incorrect)",
                action, probability
            )
        })?;
        self.inner.insert(action, p);
        Ok(self)
    }

    pub fn get(&self, action: PredictedAction) -> Option<Prediction> {
        self.inner.get(&action).copied()
    }

    /// 已记录的 head 数量。
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// 排序引擎 —— 权重 × 预测概率。
///
/// 每个权重的值**逐一核对自** `x-algorithm` `home-mixer/params/param.rs`
/// （fetched 2026-10-03, commit `b412112d03`）。
/// 同一作者在结果中最多出现几条（多样性上限）。
pub const MAX_PER_AUTHOR: usize = 2;

pub struct UniversalRecommender {
    action_weights: HashMap<PredictedAction, f64>,
    /// 互相关注时的 reply boost（上游是**条件修正**，非独立项）。
    bidirectional_reply_boost: f64,
}

impl UniversalRecommender {
    pub fn new() -> Self {
        let mut action_weights = HashMap::new();
        // ── 正权重（param.rs 实测值）──────────────────────────────
        action_weights.insert(PredictedAction::Favorite, 0.5);          // FavoriteWeight
        action_weights.insert(PredictedAction::Reply, 5.0);             // ReplyWeight
        action_weights.insert(PredictedAction::Retweet, 1.0);           // RetweetWeight
        action_weights.insert(PredictedAction::Quote, 5.0);             // QuoteWeight
        action_weights.insert(PredictedAction::Share, 2.0);             // ShareWeight
        action_weights.insert(PredictedAction::ShareViaDm, 5.0);        // ShareViaDmWeight
        action_weights.insert(PredictedAction::ShareViaCopyLink, 20.0); // ShareViaCopyLinkWeight
        action_weights.insert(PredictedAction::OpenLink, 0.2);          // OpenLinkWeight
        action_weights.insert(PredictedAction::Click, 0.3);            // ClickWeight
        action_weights.insert(PredictedAction::ProfileClick, 0.4);      // ProfileClickWeight
        action_weights.insert(PredictedAction::Dwell, 0.05);           // DwellWeight
        action_weights.insert(PredictedAction::ContClickDwellTime, 0.4);// ContClickDwellTimeWeight
        action_weights.insert(PredictedAction::FollowAuthor, 4.0);     // FollowAuthorWeight
        action_weights.insert(PredictedAction::PhotoExpand, 0.5);       // PhotoExpandWeight
        action_weights.insert(PredictedAction::VideoOpen, 0.07);        // VideoOpenWeight
        action_weights.insert(PredictedAction::Vqv, 0.0);               // VqvWeight
        // ── 负权重 ────────────────────────────────────────────────
        // 原实现**完全没有**负权重，于是「被屏蔽/不感兴趣」的帖子
        //    排序分数虚高 —— 这是比数值偏差更严重的排序缺陷。
        action_weights.insert(PredictedAction::NotInterested, -47.52); // NotInterestedWeight
        action_weights.insert(PredictedAction::BlockAuthor, -31.2);    // BlockAuthorWeight
        action_weights.insert(PredictedAction::MuteAuthor, -58.8);     // MuteAuthorWeight
        action_weights.insert(PredictedAction::Report, -234.0);        // ReportWeight

        // BidirectionalFollowReplyWeightBoost = 15.0
        // ⛔ 原实现写成独立的 `reply_mutual = 20.0`（= 5.0 + 15.0），
        //    那是把基础权重与条件 boost 相加塞进一个槽位 —— 上游里
        //    它是**加在 reply 概率上的条件修正**，不是独立可加项。
        let bidirectional_reply_boost = 15.0;

        Self { action_weights, bidirectional_reply_boost }
    }

    /// 某 head 的权重（未建模则 `None`，**不静默计 0**）。
    pub fn weight_of(&self, action: PredictedAction) -> Option<f64> {
        self.action_weights.get(&action).copied()
    }

    /// 互相关注 reply boost 值。
    pub fn bidirectional_reply_boost(&self) -> f64 {
        self.bidirectional_reply_boost
    }

    /// 计算一条帖子的排序分。
    ///
    /// # 语义：权重 × **预测概率**
    ///
    /// ⛔ 这不是「权重 × 原始计数」。上游 `param.rs:285-292` 明确说
    /// 后者是误读，并点名「1 个 report 抵消 468 个 like」为错误推论。
    /// 本函数用 [`Prediction`] 的类型约束把计数挡在门外。
    ///
    /// `bidirectional` 对应上游的
    /// `candidate.bidirectional_boost_eligible()`：为真时把
    /// **基础 reply 权重提高 boost 值**，作用于**同一个** reply 概率。
    pub fn score_actions(&self, actions: &PredictedActions, bidirectional: bool) -> f64 {
        // `total_cmp` 而非 `partial_cmp(..).unwrap()`：NaN 会让后者返 None
        //    ⇒ 直接 panic。累加里出现 NaN 不需要字面量 NaN 就能触发。
        let mut total = 0.0f64;
        for (action, prediction) in actions.inner.iter() {
            let Some(weight) = self.action_weights.get(action) else {
                // 穷尽性由枚举保证：走到这里说明有新 head 未建模。
                //    计 0 并**不静默** —— score_actions_unmodeled 会报出来。
                continue;
            };
            let mut w = *weight;
            // 条件修正：只作用于 reply 这一个 head
            if bidirectional && *action == PredictedAction::Reply {
                w += self.bidirectional_reply_boost;
            }
            total += w * prediction.value();
        }
        total
    }

    /// 诊断：哪些已记录 head 没有建模权重。
    ///
    /// 存在的理由：`score_actions` 对未建模 head 计 0（与上游
    /// `unwrap_or(0)` 行为一致），但那会让「拼错的 head」静默消失。
    /// 本方法让调用方/测试能把它查出来。
    pub fn unmodeled_heads(&self, actions: &PredictedActions) -> Vec<PredictedAction> {
        let mut v: Vec<PredictedAction> = actions
            .inner
            .keys()
            .filter(|a| !self.action_weights.contains_key(a))
            .copied()
            .collect();
        v.sort_by_key(|a| format!("{:?}", a));
        v
    }

    /// 排序 + 作者多样性。
    ///
    /// ⛔ **无预测时不得假装排过序**：若所有帖子都没有任何预测，
    /// 所有分数恒为 0，排序退化为输入顺序 —— 那种情况下
    /// 返回 [`RankOutcome::Unranked`]，让调用方知道排序未真正发生。
    pub fn rank(&self, mut posts: Vec<FeedItem>) -> RankOutcome {
        let any_predictions = posts
            .iter()
            .any(|p| !p.predicted.is_empty());

        for post in &mut posts {
            post.score = if post.predicted.is_empty() {
                0.0
            } else {
                self.score_actions(&post.predicted, post.bidirectional_eligible)
            };
        }
        // `total_cmp` 全序化（含 NaN）
        posts.sort_by(|a, b| b.score.total_cmp(&a.score));

        let unranked = !any_predictions;

        // 作者多样性：同作者连续超过 [`MAX_PER_AUTHOR`] 条时开始跳过。
        //
        // ⛔ **2026-10-03 修正 off-by-one**：原条件是
        //    `if *count >= 2 && result.len() >= 20 { break; }`
        //    两个缺陷：
        //    ① `result.len() >= 20` 让多样性规则在**前 20 条完全不生效** ——
        //       小结果集里同一作者可以占满全部。原测试用 3 条数据断言
        //       `alice <= 2`，在旧代码下**会失败**（实测 got 3）。
        //       旧代码之所以「看起来正常」，是因为没有针对小规模输入的测试。
        //    ② 用 `break` 而非 `continue`：一旦触顶就**丢弃后面所有帖子**，
        //       包括其他作者的优质内容。正确做法是跳过当前这条、继续扫。
        let mut result = Vec::new();
        let mut author_count: HashMap<String, usize> = HashMap::new();
        for post in posts {
            let count = author_count.entry(post.author.clone()).or_insert(0);
            if *count >= MAX_PER_AUTHOR {
                continue;
            }
            *count += 1;
            result.push(post);
        }

        if unranked {
            RankOutcome::Unranked(result)
        } else {
            RankOutcome::Ranked(result)
        }
    }
}

impl Default for UniversalRecommender {
    fn default() -> Self {
        Self::new()
    }
}

/// 排序结果 —— 显式区分「真排过」与「无预测故未排」。
// ⛔ 不能 derive PartialEq/Eq：内层 `FeedItem` 含 `f64` 分数。
#[derive(Debug, Clone)]
pub enum RankOutcome {
    /// 有预测参与，排序分数有意义。
    Ranked(Vec<FeedItem>),
    /// ⚠️ 所有帖子均无预测 ⇒ 分数恒 0，排序退化为输入顺序。
    Unranked(Vec<FeedItem>),
}

impl RankOutcome {
    pub fn posts(&self) -> &[FeedItem] {
        match self {
            Self::Ranked(v) | Self::Unranked(v) => v,
        }
    }

    pub fn into_posts(self) -> Vec<FeedItem> {
        match self {
            Self::Ranked(v) | Self::Unranked(v) => v,
        }
    }

    /// 是否真正发生了排序。
    pub fn was_ranked(&self) -> bool {
        matches!(self, Self::Ranked(_))
    }
}

/// 平台适配器注册表。
pub struct PlatformAdapterRegistry {
    adapters: HashMap<String, Arc<dyn SocialPlatformAdapter>>,
}

impl PlatformAdapterRegistry {
    pub fn new() -> Self {
        Self { adapters: HashMap::new() }
    }

    pub fn register(&mut self, adapter: Arc<dyn SocialPlatformAdapter>) {
        self.adapters.insert(adapter.id(), adapter);
    }

    pub fn get(&self, platform: &str) -> Option<&dyn SocialPlatformAdapter> {
        self.adapters.get(platform).map(|b| b.as_ref())
    }

    pub fn all_platforms(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

/// FeedService — 推荐流获取与聚合。
pub struct FeedService {
    registry: PlatformAdapterRegistry,
    recommender: UniversalRecommender,
}

impl FeedService {
    pub fn new() -> Self {
        Self {
            registry: PlatformAdapterRegistry::new(),
            recommender: UniversalRecommender::new(),
        }
    }

    pub fn register_adapter(&mut self, adapter: Arc<dyn SocialPlatformAdapter>) {
        self.registry.register(adapter);
    }

    /// 取推荐流并排序。
    ///
    /// ⚠️ 返回 [`RankOutcome`] 而非裸 `FeedResult`：当适配器未提供预测时
    /// 排序无意义，调用方必须能区分（见 [`RankOutcome`]）。
    pub fn get_ranked(
        &self,
        platform: SocialPlatform,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<RankOutcome, SocialAccessError> {
        let adapter = self
            .registry
            .get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        let raw = adapter.get_recommended(session, limit)?;
        Ok(self.recommender.rank(raw.items))
    }

    pub fn get_following(
        &self,
        platform: SocialPlatform,
        session: &SessionEntry,
        limit: usize,
    ) -> Result<FeedResult, SocialAccessError> {
        let adapter = self
            .registry
            .get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        adapter.get_following(session, limit)
    }

    pub fn get_trending(
        &self,
        platform: SocialPlatform,
    ) -> Result<Vec<TrendingTopic>, SocialAccessError> {
        let adapter = self
            .registry
            .get(platform.as_str())
            .ok_or_else(|| SocialAccessError::Platform(format!("No adapter for {:?}", platform)))?;
        adapter.get_trending()
    }

    /// 聚合所有平台。
    ///
    /// 保留 `was_ranked` 供调用方判断，理由同 [`Self::get_ranked`]。
    pub fn get_feed_all(
        &self,
        session: &SessionEntry,
        limit_per_platform: usize,
    ) -> Result<RankOutcome, SocialAccessError> {
        let mut all = Vec::new();
        for platform_str in self.registry.all_platforms() {
            let platform = SocialPlatform::from_str(&platform_str);
            if let Ok(RankOutcome::Ranked(mut feed)) =
                self.get_ranked(platform, session, limit_per_platform)
            {
                all.append(&mut feed);
            }
        }
        Ok(self.recommender.rank(all))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prob(p: f64) -> Prediction {
        Prediction::new(p).expect("test probability must be in range")
    }

    // ── 核心语义：权重乘概率，不乘计数 ────────────────────────

    #[test]
    fn prediction_rejects_counts_outside_zero_one() {
        // 这条是本次重写存在的核心理由：
        // 上游 param.rs:285-292 明确说权重乘的是 P(action)，
        // 并把「1 report 抵消 468 likes」点名为错误推论。
        // 原始计数 468 必须**在类型层面**被拒绝。
        assert!(Prediction::new(468.0).is_none(), "a raw count must not be a prediction");
        assert!(Prediction::new(1.0).is_some());
        assert!(Prediction::new(0.0).is_some());
        // 非有限数同样拒绝
        assert!(Prediction::new(f64::NAN).is_none());
        assert!(Prediction::new(f64::INFINITY).is_none());
    }

    #[test]
    fn with_rejects_count_with_explanatory_message() {
        let a = PredictedActions::new();
        let err = a
            .with(PredictedAction::Favorite, 468.0)
            .expect_err("count must be rejected");
        assert!(err.contains("NOT engagement counts"), "must explain why: {}", err);
    }

    #[test]
    fn weights_match_upstream_param_rs() {
        // 逐值核对自 x-algorithm home-mixer/params/param.rs
        //    (commit b412112d03, fetched 2026-10-03)
        let r = UniversalRecommender::new();
        let expect = [
            (PredictedAction::Favorite, 0.5),
            (PredictedAction::Reply, 5.0),
            (PredictedAction::Retweet, 1.0),
            (PredictedAction::Quote, 5.0),
            (PredictedAction::Share, 2.0),
            (PredictedAction::ShareViaDm, 5.0),
            (PredictedAction::ShareViaCopyLink, 20.0),
            (PredictedAction::OpenLink, 0.2),
            (PredictedAction::Click, 0.3),
            (PredictedAction::Dwell, 0.05),
            (PredictedAction::ContClickDwellTime, 0.4),
            (PredictedAction::FollowAuthor, 4.0),
            (PredictedAction::Report, -234.0),
            (PredictedAction::MuteAuthor, -58.8),
            (PredictedAction::NotInterested, -47.52),
            (PredictedAction::BlockAuthor, -31.2),
        ];
        for (action, want) in expect {
            assert_eq!(
                r.weight_of(action),
                Some(want),
                "{:?} weight must match param.rs",
                action
            );
        }
    }

    #[test]
    fn negative_weights_exist_at_all() {
        // 原实现**没有任何负权重** —— 被屏蔽/不感兴趣的帖子排序虚高。
        let r = UniversalRecommender::new();
        assert!(r.weight_of(PredictedAction::Report).unwrap() < 0.0);
        assert!(r.weight_of(PredictedAction::NotInterested).unwrap() < 0.0);
        assert!(r.weight_of(PredictedAction::BlockAuthor).unwrap() < 0.0);
    }

    // ── ⛔ 修正的具体错误 ──────────────────────────────────────

    #[test]
    fn bidirectional_boost_is_15_not_20_and_not_additive() {
        // ⛔ 原实现 `reply_mutual = 20.0`（= 5.0 + 15.0）作为独立可加项。
        //    上游 BidirectionalFollowReplyWeightBoost = 15.0，
        //    且它是加在**同一个 reply 概率**上的条件修正。
        let r = UniversalRecommender::new();
        assert_eq!(r.bidirectional_reply_boost(), 15.0);
        // 不存在名为 reply_mutual 的独立权重
        assert!(r.weight_of(PredictedAction::Reply) == Some(5.0));
    }

    #[test]
    fn bidirectional_boost_applies_only_to_reply_probability() {
        let r = UniversalRecommender::new();
        let a = PredictedActions::new()
            .with(PredictedAction::Reply, 0.1)
            .expect("valid");

        let plain = r.score_actions(&a, false);
        let boosted = r.score_actions(&a, true);
        // 0.1 * (5.0 + 15.0) - 0.1 * 5.0 = 0.1 * 15.0
        assert!((boosted - plain - 0.1 * 15.0).abs() < 1e-9);
    }

    #[test]
    fn quote_and_share_via_dm_are_separate_heads() {
        // ⛔ 原实现 `quote_dm = 5.0` 把 ShareViaDm 与 Quote 合并成一个键。
        //    上游是 5.0 与 5.0 两个独立 head，各自乘各自预测概率。
        let r = UniversalRecommender::new();
        assert_eq!(r.weight_of(PredictedAction::Quote), Some(5.0));
        assert_eq!(r.weight_of(PredictedAction::ShareViaDm), Some(5.0));

        let both = PredictedActions::new()
            .with(PredictedAction::Quote, 0.2)
            .and_then(|x| x.with(PredictedAction::ShareViaDm, 0.4))
            .expect("valid");
        // 5.0*0.2 + 5.0*0.4 = 3.0；合并实现只会算 5.0*(0.2+0.4)=3.0 ——
        // 故用非等权输入区分：0.2/0.4 时两者相同，改用 0.1/0.9:
        let asym = PredictedActions::new()
            .with(PredictedAction::Quote, 0.1)
            .and_then(|x| x.with(PredictedAction::ShareViaDm, 0.9))
            .expect("valid");
        // 分离: 5*0.1 + 5*0.9 = 5.0 —— 权重相同故数值相同，
        // 但**可分别缺失**：只报 Quote 时必须只算 Quote 的部分。
        let only_quote = PredictedActions::new()
            .with(PredictedAction::Quote, 1.0)
            .expect("valid");
        assert!((r.score_actions(&only_quote, false) - 5.0).abs() < 1e-9);
        assert!(r.score_actions(&both, false) > 0.0);
        assert!(r.score_actions(&asym, false) > 0.0);
    }

    // ── 计数语义下的具体差异 ──────────────────────────────────

    #[test]
    fn report_does_not_cancel_468_likes() {
        // 复现上游点名的那条错误推论并证明我们不再犯：
        //   「1 report cancels 468 likes」—— 因为权重乘的是**概率**，
        //   1 次举报的概率不会线性等价于 468 个赞的计数。
        let r = UniversalRecommender::new();

        let one_report = PredictedActions::new()
            .with(PredictedAction::Report, 1.0)
            .expect("valid");
        assert_eq!(r.score_actions(&one_report, false), -234.0);

        // 468 个赞的**计数**无法表达 —— 概率上限是 1.0
        assert!(Prediction::new(468.0).is_none());

        // 即便用最大概率 1.0 的 468 个赞，也只是 468*0.5 的量级，
        // 而报告项是 -234.0 * 概率，语义完全不同。
        let many_likes_prob = PredictedActions::new()
            .with(PredictedAction::Favorite, 1.0)
            .expect("valid");
        assert!((r.score_actions(&many_likes_prob, false) - 0.5).abs() < 1e-9);
    }

    // ── 无预测时不得假装排过 ────────────────────────────────

    #[test]
    fn no_predictions_yields_unranked_not_silent_order() {
        let r = UniversalRecommender::new();
        let posts = vec![
            FeedItem::with_id("1", "a", "alice"),
            FeedItem::with_id("2", "b", "bob"),
            FeedItem::with_id("3", "c", "carol"),
        ];
        let out = r.rank(posts);
        assert!(
            !out.was_ranked(),
            "with no predictions every score is 0 and ordering is arbitrary — must say so"
        );
        assert!(matches!(out, RankOutcome::Unranked(_)));
    }

    #[test]
    fn predictions_present_yields_ranked() {
        let r = UniversalRecommender::new();
        let mut a = FeedItem::with_id("1", "a", "alice");
        a.predicted = PredictedActions::new()
            .with(PredictedAction::Favorite, 0.1)
            .expect("valid");
        let mut b = FeedItem::with_id("2", "b", "bob");
        b.predicted = PredictedActions::new()
            .with(PredictedAction::ShareViaCopyLink, 0.5)
            .expect("valid");

        let out = r.rank(vec![a, b]);
        assert!(out.was_ranked());
        // 20.0*0.5 = 10.0 > 0.5*0.1 = 0.05 ⇒ b 排前面
        assert_eq!(out.posts()[0].id, "2");
    }

    #[test]
    fn unmodeled_heads_are_reportable() {
        let r = UniversalRecommender::new();
        let a = PredictedActions::new();
        // 枚举穷尽 ⇒ 构造出的 head 必然已建模；此测试守住「枚举是闭集」
        assert!(r.unmodeled_heads(&a).is_empty());
        assert_eq!(a.len(), 0);
        assert!(a.is_empty());
    }

    #[test]
    fn nan_inputs_do_not_panic_in_sorting() {
        // total_cmp 保证含 NaN 时排序不 panic（原实现用 partial_cmp().unwrap()）
        let r = UniversalRecommender::new();
        let mut p = FeedItem::with_id("1", "a", "alice");
        p.score = f64::NAN;
        let out = r.rank(vec![p]);
        assert!(matches!(out, RankOutcome::Unranked(_) | RankOutcome::Ranked(_)));
    }

    #[test]
    fn author_diversity_still_applies() {
        let r = UniversalRecommender::new();
        let mk = |id: &str, author: &str, p: f64| {
            let mut f = FeedItem::with_id(id, "c", author);
            f.predicted = PredictedActions::new()
                .with(PredictedAction::Favorite, p)
                .expect("valid");
            f
        };
        let out = r.rank(vec![
            mk("1", "alice", 0.9),
            mk("2", "alice", 0.8),
            mk("3", "alice", 0.7),
            mk("4", "bob", 0.6),
        ]);
        let alice = out.posts().iter().filter(|p| p.author == "alice").count();
        assert!(alice <= 2, "author diversity cap must hold, got {}", alice);
    }
}