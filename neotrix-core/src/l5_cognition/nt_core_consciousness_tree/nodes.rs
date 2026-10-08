use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::contract::*;
use super::types::*;

/// Provider Benchmark — Unstract LLM Challenge pattern
pub use neotrix_types::consciousness::ProviderBenchmark;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityCategory {
    Perceive,
    Understand,
    Reason,
    Model,
    Synthesize,
    Execute,
    Verify,
    Remember,
    Coordinate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAtom {
    pub name: String,
    pub tier: SelfTestTier,
    pub branch: BranchKind,
    pub category: CapabilityCategory, // MCA 9-layer capability classification
    pub self_test_fn: Option<String>, // Function name for dynamic lookup
    pub last_score: f64,
    pub generation: u64,
    pub mandatory: bool, // If true, must pass for branch to produce fruit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfTestTier {
    T1Existence,    // impl SelfTest exists
    T2Registration, // Registered in SelfTestRegistry (run.rs + pipeline.rs)
    T3Production,   // Detection function consumed by non-test code
}

impl Default for CapabilityAtom {
    fn default() -> Self {
        Self {
            name: String::new(),
            tier: SelfTestTier::T1Existence,
            branch: BranchKind::Core,
            category: CapabilityCategory::Perceive,
            self_test_fn: None,
            last_score: 0.0,
            generation: 0,
            mandatory: false,
        }
    }
}
/// Evolution Fruit — replaces CapabilityFruit with verifiable evolution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionFruit {
    pub name: String,
    pub source_branch: BranchKind,
    pub description: String,
    pub produced_at_cycle: u64,
    pub quality: f64,
    // New verifiable fields
    pub claim: String,           // What capability this fruit claims to provide
    pub evidence: EvidenceChain, // Cryptographic proof chain (WARC/SHA-256/JSONL)
    pub stop_rule: StopRule,     // Inherited from contract
    pub benchmark: ProviderBenchmark, // LLM Challenge results (Unstract pattern)
    pub generation: u64,         // MetaClaw versioning
}

impl Default for EvolutionFruit {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_branch: BranchKind::Core,
            description: String::new(),
            produced_at_cycle: 0,
            quality: 0.0,
            claim: String::new(),
            evidence: EvidenceChain::default(),
            stop_rule: StopRule::default(),
            benchmark: ProviderBenchmark::default(),
            generation: 0,
        }
    }
}

/// Evidence Chain — Claude-OSINT pattern: WARC + SHA-256 + JSONL run_id
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceChain {
    pub warc_path: Option<String>, // WARC archive path
    pub sha256: Option<String>,    // SHA-256 of artifact
    pub run_id: Option<String>,    // JSONL run_id for traceability
    pub timestamp: u64,
    pub tool_versions: Vec<String>, // Tool versions used
}

impl EvidenceChain {
    pub fn new(run_id: String, sha256: String) -> Self {
        Self {
            warc_path: None,
            sha256: Some(sha256),
            run_id: Some(run_id),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            tool_versions: vec!["neotrix".into()],
        }
    }

    /// Build an evidence chain from real branch state instead of placeholder values.
    /// Fingerprints the branch's self-test results, health, maturity, and absorbed
    /// capabilities so the chain reflects actual evolution output (Claude-OSINT pattern).
    pub fn from_branch_state(cycle: u64, kind: &BranchKind, branch: &CapabilityBranch) -> Self {
        let payload = format!(
            "cycle={}|kind={}|tests={}|health={:.6}|maturity={:.6}|absorbed={:?}",
            cycle,
            kind.label(),
            branch.self_test_count,
            branch.health,
            branch.maturity_score(),
            branch.absorbed_capabilities,
        );
        let sha256 = hex::encode(Sha256::digest(payload.as_bytes()));
        let run_id = format!("cycle-{}-{}", cycle, kind.label());
        Self {
            warc_path: None,
            sha256: Some(sha256),
            run_id: Some(run_id),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            tool_versions: vec![format!("neotrix-cycle-{}", cycle)],
        }
    }
}

#[derive(Debug, Clone)]
/// 演化趋势预测 — 由 nt_core_forecast 引擎在 growth cycle 中生成。
///
/// 意识体维度升维 (cycle 251 经验): 让 ConsciousnessTree 从"评估当前状态"
/// 升级为"预测演化趋势"。基于各 branch 当前健康/迷雾/果实数据, 用 E8 溯因
/// 桥 + 情景树预测下一 cycle 的演化方向, 使演化决策具备前瞻性而非仅回看。
pub struct EvolutionForecast {
    /// 预测目标 (如 "overall-evolution" / "NT-IO")
    pub target: String,
    /// 预测方向: +1 利多(健康上升) / -1 利空(健康下降) / 0 震荡
    pub direction: f64,
    /// 校准置信度 (0..1)
    pub confidence: f64,
    /// 弃权信号 — 信息不足时置 true (不参与决策)
    /// ⭐ 审计裁定 2026-10-07：`dead-flag` 判此字段零读点 —— **门是对的**。
    ///
    /// 我一度以为门误报（`lifecycle.rs` 有 4 处 `.abstain`），核实后：
    /// 那 4 处属于 **`nt_forecast::Forecast`**（另一文件、**同名** struct），
    /// 本字段属 **`EvolutionForecast`** ⇒ **同名异类型**，互不算读点。
    ///
    /// ⇒ 这是本会话第 4 次「以为门错、实则门对」：
    ///   同名字段跨文件/跨 struct 会互相「救活」，掩盖真死字段。
    /// ⇒ 门按 `(文件, token)` 计数 + `name_decls>1` 时**不回退**，
    ///   这个设计恰恰**避免了**同名互相救活 —— 是对的，别改。
    // ⭐ **活的（2026-10-07 切片复核）**：`lifecycle.rs:189/:985` 用同 struct
    //   的 `forecast.abstain` 判定 —— `EvolutionForecast` 的真实消费点。
    // ⛔ 死开关门把它判成「零读点」：它按「声明文件内读点」计数，而读点在
    //   兄弟文件 lifecycle.rs ⇒ **跨文件消费被当成没消费**（与 chunk_size/concurrency
    //   同类，门的第二类误报）。不删。
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
    pub abstain: bool,
    /// 情景树叶子概率摘要 (bull/bear/sideways)
    pub scenario_probs: Vec<(String, f64)>,
    /// 置信理由
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct GrowthReport {
    pub phase0_contract: Option<String>,
    pub phase1_absorbed: u64,
    pub phase2_phi: f64,
    pub phase3_fruits: usize,
    pub phase4_guidance: usize,
    pub phase6_fulfillment: Option<ContractFulfillment>,
    pub phase7_drift: Option<DriftReport>,
    /// 全仓加权雾和 (Σ w × fog.level) — 迷雾地图主量纲
    pub weighted_fog_sum: f64,
    /// 每域迷雾浓度摘要 (branch label → level)
    pub fog_by_branch: std::collections::BTreeMap<String, f64>,
    /// 演化趋势预测 (nt_core_forecast 接线, 意识体维度升维)
    pub evolution_forecast: Option<EvolutionForecast>,
}
