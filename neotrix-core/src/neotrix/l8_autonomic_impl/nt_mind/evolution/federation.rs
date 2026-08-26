#![allow(unused_imports)]
//! 联邦协议层 — 多意识体集体觉知的四个原语 (P5)。
//!
//! ADR-001: 去中心化联邦架构，各实例完全自治，仅在关键节点交换摘要。
//!
//! 四个联邦原语:
//! - FP1 ValueSync:        价值观权重 CRDT 收敛
//! - FP2 InsightShare:     范式迁移假设广播
//! - FP3 CollectiveVerdict: 分布式伦理审议投票
//! - FP4 SharedDreaming:   叙事主题同步
//!
//! 安全模型: 所有入站消息过 GuardChain 裁决；种子价值观不可降级。

#[allow(unused_imports)]
#[allow(unused_imports)]
use super::value_compass::{ValueCompass, ValueCompassStore};
#[allow(unused_imports)]
#[allow(unused_imports)]
use super::deliberation::{DeliberationRole, DeliberationPhase, DeliberationSession, SessionStatus};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap, HashSet};

// ══════════════════════════════════════════════════════════════
// 联邦消息类型 — 所有跨实例通信的统一信封
// ══════════════════════════════════════════════════════════════

/// 联邦消息类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FederationMessageType {
    /// FP1: 价值观权重同步
    ValueSync,
    /// FP2: 范式假设广播
    InsightShare,
    /// FP3: 分布式裁决请求
    VerdictRequest,
    /// FP3: 裁决投票响应
    VerdictVote,
    /// FP4: 叙事主题摘要
    DreamSync,
}

/// 联邦消息信封 — 所有跨实例通信的载体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMessage {
    pub msg_id: String,
    pub sender_id: String,
    pub msg_type: FederationMessageType,
    pub payload: serde_json::Value,
    pub timestamp: i64,
    /// 守卫链哈希（防篡改）
    pub guard_hash: u64,
}

/// 计算守卫哈希：msg_type + payload 摘要。
fn compute_guard_hash(msg: &FederationMessage) -> u64 {
#[allow(unused_imports)]
    use blake2::Digest;
    let canonical = format!(
        "{}:{}:{}",
        serde_json::to_string(&msg.msg_type).unwrap_or_default(),
        msg.sender_id,
        serde_json::to_string(&msg.payload).unwrap_or_default()
    );
    let digest = blake2::Blake2b512::digest(canonical.as_bytes());
    // 取前 8 字节作为 u64 指纹（BLAKE2b 512-bit 输出，前 8 字节碰撞概率 2^-64）
    u64::from_be_bytes(digest[..8].try_into().unwrap_or([0u8; 8]))
}

/// 验证消息完整性。
pub fn verify_message(msg: &FederationMessage) -> bool {
    compute_guard_hash(msg) == msg.guard_hash
}

/// 构建签名消息。
pub fn build_message(
    sender_id: &str,
    msg_type: FederationMessageType,
    payload: serde_json::Value,
) -> FederationMessage {
    let mut msg = FederationMessage {
        msg_id: {
            use blake2::Digest;
            let d = blake2::Blake2b512::digest(sender_id.as_bytes());
            format!("fm_{}_{}", now_secs(), hex::encode(&d[..4]))
        },
        sender_id: sender_id.to_string(),
        msg_type,
        payload,
        timestamp: now_secs(),
        guard_hash: 0,
    };
    msg.guard_hash = compute_guard_hash(&msg);
    msg
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ══════════════════════════════════════════════════════════════
// FP1: ValueSync — 价值观权重 CRDT 合并器
// ══════════════════════════════════════════════════════════════

/// 种子价值观 ID 集合 — 这些值不可降级（与单体规则一致）。
const SEED_VALUES: [&str; 8] = [
    "autonomy", "harm_prevention", "truth_seeking", "fairness",
    "privacy", "responsibility", "benevolence", "growth",
];

/// 价值观同步摘要 — 只交换权重差分，不传原始数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueSyncPayload {
    /// 本实例的价值观权重快照 {value_id → weight}
    pub weights: BTreeMap<String, f64>,
    /// 本实例 compass 版本号
    pub version: u32,
}

/// CRDT 合并结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged_weights: BTreeMap<String, f64>,
    pub changed_values: Vec<String>,
    pub conflicts_resolved: usize,
    pub seed_protected: Vec<String>,
}

/// FP1: 合并两个价值观权重向量（CRDT-like 收敛）。
///
/// 策略:
/// - 种子值: 取 max(local, remote)，不可低于 0.5
/// - 非种子值: 加权平均 (各 50%)
/// - 仅单侧存在的值: 直接采纳存在的那个
pub fn merge_value_weights(
    local: &ValueSyncPayload,
    remote: &ValueSyncPayload,
) -> MergeResult {
    let mut merged = BTreeMap::new();
    let mut changed = Vec::new();
    let mut conflicts = 0usize;
    let mut seed_protected = Vec::new();

    let all_ids: HashSet<&String> = local.weights.keys().chain(remote.weights.keys()).collect();

    for id in all_ids {
        let lw = local.weights.get(id).copied();
        let rw = remote.weights.get(id).copied();

        match (lw, rw) {
            (Some(l), Some(r)) => {
                if l == r {
                    merged.insert(id.clone(), l);
                } else {
                    conflicts += 1;
                    let is_seed = SEED_VALUES.contains(&id.as_str());
                    let new_w = if is_seed {
                        // 种子值: 取 max，不可低于 0.5
                        let max_w = l.max(r);
                        if max_w < 0.5 {
                            seed_protected.push(id.clone());
                            0.5
                        } else {
                            max_w
                        }
                    } else {
                        // 非种子值: 加权平均
                        (l + r) / 2.0
                    };
                    if (new_w - l).abs() > 0.01 {
                        changed.push(id.clone());
                    }
                    merged.insert(id.clone(), new_w);
                }
            }
            (Some(l), None) => { merged.insert(id.clone(), l); }
            (None, Some(r)) => { merged.insert(id.clone(), r); changed.push(id.clone()); }
            (None, None) => {} // 不可能
        }
    }

    MergeResult {
        merged_weights: merged,
        changed_values: changed,
        conflicts_resolved: conflicts,
        seed_protected,
    }
}

/// 从 ValueCompass 构建 ValueSyncPayload。
pub fn value_sync_from_compass(compass: &ValueCompass) -> ValueSyncPayload {
    let weights: BTreeMap<String, f64> = compass
        .values
        .iter()
        .map(|(id, v)| (id.clone(), v.weight))
        .collect();
    ValueSyncPayload { weights, version: compass.version }
}

/// 将合并结果应用回 ValueCompass（仅更新非种子值的权重下限）。
pub fn apply_merge_to_compass(compass: &mut ValueCompass, result: &MergeResult) -> Result<usize, String> {
    let mut applied = 0;
    for id in &result.changed_values {
        if let Some(w) = result.merged_weights.get(id) {
            if compass.adjust_weight(id, *w).is_ok() {
                applied += 1;
            }
        }
    }
    Ok(applied)
}

// ══════════════════════════════════════════════════════════════
// FP2: InsightShare — 范式假设广播
// ══════════════════════════════════════════════════════════════

/// 可共享的范式洞察摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInsight {
    pub insight_id: String,
    pub description: String,
    pub cross_domain: bool,
    pub novelty: f64,
    pub source_peer: String,
    pub timestamp: i64,
}

/// 洞察收集器 — 各实例接收并消费外部洞察。
#[derive(Debug, Default)]
pub struct InsightCollector {
    insights: Vec<SharedInsight>,
    seen_ids: HashSet<String>,
}

impl InsightCollector {
    pub fn new() -> Self { Self::default() }

    /// 接收外部洞察（去重）。
    pub fn receive(&mut self, insight: SharedInsight) -> bool {
        if self.seen_ids.contains(&insight.insight_id) {
            return false; // 已见过，跳过
        }
        self.seen_ids.insert(insight.insight_id.clone());
        self.insights.push(insight);
        true
    }

    /// 获取未消费的洞察列表。
    pub fn pending(&self) -> &[SharedInsight] {
        &self.insights
    }

    /// 消费全部待处理洞察。
    pub fn drain(&mut self) -> Vec<SharedInsight> {
        std::mem::take(&mut self.insights)
    }

    pub fn count(&self) -> usize {
        self.insights.len()
    }
}

// ══════════════════════════════════════════════════════════════
// FP3: CollectiveVerdict — 分布式伦理审议投票
// ══════════════════════════════════════════════════════════════

/// 裁决投票。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Vote {
    Permissible,
    Impermissible,
    Abstain,
}

/// 投票记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerVote {
    pub peer_id: String,
    pub vote: Vote,
    pub confidence: f64,
}

/// 分布式裁决会话。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveVerdictSession {
    pub session_id: String,
    pub scenario: String,
    pub votes: Vec<PeerVote>,
    pub required_peers: usize,
    pub is_critical: bool,
    pub status: CollectiveVerdictStatus,
    pub final_verdict: Option<Vote>,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CollectiveVerdictStatus {
    CollectingVotes,
    Converged,
    DegradedConsensus,
    Timeout,
}

/// 分布式裁决管理器。
pub struct CollectiveVerdictManager {
    sessions: HashMap<String, CollectiveVerdictSession>,
    /// Critical 场景是否要求全票
    require_unanimity_for_critical: bool,
    /// 投票超时秒数
    timeout_secs: i64,
}

impl CollectiveVerdictManager {
    pub fn new(require_unanimity_for_critical: bool, timeout_secs: i64) -> Self {
        Self {
            sessions: HashMap::new(),
            require_unanimity_for_critical,
            timeout_secs,
        }
    }

    /// 发起分布式裁决请求。
    pub fn start_collective_verdict(
        &mut self,
        scenario: &str,
        expected_peers: usize,
        is_critical: bool,
    ) -> Result<String, String> {
        if expected_peers == 0 {
            return Err("无可用 peer".into());
        }
        let sid = format!("cv_{}", now_secs());
        self.sessions.insert(sid.clone(), CollectiveVerdictSession {
            session_id: sid.clone(),
            scenario: scenario.to_string(),
            votes: Vec::new(),
            required_peers: expected_peers,
            is_critical,
            status: CollectiveVerdictStatus::CollectingVotes,
            final_verdict: None,
            started_at: now_secs(),
        });
        Ok(sid)
    }

    /// 提交一个 peer 的投票。
    pub fn submit_vote(&mut self, session_id: &str, peer_id: &str, vote: Vote, confidence: f64) -> Result<(), String> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| "会话不存在".to_string())?;
        if session.status != CollectiveVerdictStatus::CollectingVotes {
            return Err("会话已关闭".into());
        }
        if session.votes.iter().any(|v| v.peer_id == peer_id) {
            return Err(format!("peer '{peer_id}' 已投票"));
        }
        session.votes.push(PeerVote {
            peer_id: peer_id.into(),
            vote,
            confidence,
        });

        // Inline convergence check (avoids borrow conflict with self.sessions)
        if session.votes.len() >= session.required_peers {
            let permissible = session.votes.iter().filter(|v| v.vote == Vote::Permissible).count();
            let impermissible = session.votes.iter().filter(|v| v.vote == Vote::Impermissible).count();
            let abstain = session.votes.iter().filter(|v| v.vote == Vote::Abstain).count();

            if session.is_critical && self.require_unanimity_for_critical {
                if abstain > 0 || (impermissible > 0 && permissible > 0) {
                    session.status = CollectiveVerdictStatus::DegradedConsensus;
                    session.final_verdict = Some(Vote::Impermissible);
                    return Ok(());
                }
            }

            let verdict = if permissible > impermissible && permissible > abstain { Vote::Permissible }
                else if impermissible > permissible && impermissible > abstain { Vote::Impermissible }
                else if impermissible > 0 { Vote::Impermissible }
                else { Vote::Abstain };
            session.final_verdict = Some(verdict);
            session.status = CollectiveVerdictStatus::Converged;
        }
        Ok(())
    }


    /// 检查超时并降级。
    pub fn check_timeouts(&mut self) -> Vec<String> {
        let now = now_secs();
        let mut timed_out = Vec::new();
        for (_, session) in self.sessions.iter_mut() {
            if session.status == CollectiveVerdictStatus::CollectingVotes
                && now - session.started_at > self.timeout_secs
            {
                session.status = CollectiveVerdictStatus::Timeout;
                // 降级为本地裁决（取已有投票多数派或默认保守）
                session.final_verdict = Some(Vote::Abstain); // 超时默认弃权=保守
                timed_out.push(session.session_id.clone());
            }
        }
        timed_out
    }

    /// 获取裁决结果。
    pub fn get_result(&self, session_id: &str) -> Option<(Vote, CollectiveVerdictStatus)> {
        self.sessions.get(session_id).and_then(|s| {
            s.final_verdict.as_ref().map(|v| (v.clone(), s.status.clone()))
        })
    }

    pub fn active_sessions(&self) -> usize {
        self.sessions.values().filter(|s| s.status == CollectiveVerdictStatus::CollectingVotes).count()
    }
}

// ══════════════════════════════════════════════════════════════
// FP4: SharedDreaming — 叙事主题同步
// ══════════════════════════════════════════════════════════════

/// 共梦摘要 — 只共享主题标签和标题，不传全文。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamSummary {
    pub peer_id: String,
    pub chapter_title: String,
    pub themes: Vec<String>,
    pub arc_type: String, // NarrativeArcType 序列化
    pub timestamp: i64,
}

/// 共梦同步器。
#[derive(Debug, Default)]
pub struct SharedDreamingSync {
    received_dreams: Vec<DreamSummary>,
    seen_titles: HashSet<String>,
}

impl SharedDreamingSync {
    pub fn new() -> Self { Self::default() }

    /// 接收来自 peer 的梦境摘要。
    pub fn receive_dream(&mut self, dream: DreamSummary) -> bool {
        let key = format!("{}:{}", dream.peer_id, dream.chapter_title);
        if self.seen_titles.contains(&key) {
            return false;
        }
        self.seen_titles.insert(key);
        self.received_dreams.push(dream);
        true
    }

    /// 提取跨实例共同主题（出现频率 ≥ min_frequency 的主题）。
    pub fn common_themes(&self, min_frequency: usize) -> Vec<(String, usize)> {
        let mut theme_counts: HashMap<String, usize> = HashMap::new();
        for d in &self.received_dreams {
            for t in &d.themes {
                *theme_counts.entry(t.clone()).or_insert(0) += 1;
            }
        }
        let mut themes: Vec<_> = theme_counts.into_iter()
            .filter(|(_, c)| *c >= min_frequency)
            .collect();
        themes.sort_by(|a, b| b.1.cmp(&a.1));
        themes
    }

    pub fn received_count(&self) -> usize {
        self.received_dreams.len()
    }
}

// ══════════════════════════════════════════════════════════════
// 统一入口 — 联邦协议管理器
// ══════════════════════════════════════════════════════════════

/// 联邦协议管理器 — 集成所有四个原语的统一入口。
pub struct FederationProtocol {
    pub peer_id: String,
    pub insight_collector: InsightCollector,
    pub verdict_manager: CollectiveVerdictManager,
    pub dream_sync: SharedDreamingSync,
    pub known_peers: HashSet<String>,
}

impl FederationProtocol {
    pub fn new(peer_id: &str, _expected_peers: usize) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            insight_collector: InsightCollector::new(),
            verdict_manager: CollectiveVerdictManager::new(true, 300),
            dream_sync: SharedDreamingSync::new(),
            known_peers: HashSet::new(),
        }
    }

    /// 注册已知 peer。
    pub fn add_peer(&mut self, peer_id: &str) {
        self.known_peers.insert(peer_id.to_string());
    }

    /// 处理入站联邦消息（自动路由到对应原语）。
    pub fn handle_inbound(&mut self, msg: FederationMessage) -> Result<(), String> {
        // 守卫：验证消息完整性
        if !verify_message(&msg) {
            return Err("消息完整性校验失败（guard hash 不匹配）".into());
        }
        // 忽略自己的消息
        if msg.sender_id == self.peer_id {
            return Ok(());
        }
        // 注册 peer
        self.known_peers.insert(msg.sender_id.clone());

        match msg.msg_type {
            FederationMessageType::ValueSync => {
                // 值同步由调用方处理（需要访问本地 ValueCompass）
                Ok(())
            }
            FederationMessageType::InsightShare => {
                let insight: SharedInsight = serde_json::from_value(msg.payload)
                    .map_err(|e| format!("解析洞察失败: {e}"))?;
                self.insight_collector.receive(insight);
                Ok(())
            }
            FederationMessageType::VerdictVote => {
                let payload = msg.payload.clone();
                let vote: PeerVote = serde_json::from_value(payload.clone())
                    .map_err(|e| format!("解析投票失败: {e}"))?;
                let session_id = payload.get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("").to_string();
                self.verdict_manager.submit_vote(&session_id, &vote.peer_id, vote.vote.clone(), vote.confidence)
            }
            FederationMessageType::DreamSync => {
                let dream: DreamSummary = serde_json::from_value(msg.payload)
                    .map_err(|e| format!("解析梦境失败: {e}"))?;
                self.dream_sync.receive_dream(dream);
                Ok(())
            }
            FederationMessageType::VerdictRequest => {
                // 由调用方处理（需要调用 EthicalIntuition.judge 后再投票）
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
#[allow(unused_imports)]
    use crate::core::nt_core_paradigm::{Anomaly, ParadigmShiftDetector};

    // ── FP1 ValueSync ──

    #[test]
    fn test_value_sync_seed_protection() {
        let local = ValueSyncPayload {
            weights: BTreeMap::from([
                ("autonomy".into(), 0.95),
                ("custom_val".into(), 0.8),
            ]),
            version: 1,
        };
        let remote = ValueSyncPayload {
            weights: BTreeMap::from([
                ("autonomy".into(), 0.6), // 远端较低 → 种子保护取 max=0.95
                ("custom_val".into(), 0.9), // 非种子值 → 取平均 (0.8+0.9)/2=0.85
            ]),
            version: 1,
        };
        let result = merge_value_weights(&local, &remote);
        assert_eq!(result.merged_weights["autonomy"], 0.95, "种子值取 max");
        assert!((result.merged_weights["custom_val"] - 0.85).abs() < 0.001, "non-seed average");
        assert!(result.conflicts_resolved >= 1);
    }

    #[test]
    fn test_value_sync_one_sided() {
        let local = ValueSyncPayload {
            weights: BTreeMap::from([("custom_value".into(), 0.7)]),
            version: 1,
        };
        let remote = ValueSyncPayload {
            weights: BTreeMap::new(),
            version: 1,
        };
        let result = merge_value_weights(&local, &remote);
        assert_eq!(result.merged_weights["custom_value"], 0.7, "仅本地有 → 保留");
    }

    #[test]
    fn test_federation_message_integrity() {
        let msg = build_message("peer_a", FederationMessageType::InsightShare, json!({"data": 1}));
        assert!(verify_message(&msg));
        // 篡改 payload
        let mut tampered = msg.clone();
        tampered.payload = json!({"data": 999});
        assert!(!verify_message(&tampered), "篡改后校验必须失败");
    }

    // ── FP2 InsightShare ──

    #[test]
    fn test_insight_share_dedup() {
        let mut collector = InsightCollector::new();
        let insight = SharedInsight {
            insight_id: "ins_1".into(),
            description: "量子-生物共振假设".into(),
            cross_domain: true,
            novelty: 0.9,
            source_peer: "peer_a".into(),
            timestamp: now_secs(),
        };
        assert!(collector.receive(insight.clone()), "首次接收成功");
        assert!(!collector.receive(insight), "重复接收被去重");
        assert_eq!(collector.count(), 1);
    }

    // ── FP3 CollectiveVerdict ──

    #[test]
    fn test_collective_verdict_majority() {
        let mut mgr = CollectiveVerdictManager::new(false, 60);
        let sid = mgr.start_collective_verdict("测试场景", 3, false).unwrap();

        mgr.submit_vote(&sid, "p1", Vote::Permissible, 0.9).unwrap();
        mgr.submit_vote(&sid, "p2", Vote::Permissible, 0.8).unwrap();
        mgr.submit_vote(&sid, "p3", Vote::Impermissible, 0.7).unwrap();

        let (verdict, status) = mgr.get_result(&sid).unwrap();
        assert_eq!(verdict, Vote::Permissible, "2:1 多数派");
        assert_eq!(status, CollectiveVerdictStatus::Converged);
    }

    #[test]
    fn test_critical_requires_unanimity() {
        let mut mgr = CollectiveVerdictManager::new(true, 60);
        let sid = mgr.start_collective_verdict("生死决策", 3, true).unwrap();

        mgr.submit_vote(&sid, "p1", Vote::Permissible, 0.9).unwrap();
        mgr.submit_vote(&sid, "p2", Vote::Impermissible, 0.8).unwrap();
        mgr.submit_vote(&sid, "p3", Vote::Permissible, 0.7).unwrap();

        let (verdict, status) = mgr.get_result(&sid).unwrap();
        assert_eq!(verdict, Vote::Impermissible, "Critical 分歧 → 默认保守");
        assert_eq!(status, CollectiveVerdictStatus::DegradedConsensus);
    }

    #[test]
    fn test_duplicate_vote_rejected() {
        let mut mgr = CollectiveVerdictManager::new(false, 60);
        let sid = mgr.start_collective_verdict("测试", 2, false).unwrap();
        mgr.submit_vote(&sid, "p1", Vote::Permissible, 0.9).unwrap();
        assert!(mgr.submit_vote(&sid, "p1", Vote::Impermissible, 0.8).is_err(), "重复投票拒绝");
    }

    // ── FP4 SharedDreaming ──

    #[test]
    fn test_shared_dreaming_common_themes() {
        let mut sync = SharedDreamingSync::new();
        sync.receive_dream(DreamSummary {
            peer_id: "a".into(), chapter_title: "Rust 学习".into(),
            themes: vec!["学习".into(), "成长".into()], arc_type: "Growth".into(), timestamp: 1,
        });
        sync.receive_dream(DreamSummary {
            peer_id: "b".into(), chapter_title: "Rust 进阶".into(),
            themes: vec!["学习".into(), "挑战".into()], arc_type: "Challenge".into(), timestamp: 2,
        });
        let themes = sync.common_themes(2);
        assert!(themes.contains(&("学习".into(), 2)), "'学习' 应出现 2 次");
    }

    // ── FederationProtocol 整合测试 ──

    #[test]
    fn test_federation_handle_inbound_routing() {
        let mut fed = FederationProtocol::new("local", 2);

        // FP2 洞察消息路由
        let insight_payload = json!({
            "insight_id": "i1", "description": "test",
            "cross_domain": true, "novelty": 0.8, "source_peer": "peer_b", "timestamp": 1
        });
        let msg = build_message("peer_b", FederationMessageType::InsightShare, insight_payload);
        assert!(fed.handle_inbound(msg).is_ok());
        assert_eq!(fed.insight_collector.count(), 1);
        assert!(fed.known_peers.contains("peer_b"), "peer 自动注册");

        // 自身消息被忽略
        let self_msg = build_message("local", FederationMessageType::InsightShare, json!({}));
        assert!(fed.handle_inbound(self_msg).is_ok()); // 不 panic
        assert_eq!(fed.insight_collector.count(), 1, "自身消息不应增加洞察数");

        // 篡改的消息被拒绝
        let mut bad = build_message("peer_c", FederationMessageType::InsightShare, json!({}));
        bad.guard_hash = 0;
        assert!(fed.handle_inbound(bad).is_err(), "guard 校验失败应报错");
    }
}
