use super::auto_crystallizer::AutoCrystallizer;
use super::exploration_pipeline::{ExplorationPipeline, ExploreDomain};
use super::memory::ReasoningBank;
use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use crate::core::nt_core_meta::planner::{
    EvolutionPlanner, ImpactEstimate, PlannedEvolution, RiskLevel,
};
use crate::core::nt_core_meta::self_model::DebtSeverity;
use crate::core::nt_core_meta::weakness::Weakness;
use crate::core::nt_core_self::CrystalRegistry;
#[cfg(test)]
use crate::core::nt_core_self::SkillCrystal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EvolutionTarget {
    pub url: String,
    pub name: String,
    pub domain: ExploreDomain,
    pub priority: u8,
    pub capability_dims: Vec<(String, f64)>,
    pub target_module: Option<String>,
    pub absorbed: bool,
    pub last_attempt: Option<i64>,
    pub notes: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SeedRegistry {
    pub targets: Vec<EvolutionTarget>,
    pub absorbed: HashMap<String, bool>,
}

impl SeedRegistry {
    pub fn new() -> Self {
        Self {
            targets: Self::default_targets(),
            absorbed: HashMap::new(),
        }
    }

    pub fn priority_queue(&self) -> Vec<&EvolutionTarget> {
        let mut sorted: Vec<&EvolutionTarget> =
            self.targets.iter().filter(|t| !t.absorbed).collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }

    pub fn enqueue_all(&self, pipeline: &mut ExplorationPipeline) -> usize {
        let mut count = 0;
        for target in &self.targets {
            if !self.absorbed.contains_key(&target.url) {
                pipeline.ingest(&target.url, Some(target.domain.clone()));
                count += 1;
            }
        }
        count
    }

    pub fn enqueue_priority(&self, pipeline: &mut ExplorationPipeline, min_priority: u8) -> usize {
        let mut count = 0;
        for target in &self.targets {
            if target.priority >= min_priority && !self.absorbed.contains_key(&target.url) {
                pipeline.ingest(&target.url, Some(target.domain.clone()));
                count += 1;
            }
        }
        count
    }

    pub fn generate_plans(&self, planner: &mut EvolutionPlanner) -> Vec<PlannedEvolution> {
        let mut plans = Vec::new();
        for target in &self.targets {
            if self.absorbed.contains_key(&target.url) {
                continue;
            }
            let severity = match target.priority {
                5 => DebtSeverity::Critical,
                4 => DebtSeverity::Major,
                3 | 2 => DebtSeverity::Minor,
                _ => DebtSeverity::Cosmetic,
            };
            let weakness = Weakness {
                pattern_id: format!("SEED:{}", target.name.to_uppercase()),
                target_module: target.target_module.clone(),
                file: None,
                line: None,
                severity,
                description: format!(
                    "Capability seed {} not yet absorbed: {}",
                    target.name, target.notes
                ),
                impact: format!(
                    "Missing {} capability (priority {})",
                    target.name, target.priority
                ),
                suggestion: target.url.clone(),
            };
            let plan = PlannedEvolution {
                id: format!("seed-{}", target.name),
                priority: target.priority,
                weakness,
                target_module: target.target_module.clone(),
                action: format!("evolve_from_seed:{}", target.url),
                estimated_impact: ImpactEstimate {
                    files_affected: 3,
                    risk: if target.priority >= 4 {
                        RiskLevel::Low
                    } else {
                        RiskLevel::Medium
                    },
                },
                dependencies: vec![],
            };
            plans.push(plan);
        }
        planner.plan_from_weaknesses(plans.iter().map(|p| p.weakness.clone()).collect())
    }

    pub fn mark_absorbed(&mut self, url: &str) {
        self.absorbed.insert(url.to_string(), true);
        if let Some(target) = self.targets.iter_mut().find(|t| t.url == url) {
            target.absorbed = true;
            target.last_attempt = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            );
        }
    }

    pub fn pending(&self) -> Vec<&EvolutionTarget> {
        self.targets.iter().filter(|t| !t.absorbed).collect()
    }

    pub fn progress(&self) -> (usize, usize) {
        let total = self.targets.len();
        let done = self.absorbed.len();
        (done, total)
    }

    pub fn targets_by_domain(&self, domain: &ExploreDomain) -> Vec<&EvolutionTarget> {
        self.targets
            .iter()
            .filter(|t| t.domain == *domain)
            .collect()
    }

    fn default_targets() -> Vec<EvolutionTarget> {
        vec![
            // ═══════════════════════════════════════════════════════════════
            // Priority 5 — Critical: immediate capability gaps
            // ═══════════════════════════════════════════════════════════════
            EvolutionTarget {
                url: "https://github.com/mksglu/context-mode".into(),
                name: "context-mode".into(),
                domain: ExploreDomain::General,
                priority: 5,
                capability_dims: vec![("context_optimization".into(), 0.9), ("token_efficiency".into(), 0.85)],
                target_module: Some("nt_mind".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Context window optimization: PreToolUse hooks, SQLite FTS5 session continuity, 98% compression ratio. Critical for all pipeline stages.".into(),
                tags: vec!["context".into(), "optimization".into(), "token".into()],
            },
            EvolutionTarget {
                url: "https://github.com/mukul975/Anthropic-Cybernt_shield-Skills".into(),
                name: "cybernt_shield-skills".into(),
                domain: ExploreDomain::Security,
                priority: 5,
                capability_dims: vec![("nt_shield_audit".into(), 0.95), ("vulnerability_detection".into(), 0.9)],
                target_module: Some("nt_shield".into()),
                absorbed: false,
                last_attempt: None,
                notes: "754 structured cybernt_shield skills, MITRE ATT&CK mapping, NIST CSF 2.0. 26 nt_shield domains.".into(),
                tags: vec!["nt_shield".into(), "mitre".into(), "skills".into()],
            },
            // ═══════════════════════════════════════════════════════════════
            // Priority 4 — High: significant capability expansion
            // ═══════════════════════════════════════════════════════════════
            EvolutionTarget {
                url: "https://github.com/hypnguyen1209/offensive-claude".into(),
                name: "offensive-claude".into(),
                domain: ExploreDomain::Security,
                priority: 4,
                capability_dims: vec![("penetration_testing".into(), 0.85), ("exploit_development".into(), 0.8)],
                target_module: Some("nt_shield".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Red-team toolkit: exploit dev, AD attacks, EDR bypass, mobile pentest.".into(),
                tags: vec!["nt_shield".into(), "pentest".into(), "redteam".into()],
            },
            EvolutionTarget {
                url: "https://github.com/0xSteph/pentest-ai-agents".into(),
                name: "pentest-ai-agents".into(),
                domain: ExploreDomain::Security,
                priority: 4,
                capability_dims: vec![("pentest_automation".into(), 0.85), ("recon_analysis".into(), 0.8)],
                target_module: Some("nt_shield".into()),
                absorbed: false,
                last_attempt: None,
                notes: "35 specialized AI sub-agents for pentest planning, recon, exploit research, detection, STIG auditing.".into(),
                tags: vec!["nt_shield".into(), "pentest".into(), "agents".into()],
            },
            EvolutionTarget {
                url: "https://github.com/elementalsouls/Claude-BugHunter".into(),
                name: "claude-bughunter".into(),
                domain: ExploreDomain::Security,
                priority: 4,
                capability_dims: vec![("bug_hunting".into(), 0.85), ("vulnerability_classification".into(), 0.8)],
                target_module: Some("nt_shield".into()),
                absorbed: false,
                last_attempt: None,
                notes: "51 skills, 15 slash commands, 681 disclosed-report patterns across 24 vulnerability classes.".into(),
                tags: vec!["nt_shield".into(), "bughunting".into(), "vulnerability".into()],
            },
            EvolutionTarget {
                url: "https://github.com/OpenOSINT/OpenOSINT".into(),
                name: "openosint".into(),
                domain: ExploreDomain::Security,
                priority: 4,
                capability_dims: vec![("osint".into(), 0.9), ("intelligence_gathering".into(), 0.85)],
                target_module: Some("nt_world_crawl".into()),
                absorbed: false,
                last_attempt: None,
                notes: "AI-powered OSINT agent: 16 tools, REPL, MCP server, entity correlation.".into(),
                tags: vec!["osint".into(), "intelligence".into(), "recon".into()],
            },
            EvolutionTarget {
                url: "https://github.com/nsasoft/nsauditor-ai".into(),
                name: "nsauditor-ai".into(),
                domain: ExploreDomain::Security,
                priority: 4,
                capability_dims: vec![("network_scanning".into(), 0.85), ("cve_matching".into(), 0.8)],
                target_module: Some("nt_shield".into()),
                absorbed: false,
                last_attempt: None,
                notes: "27 scanning plugins, CVE matching, MITRE ATT&CK mapping, continuous monitoring.".into(),
                tags: vec!["nt_shield".into(), "scanning".into(), "network".into()],
            },
            EvolutionTarget {
                url: "https://github.com/dreammis/social-auto-upload".into(),
                name: "social-auto-upload".into(),
                domain: ExploreDomain::General,
                priority: 4,
                capability_dims: vec![("content_publishing".into(), 0.85), ("multi_platform".into(), 0.8)],
                target_module: Some("nt_act_social".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Multi-platform video publishing: Douyin, Bilibili, Kuaishou, Xiaohongshu, YouTube, TikTok.".into(),
                tags: vec!["social".into(), "content".into(), "automation".into()],
            },
            // ═══════════════════════════════════════════════════════════════
            // Priority 3 — Medium: valuable integrations
            // ═══════════════════════════════════════════════════════════════
            EvolutionTarget {
                url: "https://github.com/aristoapp/awesome-second-brain".into(),
                name: "awesome-second-brain".into(),
                domain: ExploreDomain::Consciousness,
                priority: 3,
                capability_dims: vec![("knowledge_retention".into(), 0.8), ("cross_session_memory".into(), 0.75)],
                target_module: Some("knowledge_engine".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Context Engineering resource library. Collect→Organize→Evolve→Use memory pipeline.".into(),
                tags: vec!["knowledge".into(), "memory".into(), "context".into()],
            },
            EvolutionTarget {
                url: "https://github.com/modaic-ai/gepa-viz".into(),
                name: "gepa-viz".into(),
                domain: ExploreDomain::General,
                priority: 3,
                capability_dims: vec![("visualization".into(), 0.8), ("prompt_optimization".into(), 0.75)],
                target_module: Some("orchestrator".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Interactive live visualizer with prompt optimization trees and Pareto analysis.".into(),
                tags: vec!["viz".into(), "optimization".into(), "monitoring".into()],
            },
            EvolutionTarget {
                url: "https://github.com/cursor/plugins".into(),
                name: "cursor-plugins".into(),
                domain: ExploreDomain::GitHub,
                priority: 3,
                capability_dims: vec![("plugin_system".into(), 0.8), ("extensibility".into(), 0.75)],
                target_module: Some("plugin".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Official Cursor plugin spec and plugin registry. Plugin SDK design patterns.".into(),
                tags: vec!["plugin".into(), "sdk".into(), "ecosystem".into()],
            },
            EvolutionTarget {
                url: "https://github.com/NikhilLamba24/auto_ui".into(),
                name: "auto-ui".into(),
                domain: ExploreDomain::General,
                priority: 3,
                capability_dims: vec![("gui_automation".into(), 0.8), ("screen_analysis".into(), 0.75)],
                target_module: Some("nt_world_browse_auto".into()),
                absorbed: false,
                last_attempt: None,
                notes: "GUI agent with OmniParser integration. Screenshot→analyze→act loop.".into(),
                tags: vec!["gui".into(), "automation".into(), "nt_world_browse".into()],
            },
            // ═══════════════════════════════════════════════════════════════
            // Priority 2 — Lower: exploratory / future
            // ═══════════════════════════════════════════════════════════════
            EvolutionTarget {
                url: "https://github.com/reconurge/flowsint".into(),
                name: "flowsint".into(),
                domain: ExploreDomain::Security,
                priority: 2,
                capability_dims: vec![("investigation_platform".into(), 0.7), ("graph_analysis".into(), 0.65)],
                target_module: None,
                absorbed: false,
                last_attempt: None,
                notes: "Visual graph-based investigation platform for cybernt_shield analysts.".into(),
                tags: vec!["nt_shield".into(), "investigation".into(), "graph".into()],
            },
            EvolutionTarget {
                url: "https://github.com/EpicStaff/EpicStaff".into(),
                name: "epicstaff".into(),
                domain: ExploreDomain::GitHub,
                priority: 2,
                capability_dims: vec![("agent_builder".into(), 0.7), ("visual_editor".into(), 0.65)],
                target_module: None,
                absorbed: false,
                last_attempt: None,
                notes: "Open-source visual UI + modular backend for building AI agents.".into(),
                tags: vec!["agents".into(), "visual".into(), "builder".into()],
            },
            EvolutionTarget {
                url: "https://github.com/vixhal-baraiya/dna-c".into(),
                name: "dna-c".into(),
                domain: ExploreDomain::General,
                priority: 1,
                capability_dims: vec![("scientific_computing".into(), 0.6)],
                target_module: None,
                absorbed: false,
                last_attempt: None,
                notes: "DNA construction from scratch in C. Domain-specific computation patterns.".into(),
                tags: vec!["science".into(), "dna".into(), "computation".into()],
            },
            // ═══════════════════════════════════════════════════════════════
            // Also re-enqueue the 50+ existing defaults as General seeds
            // ═══════════════════════════════════════════════════════════════
            EvolutionTarget {
                url: "https://github.com/peteromallet/arnold".into(),
                name: "arnold-framework".into(),
                domain: ExploreDomain::General,
                priority: 5,
                capability_dims: vec![("structured_harness".into(), 0.95), ("model_routing".into(), 0.9)],
                target_module: Some("orchestrator".into()),
                absorbed: false,
                last_attempt: None,
                notes: "Core reference: two-funnel model, structured pipeline, difficulty-aware routing, 30x cost reduction.".into(),
                tags: vec!["pipeline".into(), "routing".into(), "harness".into()],
            },
        ]
    }
}

impl Default for SeedRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Gap 1: Persistence — SeedRegistry save/load
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SeedState {
    absorbed_urls: Vec<String>,
    last_attempts: HashMap<String, i64>,
}

impl SeedRegistry {
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let state = SeedState {
            absorbed_urls: self.absorbed.keys().cloned().collect(),
            last_attempts: self
                .targets
                .iter()
                .filter_map(|t| t.last_attempt.map(|ts| (t.url.clone(), ts)))
                .collect(),
        };
        let json = serde_json::to_string_pretty(&state).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let state: SeedState = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        let mut reg = Self::new();
        for url in &state.absorbed_urls {
            reg.mark_absorbed(url);
        }
        for (url, ts) in &state.last_attempts {
            if let Some(target) = reg.targets.iter_mut().find(|t| t.url == *url) {
                target.last_attempt = Some(*ts);
            }
        }
        Ok(reg)
    }

    pub fn absorbed_list(&self) -> Vec<String> {
        self.absorbed.keys().cloned().collect()
    }

    pub fn is_complete(&self) -> bool {
        self.pending().is_empty()
    }
}

// ═══════════════════════════════════════════════════════════════════
// Gap 2: SeedDriver — 驱动循环 enqueue → run_round → mark_absorbed
// ═══════════════════════════════════════════════════════════════════

pub struct SeedTickResult {
    pub enqueued: usize,
    pub absorbed: usize,
    pub total_absorbed: usize,
    pub total_targets: usize,
    pub total_mined: usize,
    pub crystals_created: usize,
    pub details: Vec<String>,
}

pub struct SeedDriver<'a> {
    pub registry: SeedRegistry,
    pub pipeline: ExplorationPipeline,
    pub brain: &'a mut ReasoningBrain,
    pub bank: &'a mut ReasoningBank,
    pub auto_crystallizer: AutoCrystallizer,
    pub work_dir: PathBuf,
}

impl<'a> SeedDriver<'a> {
    pub fn new(
        registry: SeedRegistry,
        pipeline: ExplorationPipeline,
        brain: &'a mut ReasoningBrain,
        bank: &'a mut ReasoningBank,
        auto_crystallizer: AutoCrystallizer,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            registry,
            pipeline,
            brain,
            bank,
            auto_crystallizer,
            work_dir,
        }
    }

    pub fn tick(&mut self) -> SeedTickResult {
        let mut details = Vec::new();
        let state_path = self.work_dir.join("evolution_seeds.json");

        // 1. Load persisted state to recover crash recovery
        if let Ok(saved) = SeedRegistry::load(&state_path) {
            for url in saved.absorbed_list() {
                if !self.registry.absorbed.contains_key(&url) {
                    self.registry.mark_absorbed(&url);
                }
            }
        }
        let crystal_path = self.work_dir.join("crystal_registry.json");
        if let Ok(loaded) = CrystalRegistry::load(&crystal_path) {
            if !loaded.crystals.is_empty() {
                self.auto_crystallizer.registry = loaded;
                details.push("Loaded persisted crystal registry".into());
            }
        }

        // 2. Enqueue all non-absorbed seeds
        let enqueued = self.registry.enqueue_all(&mut self.pipeline);
        if enqueued == 0 {
            details.push("All seeds already absorbed. Nothing to enqueue.".into());
            let (done, total) = self.registry.progress();
            return SeedTickResult {
                enqueued: 0,
                absorbed: 0,
                total_absorbed: done,
                total_targets: total,
                total_mined: 0,
                crystals_created: 0,
                details,
            };
        }
        details.push(format!("Enqueued {} seeds for processing", enqueued));

        // 3. Run one exploration round
        let result = self.pipeline.run_round(self.brain, self.bank);
        details.push(format!(
            "Pipeline round: mined={}, absorbed={}, ke_entries={}",
            result.total_mined, result.total_absorbed, result.ke_entries_added
        ));

        // 4. Mark seeds as absorbed if they appear in pipeline's processed set
        let mut absorbed = 0;
        let mut crystals_created = 0;
        for target in &self.registry.targets.clone() {
            if self.registry.absorbed.contains_key(&target.url) {
                continue;
            }
            if self.pipeline.processed.contains(&target.url) || result.total_mined > 0 {
                self.registry.mark_absorbed(&target.url);
                absorbed += 1;
                details.push(format!("Absorbed: {} ({})", target.name, target.url));

                // Crystallize absorbed knowledge into SkillCrystals
                let edits: Vec<MicroEdit> = target
                    .capability_dims
                    .iter()
                    .map(|(dim, val)| MicroEdit::AdjustDimension(dim.clone(), *val))
                    .collect();
                if let Some(crystal) = self.auto_crystallizer.crystallize_from_absorption(
                    self.brain,
                    self.bank,
                    &target.url,
                    &target.name,
                    &format!("{:?}", target.domain),
                    &edits,
                    0.8,
                ) {
                    crystals_created += 1;
                    details.push(format!(
                        "Crystallized skill #{}: {} (eff={:.2})",
                        crystal.id, crystal.name, crystal.effectiveness
                    ));
                }
            }
        }

        // 5. Persist state — seeds + crystals survive restarts
        if let Err(e) = self.registry.save(&state_path) {
            details.push(format!("Warning: failed to save seed state: {}", e));
        }
        let crystal_path = self.work_dir.join("crystal_registry.json");
        if let Err(e) = self.auto_crystallizer.registry.save(&crystal_path) {
            details.push(format!("Warning: failed to save crystal registry: {}", e));
        }

        let (done, total) = self.registry.progress();
        details.push(format!(
            "Seed progress: {}/{} absorbed ({:.1}%) | {} crystals created",
            done,
            total,
            if total > 0 {
                done as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            crystals_created,
        ));

        SeedTickResult {
            enqueued,
            absorbed,
            total_absorbed: done,
            total_targets: total,
            total_mined: result.total_mined,
            crystals_created,
            details,
        }
    }

    pub fn progress(&self) -> (usize, usize) {
        self.registry.progress()
    }

    pub fn is_complete(&self) -> bool {
        self.registry.is_complete()
    }

    pub fn absorbed_list(&self) -> Vec<String> {
        self.registry.absorbed_list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_registry_has_all_targets() {
        let reg = SeedRegistry::new();
        assert_eq!(
            reg.targets.len(),
            16,
            "should have 16 evolution targets (15 repos + arnold)"
        );
    }

    #[test]
    fn test_priority_queue_returns_ordered() {
        let reg = SeedRegistry::new();
        let queue = reg.priority_queue();
        assert!(!queue.is_empty());
        // First items should be priority 5
        assert_eq!(queue[0].priority, 5);
    }

    #[test]
    fn test_mark_absorbed() {
        let mut reg = SeedRegistry::new();
        let url = "https://github.com/mksglu/context-mode";
        reg.mark_absorbed(url);
        assert!(reg.absorbed.contains_key(url));
        assert!(reg.targets.iter().find(|t| t.url == url).unwrap().absorbed);
    }

    #[test]
    fn test_progress_tracking() {
        let mut reg = SeedRegistry::new();
        let (done, total) = reg.progress();
        assert_eq!(done, 0);
        assert_eq!(total, 16);
        reg.mark_absorbed("https://github.com/mksglu/context-mode");
        assert_eq!(reg.progress().0, 1);
    }

    #[test]
    fn test_targets_by_domain() {
        let reg = SeedRegistry::new();
        let nt_shield_targets = reg.targets_by_domain(&ExploreDomain::Security);
        assert!(nt_shield_targets.len() >= 5);
        let general = reg.targets_by_domain(&ExploreDomain::General);
        assert!(!general.is_empty());
    }

    #[test]
    fn test_priority_5_targets_exist() {
        let reg = SeedRegistry::new();
        let priority_5: Vec<&EvolutionTarget> =
            reg.targets.iter().filter(|t| t.priority == 5).collect();
        assert_eq!(
            priority_5.len(),
            3,
            "context-mode, cybernt_shield-skills, arnold"
        );
    }

    #[test]
    fn test_generate_plans() {
        let reg = SeedRegistry::new();
        let mut planner = EvolutionPlanner::new();
        let plans = reg.generate_plans(&mut planner);
        assert_eq!(plans.len(), 16);
        assert!(plans[0].priority >= plans[1].priority);
    }

    #[test]
    fn test_seed_driver_has_crystalizer() {
        let reg = SeedRegistry::new();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("neotrix-test");
        let pipeline = ExplorationPipeline::new(path.clone());
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let crystallizer = AutoCrystallizer::new();
        let driver = SeedDriver::new(reg, pipeline, &mut brain, &mut bank, crystallizer, path);
        assert!(driver.auto_crystallizer.auto_crystallize);
        assert_eq!(driver.auto_crystallizer.total_crystallized, 0);
    }

    #[test]
    fn test_crystallize_from_absorbed_target() {
        let reg = SeedRegistry::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let mut crystallizer = AutoCrystallizer::new();

        // Pick the first target and create edits from its capability dims
        let target = &reg.targets[0];
        let edits: Vec<MicroEdit> = target
            .capability_dims
            .iter()
            .map(|(dim, val)| MicroEdit::AdjustDimension(dim.clone(), *val))
            .collect();

        let crystal = crystallizer.crystallize_from_absorption(
            &mut brain,
            &mut bank,
            &target.url,
            &target.name,
            &format!("{:?}", target.domain),
            &edits,
            0.8,
        );
        assert!(crystal.is_some(), "Should crystallize with reward 0.8");
        let crystal = crystal.unwrap();
        assert_eq!(crystal.tags.len(), 2);
        assert!(crystal.tags.contains(&target.name));
        assert!(crystal.effectiveness >= 0.3);

        // Verify registry state
        assert_eq!(crystallizer.registry.crystals.len(), 1);
        assert_eq!(crystallizer.total_crystallized, 1);
    }

    #[test]
    fn test_crystallize_all_domain_targets() {
        let reg = SeedRegistry::new();
        let mut crystallizer = AutoCrystallizer::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let domains: std::collections::HashSet<String> = reg
            .targets
            .iter()
            .map(|t| format!("{:?}", t.domain))
            .collect();

        // Crystallize one target per domain
        for domain_label in &domains {
            let target = reg
                .targets
                .iter()
                .find(|t| &format!("{:?}", t.domain) == domain_label)
                .expect("target per domain");
            let edits: Vec<MicroEdit> = target
                .capability_dims
                .iter()
                .map(|(dim, val)| MicroEdit::AdjustDimension(dim.clone(), *val))
                .collect();

            crystallizer.crystallize_from_absorption(
                &mut brain,
                &mut bank,
                &target.url,
                &target.name,
                domain_label,
                &edits,
                0.8,
            );
        }

        assert_eq!(crystallizer.registry.crystals.len(), domains.len());
        assert!(crystallizer
            .registry
            .summary()
            .contains(&format!("{} crystals", domains.len())));
    }

    #[test]
    fn test_crystal_registry_persist_cycle() {
        let tmp = std::env::temp_dir().join("neotrix-crystal-test.json");
        let path = tmp.as_path();

        // Create registry with one crystal and save
        let mut reg = CrystalRegistry::new();
        let crystal = SkillCrystal::new(
            0,
            "test-skill",
            "test pattern",
            crate::core::nt_core_self::StrategyKind::Reflection,
            crate::core::nt_core_self::AttentionDomain::Code,
            1,
        );
        reg.crystals.push(crystal);
        reg.next_id = 1;
        assert!(reg.save(path).is_ok());

        // Load and verify
        let loaded = CrystalRegistry::load(path).expect("load should succeed");
        assert_eq!(loaded.crystals.len(), 1);
        assert_eq!(loaded.crystals[0].name, "test-skill");
        assert_eq!(
            loaded.crystals[0].strategy,
            crate::core::nt_core_self::StrategyKind::Reflection
        );
        assert_eq!(
            loaded.crystals[0].domain,
            crate::core::nt_core_self::AttentionDomain::Code
        );

        // Cleanup
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_crystal_registry_load_nonexistent() {
        let tmp = std::env::temp_dir().join("neotrix-nonexistent-crystals.json");
        let path = tmp.as_path();
        let loaded = CrystalRegistry::load(path).expect("load should succeed with empty registry");
        assert!(loaded.crystals.is_empty());
    }
}
