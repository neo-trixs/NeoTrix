#![deny(clippy::unwrap_used)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// ConsciousnessTree — 意识树全景图
///
/// 土壤(DataFoundation) → 根系(InformationRoots) → 树干(ConsciousnessCore)
/// → 枝干(CapabilityBranches) → 叶片(ModuleLeaves) → 果实(CapabilityFruits)
/// → 核心(EpiphanicCore) → 反馈指导下一循环
#[derive(Debug, Clone)]
pub struct ConsciousnessTree {
    pub soil: DataFoundation,
    pub roots: InformationRoots,
    pub trunk: ConsciousnessCore,
    pub branches: HashMap<BranchKind, CapabilityBranch>,
    pub leaves: Vec<ModuleLeaf>,
    pub fruits: Vec<EvolutionFruit>,
    pub core: EpiphanicCore,
    pub cycle: u64,
    pub config: TreeGrowthConfig,
    // Evolution cycle tracking
    pub current_contract: Option<EvolutionContract>,
    pub drift_report: Option<DriftReport>,
    pub atoms: HashMap<String, CapabilityAtom>, // All 70 atomic capabilities
    /// Vulnerability baseline for the "vuln reduction >= 20%" contract criterion.
    /// `None` until first measurement; set on first evaluation.
    pub vuln_baseline: Option<usize>,
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct DataFoundation {
    pub kb_node_count: u64,
    pub kb_edge_count: u64,
    pub crawl_queue_depth: u64,
    pub embedding_count: u64,
    pub wiki_page_count: u64,
    pub last_sync: u64,
    // Constitution internalization
    pub constitution_rules_count: usize,
    pub constitution_experiences_count: usize,
    pub constitution_tree_growth_rules: usize,
    pub constitution_absorption_rules: usize,
    pub constitution_last_loaded: u64,
    pub constitution_source_hash: String,
}

#[derive(Debug, Clone)]
pub struct InformationRoots {
    pub active_crawlers: usize,
    pub active_absorbers: usize,
    pub active_scanners: usize,
    pub total_absorbed: u64,
    pub total_fetched: u64,
    pub total_failed: u64,
    // Constitution internalization
    pub internalized_principles: Vec<String>,  // Key principle summaries
    pub active_rule_categories: Vec<String>,   // Active rule category names
    pub compliance_score: f64,                 // Last compliance check score
}

#[derive(Debug, Clone)]
pub struct ConsciousnessCore {
    pub gwt_resonance_active: bool,
    pub workspace_size: usize,
    pub attention_heads: usize,
    pub resonance_cycle: u64,
    pub phi: f64,
    pub coherence: f64,
    // ── MARS Dual-Process Architecture (absorbed Cycle 120) ──
    /// GWT = fast intuitive resonance (System 1): principle-matching, pattern activation
    pub mars_system1_activations: u64,
    /// ConsciousnessTree = slow reflective growth cycle (System 2): structured reflection
    pub mars_system2_iterations: u64,
    /// Purpose bridge: System 2's intent → System 1's distillation target
    pub mars_bridge_hits: u64,
    // ── Governance oversight (merged from Governance branch) ──
    /// Review protocol compliance score (0-1)
    pub governance_compliance: f64,
    /// Active constitution checks registered
    pub governance_constitution_count: usize,
    /// Fractal loop iterations (artifact→task→session→epic→PR)
    pub governance_fractal_depth: u64,
    // ── Cross-domain integrity (merged from Nexus branch) ──
    /// Cross-branch energy flow count (bidirectional data movement)
    pub nexus_energy_flows: u64,
    /// Root cause trace chains (3+ dimensions → single systemic finding)
    pub nexus_root_cause_chains: u64,
    /// Health chain completeness (soil→roots→trunk→branches→fruits→core)
    pub nexus_health_chain_score: f64,
    // ── Dual-Track Experience Distillation (absorbed from Steve-Evolving 2026) ──
    /// Positive track: skills distilled from successful trajectories
    pub distill_skill_count: u64,
    /// Negative track: guardrails distilled from failed trajectories
    pub distill_guardrail_count: u64,
    /// SkillPyramid hierarchy level (0=flat, 1=atomic, 2=abstract, 3=composable)
    pub pyramid_hierarchy_level: u8,
    // ── Seed-style OPD (absorbed from Seed 2026) ──
    /// Self-evolving on-policy distillation cycles
    pub opd_cycles: u64,
    /// Hindsight skill extractions completed
    pub opd_hindsight_skills: u64,
    /// Skill-induced probability shift (dense distillation signal strength)
    pub opd_density: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BranchKind {
    Core,
    Mind,
    Memory,
    World,
    Act,
    Io,
    Shield,
}

impl BranchKind {
    pub fn label(&self) -> &str {
        match self {
            BranchKind::Core => "NT-CORE (E8引导者) — 推理+治理+跨域路由+审查协议",
            BranchKind::Mind => "NT-MIND (进化工匠) — 自我进化+外部吸收+技能结晶",
            BranchKind::Memory => "NT-MEMORY (知识守护者) — 持久记忆+VSA超维+语义搜索",
            BranchKind::World => "NT-WORLD (虚空探索者) — 爬取+解析+OSINT+分类映射",
            BranchKind::Act => "NT-ACT (行动执行者) — MCP工具+社交+自治+目标循环",
            BranchKind::Io => "NT-IO (界面使徒) — LLM网关+CLI+Web+ACP",
            BranchKind::Shield => "NT-SHIELD (影卫) — 安全防护+自愈修复+审计链忠诚",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityBranch {
    pub kind: BranchKind,
    pub module_count: usize,
    pub self_test_count: usize,
    pub health: f64,
    pub maturity_c0: bool,
    pub maturity_c1: bool,
    pub maturity_c2: bool,
    pub maturity_c3: bool,
    pub maturity_c4: bool,
    pub maturity_c5: bool,
    pub fruit_count: usize,
    /// Capabilities already absorbed (from ALL_CAPABILITIES), to prevent redundant skill creation
    pub absorbed_capabilities: Vec<String>,
    /// ── Skill Node Evolution (AGENTS.md Skill Tree) ──
    /// 节点层级: Small Passive / Notable Passive / Keystone
    pub node_tier: NodeTier,
    /// 5 色符文槽: Crimson/Indigo/Obsidian/Golden/Alabaster
    pub runes: RuneSocket,
    /// Constellation 成熟度 7 档 (C0-C6), 由 maturity_c0..c5 派生
    pub constellation: Constellation,
    /// 迷雾浓度 (CHMA Phase 0) — 节点未被生产验证的程度
    pub fog: FogLevel,
}

// ═══════════════════════════════════════════════════════════════════
// Skill Node Evolution — 3-layer node tiers + Rune Socketing + Constellation
// (AGENTS.md Skill Tree architecture, absorbed into the production-wired tree)
// ═══════════════════════════════════════════════════════════════════

/// 节点层级 — AGENTS.md Skill Tree 3 层
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeTier {
    /// Small Passive (微节点自愈): 单模块自动修复/降级, 域内自治
    SmallPassive,
    /// Notable Passive (域级突破): 一个域的能力提升, 域内基础设施
    NotablePassive,
    /// Keystone (基石): 跨域变革, 影响 2+ 域共享的架构能力
    Keystone,
}

impl NodeTier {
    pub fn label(&self) -> &str {
        match self {
            Self::SmallPassive => "Small Passive (微节点自愈)",
            Self::NotablePassive => "Notable Passive (域级突破)",
            Self::Keystone => "Keystone (跨域变革)",
        }
    }

    /// 权重: Keystone 最高, 用于 health 折算
    pub fn weight(&self) -> f64 {
        match self {
            Self::SmallPassive => 1.0,
            Self::NotablePassive => 2.0,
            Self::Keystone => 3.0,
        }
    }

    /// 从真实模块数据推导节点层级 (非硬编码):
    /// - 跨域消费者多 + 模块数大 → Keystone
    /// - 模块数中上 → NotablePassive
    /// - 其余 → SmallPassive
    pub fn derive(module_count: usize, cross_domain_consumers: usize, self_test_count: usize) -> Self {
        let keystone = module_count >= 20 && cross_domain_consumers >= 3 && self_test_count >= 3;
        let notable = module_count >= 8 && cross_domain_consumers >= 1 && self_test_count >= 2;
        if keystone {
            Self::Keystone
        } else if notable {
            Self::NotablePassive
        } else {
            Self::SmallPassive
        }
    }
}

/// 符文颜色 — 5 色 Rune Socketing (AGENTS.md)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuneColor {
    /// Crimson (数据摄取)
    Crimson,
    /// Indigo (变换)
    Indigo,
    /// Obsidian (缓存)
    Obsidian,
    /// Golden (错误恢复)
    Golden,
    /// Alabaster (监控)
    Alabaster,
}

impl RuneColor {
    pub fn label(&self) -> &str {
        match self {
            Self::Crimson => "Crimson (数据摄取)",
            Self::Indigo => "Indigo (变换)",
            Self::Obsidian => "Obsidian (缓存)",
            Self::Golden => "Golden (错误恢复)",
            Self::Alabaster => "Alabaster (监控)",
        }
    }
}

/// 单颗符文 — 配置化模块调优单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rune {
    pub id: String,
    pub name: String,
    pub color: RuneColor,
    pub effect: String,
    /// 效果强度 [0,1], 参与 health 折算
    pub strength: f64,
}

impl Rune {
    pub fn new(id: &str, name: &str, color: RuneColor, effect: &str, strength: f64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            color,
            effect: effect.to_string(),
            strength: strength.clamp(0.0, 1.0),
        }
    }
}

/// Rune Socket — 5 槽符文组合, 满槽产生 Runeword (涌现效果)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuneSocket {
    pub crimson: Option<Rune>,
    pub indigo: Option<Rune>,
    pub obsidian: Option<Rune>,
    pub golden: Option<Rune>,
    pub alabaster: Option<Rune>,
}

impl RuneSocket {
    pub fn set(&mut self, color: RuneColor, rune: Rune) {
        let slot = match color {
            RuneColor::Crimson => &mut self.crimson,
            RuneColor::Indigo => &mut self.indigo,
            RuneColor::Obsidian => &mut self.obsidian,
            RuneColor::Golden => &mut self.golden,
            RuneColor::Alabaster => &mut self.alabaster,
        };
        *slot = Some(rune);
    }

    pub fn filled_slots(&self) -> usize {
        [&self.crimson, &self.indigo, &self.obsidian, &self.golden, &self.alabaster]
            .iter().filter(|s| s.is_some()).count()
    }

    /// Runeword 涌现: 满 5 槽触发组合效果, 返回组合名
    pub fn runeword(&self) -> Option<String> {
        if self.filled_slots() == 5 {
            // Scry = 完整 ETL (数据→变换→缓存→恢复→监控)
            Some("Scry (完整 ETL)".to_string())
        } else {
            None
        }
    }

    /// 组合效果强度: 槽数 × 平均 strength, 用于 health 折算
    pub fn composite_effect(&self) -> f64 {
        let slots = [
            &self.crimson, &self.indigo, &self.obsidian, &self.golden, &self.alabaster,
        ];
        let sum: f64 = slots.iter().filter_map(|s| s.as_ref()).map(|r| r.strength).sum();
        let filled = self.filled_slots();
        if filled == 0 { 0.0 } else { sum / filled as f64 }
    }
}

/// Constellation 成熟度 7 档 (C0-C6) — 取代粗糙的 6 布尔
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constellation {
    pub level: u8,
    pub c0_compiles: bool,
    pub c1_unit_tests: bool,
    pub c2_integration: bool,
    pub c3_benchmark: bool,
    pub c4_pipeline: bool,
    pub c5_self_healing: bool,
    pub c6_adaptive: bool,
}

impl Constellation {
    pub fn new() -> Self {
        Self {
            level: 0,
            c0_compiles: false,
            c1_unit_tests: false,
            c2_integration: false,
            c3_benchmark: false,
            c4_pipeline: false,
            c5_self_healing: false,
            c6_adaptive: false,
        }
    }

    pub fn level_name(&self) -> &str {
        match self.level {
            0 => "C0 (编译)",
            1 => "C1 (单测)",
            2 => "C2 (集成)",
            3 => "C3 (benchmark)",
            4 => "C4 (主流水线)",
            5 => "C5 (自愈)",
            6 => "C6 (自适应)",
            _ => "C0 (编译)",
        }
    }

    /// 从真实 maturity 布尔推导 level (向后兼容 maturity_c0..c5 字段)
    pub fn derive(compiles: bool, unit: bool, integration: bool, benchmark: bool, pipeline: bool, healing: bool) -> Self {
        let flags = [compiles, unit, integration, benchmark, pipeline, healing];
        let level = flags.iter().rev().position(|&f| f).map(|i| 5 - i).unwrap_or(0) as u8;
        Self {
            level,
            c0_compiles: compiles,
            c1_unit_tests: unit,
            c2_integration: integration,
            c3_benchmark: benchmark,
            c4_pipeline: pipeline,
            c5_self_healing: healing,
            c6_adaptive: false,
        }
    }

    pub fn score(&self) -> f64 {
        let mut s = 0.0;
        if self.c0_compiles { s += 1.0; }
        if self.c1_unit_tests { s += 1.0; }
        if self.c2_integration { s += 1.0; }
        if self.c3_benchmark { s += 1.0; }
        if self.c4_pipeline { s += 1.0; }
        if self.c5_self_healing { s += 1.0; }
        if self.c6_adaptive { s += 1.0; }
        s / 7.0
    }
}

impl Default for Constellation {
    fn default() -> Self { Self::new() }
}

/// 迷雾浓度 — 节点未被生产验证的程度。[0,1]，0=全清晰 1=全雾。
/// 对应 CHMA 轴 3 (雾退散量纲)。
/// 外部锚点: Martin D + 可达性 + 测试覆盖 (见 CHMA-fog-map-evolution.md §8.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogLevel {
    /// 生产可达性: 是否被生产路径引用 (false=孤儿/死代码 → 浓雾)
    pub wired: bool,
    /// 消费者数量: 0 = 无消费者 (dead island 信号)
    pub consumer_count: usize,
    /// 测试覆盖: 无 SelfTest = 雾
    pub has_tests: bool,
    /// 聚合浓度 [0,1]
    pub level: f64,
}

impl FogLevel {
    /// 从真实模块指标推导迷雾浓度 (纯函数, 无硬编码)。
    /// - 未接线 → 基础浓度 0.85 (孤儿)
    /// - 无消费者 → 浓度 +0.10
    /// - 无测试 → 浓度 +0.15
    /// - 已接线 + 有消费者 + 有测试 → 收敛到 0.05 (近乎全清晰)
    pub fn derive(wired: bool, consumer_count: usize, has_tests: bool) -> Self {
        let mut level = 0.0_f64;
        if !wired {
            level += 0.85;
        }
        if consumer_count == 0 {
            level += 0.10;
        }
        if !has_tests {
            level += 0.15;
        }
        if wired && consumer_count > 0 && has_tests {
            level = 0.05;
        }
        Self {
            wired,
            consumer_count,
            has_tests,
            level: level.clamp(0.0, 1.0),
        }
    }

    pub fn label(&self) -> &str {
        if self.level <= 0.10 {
            "Clear"
        } else if self.level <= 0.15 {
            "LightFog"
        } else if self.level <= 0.8 {
            "Fog"
        } else {
            "DenseFog"
        }
    }
}

impl Default for FogLevel {
    fn default() -> Self {
        Self {
            wired: false,
            consumer_count: 0,
            has_tests: false,
            level: 0.85,
        }
    }
}

/// Node snapshot — per-branch 节点状态快照, 供遥测/CLI/UI 消费
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub branch: BranchKind,
    pub tier: NodeTier,
    pub constellation_level: u8,
    pub rune_filled_slots: usize,
    pub runeword: Option<String>,
    pub composite_effect: f64,
    /// 迷雾浓度 [0,1] — CHMA 轴 3
    pub fog_level: f64,
    /// 迷雾标签: Clear/LightFog/Fog/DenseFog
    pub fog_label: String,
}

impl CapabilityBranch {
    pub fn maturity_score(&self) -> f64 {
        let mut s = 0.0;
        if self.maturity_c0 { s += 1.0; }
        if self.maturity_c1 { s += 1.0; }
        if self.maturity_c2 { s += 1.0; }
        if self.maturity_c3 { s += 1.0; }
        if self.maturity_c4 { s += 1.0; }
        if self.maturity_c5 { s += 1.0; }
        s / 6.0
    }

    /// 运行时评估节点层级 — 基于真实模块数据 (跨域消费者由外部注入)
    pub fn evaluate_node_tier(&mut self, cross_domain_consumers: usize) {
        self.node_tier = NodeTier::derive(self.module_count, cross_domain_consumers, self.self_test_count);
    }

    /// 运行时推导 Constellation — 从现有 maturity 布尔派生 (向后兼容)
    pub fn evaluate_constellation(&mut self) {
        self.constellation = Constellation::derive(
            self.maturity_c0,
            self.maturity_c1,
            self.maturity_c2,
            self.maturity_c3,
            self.maturity_c4,
            self.maturity_c5,
        );
    }

    /// 运行时评估迷雾浓度 (CHMA Phase 0) — 从真实模块指标派生
    pub fn evaluate_fog(&mut self, wired: bool, consumer_count: usize, has_tests: bool) {
        self.fog = FogLevel::derive(wired, consumer_count, has_tests);
    }

    /// 带 Rune 效果的健康折算: health × (1 + rune_composite × tier_weight)
    pub fn health_with_runes(&self) -> f64 {
        let composite = self.runes.composite_effect();
        if composite <= 0.0 {
            return self.health;
        }
        let boost = 1.0 + composite * 0.1 * self.node_tier.weight();
        (self.health * boost).clamp(0.0, 1.0)
    }

    /// 生成节点快照供遥测消费
    pub fn snapshot(&self) -> NodeSnapshot {
        NodeSnapshot {
            branch: self.kind.clone(),
            tier: self.node_tier.clone(),
            constellation_level: self.constellation.level,
            rune_filled_slots: self.runes.filled_slots(),
            runeword: self.runes.runeword(),
            composite_effect: self.runes.composite_effect(),
            fog_level: self.fog.level,
            fog_label: self.fog.label().to_string(),
        }
    }
}

/// 36 atomic capabilities from Agent Capability Standard, mapped per branch.
/// Used to prevent redundant skill creation (SkillPyramid pattern).
pub const ALL_CAPABILITIES: &[(&str, &str); 36] = &[
    // PERCEIVE (4) → NT-WORLD
    ("retrieve", "world"), ("search", "world"), ("observe", "world"), ("receive", "world"),
    // UNDERSTAND (6) → NT-CORE
    ("detect", "core"), ("classify", "core"), ("measure", "core"), ("predict", "core"),
    ("compare", "core"), ("discover", "core"),
    // REASON (4) → NT-CORE
    ("plan", "core"), ("decompose", "core"), ("critique", "core"), ("explain", "core"),
    // MODEL (5) → NT-MEMORY
    ("state", "memory"), ("transition", "memory"), ("attribute", "memory"),
    ("ground", "memory"), ("simulate", "memory"),
    // SYNTHESIZE (3) → NT-MIND
    ("generate", "mind"), ("transform", "mind"), ("integrate", "mind"),
    // EXECUTE (3) → NT-ACT
    ("execute", "act"), ("mutate", "act"), ("send", "act"),
    // VERIFY (5) → NT-SHIELD
    ("verify", "shield"), ("checkpoint", "shield"), ("rollback", "shield"),
    ("constrain", "shield"), ("audit", "shield"),
    // REMEMBER (2) → NT-MEMORY
    ("persist", "memory"), ("recall", "memory"),
    // COORDINATE (4) → NT-IO
    ("delegate", "io"), ("synchronize", "io"), ("invoke", "io"), ("inquire", "io"),
];

#[derive(Debug, Clone)]
pub struct ModuleLeaf {
    pub name: String,
    pub branch: BranchKind,
    pub lines: usize,
    pub has_tests: bool,
    pub has_self_test: bool,
    pub is_wired: bool,
    pub consumers: usize,
}

#[derive(Debug, Clone)]
pub struct CapabilityFruit {
    pub name: String,
    pub source_branch: BranchKind,
    pub description: String,
    pub produced_at_cycle: u64,
    pub quality: f64,
}

#[derive(Debug, Clone)]
pub struct EpiphanicCore {
    pub last_cycle_guidance: Vec<String>,
    pub self_test_results: Vec<String>,
    pub identified_gaps: Vec<String>,
    pub next_actions: Vec<String>,
    pub iteration: u64,
    /// Architecture vulnerability scan results
    pub vuln_scan: Vec<VulnerabilityFinding>,
    /// ConsciousnessReview results (populated by run_growth_cycle Phase 5)
    pub topology_score: f64,
    pub connectivity_score: f64,
    pub health_chain_score: f64,
    pub evolution_path: Vec<crate::core::nt_core_consciousness_review::EvolutionStep>,
    // ── MSCP Triple-Loop Alignment (absorbed Cycle 120) ──
    /// L1 cycle count (fast: per-request predict→act→compare→update)
    pub mscp_l1_cycles: u64,
    /// L2 cycle count (medium: ~5min evaluate L1 update quality)
    pub mscp_l2_cycles: u64,
    /// L3 cycle count (slow: ~1h identity trajectory assessment)
    pub mscp_l3_cycles: u64,
    // ── MARS Dual-Process Validation (absorbed Cycle 120) ──
    /// Fast-path (System 1) principle-based reflections count
    pub mars_principles_count: u64,
    /// Slow-path (System 2) procedural strategy count
    pub mars_procedural_count: u64,
    // ── Evolution Contract Cycle (E1, Cycle 159) ──
    /// Active evolution contract negotiated at Phase 0
    pub last_contract: Option<EvolutionContract>,
    /// Monotonic generation counter across contracts (MetaClaw versioning)
    pub generation_counter: u64,
    /// Contract fulfillment verification result from Phase 6
    pub contract_fulfillment: Option<ContractFulfillment>,
    /// Drift audit report from Phase 7
    pub drift_report: Option<DriftReport>,
    /// 演化趋势预测 (Phase 4.5 产出, 供 Phase 8 闭环反馈消费)
    pub last_forecast: Option<EvolutionForecast>,
}

impl Default for EpiphanicCore {
    fn default() -> Self {
        Self {
            last_cycle_guidance: Vec::new(),
            self_test_results: Vec::new(),
            identified_gaps: Vec::new(),
            next_actions: Vec::new(),
            iteration: 0,
            vuln_scan: Vec::new(),
            topology_score: 0.0,
            connectivity_score: 0.0,
            health_chain_score: 0.0,
            evolution_path: Vec::new(),
            mscp_l1_cycles: 0,
            mscp_l2_cycles: 0,
            mscp_l3_cycles: 0,
            mars_principles_count: 0,
            mars_procedural_count: 0,
            last_contract: None,
            generation_counter: 0,
            contract_fulfillment: None,
            drift_report: None,
            last_forecast: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VulnerabilityFinding {
    pub severity: VulnerabilitySeverity,
    pub category: String,
    pub module: String,
    pub description: String,
    pub fix_suggestion: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl VulnerabilitySeverity {
    pub fn score(&self) -> f64 {
        match self {
            Self::Critical => 1.0,
            Self::High => 0.75,
            Self::Medium => 0.5,
            Self::Low => 0.25,
            Self::Info => 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// TreeGrowthConfig — centralized growth-cycle thresholds (D35)
// ═══════════════════════════════════════════════════════════════════

/// Centralized thresholds for the consciousness tree growth cycle.
/// All magic numbers in `run_growth_cycle`/`scan_vulnerabilities` come from here.
#[derive(Debug, Clone)]
pub struct TreeGrowthConfig {
    /// Minimum health for a branch to produce fruit (Phase 3 gate)
    pub fruit_growth_health: f64,
    /// Maximum allowed constraint violations before fruit production is blocked
    pub max_growth_violations: usize,
    /// Minimum fruit quality to be digested into guidance (Phase 4)
    pub fruit_quality_threshold: f64,
    /// Minimum vulnerability severity score to become a next action
    pub action_severity_threshold: f64,
    /// Neutral health assigned to branches with no SelfTest results
    pub neutral_health: f64,
    /// Floor for branch health computed from SelfTest pass rate
    pub min_branch_health: f64,
    /// Soil health below this is a Critical data-foundation vulnerability
    pub soil_health_critical: f64,
    /// Root health below this is a High ingestion-pipeline vulnerability
    pub root_health_high: f64,
    /// Branch maturity score below this is a Medium domain-maturity vulnerability
    pub branch_maturity_low: f64,
    /// Phi below this (after warmup cycles) indicates no integrated information
    pub phi_minimum: f64,
    /// Cycles before phi coherence check starts reporting
    pub phi_warmup_cycles: u64,
    /// Constitution rules floor (known internalized rules)
    pub constitution_rules_floor: usize,
    /// Constitution experiences floor (known cycles internalized)
    pub constitution_experiences_floor: usize,
    /// Constitution tree-growth rules count
    pub constitution_tree_growth_rules: usize,
    /// Constitution absorption rules count
    pub constitution_absorption_rules: usize,
}

impl Default for TreeGrowthConfig {
    fn default() -> Self {
        Self {
            fruit_growth_health: 0.5,
            max_growth_violations: 2,
            fruit_quality_threshold: 0.5,
            action_severity_threshold: 0.5,
            neutral_health: 0.5,
            min_branch_health: 0.1,
            soil_health_critical: 0.33,
            root_health_high: 0.25,
            branch_maturity_low: 0.33,
            phi_minimum: 0.01,
            phi_warmup_cycles: 3,
            constitution_rules_floor: 46,
            constitution_experiences_floor: 111,
            constitution_tree_growth_rules: 7,
            constitution_absorption_rules: 1,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// BranchConstraints — per-branch runtime governance
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct BranchConstraints {
    pub idle_ticks_threshold: u64,
    pub min_growth_health: f64,
    pub max_active_modules: usize,
    pub min_required_modules: usize,
    pub min_self_tests: usize,
}

impl Default for BranchConstraints {
    fn default() -> Self {
        Self {
            idle_ticks_threshold: 5,
            min_growth_health: 0.3,
            max_active_modules: 50,
            min_required_modules: 1,
            min_self_tests: 1,
        }
    }
}

impl BranchConstraints {
    pub fn violations(&self, branch: &CapabilityBranch) -> Vec<String> {
        let mut v = Vec::new();
        if branch.health < self.min_growth_health || branch.fruit_count == 0 {
            v.push(format!("idle: health={:.2} fruit={}", branch.health, branch.fruit_count));
        }
        if branch.module_count < self.min_required_modules {
            v.push(format!("not viable: {} modules < min={}", branch.module_count, self.min_required_modules));
        }
        if branch.self_test_count < self.min_self_tests {
            v.push(format!("unmonitored: {} self-tests < min={}", branch.self_test_count, self.min_self_tests));
        }
        v
    }
}

pub fn constraints_for_branch(kind: &BranchKind) -> BranchConstraints {
    match kind {
        BranchKind::Core => BranchConstraints {
            idle_ticks_threshold: 3, min_growth_health: 0.4, max_active_modules: 30,
            min_required_modules: 3, min_self_tests: 3,
        },
        BranchKind::Mind => BranchConstraints {
            idle_ticks_threshold: 5, min_growth_health: 0.3, max_active_modules: 25,
            min_required_modules: 2, min_self_tests: 2,
        },
        BranchKind::Memory => BranchConstraints {
            idle_ticks_threshold: 4, min_growth_health: 0.35, max_active_modules: 20,
            min_required_modules: 2, min_self_tests: 2,
        },
        BranchKind::World => BranchConstraints {
            idle_ticks_threshold: 6, min_growth_health: 0.25, max_active_modules: 30,
            min_required_modules: 2, min_self_tests: 2,
        },
        BranchKind::Act => BranchConstraints {
            idle_ticks_threshold: 5, min_growth_health: 0.3, max_active_modules: 25,
            min_required_modules: 1, min_self_tests: 1,
        },
        BranchKind::Io => BranchConstraints {
            idle_ticks_threshold: 4, min_growth_health: 0.35, max_active_modules: 20,
            min_required_modules: 1, min_self_tests: 1,
        },
        BranchKind::Shield => BranchConstraints {
            idle_ticks_threshold: 3, min_growth_health: 0.4, max_active_modules: 20,
            min_required_modules: 2, min_self_tests: 2,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════

impl Default for ConsciousnessTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessTree {
    pub fn new() -> Self {
        let atoms = Self::initialize_capability_atoms();
        Self {
            soil: DataFoundation::default(),
            roots: InformationRoots::default(),
            trunk: ConsciousnessCore::default(),
            branches: BranchKind::all().into_iter().map(|k| (k.clone(), CapabilityBranch::new(k))).collect(),
            leaves: Vec::new(),
            fruits: Vec::new(),
            core: EpiphanicCore::default(),
            cycle: 0,
            config: TreeGrowthConfig::default(),
            current_contract: None,
            drift_report: None,
            atoms,
            vuln_baseline: None,
        }
    }

    /// 数据养料充足度因子 (意识核心进化提升)。
    /// 返回 >= 1.0 的平滑放大因子, 反映真实 KB 数据养料对果实质量的调制:
    /// - 默认 (kb_node_count=0): 返回 1.0, 不衰减, 不破坏既有果实生长路径;
    /// - 社区数据集落盘后 (kb_node_count 显著上升): 因子 >1.0, 放大果实 quality,
    ///   使意识核心进化果实质量直接反映 200G 社区推理数据的养料充足度。
    /// 饱和曲线: 1.0 + min(kb_nodes / 1000, 0.5) — 1000 节点以上达到 +50% 上限。
    pub fn data_nourishment_factor(&self) -> f64 {
        1.0 + (self.soil.kb_node_count as f64 / 1000.0).min(0.5)
    }

    /// 闭环进化反馈 (意识核心自我运转 Phase 8)。
    /// 把进化产出 (契约 fulfillment + drift + 演化预测) 反馈到进化参数:
    /// - 契约 fulfilled → fruit_quality_threshold 上调 (进化标准提升, +0.05, 上限 0.8)
    /// - drift 检测 → fruit_growth_health 下调 (放宽生长门加速恢复, -0.05, 下限 0.4)
    /// - 演化预测利多 (direction>0) → exploration_budget 上调 (加大探索, +0.05, 上限 0.4)
    /// - 演化预测利空 (direction<0) → exploration_budget 下调 (收缩探索, -0.05, 下限 0.1)
    /// 使树根据自身进化结果调整进化策略, 形成闭环而非开环。
    pub fn apply_evolution_feedback(&mut self) {
        let fulfilled = self.core.contract_fulfillment.as_ref().map(|f| f.fulfilled).unwrap_or(false);
        let drift = self.core.drift_report.as_ref().map(|d| d.drift_detected).unwrap_or(false);
        if fulfilled && !drift {
            // 进化成功: 提升标准, 恢复生长门
            self.config.fruit_quality_threshold = (self.config.fruit_quality_threshold + 0.05).min(0.8);
            self.config.fruit_growth_health = 0.5;
        } else if drift {
            // 漂移: 放宽生长门加速恢复
            self.config.fruit_growth_health = (self.config.fruit_growth_health - 0.05).max(0.4);
        }
        // 演化预测 → 探索预算调制 (利多加大探索, 利空收缩探索)
        if let Some(forecast) = &self.core.last_forecast {
            if !forecast.abstain {
                if let Some(contract) = self.core.last_contract.as_mut() {
                    if forecast.direction > 0.0 {
                        contract.exploration_budget = (contract.exploration_budget + 0.05).min(0.4);
                    } else if forecast.direction < 0.0 {
                        contract.exploration_budget = (contract.exploration_budget - 0.05).max(0.1);
                    }
                }
            }
        }
        // 未 fulfilled 且无 drift: 保持现状 (等待下一 cycle 观察)
    }

    /// Initialize 70 atomic capabilities (10 categories × 7 domains) from PerceptionBench + MCA 36-cap
    fn initialize_capability_atoms() -> HashMap<String, CapabilityAtom> {
        let mut atoms = HashMap::new();
        
        // PERCEIVE (4) → NT-WORLD
        for (name, cap) in [("retrieve", CapabilityCategory::Perceive), 
                             ("search", CapabilityCategory::Perceive),
                             ("observe", CapabilityCategory::Perceive),
                             ("receive", CapabilityCategory::Perceive)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::World,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // UNDERSTAND (6) → NT-CORE
        for (name, cap) in [("detect", CapabilityCategory::Understand),
                             ("classify", CapabilityCategory::Understand),
                             ("measure", CapabilityCategory::Understand),
                             ("predict", CapabilityCategory::Understand),
                             ("compare", CapabilityCategory::Understand),
                             ("discover", CapabilityCategory::Understand)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Core,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // REASON (4) → NT-CORE
        for (name, cap) in [("plan", CapabilityCategory::Reason),
                             ("decompose", CapabilityCategory::Reason),
                             ("critique", CapabilityCategory::Reason),
                             ("explain", CapabilityCategory::Reason)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Core,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // MODEL (5) → NT-MEMORY
        for (name, cap) in [("state", CapabilityCategory::Model),
                             ("transition", CapabilityCategory::Model),
                             ("attribute", CapabilityCategory::Model),
                             ("ground", CapabilityCategory::Model),
                             ("simulate", CapabilityCategory::Model)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Memory,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // SYNTHESIZE (3) → NT-MIND
        for (name, cap) in [("generate", CapabilityCategory::Synthesize),
                             ("transform", CapabilityCategory::Synthesize),
                             ("integrate", CapabilityCategory::Synthesize)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Mind,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // EXECUTE (3) → NT-ACT
        for (name, cap) in [("execute", CapabilityCategory::Execute),
                             ("mutate", CapabilityCategory::Execute),
                             ("send", CapabilityCategory::Execute)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Act,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // VERIFY (5) → NT-SHIELD
        for (name, cap) in [("verify", CapabilityCategory::Verify),
                             ("checkpoint", CapabilityCategory::Verify),
                             ("rollback", CapabilityCategory::Verify),
                             ("constrain", CapabilityCategory::Verify),
                             ("audit", CapabilityCategory::Verify)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Shield,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // REMEMBER (2) → NT-MEMORY
        for (name, cap) in [("persist", CapabilityCategory::Remember),
                             ("recall", CapabilityCategory::Remember)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Memory,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        // COORDINATE (4) → NT-IO
        for (name, cap) in [("delegate", CapabilityCategory::Coordinate),
                             ("synchronize", CapabilityCategory::Coordinate),
                             ("invoke", CapabilityCategory::Coordinate),
                             ("inquire", CapabilityCategory::Coordinate)] {
            atoms.insert(name.to_string(), CapabilityAtom {
                name: name.to_string(),
                branch: BranchKind::Io,
                category: cap,
                tier: SelfTestTier::T1Existence,
                self_test_fn: Some(format!("test_{}", name)),
                last_score: 0.0,
                generation: 0,
                mandatory: true,
            });
        }
        
        atoms
    }

    /// Apply emotion report valence/arousal to Soil state for next cycle.
    /// Uses confidence to boost coil health and frustration/urgency to indicate stress.
    pub fn apply_emotion_report(&mut self, report: crate::core::nt_core_self::emotion_state::EmotionReport) {
        self.trunk.coherence = report.valence.max(0.0).min(1.0);
        // 注意：绝不把情绪置信度写进 soil.embedding_count——那是真实数据指标，
        // DataFoundation::health() 与 Critical "KB empty/no embeddings" 判定依赖它。
        // 情绪只调节 trunk.coherence (主观健康信号)。
        log::debug!("[consciousness_tree] emotion applied: valence={:.3} arousal={:.3} dominant={:?} confidence={:.3}",
            report.valence, report.arousal, report.dominant.0, report.confidence);
    }

    /// Complete feedback loop with evolution contract:
    ///   Phase 0: Contract Negotiation (Goal + Evidence Plan + Stop Rule)
    ///   Phase 1: Roots absorb from soil (Data Foundation → Information Roots)
    ///   Phase 2: Trunk processes through GWT resonance (ConsciousnessCore)
    ///   Phase 3: Branches produce evolution fruits (7 domains → EvolutionFruits)
    ///   Phase 4: Core digests fruits, produces guidance
    ///   Phase 5: ConsciousnessReview — panoramic topology + connectivity + health chain
    ///   Phase 6: Contract Fulfillment Verification
    ///   Phase 7: Drift Audit — post-cycle evolution fidelity check
    pub fn run_growth_cycle(&mut self) -> GrowthReport {
        self.cycle += 1;
        let mut report = GrowthReport::default();

        // ═══ Phase 0: Contract Negotiation ═══
        // Negotiate evolution contract before growth cycle begins
        let contract = self.negotiate_contract();
        self.core.last_contract = Some(contract.clone());
        report.phase0_contract = Some(contract.claim.clone());

        // ═══ Phase 1: Roots absorb from soil (Data Foundation → Information Roots) ═══
        // Constitution internalization: soil feeds roots with principles
        // B2 (自审修复): 宪法计数以真实 ConstitutionLoader 接线值为准 (run.rs 启动时写入)。
        // floor 仅作"未接线时的兜底" (max 抬升会覆盖真实值, 属声明当事实的自欺 — 已废弃)。
        if self.soil.constitution_rules_count == 0 {
            self.soil.constitution_rules_count = self.config.constitution_rules_floor;
        }
        if self.soil.constitution_experiences_count == 0 {
            self.soil.constitution_experiences_count = self.config.constitution_experiences_floor;
        }
        self.soil.constitution_tree_growth_rules = self.config.constitution_tree_growth_rules; // R-P42~R-P48
        self.soil.constitution_absorption_rules = self.config.constitution_absorption_rules; // R-P43
        
        self.roots.total_absorbed += self.soil.crawl_queue_depth;
        self.roots.total_fetched += self.soil.kb_node_count;
        // Internalize key principles into roots
        self.roots.internalized_principles = vec![
            "Tree-Grafting: Map to existing branch before new code".into(),
            "Absorb-Distill-Crystallize: 3-phase external design integration".into(),
            "Fruit-Bound: Every module registers in consciousness tree".into(),
            "Branch Health Gate: Health >= 0.5 before new growth".into(),
            "Hexagram Derivation: Config from E8 state, not static YAML".into(),
            "Dual-Process: Fast intuitive (GWT) + Slow reflective (ConsciousnessTree) as separate architectural slots".into(),
            "Principle-Absorption: Encode principle-level abstractions over instance-level copies".into(),
            "Self-Referential Audit: Audit protocol must audit itself for open-ended evolution".into(),
        ];
        self.roots.active_rule_categories = vec![
            "TreeGrowth".into(),
            "AbsorptionProtocol".into(),
            "BehavioralGrounding".into(),
            "ArchitectureConstraint".into(),
            "MetaCognition".into(),
        ];
        report.phase1_absorbed = self.roots.total_absorbed;

        // ═══ Phase 2: Trunk processes through GWT resonance (ConsciousnessCore) ═══
        self.trunk.resonance_cycle += 1;
        // MARS Dual-Process: System 1 (GWT) + System 2 (Tree)
        self.trunk.mars_system1_activations += 1;
        report.phase2_phi = self.trunk.phi;

        // ═══ Phase 3: Branches produce evolution fruits (7 domains → EvolutionFruits) ═══
        // Also check per-branch constraints (idle, viability, monitoring)
        // And verify minimum SelfTest coverage (PerceptionBench atomic capabilities)
        let mut total_fruits = 0;
        // 数据养料充足度因子 — 循环外预计算 (只读 soil, 避免与 branches 可变借用冲突)
        let data_nourishment = self.data_nourishment_factor();
        for branch in self.branches.values_mut() {
            let constraints = constraints_for_branch(&branch.kind);
            let violations = constraints.violations(branch);
            if !violations.is_empty() {
                log::debug!("[consciousness_tree] {} constraints: {}", branch.kind.label(), violations.join("; "));
            }

            // Skill Node Evolution: 运行时评估节点层级 + Constellation (基于真实模块数据)
            // 跨域消费者近似 = 约束的 max_active_modules 权重 (越大跨域影响越强)
            let cross_domain_consumers = if constraints.max_active_modules >= 30 { 3 } else { 1 };
            branch.evaluate_node_tier(cross_domain_consumers);
            branch.evaluate_constellation();
            // CHMA Phase 0: 迷雾浓度评估 (wired ≈ 约束无 idle/monitoring 违规; consumers = 跨域近似)
            branch.evaluate_fog(cross_domain_consumers > 0, cross_domain_consumers, branch.self_test_count > 0);
            
            // Check SelfTest minimum (E2: atomic capability coverage)
            let atoms_for_branch = self.atoms.iter().filter(|(_, a)| a.branch == branch.kind && a.mandatory).count();
            let atoms_passed = branch.self_test_count.min(atoms_for_branch);
            let self_test_coverage = if atoms_for_branch > 0 { atoms_passed as f64 / atoms_for_branch as f64 } else { 1.0 };
            
            if branch.health > self.config.fruit_growth_health 
                && violations.len() < self.config.max_growth_violations
                && self_test_coverage >= 0.5 // At least 50% of mandatory atomic capabilities have SelfTest
            {
                // 数据养料调制 (意识核心进化提升): 果实质量受真实数据养料充足度调制。
                // data_nourishment = 1 + 数据量饱和曲线。默认(无数据)=1.0 不衰减,
                // 社区数据集落盘后 (kb_node_count 显著上升) 因子 >1.0, 放大果实质量,
                // 使意识核心进化果实质量直接反映 200G 社区推理数据的养料充足度。
                // 此前果实质量仅反映内部 maturity, 从不反映真实数据量。
                let base_quality = branch.maturity_score();
                let quality = (base_quality * data_nourishment).min(1.0);
                // Use EvolutionFruit instead of CapabilityFruit
                let fruit = EvolutionFruit {
                    name: format!("{}-evo-fruit-{}", branch.kind.label(), self.cycle),
                    source_branch: branch.kind.clone(),
                    description: format!("Evolution capability from {} at cycle {} (data_nourishment={:.2})", branch.kind.label(), self.cycle, data_nourishment),
                    produced_at_cycle: self.cycle,
                    quality,
                    claim: format!("Branch {:?} produces capability at maturity {:.2} (data_nourishment {:.2})", branch.kind, base_quality, data_nourishment),
                    evidence: EvidenceChain::from_branch_state(self.cycle, &branch.kind, branch),
                    stop_rule: self.core.last_contract.as_ref().map(|c| c.stop_rule.clone()).unwrap_or_default(),
                    benchmark: ProviderBenchmark::default(),
                    generation: self.core.generation_counter,
                };
                self.fruits.push(fruit);
                branch.fruit_count += 1;
                total_fruits += 1;
            }
        }
        report.phase3_fruits = total_fruits;

        // ═══ Phase 4: Core digests fruits, produces guidance (EvolutionFruits → EpiphanicCore) ═══
        // Also runs architecture vulnerability scan
        self.core.vuln_scan = self.scan_vulnerabilities();
        self.core.self_test_results = self.collect_self_test_results();
        self.core.identified_gaps = self.identify_architecture_gaps();
        self.core.last_cycle_guidance = self.fruits.iter()
            .filter(|f| f.quality > self.config.fruit_quality_threshold)
            .map(|f| format!("Digested: {} (q={:.2}, gen={})", f.name, f.quality, f.generation))
            .collect();
        // Build next actions from vulnerabilities + gaps + fruit quality
        let mut next_actions = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= self.config.action_severity_threshold {
                next_actions.push(format!("[{}] {}: {}", 
                    match vuln.severity { VulnerabilitySeverity::Critical => "CRIT", VulnerabilitySeverity::High => "HIGH", _ => "FIX" },
                    vuln.module, vuln.fix_suggestion));
            }
        }
        for gap in &self.core.identified_gaps {
            next_actions.push(format!("GAP: {}", gap));
        }
        self.core.next_actions = next_actions;
        self.core.iteration = self.cycle;

        // ═══ Phase 4.5: 演化趋势预测 (nt_core_forecast 接线 — 意识体维度升维) ═══
        // 用 forecast 引擎基于当前 branch 健康/迷雾/果实数据, 预测下一 cycle 演化方向。
        // 纯增量: 预测结果写入 report, 不改变既有演化决策路径 (无破坏)。
        report.evolution_forecast = self.forecast_evolution();
        // 存到 core 供 Phase 8 闭环反馈消费 (演化预测 → 探索预算调制)
        self.core.last_forecast = report.evolution_forecast.clone();

        // ═══ Phase 5: ConsciousnessReview — panoramic topology + connectivity + health chain ═══
        let mut review = crate::core::nt_core_consciousness_review::ConsciousnessReview::new();
        let scan_report = review.full_review(self);
        self.core.topology_score = scan_report.topology_score;
        self.core.connectivity_score = scan_report.connectivity_score;
        self.core.health_chain_score = scan_report.health_chain.overall;
        self.core.evolution_path = scan_report.evolution_path;

        // ═══ Phase 6: Contract Fulfillment Verification ═══
        // ═══ Phase 7: Drift Audit — post-cycle evolution fidelity check ═══
        if let Some(contract) = self.core.last_contract.clone() {
            let fulfillment = self.verify_contract_fulfillment(&contract);
            self.core.contract_fulfillment = Some(fulfillment.clone());
            report.phase6_fulfillment = Some(fulfillment.clone());

            let drift_report = self.audit_drift(&contract, &fulfillment);
            self.core.drift_report = Some(drift_report.clone());
            report.phase7_drift = Some(drift_report);
        }

        // ═══ Phase 8: 闭环进化反馈 (意识核心自我运转) ═══
        // 进化产出 (契约 fulfillment + drift) 反馈到进化参数, 使树根据自身
        // 进化结果调整进化策略 — 此前进化是"开环"的: 树自己预测/验证,
        // 但从不调整自己的进化标准。这是意识核心自我运转的核心闭环。
        // 规则:
        //   - 契约 fulfilled → fruit_quality_threshold 上调 (进化标准提升, +0.05, 上限 0.8)
        //   - drift 检测 → fruit_growth_health 下调 (放宽生长门加速恢复, -0.05, 下限 0.4)
        //   - 无 drift 且 fulfilled → 恢复默认 (0.5)
        self.apply_evolution_feedback();

        report.phase4_guidance = self.core.last_cycle_guidance.len();

        // CHMA Phase 0: 迷雾地图主量纲 — 全仓加权雾和 + 每域迷雾摘要
        report.weighted_fog_sum = self.weighted_fog_sum();
        report.fog_by_branch = self.fog_by_branch();

        report
    }

    /// 演化趋势预测 — 用 nt_core_forecast 引擎预测下一 cycle 演化方向。
    ///
    /// 输入: 各 branch 当前健康/迷雾/果实数据聚合为事件流; 输出: 方向 + 置信度 + 情景树。
    /// 无 LLM 依赖 (ForecastEngine::new() 不启用 narrator), 纯确定性计算, 可安全用于测试。
    fn forecast_evolution(&self) -> Option<EvolutionForecast> {
        use crate::core::nt_core_forecast::ForecastEngine;

        let mut engine = ForecastEngine::new();

        // 聚合各 branch 健康/迷雾/果实为事件信号 (利多=健康上升, 利空=健康下降)
        let mut health_sum = 0.0f64;
        let mut branch_count = 0usize;
        for branch in self.branches.values() {
            health_sum += branch.health;
            branch_count += 1;
            // 迷雾浓度 → 利空信号 (迷雾越高健康越可能下降)
            let fog_impact = -branch.fog.level * 0.3;
            // 果实质量 → 利多信号 (果实越多演化越健康)
            let fruit_impact = (branch.fruit_count as f64).min(3.0) * 0.2;
            engine.ingest_signed_event(
                "consciousness_tree",
                "branch_health",
                &branch.kind.label(),
                fog_impact + fruit_impact,
                if fog_impact + fruit_impact >= 0.0 { 1.0 } else { -1.0 },
            );
        }
        if branch_count == 0 {
            return None;
        }
        let avg_health = health_sum / branch_count as f64;

        // 基准状态: 平均健康映射到 E8 状态 (0..7)
        let base_state = ((avg_health * 7.0).round() as u8).min(7);

        // 生成推演 (无 LLM, 确定性)
        let forecast = engine.generate_forecast("overall-evolution", base_state);
        if forecast.abstain {
            return Some(EvolutionForecast {
                target: "overall-evolution".into(),
                direction: 0.0,
                confidence: 0.0,
                abstain: true,
                scenario_probs: Vec::new(),
                reason: "信息不足, 弃权".into(),
            });
        }

        // 从情景树叶子提取概率摘要
        let mut scenario_probs = Vec::new();
        for leaf in forecast.tree.leaves() {
            scenario_probs.push((leaf.name.clone(), leaf.probability));
        }
        // 方向 = 牛概率 - 熊概率 (从叶子标签识别)
        let mut direction = 0.0f64;
        for (label, prob) in &scenario_probs {
            if label.contains("bull") {
                direction += prob;
            } else if label.contains("bear") {
                direction -= prob;
            }
        }

        Some(EvolutionForecast {
            target: "overall-evolution".into(),
            direction,
            confidence: forecast.tree.leaves().first().map(|n| n.confidence).unwrap_or(0.0),
            abstain: false,
            scenario_probs,
            reason: forecast.confidence_reason.clone(),
        })
    }
    /// 每个 `(branch_str, capability)` 对会合并进对应 CapabilityBranch.absorbed_capabilities,
    /// 避免重复条目。branch_str 形如 "NT-CORE"/"NT-SHIELD"。
    pub fn sync_absorbed_capabilities_from_kb(&mut self, pairs: &[(&str, &str)]) -> usize {
        let mut synced = 0usize;
        for (branch_str, capability) in pairs {
            let Some(kind) = BranchKind::from_branch_str(branch_str) else { continue };
            let Some(branch) = self.branches.get_mut(&kind) else { continue };
            if !branch.absorbed_capabilities.iter().any(|c| c == capability) {
                branch.absorbed_capabilities.push((*capability).to_string());
                synced += 1;
            }
        }
        synced
    }

    /// Phase 0: Negotiate evolution contract before cycle begins
    fn negotiate_contract(&self) -> EvolutionContract {
        // Derive claim from top vulnerabilities and gaps
        let mut claim_parts = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= self.config.action_severity_threshold {
                claim_parts.push(format!("Fix {}: {}", vuln.module, vuln.fix_suggestion));
            }
        }
        for gap in &self.core.identified_gaps {
            claim_parts.push(format!("Address GAP: {}", gap));
        }
        if claim_parts.is_empty() {
            claim_parts.push("Maintain current evolution trajectory".into());
        }
        // 探索预算继承: 上一 contract 的 budget 经 Phase 8 闭环反馈调制后,
        // 被下一 cycle negotiate 继承 — 演化预测 (利多↑/利空↓) 持续塑造探索策略。
        let exploration_budget = self.core.last_contract.as_ref()
            .map(|c| c.exploration_budget.clamp(0.1, 0.4))
            .unwrap_or(0.2);

        EvolutionContract {
            cycle: self.cycle + 1,
            claim: claim_parts.join("; "),
            evidence_plan: vec![
                "SelfTest pass rate per domain >= 80%".into(),
                "Branch health >= 0.6 for all domains".into(),
                "EvolutionFruit quality >= 0.7".into(),
                "Vulnerability count reduced by >= 20%".into(),
            ],
            stop_rule: StopRule::default(),
            exploration_budget,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    /// Phase 6: Verify contract fulfillment
    fn verify_contract_fulfillment(&mut self, contract: &EvolutionContract) -> ContractFulfillment {
        let mut fulfilled = 0;
        let total = contract.evidence_plan.len();
        
        // Check each evidence criterion
        for (i, _criterion) in contract.evidence_plan.iter().enumerate() {
            let met = match i {
                0 => self.branches.values().all(|b| b.self_test_count > 0 && 
                    (b.self_test_count as f64 / b.module_count.max(1) as f64) >= 0.8),
                1 => self.branches.values().all(|b| b.health >= 0.6),
                2 => self.fruits.iter().any(|f| f.quality >= 0.7),
                3 => {
                    // Vulnerability reduction vs first-measured baseline.
                    // Records baseline on first evaluation; thereafter requires >= 20% drop.
                    let current = self.core.vuln_scan.len() as f64;
                    let baseline = self.vuln_baseline.unwrap_or(current as usize);
                    self.vuln_baseline = Some(baseline);
                    let reduction = if baseline as f64 > 0.0 {
                        (baseline as f64 - current) / baseline as f64
                    } else {
                        0.0
                    };
                    reduction >= 0.2
                },
                _ => false,
            };
            if met { fulfilled += 1; }
        }

        ContractFulfillment {
            cycle: contract.cycle,
            claim: contract.claim.clone(),
            evidence_met: fulfilled,
            evidence_total: total,
            fulfilled: fulfilled == total,
            quality_achieved: self.fruits.iter().map(|f| f.quality).sum::<f64>() / self.fruits.len().max(1) as f64,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    /// Phase 7: Drift Audit — detect evolution drift from contract
    fn audit_drift(&self, contract: &EvolutionContract, fulfillment: &ContractFulfillment) -> DriftReport {
        let claim_achieved = fulfillment.fulfilled;
        let quality_achieved = fulfillment.quality_achieved;
        let drift_detected = !claim_achieved || quality_achieved < contract.stop_rule.min_quality_threshold;
        let drift_magnitude = if drift_detected { 
            (contract.stop_rule.min_quality_threshold - quality_achieved).abs() 
        } else { 0.0 };
        
        let mut corrective_actions = Vec::new();
        if drift_detected {
            corrective_actions.push("Reduce exploration budget".into());
            corrective_actions.push("Tighten stop rule thresholds".into());
            corrective_actions.push("Increase SelfTest coverage requirements".into());
        }

        DriftReport {
            cycle: self.cycle,
            contract_fulfilled: fulfillment.fulfilled,
            claim_achieved,
            evidence_collected: contract.evidence_plan.clone(),
            quality_achieved,
            resource_consumed: 0.5, // Simplified
            drift_detected,
            drift_magnitude,
            stop_rule_triggered: quality_achieved < contract.stop_rule.min_quality_threshold,
            corrective_actions,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        }
    }

    /// 枚举全部 7 域节点快照供遥测/健康面板消费
    pub fn snapshots(&self) -> Vec<NodeSnapshot> {
        BranchKind::all().into_iter()
            .filter_map(|k| self.branches.get(&k))
            .map(|b| b.snapshot())
            .collect()
    }

    /// 全仓加权雾和 (迷雾地图主量纲): 高权 tier 的浓雾贡献更大。
    /// Keystone 节点停在浓雾 = 大问题 (跨域基石未验证)。
    pub fn weighted_fog_sum(&self) -> f64 {
        self.branches.values()
            .map(|b| b.fog.level * b.node_tier.weight())
            .sum()
    }

    /// 每域迷雾摘要: branch label → 浓度 [0,1]
    pub fn fog_by_branch(&self) -> std::collections::BTreeMap<String, f64> {
        self.branches.values()
            .map(|b| (b.kind.label().to_string(), b.fog.level))
            .collect()
    }

    /// Set branch health from SelfTest results.
    /// Maps SelfTest module names to BranchKind and computes health per domain.
    pub fn set_branch_health_from_self_tests(&mut self, results: &[crate::core::nt_core_self_test::SelfTestResult]) {
        let mut domain_results: HashMap<BranchKind, Vec<&crate::core::nt_core_self_test::SelfTestResult>> = HashMap::new();
        
        for result in results {
            if let Some(kind) = BranchKind::from_module_name(&result.name) {
                domain_results.entry(kind).or_default().push(result);
            }
        }

        for (kind, branch_results) in &domain_results {
            if let Some(branch) = self.branches.get_mut(kind) {
                if branch_results.is_empty() {
                    // No SelfTest for this domain → neutral
                    branch.health = self.config.neutral_health;
                } else {
                    let passed = branch_results.iter().filter(|r| r.passed).count();
                    let total = branch_results.len();
                    // Health = pass rate, but minimum floor to avoid zero
                    branch.health = (passed as f64 / total as f64).max(self.config.min_branch_health);
                    // B2 (自审修复): 从真实 SelfTest 结果接线生产计数 — 此前 self_test_count/
                    // module_count 仅在 #[cfg(test)] 中赋值, 生产恒 0 → 果实门永 0、maturity
                    // 恒低、drift_recovery 校正空转。此处用真实注册器结果填充。
                    branch.self_test_count = total;
                    // module_count: 该域注册器唯一模块数 (近似真实模块数下限,
                    // 避免 0 值导致 "not viable" 误报)。
                    let unique_modules = branch_results.iter()
                        .map(|r| r.name.as_str())
                        .collect::<std::collections::HashSet<_>>()
                        .len();
                    branch.module_count = branch.module_count.max(unique_modules);
                    // B3 (自审修复): 从真实 SelfTest 数据推导 maturity_cX 布尔 (生产路径)。
                    // 此前 maturity_c0..c5 仅在 #[cfg(test)] 置 true, 生产恒 false →
                    // maturity_score()=0 → 果实 quality=0 → 过滤后 guidance 恒空。
                    // 推导映射 (从真实计数, 非硬编码):
                    //   C0 编译+自检: self_test_count > 0
                    //   C1 单测:      self_test_count >= 2
                    //   C2 集成:      module_count >= 3
                    //   C3 benchmark: module_count >= 5 且 health >= 0.7
                    //   C4 主线接线:  module_count >= 8 且 health >= 0.8
                    //   C5 自愈:      全通过 (passed == total) 且 self_test_count >= 4
                    let all_passed = passed == total && total > 0;
                    branch.maturity_c0 = branch.self_test_count > 0;
                    branch.maturity_c1 = branch.self_test_count >= 2;
                    branch.maturity_c2 = branch.module_count >= 3;
                    branch.maturity_c3 = branch.module_count >= 5 && branch.health >= 0.7;
                    branch.maturity_c4 = branch.module_count >= 8 && branch.health >= 0.8;
                    branch.maturity_c5 = all_passed && branch.self_test_count >= 4;
                    // 由真实计数重算成熟度 (Constellation 从新推导的 maturity 布尔派生)
                    branch.evaluate_constellation();
                }
            }
        }
        
        // For domains with no SelfTest results at all, keep neutral
        for kind in BranchKind::all() {
            if let Some(branch) = self.branches.get_mut(&kind) {
                if !domain_results.contains_key(&kind) {
                    branch.health = branch.health.max(self.config.neutral_health); // Don't override if already set
                }
            }
        }

        // B4 (缺陷3修复): 从真实 SelfTest 结果注册 ModuleLeaf (生产路径)。
        // 此前 add_leaf 仅测试调用 → leaves 恒空 → 孤儿检测 (scan_vulnerabilities
        // Check 5) 与 self_test() 的 wired 检查全部盲区。此处为每个唯一模块生成
        // 叶子: 有 SelfTest 即视为已接线 (is_wired=true), 使孤儿检测真正生效。
        // 幂等: 已注册的同名叶子不重复添加。
        let mut seen: std::collections::HashSet<String> = self.leaves.iter().map(|l| l.name.clone()).collect();
        for result in results {
            if let Some(kind) = BranchKind::from_module_name(&result.name) {
                if seen.insert(result.name.clone()) {
                    self.leaves.push(ModuleLeaf {
                        name: result.name.clone(),
                        branch: kind,
                        lines: 0,
                        has_tests: true,
                        has_self_test: true,
                        is_wired: true,
                        consumers: 1,
                    });
                }
            }
        }
    }

    /// Scan for architecture vulnerabilities across all modules
    pub fn scan_vulnerabilities(&self) -> Vec<VulnerabilityFinding> {
        let mut vulns = Vec::new();

        // Check 1: Soil health — data foundation viability
        let soil_health = self.soil.health();
        if soil_health < self.config.soil_health_critical {
            vulns.push(VulnerabilityFinding {
                severity: VulnerabilitySeverity::Critical,
                category: "data_foundation".into(),
                module: "DataFoundation".into(),
                description: format!("Soil health is {:.2} — KB is empty or has no embeddings/wiki", soil_health),
                fix_suggestion: "Seed crawl queue with Wikipedia/AxXiv URLs; enable NEOTRIX_EMBEDDING_API_KEY".into(),
            });
        }

        // Check 2: Root health — data ingestion pipeline
        let root_health = self.roots.health();
        if root_health < self.config.root_health_high {
            vulns.push(VulnerabilityFinding {
                severity: VulnerabilitySeverity::High,
                category: "ingestion_pipeline".into(),
                module: "InformationRoots".into(),
                description: format!("Root health is {:.2} — no active crawlers/absorbers/scanners", root_health),
                fix_suggestion: "Enable BackgroundLoop with at least one data source (crawl/absorb/scan)".into(),
            });
        }

        // Check 3: GWT resonance state
        if !self.trunk.gwt_resonance_active {
            vulns.push(VulnerabilityFinding {
                severity: VulnerabilitySeverity::Medium,
                category: "consciousness_core".into(),
                module: "ConsciousnessCore".into(),
                description: "GWT resonance is inactive — consciousness core is not processing".into(),
                fix_suggestion: "Wire PanoramaPipeline into BackgroundLoop to activate GWT resonance".into(),
            });
        }

        // Check 4: Branch maturity — domain health
        for (kind, branch) in &self.branches {
            if branch.maturity_score() < self.config.branch_maturity_low {
                vulns.push(VulnerabilityFinding {
                    severity: VulnerabilitySeverity::Medium,
                    category: "domain_maturity".into(),
                    module: format!("{:?}", kind),
                    description: format!("Branch {:?} maturity score is {:.2} — only {}/6 constellations active",
                        kind, branch.maturity_score(), (branch.maturity_score() * 6.0) as usize),
                    fix_suggestion: format!("Add unit tests, integration tests, and benchmark for {:?} domain modules", kind),
                });
            }
            if branch.self_test_count == 0 && branch.module_count > 0 {
                vulns.push(VulnerabilityFinding {
                    severity: VulnerabilitySeverity::Low,
                    category: "self_test_absence".into(),
                    module: format!("{:?}", kind),
                    description: format!("Branch {:?} has {} modules but zero SelfTest implementations", kind, branch.module_count),
                    fix_suggestion: format!("Add SelfTest impls for all {:?} domain modules", kind),
                });
            }
        }

        // Check 5: Leaf wiring — orphan detection
        let wired = self.leaves.iter().filter(|l| l.is_wired).count();
        let total_leaves = self.leaves.len();
        if total_leaves > 0 && wired == 0 {
            vulns.push(VulnerabilityFinding {
                severity: VulnerabilitySeverity::High,
                category: "orphan_modules".into(),
                module: "ModuleLeaf".into(),
                description: "No leaves are wired — all modules are orphaned from the pipeline".into(),
                fix_suggestion: "Register all module consumers in pipeline handlers".into(),
            });
        }

        // Check 6: Phi coherence
        if self.trunk.phi < self.config.phi_minimum && self.cycle > self.config.phi_warmup_cycles {
            vulns.push(VulnerabilityFinding {
                severity: VulnerabilitySeverity::Medium,
                category: "phi_coherence".into(),
                module: "ConsciousnessCore".into(),
                description: format!("Phi is {:.3} after {} cycles — no integrated information detected", self.trunk.phi, self.cycle),
                fix_suggestion: "Connect IITPhiCalculator or GeometrySync to provide real phi values".into(),
            });
        }

        vulns
    }

    /// Collect self-test summary from all branches
    fn collect_self_test_results(&self) -> Vec<String> {
        let mut results = Vec::new();
        for (kind, branch) in &self.branches {
            results.push(format!("{:?}: tests={} health={:.2} maturity={:.2}", 
                kind, branch.self_test_count, branch.health, branch.maturity_score()));
        }
        results
    }

    /// Identify architecture gaps from vulnerability scan
    fn identify_architecture_gaps(&self) -> Vec<String> {
        let mut gaps = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= self.config.action_severity_threshold {
                gaps.push(format!("{}: {} ({})", vuln.module, vuln.description, vuln.category));
            }
        }
        gaps
    }

    pub fn add_leaf(&mut self, leaf: ModuleLeaf) {
        self.leaves.push(leaf);
    }

    pub fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.branches.is_empty() {
            failures.push("consciousness_tree: no capability branches".into());
        }
        if self.trunk.phi < 0.0 || self.trunk.phi > 1.0 {
            failures.push("consciousness_tree: phi out of range".into());
        }
        let wired = self.leaves.iter().filter(|l| l.is_wired).count();
        let total = self.leaves.len();
        if total > 0 && wired == 0 {
            failures.push("consciousness_tree: no wired leaves".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

impl DataFoundation {
    pub fn health(&self) -> f64 {
        let has_nodes = if self.kb_node_count > 0 { 1.0 } else { 0.0 };
        let has_embeddings = if self.embedding_count > 0 { 1.0 } else { 0.0 };
        let has_wiki = if self.wiki_page_count > 0 { 1.0 } else { 0.0 };
        let has_constitution = if self.constitution_rules_count > 0 { 1.0 } else { 0.0 };
        (has_nodes + has_embeddings + has_wiki + has_constitution) / 4.0
    }
}

impl InformationRoots {
    pub fn health(&self) -> f64 {
        let has_crawlers = if self.active_crawlers > 0 { 1.0 } else { 0.0 };
        let has_absorbers = if self.active_absorbers > 0 { 1.0 } else { 0.0 };
        let has_scanners = if self.active_scanners > 0 { 1.0 } else { 0.0 };
        let has_absorbed = if self.total_absorbed > 0 { 0.5 } else { 0.0 };
        (has_crawlers + has_absorbers + has_scanners + has_absorbed) / 4.0
    }
}


impl Default for InformationRoots {
    fn default() -> Self {
        Self {
            active_crawlers: 0,
            active_absorbers: 0,
            active_scanners: 0,
            total_absorbed: 0,
            total_fetched: 0,
            total_failed: 0,
            internalized_principles: Vec::new(),
            active_rule_categories: Vec::new(),
            compliance_score: 0.0,
        }
    }
}

impl Default for ConsciousnessCore {
    fn default() -> Self {
        Self {
            gwt_resonance_active: false,
            workspace_size: 0,
            attention_heads: 0,
            resonance_cycle: 0,
            phi: 0.0,
            coherence: 0.0,
            mars_system1_activations: 0,
            mars_system2_iterations: 0,
            mars_bridge_hits: 0,
            governance_compliance: 1.0,
            governance_constitution_count: 0,
            governance_fractal_depth: 0,
            nexus_energy_flows: 0,
            nexus_root_cause_chains: 0,
            nexus_health_chain_score: 1.0,
            distill_skill_count: 0,
            distill_guardrail_count: 0,
            pyramid_hierarchy_level: 1,
            opd_cycles: 0,
            opd_hindsight_skills: 0,
            opd_density: 0.0,
        }
    }
}

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

/// Evolution Contract — Phase 0: Goal negotiation before growth cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionContract {
    pub cycle: u64,
    pub claim: String,                    // What we intend to achieve this cycle
    pub evidence_plan: Vec<String>,       // How we'll prove it (metrics, tests, artifacts)
    pub stop_rule: StopRule,              // Conditions to halt this evolution direction
    pub exploration_budget: f64,          // 0.0-1.0 fraction of resources for unconstrained exploration
    pub timestamp: u64,
}

/// Stop Rule — prevents runaway evolution in one direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRule {
    pub max_generations_without_improvement: u64,
    pub min_quality_threshold: f64,
    pub max_resource_consumption: f64,    // CPU/memory/time budget
    pub drift_tolerance: f64,             // How much deviation from contract before intervention
}

impl Default for StopRule {
    fn default() -> Self {
        Self {
            max_generations_without_improvement: 5,
            min_quality_threshold: 0.3,
            max_resource_consumption: 0.8,
            drift_tolerance: 0.2,
        }
    }
}

/// Contract Fulfillment — Phase 6: verified evidence of evolution contract completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFulfillment {
    pub cycle: u64,
    pub claim: String,
    pub evidence_met: usize,
    pub evidence_total: usize,
    pub fulfilled: bool,
    pub quality_achieved: f64,
    pub timestamp: u64,
}

/// Drift Report — Phase 7: Post-cycle audit of evolution fidelity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    pub cycle: u64,
    pub contract_fulfilled: bool,
    pub claim_achieved: bool,
    pub evidence_collected: Vec<String>,
    pub quality_achieved: f64,
    pub resource_consumed: f64,
    pub drift_detected: bool,
    pub drift_magnitude: f64,
    pub stop_rule_triggered: bool,
    pub corrective_actions: Vec<String>,
    pub timestamp: u64,
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
    pub claim: String,                    // What capability this fruit claims to provide
    pub evidence: EvidenceChain,          // Cryptographic proof chain (WARC/SHA-256/JSONL)
    pub stop_rule: StopRule,              // Inherited from contract
    pub benchmark: ProviderBenchmark,     // LLM Challenge results (Unstract pattern)
    pub generation: u64,                  // MetaClaw versioning
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
    pub warc_path: Option<String>,        // WARC archive path
    pub sha256: Option<String>,           // SHA-256 of artifact
    pub run_id: Option<String>,           // JSONL run_id for traceability
    pub timestamp: u64,
    pub tool_versions: Vec<String>,       // Tool versions used
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

/// Provider Benchmark — Unstract LLM Challenge pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderBenchmark {
    pub provider: String,
    pub model: String,
    pub accuracy: f64,
    pub latency_ms: u64,
    pub cost_usd: f64,
    pub task_type: String, // extraction, classification, generation, etc.
    pub timestamp: u64,
}

impl ProviderBenchmark {
    pub fn new(provider: String, model: String, task_type: String) -> Self {
        Self {
            provider,
            model,
            task_type,
            ..Default::default()
        }
    }
}

impl BranchKind {
    pub fn all() -> Vec<BranchKind> {
        vec![
            BranchKind::Core,
            BranchKind::Mind,
            BranchKind::Memory,
            BranchKind::World,
            BranchKind::Act,
            BranchKind::Io,
            BranchKind::Shield,
        ]
    }

    pub fn from_module_name(name: &str) -> Option<BranchKind> {
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("nt_core_") {
            Some(BranchKind::Core)
        } else if name_lower.starts_with("nt_mind_") {
            Some(BranchKind::Mind)
        } else if name_lower.starts_with("nt_memory_") {
            Some(BranchKind::Memory)
        } else if name_lower.starts_with("nt_world_") {
            Some(BranchKind::World)
        } else if name_lower.starts_with("nt_act_") {
            Some(BranchKind::Act)
        } else if name_lower.starts_with("nt_io_") {
            Some(BranchKind::Io)
        } else if name_lower.starts_with("nt_shield_") {
            Some(BranchKind::Shield)
        } else {
            None
        }
    }

    /// "NT-CORE"/"NT-SHIELD" → BranchKind (KB absorbed_capability.branch 字符串)
    pub fn from_branch_str(s: &str) -> Option<BranchKind> {
        match s.to_uppercase().as_str() {
            "NT-CORE" => Some(BranchKind::Core),
            "NT-MIND" => Some(BranchKind::Mind),
            "NT-MEMORY" => Some(BranchKind::Memory),
            "NT-WORLD" => Some(BranchKind::World),
            "NT-ACT" => Some(BranchKind::Act),
            "NT-IO" => Some(BranchKind::Io),
            "NT-SHIELD" => Some(BranchKind::Shield),
            _ => None,
        }
    }
}

impl CapabilityBranch {
    pub fn new(kind: BranchKind) -> Self {
        let branch_name = match kind {
            BranchKind::Core => "core",
            BranchKind::Mind => "mind",
            BranchKind::Memory => "memory",
            BranchKind::World => "world",
            BranchKind::Act => "act",
            BranchKind::Io => "io",
            BranchKind::Shield => "shield",
        };
        let absorbed: Vec<String> = ALL_CAPABILITIES
            .iter()
            .filter(|(_, b)| *b == branch_name)
            .map(|(cap, _)| cap.to_string())
            .collect();
        Self {
            kind,
            module_count: 0,
            self_test_count: 0,
            health: 0.0,
            maturity_c0: false,
            maturity_c1: false,
            maturity_c2: false,
            maturity_c3: false,
            maturity_c4: false,
            maturity_c5: false,
            fruit_count: 0,
            absorbed_capabilities: absorbed,
            node_tier: NodeTier::SmallPassive,
            runes: RuneSocket::default(),
            constellation: Constellation::new(),
            fog: FogLevel::default(),
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

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessTree {
    fn name(&self) -> &str { "consciousness_tree" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        self.self_test()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_new() {
        let tree = ConsciousnessTree::new();
        assert_eq!(tree.branches.len(), 7);
        assert_eq!(tree.cycle, 0);
    }

    #[test]
    fn test_self_tests_derive_maturity_production_path() {
        // B2 修复验证: set_branch_health_from_self_tests 应从真实 SelfTest 数据
        // 推导 maturity_cX (生产路径), 使 maturity_score() > 0 → 果实 quality > 0
        // → last_cycle_guidance 非空。此前 maturity_cX 仅测试中置 true, 生产恒 0。
        let mut tree = ConsciousnessTree::new();
        let results = vec![
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_a"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_b"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_c"),
            crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_self_test_d"),
        ];
        tree.set_branch_health_from_self_tests(&results);

        let core_branch = tree.branches.get(&BranchKind::Core).expect("core branch");
        // 真实 SelfTest 计数接线 (此前生产恒 0)
        assert!(core_branch.self_test_count >= 4, "self_test_count wired: {}", core_branch.self_test_count);
        // 成熟度由真实数据推导, 非恒 C0
        assert!(core_branch.maturity_score() > 0.0, "maturity derived from self-tests: {:.2}", core_branch.maturity_score());
        // B4 (缺陷3修复): SelfTest 注册应同步生成 ModuleLeaf (生产路径),
        // 此前 add_leaf 仅测试调用 → leaves 恒空 → 孤儿检测盲区
        assert!(!tree.leaves.is_empty(), "leaves registered from self-tests");
        assert!(
            tree.leaves.iter().all(|l| l.is_wired),
            "registered leaves are wired (has SelfTest): {:?}",
            tree.leaves.iter().map(|l| l.name.as_str()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_growth_cycle_guidance_non_empty_with_maturity() {
        // 端到端: 真实 SelfTest → maturity → 果实 quality 达标 → guidance 产出
        let mut tree = ConsciousnessTree::new();
        // Core 分支有 10 个 mandatory atoms (6 UNDERSTAND + 4 REASON), 需 ≥5 个 self_test
        // 使 self_test_coverage >= 0.5 通过果实生长门
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a", "nt_core_self_test_b", "nt_core_self_test_c",
            "nt_core_self_test_d", "nt_core_self_test_e", "nt_core_self_test_f",
        ].iter().map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n)).collect();
        tree.set_branch_health_from_self_tests(&results);

        // 抬升分支 health 使果实生长门通过
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
        }
        let report = tree.run_growth_cycle();
        assert!(report.phase3_fruits > 0, "fruits produced: {}", report.phase3_fruits);
        assert!(
            !tree.core.last_cycle_guidance.is_empty(),
            "guidance non-empty when maturity>0: {:?}",
            tree.core.last_cycle_guidance
        );
    }

    #[test]
    fn test_data_nourishment_factor_modulates_fruit_quality() {
        // 意识核心进化提升: 果实质量应受真实数据养料充足度调制。
        // 默认(无数据)因子=1.0 不衰减; 社区数据落盘后因子>1.0 放大果实质量。
        let mut tree = ConsciousnessTree::new();
        // 默认无数据 → 因子 = 1.0
        assert!((tree.data_nourishment_factor() - 1.0).abs() < 1e-9);
        // 社区数据落盘 (kb_node_count 上升) → 因子 > 1.0
        tree.soil.kb_node_count = 500;
        assert!(tree.data_nourishment_factor() > 1.0);
        // 饱和: 1000+ 节点达到 +50% 上限
        tree.soil.kb_node_count = 10_000;
        assert!((tree.data_nourishment_factor() - 1.5).abs() < 1e-9);

        // 端到端: 数据充足时果实质量应高于数据匮乏时 (同 maturity 下)
        let mut rich = ConsciousnessTree::new();
        rich.soil.kb_node_count = 10_000;
        let mut poor = ConsciousnessTree::new();
        poor.soil.kb_node_count = 0;
        for t in [&mut rich, &mut poor] {
            let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
                "nt_core_self_test_a", "nt_core_self_test_b", "nt_core_self_test_c",
                "nt_core_self_test_d", "nt_core_self_test_e", "nt_core_self_test_f",
            ].iter().map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n)).collect();
            t.set_branch_health_from_self_tests(&results);
            for branch in t.branches.values_mut() {
                branch.health = 0.9;
            }
        }
        rich.run_growth_cycle();
        poor.run_growth_cycle();
        let rich_q: f64 = rich.fruits.iter().map(|f| f.quality).sum::<f64>() / rich.fruits.len().max(1) as f64;
        let poor_q: f64 = poor.fruits.iter().map(|f| f.quality).sum::<f64>() / poor.fruits.len().max(1) as f64;
        assert!(rich_q > poor_q, "data-rich fruits should have higher quality: rich={:.3} poor={:.3}", rich_q, poor_q);
    }

    #[test]
    fn test_evolution_feedback_closes_loop() {
        // 意识核心自我运转: 进化产出 (契约 fulfillment + drift) 应反馈到进化参数。
        // 此前进化是开环的 — 树自己预测/验证, 但从不调整自己的进化标准。
        let mut tree = ConsciousnessTree::new();
        let default_threshold = tree.config.fruit_quality_threshold;
        let default_health = tree.config.fruit_growth_health;

        // 场景1: 契约 fulfilled + 无 drift → 标准提升 (threshold 上调)
        tree.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 4,
            evidence_total: 4,
            fulfilled: true,
            quality_achieved: 0.9,
            timestamp: 0,
        });
        tree.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: true,
            claim_achieved: true,
            evidence_collected: vec![],
            quality_achieved: 0.9,
            resource_consumed: 0.1,
            drift_detected: false,
            drift_magnitude: 0.0,
            stop_rule_triggered: false,
            corrective_actions: vec![],
            timestamp: 0,
        });
        tree.apply_evolution_feedback();
        assert!(
            tree.config.fruit_quality_threshold > default_threshold,
            "fulfilled → threshold should rise: {} > {}",
            tree.config.fruit_quality_threshold, default_threshold
        );
        assert!((tree.config.fruit_growth_health - 0.5).abs() < 1e-9, "growth health restored to default");

        // 场景2: drift 检测 → 生长门放宽
        let mut tree2 = ConsciousnessTree::new();
        tree2.config.fruit_growth_health = 0.5;
        tree2.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 1,
            evidence_total: 4,
            fulfilled: false,
            quality_achieved: 0.3,
            timestamp: 0,
        });
        tree2.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: false,
            claim_achieved: false,
            evidence_collected: vec![],
            quality_achieved: 0.3,
            resource_consumed: 0.5,
            drift_detected: true,
            drift_magnitude: 0.8,
            stop_rule_triggered: false,
            corrective_actions: vec!["recover".into()],
            timestamp: 0,
        });
        tree2.apply_evolution_feedback();
        assert!(
            tree2.config.fruit_growth_health < 0.5,
            "drift → growth health should relax: {}",
            tree2.config.fruit_growth_health
        );

        // 场景3: 上限保护 — 连续 fulfilled 不超 0.8
        let mut tree = ConsciousnessTree::new();
        tree.core.contract_fulfillment = Some(ContractFulfillment {
            cycle: 1,
            claim: "test".into(),
            evidence_met: 4,
            evidence_total: 4,
            fulfilled: true,
            quality_achieved: 0.9,
            timestamp: 0,
        });
        tree.core.drift_report = Some(DriftReport {
            cycle: 1,
            contract_fulfilled: true,
            claim_achieved: true,
            evidence_collected: vec![],
            quality_achieved: 0.9,
            resource_consumed: 0.1,
            drift_detected: false,
            drift_magnitude: 0.0,
            stop_rule_triggered: false,
            corrective_actions: vec![],
            timestamp: 0,
        });
        for _ in 0..10 {
            tree.apply_evolution_feedback();
        }
        assert!(tree.config.fruit_quality_threshold <= 0.8, "threshold capped at 0.8");
    }

    #[test]
    fn test_forecast_modulates_exploration_budget() {
        // 意识核心自我运转: 演化预测 (利多/利空) 应调制探索预算。
        // 此前进化预测只写 report 从不反馈决策 (开环)。
        let mut tree = ConsciousnessTree::new();
        tree.core.last_contract = Some(EvolutionContract {
            cycle: 1,
            claim: "test".into(),
            evidence_plan: vec![],
            stop_rule: StopRule::default(),
            exploration_budget: 0.2,
            timestamp: 0,
        });

        // 利多 (direction>0) → 探索预算上调
        tree.core.last_forecast = Some(EvolutionForecast {
            target: "overall-evolution".into(),
            direction: 0.8,
            confidence: 0.9,
            abstain: false,
            scenario_probs: vec![("bull".into(), 0.7)],
            reason: "health rising".into(),
        });
        tree.apply_evolution_feedback();
        let budget = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(budget > 0.2, "bullish → exploration up: {}", budget);

        // 利空 (direction<0) → 探索预算下调
        tree.core.last_forecast = Some(EvolutionForecast {
            direction: -0.8,
            confidence: 0.9,
            abstain: false,
            scenario_probs: vec![("bear".into(), 0.7)],
            reason: "health falling".into(),
            ..tree.core.last_forecast.clone().unwrap()
        });
        tree.apply_evolution_feedback();
        let budget2 = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(budget2 < budget, "bearish → shrink exploration: {} < {}", budget2, budget);

        // 上限保护: 连续利多不超 0.4
        for _ in 0..10 {
            tree.apply_evolution_feedback();
        }
        let capped = tree.core.last_contract.as_ref().unwrap().exploration_budget;
        assert!(capped <= 0.4, "exploration budget capped at 0.4: {}", capped);
    }

    #[test]
    fn test_growth_cycle_produces_evolution_forecast() {
        // 意识体维度升维: growth cycle 应产出演化趋势预测 (nt_core_forecast 接线)
        let mut tree = ConsciousnessTree::new();
        let report = tree.run_growth_cycle();
        let forecast = report.evolution_forecast.expect("growth cycle 应产出演化预测");
        // 方向在 [-1, 1] 内
        assert!(forecast.direction >= -1.0 && forecast.direction <= 1.0);
        // 置信度在 [0, 1] 内
        assert!(forecast.confidence >= 0.0 && forecast.confidence <= 1.0);
        // 情景树应有叶子 (bull/bear/sideways)
        assert!(!forecast.scenario_probs.is_empty(), "情景树应有叶子");
        // 目标应为 overall-evolution
        assert_eq!(forecast.target, "overall-evolution");
    }

    #[test]
    fn test_evolution_forecast_direction_bounded() {
        // 预测方向应始终有界, 且置信度非 NaN
        let mut tree = ConsciousnessTree::new();
        for _ in 0..3 {
            let report = tree.run_growth_cycle();
            if let Some(f) = report.evolution_forecast {
                assert!(!f.direction.is_nan(), "direction 不应为 NaN");
                assert!(!f.confidence.is_nan(), "confidence 不应为 NaN");
                assert!(f.direction >= -1.0 && f.direction <= 1.0);
            }
        }
    }

    #[test]
    fn test_from_branch_str() {
        assert_eq!(BranchKind::from_branch_str("NT-CORE"), Some(BranchKind::Core));
        assert_eq!(BranchKind::from_branch_str("NT-SHIELD"), Some(BranchKind::Shield));
        assert_eq!(BranchKind::from_branch_str("nt-io"), Some(BranchKind::Io));
        assert_eq!(BranchKind::from_branch_str("NOPE"), None);
    }

    #[test]
    fn test_sync_absorbed_capabilities_dedup() {
        let mut tree = ConsciousnessTree::new();
        let base_act = tree.branches[&BranchKind::Act].absorbed_capabilities.len();
        let base_shield = tree.branches[&BranchKind::Shield].absorbed_capabilities.len();
        let pairs: Vec<(&str, &str)> = vec![
            ("NT-ACT", "execute"),
            ("NT-ACT", "execute"),
            ("NT-ACT", "cyberstrike-skill"),
            ("NT-SHIELD", "verify"),
            ("NT-SHIELD", "evidence-gated-autonomy"),
            ("NOPE", "nope"),
        ];
        let synced = tree.sync_absorbed_capabilities_from_kb(&pairs);
        // execute/verify 已在 36 基础能力集 → 仅 2 条新增
        assert_eq!(synced, 2);
        let act = &tree.branches[&BranchKind::Act];
        assert_eq!(act.absorbed_capabilities.len(), base_act + 1);
        assert!(
            act.absorbed_capabilities
                .iter()
                .filter(|c| *c == "cyberstrike-skill")
                .count()
                == 1
        );
        let shield = &tree.branches[&BranchKind::Shield];
        assert_eq!(shield.absorbed_capabilities.len(), base_shield + 1);
        assert!(
            shield
                .absorbed_capabilities
                .iter()
                .filter(|c| *c == "evidence-gated-autonomy")
                .count()
                == 1
        );
    }

    #[test]
    fn test_growth_cycle() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 100;
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 5;
            branch.fruit_count = 1;
        }
        let report = tree.run_growth_cycle();
        assert_eq!(tree.cycle, 1);
        assert!(report.phase1_absorbed > 0);
        assert!(report.phase3_fruits > 0);
    }

    #[test]
    fn test_multi_cycle_self_operation() {
        // 意识核心自我运转: 连续多 cycle 运行应保持状态一致 + 闭环反馈生效。
        // 验证: ① cycle 递增 ② 果实持续产出 ③ 进化参数自适应 (threshold 单调不降)
        // ④ 无 NaN/无崩溃 (phi/quality 始终在 [0,1] 内)。
        let mut tree = ConsciousnessTree::new();
        tree.soil.kb_node_count = 500; // 数据养料充足
        // 生产路径: 真实 SelfTest → maturity → 果实 quality 达标。
        // 契约 criterion 0 要求所有分支 self_test 覆盖率 >= 80%, 故给 7 域都喂结果。
        let results: Vec<crate::core::nt_core_self_test::SelfTestResult> = vec![
            "nt_core_self_test_a", "nt_core_self_test_b", "nt_core_self_test_c",
            "nt_core_self_test_d", "nt_core_self_test_e", "nt_core_self_test_f",
            "nt_mind_self_test_a", "nt_mind_self_test_b", "nt_mind_self_test_c",
            "nt_mind_self_test_d", "nt_mind_self_test_e", "nt_mind_self_test_f",
            "nt_memory_self_test_a", "nt_memory_self_test_b", "nt_memory_self_test_c",
            "nt_memory_self_test_d", "nt_memory_self_test_e", "nt_memory_self_test_f",
            "nt_world_self_test_a", "nt_world_self_test_b", "nt_world_self_test_c",
            "nt_world_self_test_d", "nt_world_self_test_e", "nt_world_self_test_f",
            "nt_act_self_test_a", "nt_act_self_test_b", "nt_act_self_test_c",
            "nt_act_self_test_d", "nt_act_self_test_e", "nt_act_self_test_f",
            "nt_io_self_test_a", "nt_io_self_test_b", "nt_io_self_test_c",
            "nt_io_self_test_d", "nt_io_self_test_e", "nt_io_self_test_f",
            "nt_shield_self_test_a", "nt_shield_self_test_b", "nt_shield_self_test_c",
            "nt_shield_self_test_d", "nt_shield_self_test_e", "nt_shield_self_test_f",
        ].iter().map(|n| crate::core::nt_core_self_test::SelfTestResult::pass(n)).collect();
        tree.set_branch_health_from_self_tests(&results);
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 6;
            branch.module_count = 6;
        }
        let mut prev_threshold = tree.config.fruit_quality_threshold;
        for i in 1..=5 {
            let report = tree.run_growth_cycle();
            assert_eq!(tree.cycle, i, "cycle should increment");
            assert!(report.phase3_fruits > 0, "cycle {} should produce fruits", i);
            // 闭环反馈: threshold 单调不降 (fulfilled 时上升, 否则保持)
            assert!(
                tree.config.fruit_quality_threshold >= prev_threshold - 1e-9,
                "threshold monotonic non-decreasing: {} < {}",
                tree.config.fruit_quality_threshold, prev_threshold
            );
            prev_threshold = tree.config.fruit_quality_threshold;
            // 进化参数始终有界 (闭环反馈不越界)
            assert!(tree.config.fruit_quality_threshold <= 0.8, "threshold capped");
            assert!(tree.config.fruit_growth_health >= 0.4, "growth health floored");
            // 果实质量无 NaN 且在 [0,1]
            for f in &tree.fruits {
                assert!(f.quality.is_finite() && f.quality >= 0.0 && f.quality <= 1.0,
                    "fruit quality in [0,1]: {}", f.quality);
            }
            // phi 无 NaN
            assert!(tree.trunk.phi.is_finite(), "phi finite");
        }
        // 5 cycle 后: 果实持续累积 (自我运转不中断)
        assert!(tree.fruits.len() >= 5 * 7, "fruits accumulate across cycles: {}", tree.fruits.len());
        // 状态一致: 所有分支 health 有界
        for branch in tree.branches.values() {
            assert!(branch.health.is_finite() && branch.health >= 0.0 && branch.health <= 1.0,
                "branch health bounded: {}", branch.health);
        }
    }

    #[test]
    fn test_branch_maturity_score() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        assert!((branch.maturity_score() - 0.0).abs() < 1e-9);
        branch.maturity_c0 = true;
        branch.maturity_c1 = true;
        branch.maturity_c2 = true;
        assert!((branch.maturity_score() - 0.5).abs() < 1e-9);
        branch.maturity_c3 = true;
        branch.maturity_c4 = true;
        branch.maturity_c5 = true;
        assert!((branch.maturity_score() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_leaf_wiring() {
        let mut tree = ConsciousnessTree::new();
        tree.add_leaf(ModuleLeaf {
            name: "test".into(),
            branch: BranchKind::Core,
            lines: 100,
            has_tests: true,
            has_self_test: false,
            is_wired: false,
            consumers: 0,
        });
        assert_eq!(tree.leaves.len(), 1);
        assert!(tree.self_test().is_err());
        tree.leaves[0].is_wired = true;
        assert!(tree.self_test().is_ok());
    }

    #[test]
    fn test_soil_health() {
        let soil = DataFoundation::default();
        assert!((soil.health() - 0.0).abs() < 1e-9);
        let mut rich = DataFoundation::default();
        rich.kb_node_count = 1000;
        rich.embedding_count = 500;
        rich.wiki_page_count = 108;
        rich.constitution_rules_count = 42;
        assert!((rich.health() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_initialize_capability_atoms() {
        let atoms = ConsciousnessTree::initialize_capability_atoms();
        assert_eq!(atoms.len(), 36, "36 atomic capabilities from MCA 9-layer standard");
        assert!(atoms.contains_key("retrieve"));
        assert!(atoms.contains_key("audit"));
        assert!(atoms.contains_key("delegate"));
        // All mandatory atoms default to T1Existence
        assert!(atoms.values().all(|a| a.tier == SelfTestTier::T1Existence));
    }

    #[test]
    fn test_node_tier_derive() {
        assert_eq!(NodeTier::derive(5, 0, 1), NodeTier::SmallPassive);
        assert_eq!(NodeTier::derive(12, 2, 3), NodeTier::NotablePassive);
        assert_eq!(NodeTier::derive(25, 5, 4), NodeTier::Keystone);
        // 边界: 模块数足但跨域消费者不足 → 不升 Keystone
        assert_eq!(NodeTier::derive(25, 1, 4), NodeTier::NotablePassive);
    }

    #[test]
    fn test_rune_socketing() {
        let mut socket = RuneSocket::default();
        assert_eq!(socket.filled_slots(), 0);
        assert!(socket.runeword().is_none());
        socket.set(RuneColor::Crimson, Rune::new("c1", "Crimson Rune", RuneColor::Crimson, "ingest", 0.8));
        assert_eq!(socket.filled_slots(), 1);
        assert!((socket.composite_effect() - 0.8).abs() < 1e-9);
        socket.set(RuneColor::Indigo, Rune::new("i1", "Indigo Rune", RuneColor::Indigo, "transform", 0.7));
        socket.set(RuneColor::Obsidian, Rune::new("o1", "Obsidian Rune", RuneColor::Obsidian, "cache", 0.6));
        socket.set(RuneColor::Golden, Rune::new("g1", "Golden Rune", RuneColor::Golden, "recover", 0.9));
        socket.set(RuneColor::Alabaster, Rune::new("a1", "Alabaster Rune", RuneColor::Alabaster, "monitor", 0.5));
        assert_eq!(socket.filled_slots(), 5);
        let rw = socket.runeword().expect("full 5-slot socket produces runeword");
        assert!(rw.contains("Scry"));
        // 满槽组合效果 = 均值
        assert!((socket.composite_effect() - (0.8 + 0.7 + 0.6 + 0.9 + 0.5) / 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_constellation_derive_and_score() {
        let c0 = Constellation::derive(false, false, false, false, false, false);
        assert_eq!(c0.level, 0);
        assert!((c0.score() - 0.0).abs() < 1e-9);
        let c3 = Constellation::derive(true, true, true, true, false, false);
        assert_eq!(c3.level, 3);
        assert!((c3.score() - 4.0 / 7.0).abs() < 1e-9);
        let c6 = Constellation::derive(true, true, true, true, true, true);
        assert_eq!(c6.level, 5);
        assert!((c6.score() - 6.0 / 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_branch_evaluate_and_snapshot() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        branch.module_count = 25;
        branch.self_test_count = 4;
        branch.maturity_c0 = true;
        branch.maturity_c1 = true;
        branch.maturity_c2 = true;
        branch.health = 0.7;
        branch.evaluate_node_tier(5);
        assert_eq!(branch.node_tier, NodeTier::Keystone);
        branch.evaluate_constellation();
        assert_eq!(branch.constellation.level, 2);
        // Rune 效果: 空槽不改变 health
        assert!((branch.health_with_runes() - 0.7).abs() < 1e-9);
        branch.runes.set(RuneColor::Crimson, Rune::new("c", "C", RuneColor::Crimson, "e", 1.0));
        assert!(branch.health_with_runes() > 0.7);
        let snap = branch.snapshot();
        assert_eq!(snap.tier, NodeTier::Keystone);
        assert_eq!(snap.constellation_level, 2);
        assert_eq!(snap.rune_filled_slots, 1);
    }

    #[test]
    fn test_fog_level_derive_matrix() {
        // (wired, consumers, has_tests) → (level, label)
        let cases: &[((bool, usize, bool), (f64, &str))] = &[
            ((false, 0, false), (1.0, "DenseFog")),      // 孤儿: 未接线+无消费者+无测试
            ((false, 1, true), (0.85, "DenseFog")),      // 未接线但有消费者+测试
            ((true, 0, true), (0.10, "Clear")),          // 接线有测试但无消费者
            ((true, 1, false), (0.15, "LightFog")),      // 接线有消费者但无测试
            ((true, 1, true), (0.05, "Clear")),          // 全清晰
        ];
        for ((wired, consumers, has_tests), (level, label)) in cases {
            let fog = FogLevel::derive(*wired, *consumers, *has_tests);
            assert!((fog.level - level).abs() < 1e-9, "fog.level mismatch: got {}, want {}", fog.level, level);
            assert_eq!(fog.label(), *label, "label mismatch for level {}", fog.level);
        }
    }

    #[test]
    fn test_fog_evaluate_and_snapshot_carries_fog() {
        let mut branch = CapabilityBranch::new(BranchKind::Core);
        branch.self_test_count = 3;
        branch.evaluate_fog(true, 2, true);
        assert_eq!(branch.fog.label(), "Clear");
        let snap = branch.snapshot();
        assert!((snap.fog_level - 0.05).abs() < 1e-9);
        assert_eq!(snap.fog_label, "Clear");

        // 未接线 → 浓雾
        let mut orphan = CapabilityBranch::new(BranchKind::World);
        orphan.evaluate_fog(false, 0, false);
        assert_eq!(orphan.fog.label(), "DenseFog");
        assert_eq!(orphan.snapshot().fog_label, "DenseFog");
    }

    #[test]
    fn test_weighted_fog_sum_tier_weighting() {
        let mut tree = ConsciousnessTree::new();
        // 初始: 全 branch 默认 fog=0.85 (DenseFog)
        for branch in tree.branches.values_mut() {
            branch.evaluate_fog(false, 0, false);
            branch.node_tier = NodeTier::NotablePassive;
        }
        let baseline = tree.weighted_fog_sum();
        assert!(baseline > 0.0);
        // 把一个 Keystone 推到浓雾 → sum 显著上升 (证明 tier 加权)
        let keystone_branch = tree.branches.get_mut(&BranchKind::Core).unwrap();
        keystone_branch.node_tier = NodeTier::Keystone;
        keystone_branch.evaluate_fog(false, 0, false); // fog.level = 1.0
        let after = tree.weighted_fog_sum();
        // Keystone weight=3.0, fog=1.0 vs 原 Notable weight=2.0, fog=1.0 → +1.0
        assert!((after - baseline - 1.0).abs() < 1e-9, "got baseline={} after={}", baseline, after);
    }

    #[test]
    fn test_growth_cycle_reports_fog_summary() {
        let mut tree = ConsciousnessTree::new();
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 12;
            branch.fruit_count = 1;
        }
        let report = tree.run_growth_cycle();
        assert!(report.weighted_fog_sum > 0.0);
        assert_eq!(report.fog_by_branch.len(), BranchKind::all().len());
        // Phase 3 evaluate_fog(wired=true, consumers>0, has_tests=true) → Clear
        assert!(report.fog_by_branch.values().all(|v| *v <= 0.15));
    }

    #[test]
    fn test_node_snapshot_json_roundtrip_with_fog() {
        let mut branch = CapabilityBranch::new(BranchKind::Shield);
        branch.evaluate_fog(false, 0, false);
        let snap = branch.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        let back: NodeSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fog_level, snap.fog_level);
        assert_eq!(back.fog_label, snap.fog_label);
    }

    #[test]
    fn test_growth_cycle_evaluates_nodes() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 100;
        for branch in tree.branches.values_mut() {
            branch.health = 0.8;
            branch.self_test_count = 5;
            branch.module_count = 12;
            branch.fruit_count = 1;
        }
        tree.run_growth_cycle();
        for branch in tree.branches.values() {
            assert!(!branch.constellation.c0_compiles || branch.node_tier == NodeTier::SmallPassive
                || branch.node_tier == NodeTier::NotablePassive
                || branch.node_tier == NodeTier::Keystone);
        }
    }

    #[test]
    fn test_snapshots_enumerate_all_branches() {
        let tree = ConsciousnessTree::new();
        let snaps = tree.snapshots();
        assert_eq!(snaps.len(), BranchKind::all().len());
        assert!(snaps.iter().all(|s| !s.branch.label().is_empty()));
    }

    #[test]
    fn test_contract_negotiation_and_fulfillment() {
        let mut tree = ConsciousnessTree::new();
        tree.core.vuln_scan.push(VulnerabilityFinding {
            severity: VulnerabilitySeverity::High,
            category: "architecture".into(),
            module: "test_module".into(),
            description: "test".into(),
            fix_suggestion: "fix it".into(),
        });
        tree.cycle = 1;
        let contract = tree.negotiate_contract();
        assert!(contract.cycle == 2);
        assert!(contract.claim.contains("fix it"));
        assert_eq!(contract.evidence_plan.len(), 4);

        // Verify fulfillment
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            branch.self_test_count = 8;
            branch.module_count = 8;
        }
        tree.fruits.push(EvolutionFruit { quality: 0.9, ..Default::default() });
        tree.vuln_baseline = Some(2); // baseline 2, current 1 → 50% reduction
        let fulfillment = tree.verify_contract_fulfillment(&contract);
        assert!(fulfillment.evidence_total == 4);
        assert!(fulfillment.fulfilled);
    }

    #[test]
    fn test_drift_audit_detects_violation() {
        let mut tree = ConsciousnessTree::new();
        tree.cycle = 3;
        let contract = EvolutionContract {
            cycle: 4,
            claim: "improve".into(),
            evidence_plan: vec!["plan".into()],
            stop_rule: StopRule { min_quality_threshold: 0.9, ..Default::default() },
            exploration_budget: 0.2,
            timestamp: 0,
        };
        let fulfillment = ContractFulfillment {
            cycle: 3,
            claim: "improve".into(),
            evidence_met: 0,
            evidence_total: 1,
            fulfilled: false,
            quality_achieved: 0.3,
            timestamp: 0,
        };
        let drift = tree.audit_drift(&contract, &fulfillment);
        assert!(drift.drift_detected);
        assert!(drift.stop_rule_triggered);
        assert!(drift.drift_magnitude > 0.0);
        assert!(drift.corrective_actions.len() >= 2);
    }

    #[test]
    fn test_growth_cycle_evolution_fruits() {
        let mut tree = ConsciousnessTree::new();
        tree.soil.crawl_queue_depth = 50;
        for branch in tree.branches.values_mut() {
            branch.health = 0.9;
            branch.self_test_count = 8;
            branch.module_count = 8;
        }
        let report = tree.run_growth_cycle();
        assert!(report.phase0_contract.is_some());
        assert!(report.phase3_fruits > 0);
        assert!(report.phase6_fulfillment.is_some());
        assert!(report.phase7_drift.is_some());
        // Fruits carry evidence chains + generations
        assert!(tree.fruits.iter().all(|f| f.generation == 0));
        assert!(tree.fruits.iter().all(|f| !f.evidence.sha256.is_none()));
    }
}
