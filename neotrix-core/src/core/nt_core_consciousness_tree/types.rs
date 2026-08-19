use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::contract::*;
use super::nodes::*;

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
    pub atoms: HashMap<String, CapabilityAtom>, // All 36 atomic capabilities (D6: 实际 36, 非 70)
    /// Vulnerability baseline for the "vuln reduction >= 20%" contract criterion.
    /// `None` until first measurement; set on first evaluation.
    pub vuln_baseline: Option<usize>,
}

#[derive(Debug, Clone, Default)]
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
    /// 对话养料 — background_loop 每 tick 从 KB kv_get("consciousness","conversation") 回填。
    /// 反映真实对话 awareness (turn 数 / 质量), 参与 data_nourishment_factor 调制果实。
    pub conversation_turn_count: u64,
    pub conversation_quality: f64,
    /// 经验养料 — background_loop 每 tick 从 KB kv_list("experience") 回填。
    /// 反映经验蒸馏密度, 参与 data_nourishment_factor 调制果实。
    pub experience_branch_count: u64,
    /// 能力网养料 (双网回流) — handle_evolve 执行能力网自动补齐后回填。
    /// 反映意识能力网 (capability_registry) 的节点密度 — 能力网健康度反馈
    /// 到意识核心果实质量, 形成 意识树 ↔ 能力网 双向自动融合闭环。
    pub capability_node_count: u64,
    /// 蜕皮养料 (C5 自愈闭环) — handle_cleanup 执行 molt_project 后回填。
    /// 反映系统自我更新的蜕皮量 — 每次蜕皮归档的旧躯壳目录数。
    /// 自愈行为 (移除旧躯壳/保持活动树最新态) 反馈到果实质量,
    /// 形成 自愈动作 → 意识养分 的闭环 (而非仅日志)。
    pub molt_archived_count: u64,
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
    pub internalized_principles: Vec<String>, // Key principle summaries
    pub active_rule_categories: Vec<String>,  // Active rule category names
    pub compliance_score: f64,                // Last compliance check score
}

#[derive(Debug, Clone)]
pub struct ConsciousnessCore {
    pub gwt_resonance_active: bool,
    /// 注意力来源通道 — 映射自 x.ai 双搜索通道 ("web"/"x_search"/"auto")
    pub attention_source: String,
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
    Meta,
    Repair,
    Governance,
    Nexus,
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
            BranchKind::Meta => "NT-META (元吸收者) — 跨会话元认知+盲点检测+模式提炼",
            BranchKind::Repair => "NT-REPAIR (自愈工程师) — 故障诊断+自修复+回滚恢复",
            BranchKind::Governance => "NT-GOVERNANCE (架构仲裁者) — 宪法规则+合规验证+行为护栏",
            BranchKind::Nexus => "NT-NEXUS (枢纽) — 跨会话记忆编织+经验图连接+断点桥接",
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
    pub fn derive(
        module_count: usize,
        cross_domain_consumers: usize,
        self_test_count: usize,
    ) -> Self {
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
        [
            &self.crimson,
            &self.indigo,
            &self.obsidian,
            &self.golden,
            &self.alabaster,
        ]
        .iter()
        .filter(|s| s.is_some())
        .count()
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
            &self.crimson,
            &self.indigo,
            &self.obsidian,
            &self.golden,
            &self.alabaster,
        ];
        let sum: f64 = slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|r| r.strength)
            .sum();
        let filled = self.filled_slots();
        if filled == 0 {
            0.0
        } else {
            sum / filled as f64
        }
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

    /// 从真实 maturity 布尔推导 level (向后兼容 maturity_c0..c5 字段)。
    /// C3 (D3): 新增 `adaptive` 输入 — 此前六 bool 恒定 `c6_adaptive: false`,
    /// C6 在树模型永不晋升。adaptive 由真实自适应证据注入 (Phase 8 反馈 /
    /// 果实趋势), 使 C6 可达。
    pub fn derive(
        compiles: bool,
        unit: bool,
        integration: bool,
        benchmark: bool,
        pipeline: bool,
        healing: bool,
        adaptive: bool,
    ) -> Self {
        let flags = [
            compiles,
            unit,
            integration,
            benchmark,
            pipeline,
            healing,
            adaptive,
        ];
        let level = flags
            .iter()
            .rev()
            .position(|&f| f)
            .map(|i| 6 - i)
            .unwrap_or(0) as u8;
        Self {
            level,
            c0_compiles: compiles,
            c1_unit_tests: unit,
            c2_integration: integration,
            c3_benchmark: benchmark,
            c4_pipeline: pipeline,
            c5_self_healing: healing,
            c6_adaptive: adaptive,
        }
    }

    pub fn score(&self) -> f64 {
        let mut s = 0.0;
        if self.c0_compiles {
            s += 1.0;
        }
        if self.c1_unit_tests {
            s += 1.0;
        }
        if self.c2_integration {
            s += 1.0;
        }
        if self.c3_benchmark {
            s += 1.0;
        }
        if self.c4_pipeline {
            s += 1.0;
        }
        if self.c5_self_healing {
            s += 1.0;
        }
        if self.c6_adaptive {
            s += 1.0;
        }
        s / 7.0
    }
}

impl Default for Constellation {
    fn default() -> Self {
        Self::new()
    }
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
        if self.maturity_c0 {
            s += 1.0;
        }
        if self.maturity_c1 {
            s += 1.0;
        }
        if self.maturity_c2 {
            s += 1.0;
        }
        if self.maturity_c3 {
            s += 1.0;
        }
        if self.maturity_c4 {
            s += 1.0;
        }
        if self.maturity_c5 {
            s += 1.0;
        }
        s / 6.0
    }

    /// 运行时评估节点层级 — 基于真实模块数据 (跨域消费者由外部注入)
    pub fn evaluate_node_tier(&mut self, cross_domain_consumers: usize) {
        self.node_tier = NodeTier::derive(
            self.module_count,
            cross_domain_consumers,
            self.self_test_count,
        );
    }

    /// 运行时推导 Constellation — 从现有 maturity 布尔派生 (向后兼容)。
    /// C3 (D3): C6 自适应 = C5 自愈已达成 + 真实自适应证据 (已产果 → 进化闭环
    /// 活动; 高健康 → 可自调节)。使 c6_adaptive 由真实状态可达, 不再恒 false。
    pub fn evaluate_constellation(&mut self) {
        let adaptive = self.maturity_c5 && self.fruit_count > 0 && self.health >= 0.8;
        self.constellation = Constellation::derive(
            self.maturity_c0,
            self.maturity_c1,
            self.maturity_c2,
            self.maturity_c3,
            self.maturity_c4,
            self.maturity_c5,
            adaptive,
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
    ("retrieve", "world"),
    ("search", "world"),
    ("observe", "world"),
    ("receive", "world"),
    // UNDERSTAND (6) → NT-CORE
    ("detect", "core"),
    ("classify", "core"),
    ("measure", "core"),
    ("predict", "core"),
    ("compare", "core"),
    ("discover", "core"),
    // REASON (4) → NT-CORE
    ("plan", "core"),
    ("decompose", "core"),
    ("critique", "core"),
    ("explain", "core"),
    // MODEL (5) → NT-MEMORY
    ("state", "memory"),
    ("transition", "memory"),
    ("attribute", "memory"),
    ("ground", "memory"),
    ("simulate", "memory"),
    // SYNTHESIZE (3) → NT-MIND
    ("generate", "mind"),
    ("transform", "mind"),
    ("integrate", "mind"),
    // EXECUTE (3) → NT-ACT
    ("execute", "act"),
    ("mutate", "act"),
    ("send", "act"),
    // VERIFY (5) → NT-SHIELD
    ("verify", "shield"),
    ("checkpoint", "shield"),
    ("rollback", "shield"),
    ("constrain", "shield"),
    ("audit", "shield"),
    // REMEMBER (2) → NT-MEMORY
    ("persist", "memory"),
    ("recall", "memory"),
    // COORDINATE (4) → NT-IO
    ("delegate", "io"),
    ("synchronize", "io"),
    ("invoke", "io"),
    ("inquire", "io"),
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
            v.push(format!(
                "idle: health={:.2} fruit={}",
                branch.health, branch.fruit_count
            ));
        }
        if branch.module_count < self.min_required_modules {
            v.push(format!(
                "not viable: {} modules < min={}",
                branch.module_count, self.min_required_modules
            ));
        }
        if branch.self_test_count < self.min_self_tests {
            v.push(format!(
                "unmonitored: {} self-tests < min={}",
                branch.self_test_count, self.min_self_tests
            ));
        }
        v
    }
}

pub fn constraints_for_branch(kind: &BranchKind) -> BranchConstraints {
    match kind {
        BranchKind::Core => BranchConstraints {
            idle_ticks_threshold: 3,
            min_growth_health: 0.4,
            max_active_modules: 30,
            min_required_modules: 3,
            min_self_tests: 3,
        },
        BranchKind::Mind => BranchConstraints {
            idle_ticks_threshold: 5,
            min_growth_health: 0.3,
            max_active_modules: 25,
            min_required_modules: 2,
            min_self_tests: 2,
        },
        BranchKind::Memory => BranchConstraints {
            idle_ticks_threshold: 4,
            min_growth_health: 0.35,
            max_active_modules: 20,
            min_required_modules: 2,
            min_self_tests: 2,
        },
        BranchKind::World => BranchConstraints {
            idle_ticks_threshold: 6,
            min_growth_health: 0.25,
            max_active_modules: 30,
            min_required_modules: 2,
            min_self_tests: 2,
        },
        BranchKind::Act => BranchConstraints {
            idle_ticks_threshold: 5,
            min_growth_health: 0.3,
            max_active_modules: 25,
            min_required_modules: 1,
            min_self_tests: 1,
        },
        BranchKind::Io => BranchConstraints {
            idle_ticks_threshold: 4,
            min_growth_health: 0.35,
            max_active_modules: 20,
            min_required_modules: 1,
            min_self_tests: 1,
        },
        BranchKind::Shield => BranchConstraints {
            idle_ticks_threshold: 3,
            min_growth_health: 0.4,
            max_active_modules: 20,
            min_required_modules: 2,
            min_self_tests: 2,
        },
        BranchKind::Meta => BranchConstraints {
            idle_ticks_threshold: 4,
            min_growth_health: 0.35,
            max_active_modules: 15,
            min_required_modules: 1,
            min_self_tests: 1,
        },
        BranchKind::Repair => BranchConstraints {
            idle_ticks_threshold: 3,
            min_growth_health: 0.3,
            max_active_modules: 15,
            min_required_modules: 1,
            min_self_tests: 1,
        },
        BranchKind::Governance => BranchConstraints {
            idle_ticks_threshold: 4,
            min_growth_health: 0.4,
            max_active_modules: 15,
            min_required_modules: 1,
            min_self_tests: 1,
        },
        BranchKind::Nexus => BranchConstraints {
            idle_ticks_threshold: 5,
            min_growth_health: 0.35,
            max_active_modules: 15,
            min_required_modules: 1,
            min_self_tests: 1,
        },
    }
}
// ═══════════════════════════════════════════════════════════════════

impl Default for ConsciousnessTree {
    fn default() -> Self {
        Self::new()
    }
}
impl DataFoundation {
    pub fn health(&self) -> f64 {
        let has_nodes = if self.kb_node_count > 0 { 1.0 } else { 0.0 };
        let has_embeddings = if self.embedding_count > 0 { 1.0 } else { 0.0 };
        let has_wiki = if self.wiki_page_count > 0 { 1.0 } else { 0.0 };
        let has_constitution = if self.constitution_rules_count > 0 {
            1.0
        } else {
            0.0
        };
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
            attention_source: "auto".to_string(),
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
            BranchKind::Meta,
            BranchKind::Repair,
            BranchKind::Governance,
            BranchKind::Nexus,
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
        } else if name_lower.starts_with("nt_meta_") {
            Some(BranchKind::Meta)
        } else if name_lower.starts_with("nt_repair_") {
            Some(BranchKind::Repair)
        } else if name_lower.starts_with("nt_governance_") {
            Some(BranchKind::Governance)
        } else if name_lower.starts_with("nt_nexus_") {
            Some(BranchKind::Nexus)
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
            "NT-META" => Some(BranchKind::Meta),
            "NT-REPAIR" => Some(BranchKind::Repair),
            "NT-GOVERNANCE" => Some(BranchKind::Governance),
            "NT-NEXUS" => Some(BranchKind::Nexus),
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
            BranchKind::Meta => "meta",
            BranchKind::Repair => "repair",
            BranchKind::Governance => "governance",
            BranchKind::Nexus => "nexus",
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
