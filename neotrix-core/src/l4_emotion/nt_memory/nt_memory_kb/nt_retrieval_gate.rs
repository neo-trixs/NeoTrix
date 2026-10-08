#![forbid(unsafe_code)]

//! **检索准入闸** —— 「这条消息**需要**记忆吗?」在**碰存储之前**回答。
//!
//! ## 为什么需要它（2026-10-03 实测）
//!
//! **本仓的真实空白**：实测 `hybrid_retrieval/`（`bm25_search` / `entity_search` /
//! `semantic_search` / `fusion_engine` / `temporal_scoring`）**全是检索原语**，
//! 且 ⛔ 全模块与全仓**没有任何**「是否值得检索」的判定
//! （`rg -in 'need.*memor|should_retrieve|necess|gate'` 于该目录 = **0 命中**）。
//! 而 `nt_memory_dual_brain.rs::recall_ltm:138` **无条件** `KnowledgeBase::open(None)`
//! ⇒ **每轮都碰存储**。
//!
//! ## 论证来自外部吸收（`waku-agent`，MIT，一手源码）
//!
//! `waku/memory/retrieval_gate.py` 自称「**HERO MOMENT #1**」，其 docstring 的论证：
//!
//! > 「Default-on retrieval is (a) slow — an extra search before every reply — and
//! > (b) worse: **irrelevant memories bias the answer ("over-interpretation")**.」
//!
//! 关键点：**「无关记忆会偏置答案」比「慢」更糟** —— 这是**过度检索**的代价，
//! 而本仓此前只把「慢」当成本。
//!
//! ## 三条移植纪律（每条都来自 waku 的实测，且我方有对应教训）
//!
//! 1. **判据是「反事实必要性」，不是相似度**（waku 原文）
//!    > 「how much **does leaving this one out change the answer**?」
//!    > 本模块 `RetrievalQuestion::OmissionImpact` 表达同一判据。
//!
//! 2. **fail-open，且理由必须写明**（waku 原文）
//!    > 「a slow or broken judge **must never cost a memory**」
//!    > ⇒ 闸失败 ⇒ **照旧检索**（`GateError` ⇒ admit）。
//!    > **对照本仓的 SSRF 守卫是 fail-closed**（失败 = 发出内网请求，**不可撤销**）。
//!    > ⇒ **两边都对**：*失败方向由「失败时损失什么」决定*。
//!    > **「永远 fail-closed」同样是未经论证的教条。**
//!
//! 3. **不发明没有测量的阈值**（守本仓自己的纪律）
//!    waku 能写 `0.5, measured 12 of 12`，是因为它**测过**
//!    （并把单位成本 `260ms / $0.02 per 1000 decisions` 写进 `jev.py:11-12`）。
//!    ⛔ **我方没有任何测量** ⇒ 因此本模块**不提供**任何默认阈值，
//!    默认实现是 **`NoGate`（恒 admit = 现状行为）**。
//!    ⇒ 换言之：**接线本身零行为变化**，直到有人注入一个真正的闸。
//!    这正是 `LESSONS-20260929-checked-is-not-verified.md` 的教训要求
//!    （mu 的准入：成本最高 54% token、收益为零）。

use std::fmt;

/// 闸的判据 —— 表达「**必要性**，不是相似度」。
///
/// 为什么不叫 `Relevance`：相似度高 ≠ 需要记忆。
/// waku 的判据是**反事实**：「抽掉它，答案会变多少？」
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalQuestion {
    /// 主判据：**抽掉这条记忆，答案会变多少？**
    ///
    /// 这是 waku 的原话所对应的形式
    /// （「how much does leaving this one out change the answer?」）。
    OmissionImpact,
}

impl fmt::Display for RetrievalQuestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OmissionImpact => {
                f.write_str("how much does leaving this one out change the answer?")
            }
        }
    }
}

/// 闸的判定结果。
#[derive(Debug, Clone, PartialEq)]
pub enum GateDecision {
    /// 检索（`query` 是闸**顺带产出**的检索词 —— waku 的闸也顺带产出 query）
    Admit { query: String },
    /// 不检索（省一次存储往返）
    Skip { reason: String },
}

/// 闸的失败态 —— **一律 fail-open**（照旧检索）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// 闸自己崩了 / 超时 / 返回了无法解析的答案
    Unavailable(String),
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(d) => write!(f, "检索闸不可用（fail-open ⇒ 照旧检索）: {d}"),
        }
    }
}

/// 检索闸 —— 可插拔，**默认不改变任何行为**。
pub trait RetrievalGate: Send + Sync {
    /// 判定「这条消息需要记忆吗」。**`Err` ⇒ 调用方必须照旧检索。**
    fn decide(&self, message: &str) -> Result<GateDecision, GateError>;
}

/// **默认闸：恒 admit** = 现状行为。
///
/// 为什么默认是它而不是「一个启发式」：
/// ⛔ **我方没有测量**，而 waku 的阈值是实测得来的。
/// 塞一个未测的启发式 = `LESSONS-20260929-checked-is-not-verified.md`
/// 警告的那种「听起来对味」的伪机制。
#[derive(Debug, Clone, Copy, Default)]
pub struct NoGate;

impl RetrievalGate for NoGate {
    fn decide(&self, message: &str) -> Result<GateDecision, GateError> {
        Ok(GateDecision::Admit { query: message.to_owned() })
    }
}

/// **fail-open 的一次性闸**（把失败语义收进类型，避免调用方写错）。
///
/// **为什么单独给这个类型**：本模块最容易被误用的方式是
/// 「拿到 `Err` 就当 Skip」⇒ 那会把**闸的故障**变成**记忆的丢失**，
/// 正是 waku 那句「a slow or broken judge must never cost a memory」
/// 要防的后果。⇒ 这里让「fail-open」成为**唯一可表达的路径**。
pub struct FailOpenGate<G: RetrievalGate> {
    inner: G,
}

impl<G: RetrievalGate> FailOpenGate<G> {
    pub fn new(inner: G) -> Self {
        Self { inner }
    }

    /// 唯一入口：**永不返回 Skip**。
    /// 闸失败 ⇒ `log::warn!` + 退回 `Admit`（用原 query）。
    pub fn decide_or_admit(&self, message: &str) -> GateDecision {
        match self.inner.decide(message) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("[retrieval-gate] {e}");
                GateDecision::Admit { query: message.to_owned() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Broken;
    impl RetrievalGate for Broken {
        fn decide(&self, _m: &str) -> Result<GateDecision, GateError> {
            Err(GateError::Unavailable("模拟闸崩溃".into()))
        }
    }

    struct Skipper;
    impl RetrievalGate for Skipper {
        fn decide(&self, _m: &str) -> Result<GateDecision, GateError> {
            Ok(GateDecision::Skip { reason: "算术题".into() })
        }
    }

    /// **本模块的核心不变量**：坏掉的闸 **绝不** 让人失去记忆。
    /// 这条直接对应 waku 的「must never cost a memory」。
    #[test]
    fn 闸崩溃时必须照旧检索() {
        let g = FailOpenGate::new(Broken);
        match g.decide_or_admit("任意消息") {
            GateDecision::Admit { query } => assert_eq!(query, "任意消息"),
            GateDecision::Skip { .. } => panic!("⛔ 闸崩溃却 Skip 了 —— 这正是 fail-open 要防的后果"),
        }
    }

    /// 默认闸 = 现状行为（恒 admit，query 原样）⇒ **接线零行为变化**
    #[test]
    fn 默认闸恒admit且query原样() {
        match FailOpenGate::new(NoGate).decide_or_admit("当我和 Alex 见面是什么时候？") {
            GateDecision::Admit { query } => assert_eq!(query, "当我和 Alex 见面是什么时候？"),
            GateDecision::Skip { reason } => panic!("默认闸不该 Skip: {reason}"),
        }
    }

    /// 闸说 Skip 就真的省掉检索（否则闸毫无用处）
    #[test]
    fn 闸判跳过时确实跳过() {
        match FailOpenGate::new(Skipper).decide_or_admit("2+2 等于几") {
            GateDecision::Skip { reason } => assert_eq!(reason, "算术题"),
            GateDecision::Admit { .. } => panic!("Skipper 应 Skip"),
        }
    }

    /// 判据字符串就是 waku 的原句 —— 钉住它，防止被无声改成「相关性」
    #[test]
    fn 判据是反事实必要性而非相似度() {
        let s = RetrievalQuestion::OmissionImpact.to_string();
        assert!(s.contains("leaving this one out"), "判据必须是 omission impact: {s}");
        assert!(!s.to_lowercase().contains("relevance"), "⛔ 判据不得退化成相似度: {s}");
    }
}