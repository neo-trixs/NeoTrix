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

// ─── ArchLayer（2026-10-07 裁定 ②B：彻底归一到目录层 7 变体）───
/// NeoTrix 目录分层 —— 与 `neotrix-core/src/l0_substrate`…`l6_meta` **逐字对齐**。
///
/// # 为什么彻底归一（②B）
///
/// 归一前本仓有**四套**层词汇并存（见
/// `ROADMAP-REDUNDANCY-FLAT-MISALIGN-2026-10-07.md` §L10 的三轴表 + 本文件旧版）：
/// | 套 | 取值 | 强制者 |
/// |---|---|---|
/// | 目录层（正典） | `l0_substrate`…`l6_meta`（7） | `check-layer-deps.sh` |
/// | 域 | `NT-CORE`…`NT-REPAIR`（11） | ⛔ 无门 |
/// | 能力高度 | `l0primitive`…`l8autonomic`（11） | ⛔ 无门 |
/// | **本枚举（曾为 ArchLayer）** | `L0Core`…`L9Transcendent`（10） | ⛔ 无门，且**目录名全不存在** |
///
/// 旧枚举的 10 个名字（`l0_core`/`l1_body`/`l2_world`/`l3_memory`/
/// `l4_cognition`/`l5_prm`/`l6_self`/`l7_capability`/`l8_seal`/
/// `l9_transcendent`）**在本仓一个目录都不存在** ⇒ 它既不能分类真实文件，
/// 又与正典层号**语义错位**（旧 `L3Memory` 实际要装 `l3_embodiment` 具体化/盾，
/// 旧 `L4Cognition` 实际要装 `l4_emotion` 情绪/记忆）。
///
/// ⇒ 旧枚举的 3 个多余变体（`L7Capability`/`L8Seal`/`L9Transcendent`，
/// 对应目录不存在）+ 2 个错语义映射，在本轮**一并消除**。
/// ⛔ 保留的**不是**旧词汇，而是**目录层真名** —— 这才叫归一。
///
/// ⛔ **仍未归一的（有意）**：域轴（11）与能力高度轴（11）**不在本轮范围**。
/// 它们描述的是「能力属哪一支」与「能力有多高」，与「代码在第几层」**正交**
/// （实测 255/255 可解析节点里只有 13 格偏离对角 ⇒ 那是一条穿了两个名字的轴）。
/// 强行塞进层枚举会让一个枚举同时表达三件事。⇒ 归一属 roadmap T0-4 后续。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchLayer {
    /// `neotrix-core/src/l0_substrate` —— 基座，全仓都依赖它
    L0Substrate,
    /// `neotrix-core/src/l1_action` —— 动作层
    L1Action,
    /// `neotrix-core/src/l2_perception` —— 感知层
    L2Perception,
    /// `neotrix-core/src/l3_embodiment` —— 具体化层（盾 / OSINT / 交通）
    L3Embodiment,
    /// `neotrix-core/src/l4_emotion` —— 情绪/记忆层
    L4Emotion,
    /// `neotrix-core/src/l5_cognition` —— 认知层
    L5Cognition,
    /// `neotrix-core/src/l6_meta` —— 元/治理层
    L6Meta,
    /// 不在任何层目录内（`neotrix-core/src/*.rs`、`bin/`、`examples/` 等）
    Outside,
}

impl ArchLayer {
    /// 目录层真名 ⇄ 枚举的映射表 —— **归一的唯一定义处**。
    ///
    /// ⛔ 其他地方（`detect_import_layer`、`from_path`、将来的规则注册表）
    /// **必须引用本表**，⛔ 不得各写一份字符串匹配 —— 那正是"四套词汇"的成因。
    pub const REAL_DIRS: &'static [(&'static str, ArchLayer)] = &[
        ("l0_substrate", ArchLayer::L0Substrate),
        ("l1_action", ArchLayer::L1Action),
        ("l2_perception", ArchLayer::L2Perception),
        ("l3_embodiment", ArchLayer::L3Embodiment),
        ("l4_emotion", ArchLayer::L4Emotion),
        ("l5_cognition", ArchLayer::L5Cognition),
        ("l6_meta", ArchLayer::L6Meta),
    ];

    /// 层目录名 → 枚举。**不含 `Outside`**（Outside 只在 `layer_index` 里表达）。
    pub fn from_dir_name(name: &str) -> Option<Self> {
        Self::REAL_DIRS
            .iter()
            .find(|(d, _)| *d == name)
            .map(|(_, l)| *l)
    }

    /// 枚举 → 层目录名。`Outside` 返回 `None`。
    pub fn dir_name(self) -> Option<&'static str> {
        Self::REAL_DIRS
            .iter()
            .find(|(_, l)| *l == self)
            .map(|(d, _)| *d)
    }

    /// 判定一个源文件属于哪一层 —— **归一后的实现**。
    ///
    /// ⛔ 旧实现有两处缺陷（均已实测）：
    /// 1. 只认 10 个**本仓不存在**的目录名 ⇒ 可分类文件 = 0
    ///    （配合 `check_architecture_layer_depth` 跳过目录 ⇒ 该检查结构上不可能失败）。
    /// 2. `p.contains("/core/")` 会被 `nt_core_*` 模块路径里的 `/core/` 子串误抓。
    ///
    /// ⇒ 现在只按**真实目录名**判定，且**按路径段**（含 `/` 前缀）而非裸子串。
    pub fn from_path(path: &Path) -> Self {
        let p = path.to_string_lossy();
        // 从后往前找层目录（路径越深越具体，但层目录通常靠前，两种都覆盖）
        for (dir, layer) in Self::REAL_DIRS {
            // 段匹配：`/l5_cognition/` 或 `src/l5_cognition/`
            if p.contains(&format!("/{dir}/")) || p.contains(&format!("/src/{dir}/")) {
                return *layer;
            }
        }
        // 顶层 `neotrix-core/src/*.rs` 不属任何层
        Self::Outside
    }

    /// 层号（0-6）。`Outside` 返回 -1，与旧语义一致（`layer_index() < 0` 表"不可分类"）。
    pub fn layer_index(self) -> i32 {
        match self {
            Self::L0Substrate => 0,
            Self::L1Action => 1,
            Self::L2Perception => 2,
            Self::L3Embodiment => 3,
            Self::L4Emotion => 4,
            Self::L5Cognition => 5,
            Self::L6Meta => 6,
            Self::Outside => -1,
        }
    }

    /// 层号 → 枚举（`layer_index()` 的逆，供映射表使用）。
    pub fn from_index(i: i32) -> Option<Self> {
        Self::REAL_DIRS.get(i as usize).map(|(_, l)| *l)
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
    /// ⛔ **未接线规格**（nt-unwired-spec）：本字段零读点 ——
    ///    功能已**声明**但读者未实现。⛔ **不要删**（删掉即销毁规格）。
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
