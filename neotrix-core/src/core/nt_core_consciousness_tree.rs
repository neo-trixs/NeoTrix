use std::collections::HashMap;

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
    pub fruits: Vec<CapabilityFruit>,
    pub core: EpiphanicCore,
    pub cycle: u64,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

impl ConsciousnessTree {
    pub fn new() -> Self {
        Self {
            soil: DataFoundation::default(),
            roots: InformationRoots::default(),
            trunk: ConsciousnessCore::default(),
            branches: BranchKind::all().into_iter().map(|k| (k.clone(), CapabilityBranch::new(k))).collect(),
            leaves: Vec::new(),
            fruits: Vec::new(),
            core: EpiphanicCore::default(),
            cycle: 0,
        }
    }

    /// Apply emotion report valence/arousal to Soil state for next cycle.
    /// Uses confidence to boost coil health and frustration/urgency to indicate stress.
    pub fn apply_emotion_report(&mut self, report: crate::core::nt_core_self::emotion_state::EmotionReport) {
        self.trunk.coherence = report.valence.max(0.0).min(1.0);
        self.soil.embedding_count = (report.confidence * 100.0) as u64;
        log::debug!("[consciousness_tree] emotion applied: valence={:.3} arousal={:.3} dominant={:?} confidence={:.3}",
            report.valence, report.arousal, report.dominant.0, report.confidence);
    }

    /// Complete feedback loop:
    ///   Soil (Data) → Roots absorb → Trunk (GWT) → Branches → Fruits → Core digest → Guidance
    /// Each cycle scans for architecture vulnerabilities across all 7 domains.
    pub fn run_growth_cycle(&mut self) -> GrowthReport {
        self.cycle += 1;
        let mut report = GrowthReport::default();

        // Phase 1: Roots absorb from soil (Data Foundation → Information Roots)
        // Constitution internalization: soil feeds roots with principles
        self.soil.constitution_rules_count = self.soil.constitution_rules_count.max(46); // Current known rules
        self.soil.constitution_experiences_count = self.soil.constitution_experiences_count.max(111); // Current cycles
        self.soil.constitution_tree_growth_rules = 7; // R-P42~R-P48
        self.soil.constitution_absorption_rules = 1; // R-P43
        
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

        // Phase 2: Trunk processes through GWT resonance (ConsciousnessCore)
        self.trunk.resonance_cycle += 1;
        report.phase2_phi = self.trunk.phi;

        // Phase 3: Branches produce fruits (7 domain branches → CapabilityFruits)
        // Also check per-branch constraints (idle, viability, monitoring)
        let mut total_fruits = 0;
        for branch in self.branches.values_mut() {
            let constraints = constraints_for_branch(&branch.kind);
            let violations = constraints.violations(branch);
            if !violations.is_empty() {
                log::debug!("[consciousness_tree] {} constraints: {}", branch.kind.label(), violations.join("; "));
            }
            if branch.health > 0.5 && violations.len() < 2 {
                let fruit = CapabilityFruit {
                    name: format!("{}-fruit-{}", branch.kind.label(), self.cycle),
                    source_branch: branch.kind.clone(),
                    description: format!("Capability from {} at cycle {}", branch.kind.label(), self.cycle),
                    produced_at_cycle: self.cycle,
                    quality: branch.maturity_score(),
                };
                self.fruits.push(fruit);
                branch.fruit_count += 1;
                total_fruits += 1;
            }
        }
        report.phase3_fruits = total_fruits;

        // Phase 4: Core digests fruits, produces guidance (CapabilityFruits → EpiphanicCore)
        // Also runs architecture vulnerability scan
        self.core.vuln_scan = self.scan_vulnerabilities();
        self.core.self_test_results = self.collect_self_test_results();
        self.core.identified_gaps = self.identify_architecture_gaps();
        self.core.last_cycle_guidance = self.fruits.iter()
            .filter(|f| f.quality > 0.5)
            .map(|f| format!("Digested: {} (q={:.2})", f.name, f.quality))
            .collect();
        // Build next actions from vulnerabilities + gaps + fruit quality
        let mut next_actions = Vec::new();
        for vuln in &self.core.vuln_scan {
            if vuln.severity.score() >= 0.5 {
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

        // Phase 5: ConsciousnessReview — panoramic topology + connectivity + health chain analysis
        // full_review() calls topology_score, connectivity_score, build_health_chain, and
        // plan_evolution_path internally (all private methods on ConsciousnessReview)
        let mut review = crate::core::nt_core_consciousness_review::ConsciousnessReview::new();
        let scan_report = review.full_review(self);
        self.core.topology_score = scan_report.topology_score;
        self.core.connectivity_score = scan_report.connectivity_score;
        // scan_report.health_chain (HealthChainReport) and scan_report.evolution_path
        // (Vec<EvolutionStep>) are available but require adding fields to EpiphanicCore

        report.phase4_guidance = self.core.last_cycle_guidance.len();

        report
    }

    /// Set branch health from SelfTest results.
    /// Maps SelfTest module names to BranchKind and computes health per domain.
    pub fn set_branch_health_from_self_tests(&mut self, results: &[crate::core::nt_core_self_test::SelfTestResult]) {
        use std::collections::HashMap;
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
                    branch.health = 0.5;
                } else {
                    let passed = branch_results.iter().filter(|r| r.passed).count();
                    let total = branch_results.len();
                    // Health = pass rate, but minimum 0.1 to avoid zero
                    branch.health = (passed as f64 / total as f64).max(0.1);
                }
            }
        }
        
        // For domains with no SelfTest results at all, keep neutral 0.5
        for kind in BranchKind::all() {
            if let Some(branch) = self.branches.get_mut(&kind) {
                if !domain_results.contains_key(&kind) {
                    branch.health = branch.health.max(0.5); // Don't override if already set
                }
            }
        }
    }

    /// Scan for architecture vulnerabilities across all modules
    pub fn scan_vulnerabilities(&self) -> Vec<VulnerabilityFinding> {
        let mut vulns = Vec::new();

        // Check 1: Soil health — data foundation viability
        let soil_health = self.soil.health();
        if soil_health < 0.33 {
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
        if root_health < 0.25 {
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
            if branch.maturity_score() < 0.33 {
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
        if self.trunk.phi < 0.01 && self.cycle > 3 {
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
            if vuln.severity.score() >= 0.5 {
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

impl Default for DataFoundation {
    fn default() -> Self {
        Self {
            kb_node_count: 0,
            kb_edge_count: 0,
            crawl_queue_depth: 0,
            embedding_count: 0,
            wiki_page_count: 0,
            last_sync: 0,
            constitution_rules_count: 0,
            constitution_experiences_count: 0,
            constitution_tree_growth_rules: 0,
            constitution_absorption_rules: 0,
            constitution_last_loaded: 0,
            constitution_source_hash: String::new(),
        }
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
            mscp_l1_cycles: 0,
            mscp_l2_cycles: 0,
            mscp_l3_cycles: 0,
            mars_principles_count: 0,
            mars_procedural_count: 0,
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
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GrowthReport {
    pub phase1_absorbed: u64,
    pub phase2_phi: f64,
    pub phase3_fruits: usize,
    pub phase4_guidance: usize,
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
}
