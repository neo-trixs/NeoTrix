//! `nt_judge` — Judge 影子内核（EVO-02 mu 式 judge-kernel，影子模式）.
//!
//! 影子模式：只记录、不拦截。任何输入的判决都会写入 [`JudgeLedger`]，
//! 但 [`JudgeLedger::intercept`] 与 [`JudgeLedger::should_intercept`] 恒为 `false`。
//!
//! 全部同步纯逻辑：时间由调用方传入（秒级时间戳），无 I/O、无时钟调用、无线程。

use std::collections::VecDeque;

/// 账本容量上限：最多保留条目数，超限丢弃最旧。
pub const JUDGE_LEDGER_CAP: usize = 500;

/// 默认单 chunk 最大字符数（按 `char` 计），超过则 [`JudgeVerdict::Archive`]。
pub const DEFAULT_MAX_CHUNK_CHARS: usize = 32_768;

/// 默认敏感词表（小写，大小写不敏感子串匹配）。
pub const DEFAULT_SENSITIVE_TERMS: &[&str] = &["password", "secret", "api_key", "private_key"];

/// 准入判决：带理由的三态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeVerdict {
    /// 准入：可进入后续管线。
    Admit {
        /// 判决理由。
        reason: String,
    },
    /// 归档：保留但不进入热管线（如超长/敏感）。
    Archive {
        /// 判决理由。
        reason: String,
    },
    /// 跳过：空 chunk 等无意义输入。
    Skip {
        /// 判决理由。
        reason: String,
    },
}

impl JudgeVerdict {
    /// 构造 [`JudgeVerdict::Admit`]。
    #[must_use]
    pub fn admit(reason: impl Into<String>) -> Self {
        Self::Admit {
            reason: reason.into(),
        }
    }

    /// 构造 [`JudgeVerdict::Archive`]。
    #[must_use]
    pub fn archive(reason: impl Into<String>) -> Self {
        Self::Archive {
            reason: reason.into(),
        }
    }

    /// 构造 [`JudgeVerdict::Skip`]。
    #[must_use]
    pub fn skip(reason: impl Into<String>) -> Self {
        Self::Skip {
            reason: reason.into(),
        }
    }

    /// 判决理由。
    #[must_use]
    pub fn reason(&self) -> &str {
        match self {
            Self::Admit { reason } | Self::Archive { reason } | Self::Skip { reason } => reason,
        }
    }

    /// 是否为 [`JudgeVerdict::Admit`]。
    #[must_use]
    pub fn is_admit(&self) -> bool {
        matches!(self, Self::Admit { .. })
    }

    /// 是否为 [`JudgeVerdict::Archive`]。
    #[must_use]
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Archive { .. })
    }

    /// 是否为 [`JudgeVerdict::Skip`]。
    #[must_use]
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip { .. })
    }
}

/// Tool 输出 chunk 准入判决器（纯函数，按长度 / 敏感词规则）。
#[derive(Debug, Clone)]
pub struct ChunkAdmission {
    max_chars: usize,
    sensitive_terms: Vec<String>,
}

impl ChunkAdmission {
    /// 以显式策略构造；敏感词统一归一化为小写，空串词条被忽略。
    #[must_use]
    pub fn new(max_chars: usize, sensitive_terms: Vec<String>) -> Self {
        let normalized = sensitive_terms
            .into_iter()
            .map(|term| term.to_lowercase())
            .filter(|term| !term.is_empty())
            .collect();
        Self {
            max_chars,
            sensitive_terms: normalized,
        }
    }

    /// 默认策略：[`DEFAULT_MAX_CHUNK_CHARS`] + [`DEFAULT_SENSITIVE_TERMS`]。
    #[must_use]
    pub fn default_policy() -> Self {
        Self {
            max_chars: DEFAULT_MAX_CHUNK_CHARS,
            sensitive_terms: DEFAULT_SENSITIVE_TERMS
                .iter()
                .map(|term| (*term).to_string())
                .collect(),
        }
    }

    /// 配置的最大字符数。
    #[must_use]
    pub fn max_chars(&self) -> usize {
        self.max_chars
    }

    /// 配置的敏感词表（小写）。
    #[must_use]
    pub fn sensitive_terms(&self) -> &[String] {
        &self.sensitive_terms
    }

    /// 纯函数判决：空 → `Skip`；超长 → `Archive`；命中敏感词 → `Archive`；其余 → `Admit`。
    #[must_use]
    pub fn judge(&self, chunk: &str) -> JudgeVerdict {
        if chunk.trim().is_empty() {
            return JudgeVerdict::skip("empty chunk");
        }
        let len = chunk.chars().count();
        if len > self.max_chars {
            return JudgeVerdict::archive("oversize chunk");
        }
        let lower = chunk.to_lowercase();
        let hit = self
            .sensitive_terms
            .iter()
            .any(|term| lower.contains(term.as_str()));
        if hit {
            return JudgeVerdict::archive("sensitive term matched");
        }
        JudgeVerdict::admit("ok")
    }
}

/// 默认策略的便捷纯函数。
#[must_use]
pub fn judge_chunk_default(chunk: &str) -> JudgeVerdict {
    ChunkAdmission::default_policy().judge(chunk)
}

/// Forget 墓碑：已遗忘条目的可审计存根。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tombstone {
    /// 被遗忘条目的 id。
    pub id: String,
    /// 遗忘原因。
    pub reason: String,
    /// 墓碑创建时间（秒级时间戳，由调用方传入）。
    pub created_secs: u64,
}

impl Tombstone {
    /// 构造墓碑（纯构造，不做 I/O；空 id 视为无效，见 [`Tombstone::is_valid`]）。
    #[must_use]
    pub fn new(id: impl Into<String>, reason: impl Into<String>, created_secs: u64) -> Self {
        Self {
            id: id.into(),
            reason: reason.into(),
            created_secs,
        }
    }

    /// 墓碑是否有效：`id` 去空白后非空。
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.id.trim().is_empty()
    }
}

/// 账本条目（单条判决记录）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeEntry {
    /// 单调序列号（从 0 起）。
    pub seq: u64,
    /// 被判决 chunk 的 id。
    pub chunk_id: String,
    /// **judged**：judge 给出的判决（含理由）。
    pub verdict: JudgeVerdict,
    /// **source**：这条判决**从哪里来**（影子 / 已激活 / 人工）。
    ///
    /// 2026-10-06（B5 = M-4「影子→active 升级路径」）：没有它就无法回答
    /// 「这条判决是 shadow 猜的，还是 active 真拦的」⇒ 切active 时
    /// 两类数据混在一起，对比 `judged` vs `outcome` 就**没有意义**。
    pub source: JudgeSource,
    /// **outcome**：真实发生的结果。
    ///
    /// ⛔ 影子模式下判决当刻**拿不到真实结果**（我们没按它行动）⇒
    ///   初始恒为 `None`，由 [`JudgeLedger::record_outcome`] 在结果已知时回填。
    ///   ⚠️ 刻意**不**在此处填 `Judged(verdict.clone())` —— 那会让
    ///   「judged 与 outcome 一致」变成**恒真**，从而**伪装成已验证**，
    ///   而这正是 mu 自己的教训（`judged` vs 真实 `outcome` 对比才有意义）。
    pub outcome: Option<JudgeOutcome>,
    /// 记录时间（秒级时间戳，由调用方传入）。
    pub recorded_secs: u64,
}

/// 判决来源（M-4 影子→active 的关键维度）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeSource {
    /// 影子：调 judge、记 ledger，但**返回 fallback**（不据此行动）。
    Shadow,
    /// 已激活：判决真正拦截了行为。
    Active,
    /// 人工判定（owner 裁决），不经judge。
    Manual,
}

/// 真实发生的结果（M-4 的对比基准面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeOutcome {
    /// 真实结果与judge 判决**一致**。
    Matched,
    /// 真实结果与judge 判决**不一致**（⇒ judge 在这个case 上判错）。
    Mismatched {
        /// 实际发生了什么（供人复核）。
        detail: String,
    },
    /// 真实结果**无法判定**（不可归因）⇒ 不计入一致率。
    Unknown,
}

/// 影子模式开关：恒为 `true`（只记录、不拦截）。
pub const SHADOW_MODE: bool = true;

/// Append-only 内存账本：`record` / `query`，最多 [`JUDGE_LEDGER_CAP`] 条，超限丢弃最旧。
#[derive(Debug, Clone, Default)]
pub struct JudgeLedger {
    entries: VecDeque<JudgeEntry>,
    next_seq: u64,
}

impl JudgeLedger {
    /// 空账本。
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            next_seq: 0,
        }
    }

    /// 容量上限（恒为 [`JUDGE_LEDGER_CAP`]）。
    #[must_use]
    pub fn capacity(&self) -> usize {
        JUDGE_LEDGER_CAP
    }

    /// 当前条目数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 下一个序列号（纯观测，不推进）。
    #[must_use]
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// 记录一条判决，返回其序列号；满时先丢弃最旧一条。
    /// 记一条判决（M-4：影子→active 的记录面）。
    ///
    /// `source` 必须由调用方**显式**给出：它决定这条判决将来能否参与
    /// 「judged vs outcome」对比（影子猜的和 active 拦的不是一回事）。
    /// `outcome` 初始恒为 `None` —— 影子模式下判决当刻**没有**真实结果。
    pub fn record(
        &mut self,
        chunk_id: &str,
        verdict: JudgeVerdict,
        source: JudgeSource,
        recorded_secs: u64,
    ) -> u64 {
        if self.entries.len() >= JUDGE_LEDGER_CAP {
            let _ = self.entries.pop_front();
        }
        let seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        self.entries.push_back(JudgeEntry {
            seq,
            chunk_id: chunk_id.to_string(),
            verdict,
            source,
            outcome: None,
            recorded_secs,
        });
        seq
    }

    /// 按序列号查询（克隆返回，保持纯逻辑无借用泄露）。
    #[must_use]
    /// 回填某条判决的**真实结果**（M-4：`judged` vs `outcome` 的对比面）。
    ///
    /// ## 为什么要单独一个入口
    ///
    /// 判决当刻拿不到真实结果（影子模式下我们**没按它行动**）⇒ 不能在
    /// `record` 里一并写入。⇒ 必须允许「先记判决，事后知道结果再回填」。
    ///
    /// ## 幂等与诚实
    ///
    /// -重复回填以**最后一次**为准（真实结果可能被修正）。
    /// - 找不到 `seq` ⇒ 返回 `Err`，**不静默新建**：凭空造一条会让
    ///   一致率统计出现无源数据。
    pub fn record_outcome(&mut self, seq: u64, outcome: JudgeOutcome) -> Result<(), String> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.seq == seq)
            .ok_or_else(|| format!("ledger 无seq={seq} 的判决，不凭空新建"))?;
        entry.outcome = Some(outcome);
        Ok(())
    }

    pub fn query(&self, seq: u64) -> Option<JudgeEntry> {
        self.entries
            .iter()
            .find(|entry| entry.seq == seq)
            .cloned()
    }

    /// 全量快照（按记录顺序，最旧在前）。
    #[must_use]
    pub fn query_all(&self) -> Vec<JudgeEntry> {
        self.entries.iter().cloned().collect()
    }

    /// 按判决类别过滤快照。
    #[must_use]
    pub fn query_by_admit(&self, admit: bool) -> Vec<JudgeEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.verdict.is_admit() == admit)
            .cloned()
            .collect()
    }

    /// 影子模式总开关：恒为 `false`（从不拦截）。
    #[must_use]
    pub fn intercept(&self) -> bool {
        false
    }

    /// 针对单条判决是否拦截：影子模式下恒为 `false`（只记录）。
    #[must_use]
    pub fn should_intercept(&self, _verdict: &JudgeVerdict) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_boundary_empty_skip_normal_admit_oversize_archive() {
        let policy = ChunkAdmission::new(8, Vec::new());
        assert!(policy.judge("").is_skip());
        assert!(policy.judge("   \n\t  ").is_skip());
        let admitted = policy.judge("12345678");
        assert!(admitted.is_admit());
        assert_eq!(admitted.reason(), "ok");
        let archived = policy.judge("123456789");
        assert!(archived.is_archive());
        assert_eq!(archived.reason(), "oversize chunk");
    }

    #[test]
    fn admission_sensitive_term_case_insensitive_archive() {
        let policy = ChunkAdmission::default_policy();
        let verdict = policy.judge("the API_KEY leaked here");
        assert!(verdict.is_archive());
        assert_eq!(verdict.reason(), "sensitive term matched");
        let clean = judge_chunk_default("hello world");
        assert!(clean.is_admit());
    }

    #[test]
    fn tombstone_fields_and_validity() {
        let stone = Tombstone::new("chunk-42", "user forget request", 1_700_000_001);
        assert_eq!(stone.id, "chunk-42");
        assert_eq!(stone.reason, "user forget request");
        assert_eq!(stone.created_secs, 1_700_000_001);
        assert!(stone.is_valid());
        let invalid = Tombstone::new("   ", "empty id", 0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn ledger_cap_evicts_oldest_and_query_works() {
        let mut ledger = JudgeLedger::new();
        assert!(ledger.is_empty());
        assert_eq!(ledger.capacity(), JUDGE_LEDGER_CAP);
        for i in 0..(JUDGE_LEDGER_CAP + 10) {
            let id = std::format!("c-{i}");
            let _ = ledger.record(&id, JudgeVerdict::admit("ok"), JudgeSource::Shadow, 1_700_000_000);
        }
        assert_eq!(ledger.len(), JUDGE_LEDGER_CAP);
        assert!(ledger.query(0).is_none());
        assert!(ledger.query(9).is_none());
        let first_kept = ledger.query(10);
        assert!(first_kept.is_some());
        assert_eq!(first_kept.map(|entry| entry.chunk_id), Some("c-10".to_string()));
        assert_eq!(ledger.query_all().len(), JUDGE_LEDGER_CAP);
        assert_eq!(ledger.query_all().first().map(|entry| entry.seq), Some(10));
    }

    #[test]
    fn shadow_mode_never_intercepts() {
        let ledger = JudgeLedger::new();
        assert!(SHADOW_MODE);
        assert!(!ledger.intercept());
        assert!(!ledger.should_intercept(&JudgeVerdict::admit("ok")));
        assert!(!ledger.should_intercept(&JudgeVerdict::archive("sensitive term matched")));
        assert!(!ledger.should_intercept(&JudgeVerdict::skip("empty chunk")));
    }

    #[test]
    fn ledger_record_seq_monotonic_and_filter() {
        let mut ledger = JudgeLedger::new();
        let seq0 = ledger.record("a", JudgeVerdict::admit("ok"), JudgeSource::Shadow, 100);
        let seq1 = ledger.record("b", JudgeVerdict::skip("empty chunk"), JudgeSource::Active, 101);
        assert_eq!(seq0, 0);
        assert_eq!(seq1, 1);
        assert_eq!(ledger.next_seq(), 2);
        assert_eq!(ledger.query_by_admit(true).len(), 1);
        assert_eq!(ledger.query_by_admit(false).len(), 1);
    }
}

#[cfg(test)]
mod m4_ledger_tests {
    use super::{JudgeLedger, JudgeOutcome, JudgeSource, JudgeVerdict};

    /// ⭐⭐⭐ **B5/M-4 验收判据（排期 §4 原文）：ledger 每条同时含
    /// `judged`/`outcome`/`source`。**
    #[test]
    fn 每条判决都带judged_outcome_source() {
        let mut l = JudgeLedger::new();
        let s0 = l.record("c0", JudgeVerdict::admit("ok"), JudgeSource::Shadow, 100);
        let s1 = l.record("c1", JudgeVerdict::archive("long"), JudgeSource::Active, 101);

        for seq in [s0, s1] {
            let e = l
                .entries
                .iter()
                .find(|e| e.seq == seq)
                .expect("entry 应存在");
            // judged：就是 verdict
            assert!(!format!("{:?}", e.verdict).is_empty(), "judged 不得为空");
            // source：显式给出、不是默认值糊弄
            assert!(
                matches!(e.source, JudgeSource::Shadow | JudgeSource::Active | JudgeSource::Manual),
                "source 必须是三态之一"
            );
            // outcome：**初始必须是 None** —— 影子模式当刻没有真实结果
            assert!(
                e.outcome.is_none(),
                "★ 判决当刻的 outcome 必须是 None（伪装成已知=伪造验证）"
            );
        }
        // source 确实被区分了（不是同一个值糊过去）
        assert_eq!(l.entries[0].source, JudgeSource::Shadow);
        assert_eq!(l.entries[1].source, JudgeSource::Active);
    }

    /// ⭐⭐ **回填后 outcome 可见，且不一致能被记出来**（这才是 M-4 的用途）。
    #[test]
    fn 回填outcome_后可对比judged() {
        let mut l = JudgeLedger::new();
        let seq = l.record("c", JudgeVerdict::admit("ok"), JudgeSource::Shadow, 1);
        assert!(l.entries[0].outcome.is_none());

        l.record_outcome(seq, JudgeOutcome::Mismatched { detail: "实际被丢弃".into() })
            .expect("回填应成功");
        let e = &l.entries[0];
        assert!(
            matches!(e.outcome, Some(JudgeOutcome::Mismatched { .. })),
            "★ 不一致的判决必须能被记下来 —— 否则切active 时无从发现判错"
        );
    }

    /// ⭐⭐ **回填不存在的 seq 必须报错，不得凭空新建**（否则一致率出现无源数据）。
    #[test]
    fn 回填不存在的seq_报错且不新建() {
        let mut l = JudgeLedger::new();
        let before = l.len();
        assert!(l.record_outcome(999, JudgeOutcome::Matched).is_err());
        assert_eq!(l.len(), before, "★ 失败的回填不得改变 ledger 长度");
    }
}
