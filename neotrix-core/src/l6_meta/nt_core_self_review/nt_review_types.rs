//! Self-review data types — findings, reports, config, gate shape.
//! Pure move from `mod.rs` (God-file split) — zero behavior change.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use neotrix_types::shared::Severity;
use serde::{Deserialize, Serialize};

// ─── moved: mod.rs:27-58 ReviewFinding/SelfReviewReport ───
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReviewReport {
    pub findings: Vec<ReviewFinding>,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
}

impl SelfReviewReport {
    pub fn is_pass(&self) -> bool {
        self.failed == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Self-review: {} passed, {} failed, {} warnings — overall {}",
            self.passed,
            self.failed,
            self.warnings,
            if self.is_pass() { "PASS" } else { "FAIL" }
        )
    }
}

// ─── moved: mod.rs:60-93 BlastRadiusReport/BlastRisk ───
/// Blast-radius report — estimates cross-file impact of findings.
/// Inspired by Revet's blast-radius summary (deterministic, risk-scored).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlastRadiusReport {
    /// Total source files scanned
    pub files_scanned: usize,
    /// Files with at least one finding
    pub affected_files: usize,
    /// Module boundary crossings (src/domain-a -> src/domain-b imports)
    pub module_crossings: usize,
    /// Risk level summary
    pub risk: BlastRisk,
    /// Per-domain finding density
    pub domain_density: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlastRisk {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for BlastRisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlastRisk::Low => write!(f, "LOW"),
            BlastRisk::Medium => write!(f, "MEDIUM"),
            BlastRisk::High => write!(f, "HIGH"),
            BlastRisk::Critical => write!(f, "CRITICAL"),
        }
    }
}

// ─── moved: mod.rs:95-154 ArchLayer ───
/// Architecture depth category — which NeoTrix domain layer a module belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArchLayer {
    L0Core,
    L1Act,
    L2World,
    L3Memory,
    L4Cognition,
    L5Prm,
    L6Self,
    L7Capability,
    L8Seal,
    L9Transcendent,
    Unknown,
}

impl ArchLayer {
    /// 判定一个源文件属于哪一层。
    ///
    /// # 2026-10-07（裁定 5A 第一批）：补真实目录名
    ///
    /// ⛔ **原实现只认 `l0_core` / `l1_body` / `l2_world` / `l3_memory` /
    /// `l4_cognition` / `l5_prm` / `l6_self` / `l7_capability` / `l8_seal` /
    /// `l9_transcendent` —— 这 10 个目录名在本仓**一个都不存在**
    /// （实测 2026-10-07）。真实层目录是 `l0_substrate` / `l1_action` /
    /// `l2_perception` / `l3_embodiment` / `l4_emotion` / `l5_cognition` /
    /// `l6_meta`。
    /// ⇒ 配合 `check_architecture_layer_depth` 里 `read_dir` 跳过目录，
    ///   **可分类文件实测为 0** ⇒ 该检查结构上不可能失败（见 roadmap 5C/5A）。
    ///
    /// **旧模式全部保留**（不删）—— 因为 `tests.rs:80-104` 的
    /// `test_arch_layer_detection` 断言的就是那些路径，且它们是本仓
    /// 历史词汇的一部分（registry 的能力高度轴也用 `l0primitive` 一类命名，
    /// 见 `ROADMAP-REDUNDANCY-FLAT-MISALIGN` §L10 的三轴表）。
    /// ⛔ 保留 ≠ 认可：这是**第四套词汇**与第三套并存的历史包袱，
    /// 归一属 roadmap T0-4（层词汇裁决），本轮只让检测能工作。
    ///
    /// ⚠️ **映射不是语义等价**：`l3_embodiment`（具体化/盾）被映到 `L3Memory`
    /// 是因为枚举里没有 embodiment 变体；`l4_emotion` 映到 `L4Cognition` 同理。
    /// 这两处**分层语义是错的**，但改枚举会动 10+ 处 `layer_index()` 匹配，
    /// 属 T0-4 的范围。此处只保证「能分类」，不宣称「分类正确」。
    pub fn from_path(path: &Path) -> Self {
        let p = path.to_string_lossy();
        // ── 真实层目录（先匹配，因为它们是活的）─────────────────────
        // ⛔ 注意：`l0_substrate` 等必须先于 `/core/` 判定，否则
        // `nt_core_*` 模块里的 `/core/` 子串会误抓。
        if p.contains("/l0_substrate/") || p.contains("l0_substrate") {
            Self::L0Core
        } else if p.contains("/l1_action/") || p.contains("l1_action") {
            Self::L1Act
        } else if p.contains("/l2_perception/") || p.contains("l2_perception") {
            Self::L2World
        } else if p.contains("/l3_embodiment/") || p.contains("l3_embodiment") {
            Self::L3Memory
        } else if p.contains("/l4_emotion/") || p.contains("l4_emotion") {
            Self::L4Cognition
        } else if p.contains("/l5_cognition/") || p.contains("l5_cognition") {
            Self::L5Prm
        } else if p.contains("/l6_meta/") || p.contains("l6_meta") {
            Self::L6Self
        }
        // ── 历史词汇（保留，见上方说明）─────────────────────────────
        else if p.contains("l0_core") || p.contains("/core/") {
            Self::L0Core
        } else if p.contains("l1_body") || p.contains("l1_act") {
            Self::L1Act
        } else if p.contains("l2_world") {
            Self::L2World
        } else if p.contains("l3_memory") {
            Self::L3Memory
        } else if p.contains("l4_cognition") {
            Self::L4Cognition
        } else if p.contains("l5_prm") || p.contains("nt_core_prm") {
            Self::L5Prm
        } else if p.contains("l6_self") || p.contains("l6_autonomic") {
            Self::L6Self
        } else if p.contains("l7_capability") {
            Self::L7Capability
        } else if p.contains("l8_autonomic") || p.contains("l8_seal") {
            Self::L8Seal
        } else if p.contains("l9_transcendent") {
            Self::L9Transcendent
        } else {
            Self::Unknown
        }
    }

    pub fn layer_index(&self) -> i32 {
        match self {
            ArchLayer::L0Core => 0,
            ArchLayer::L1Act => 1,
            ArchLayer::L2World => 2,
            ArchLayer::L3Memory => 3,
            ArchLayer::L4Cognition => 4,
            ArchLayer::L5Prm => 5,
            ArchLayer::L6Self => 6,
            ArchLayer::L7Capability => 7,
            ArchLayer::L8Seal => 8,
            ArchLayer::L9Transcendent => 9,
            ArchLayer::Unknown => -1,
        }
    }
}

// ─── moved: mod.rs:156-209 SelfReviewConfig ───
#[derive(Debug, Clone)]
pub struct SelfReviewConfig {
    pub unwrap_max: usize,
    pub expect_max: usize,
    pub todo_max: usize,
    pub unimplemented_max: usize,
    pub allow_dead_max: usize,
    pub empty_match_max: usize,
    pub index_multiplier: usize,
    pub index_absolute: usize,
    pub lock_unwrap_max: usize,
    pub exit_max: usize,
    pub observer_quality_threshold: f64,
    pub uncovered_test_max: usize,
    pub unwrap_in_lazy_max: usize,
    pub unused_imports_max: usize,
    pub karpathy_simplicity_max: usize,
    pub karpathy_surgical_files: usize,
    pub karpathy_surgical_lines: usize,
    pub karpathy_complexity_max: usize,
    pub karpathy_goal_driven_max: usize,
    pub seal_stub_max: usize,
    pub min_test_line_count: usize,
    pub scan_safety_bound: usize,
}

impl Default for SelfReviewConfig {
    fn default() -> Self {
        Self {
            unwrap_max: 120,
            expect_max: 50,
            todo_max: 5,
            unimplemented_max: 3,
            allow_dead_max: 50,
            empty_match_max: 15,
            index_multiplier: 5,
            index_absolute: 50,
            lock_unwrap_max: 3,
            exit_max: 3,
            observer_quality_threshold: 0.3,
            uncovered_test_max: 5,
            unwrap_in_lazy_max: 3,
            unused_imports_max: 20,
            karpathy_simplicity_max: 30,
            karpathy_surgical_files: 10,
            karpathy_surgical_lines: 500,
            karpathy_complexity_max: 20,
            karpathy_goal_driven_max: 30,
            seal_stub_max: 3,
            min_test_line_count: 50,
            scan_safety_bound: 300,
        }
    }
}

// ─── moved: mod.rs:211-232 SelfReviewGate struct+Default ───
pub struct SelfReviewGate {
    pub strict_mode: bool,
    pub findings: Vec<ReviewFinding>,
    /// Configurable threshold overrides
    pub config: SelfReviewConfig,
    /// Optional observer feedback: quality score from OneObserver (0.0–1.0)
    pub observer_quality: Option<f64>,
    /// Optional observer patterns detected
    pub observer_patterns: Vec<String>,
}

impl Default for SelfReviewGate {
    fn default() -> Self {
        Self {
            strict_mode: true,
            findings: Vec::new(),
            config: SelfReviewConfig::default(),
            observer_quality: None,
            observer_patterns: Vec::new(),
        }
    }
}

// ─── moved: mod.rs:1633-1646 PatternConfig/PatternMatch ───
#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub name: String,
    pub pattern: String,
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub pattern_name: String,
    pub matched_text: String,
}

#[cfg(test)]
mod arch_depth_probe {
    use super::*;
    use super::super::nt_review_runner::*;

    /// 裁定 5A 的验收探针：确认 `check_architecture_layer_depth` 真的检到东西了。
    ///
    /// ⛔ 上一轮（5C）这个检查报「0 violations」而**实际一个文件都没检到** ——
    /// 那正是本仓最熟的那个病（报 PASS 却结构上不可能失败，LESSONS L8）。
    /// ⇒ 本探针断言的是**可分类文件数 > 0**，而不是「违规数 = 0」
    ///    （后者在门修好之前恰好也是 0，无法区分"检了且干净"与"没检"）。
    #[test]
    fn arch_depth_actually_classifies_files() {
        let mut gate = SelfReviewGate::new(true);
        gate.run_all();
        let arch: Vec<_> = gate
            .findings
            .into_iter()
            .filter(|f| f.category.contains("arch_depth"))
            .collect();
        for f in &arch {
            println!("[{}] {}", f.severity, f.message);
        }
        assert!(
            arch.iter().any(|f| f.message.contains("classifiable files")),
            "arch_depth 报告必须含可分类文件数 —— 否则无法区分「检了 N 个、发现 0」\
             与「一个都没检到」（L8 判据）。实际报告：{arch:?}"
        );
        assert!(
            !arch.iter().any(|f| f.message.contains("NOT CHECKED")),
            "arch_depth 不应再报 NOT CHECKED —— from_path 已指向真实层目录"
        );
        // ⭐ 2026-10-07 追加：**检出数必须落在窄区间内**。
        // 门在修 `from_path` 之后一度报 401，其中 400 是假阳性（判据方向反了
        // + `core::` 子串误判）；修完是 6。⇒ 区间卡住，防止将来再次漂移到
        // "几百处"或"0 处"而无人察觉 —— 两者都是本仓最熟的那个病。
        let n: usize = arch
            .iter()
            .filter_map(|f| {
                f.message
                    .split("Architecture depth: ")
                    .nth(1)
                    .and_then(|s| s.split(' ').next())
                    .and_then(|s| s.parse::<usize>().ok())
            })
            .sum();
        assert!(
            (1..=40).contains(&n),
            "arch_depth 检出数应落在 1..=40（当前实测 6，与权威门 baseline 的 13 条同批债）。\
             实测 {n} —— 偏大 ⇒ 多半是 `core::` 子串误判或判据方向反了；\
             偏小 ⇒ 可能又退回「检不到东西」。"
        );
    }
}
