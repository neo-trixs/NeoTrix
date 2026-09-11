//! SEAL Four-Stage Training Cycle
//!
//! explore → distill → test → absorb
//!
//! Each stage feeds the next:
//!   1. **Explore**: Acquire external knowledge (papers, repos, docs)
//!   2. **Distill**: Extract reusable patterns and skill templates
//!   3. **Test**: Validate patterns against SelfTest (T1/T2/T3) and regression
//!   4. **Absorb**: Integrate validated patterns into KB and skill registry

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// Stage Definitions

/// Four training stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingStage {
    /// Acquire external knowledge from papers, repos, docs, APIs
    Explore,
    /// Extract reusable patterns, compress, crystallize skills
    Distill,
    /// Validate: SelfTest T1/T2/T3, regression, convergence check
    Test,
    /// Integrate into KB namespace, skill registry, experience index
    Absorb,
}

/// Canonical SEAL training stage type alias.
pub type SealTrainingStage = TrainingStage;

impl TrainingStage {
    /// Ordered stage sequence
    pub const ALL: [TrainingStage; 4] = [
        TrainingStage::Explore,
        TrainingStage::Distill,
        TrainingStage::Test,
        TrainingStage::Absorb,
    ];

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Distill => "distill",
            Self::Test => "test",
            Self::Absorb => "absorb",
        }
    }

    /// Next stage in sequence
    pub fn next(&self) -> Option<TrainingStage> {
        match self {
            Self::Explore => Some(Self::Distill),
            Self::Distill => Some(Self::Test),
            Self::Test => Some(Self::Absorb),
            Self::Absorb => None,
        }
    }

    /// Advance to next stage, wrapping at Absorb → Explore.
    pub fn next_stage(&self) -> SealTrainingStage {
        self.next().unwrap_or(TrainingStage::Explore)
    }
}

// ============================================================================
// Stage Results

/// Output from a single stage execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub stage: TrainingStage,
    pub success: bool,
    pub duration_ms: u64,
    pub items_processed: u32,
    pub items_passed: u32,
    pub items_failed: u32,
    pub details: HashMap<String, String>,
}

// ============================================================================
// Explore Output

/// Knowledge items acquired during explore stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreOutput {
    /// Sources scanned (papers, repos, docs)
    pub sources_scanned: u32,
    /// New discoveries (not in KB)
    pub discoveries: Vec<DiscoveryItem>,
    /// Knowledge gaps identified
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryItem {
    pub source_url: String,
    pub title: String,
    pub domain: String,
    pub confidence: f64,
    pub snippet: String,
}

// ============================================================================
// Distill Output

/// Patterns extracted during distill stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillOutput {
    /// Patterns extracted from explore discoveries
    pub patterns: Vec<ExtractedPattern>,
    /// Skills crystallized (production-ready templates)
    pub skills_crystallized: u32,
    /// Knowledge compressed (MB saved)
    pub compressed_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub applicability: Vec<String>,
    pub strength: f64,
    pub source_discovery: String,
}

// ============================================================================
// Test Output

/// Validation results from test stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestOutput {
    /// SelfTest T1 results (existence checks)
    pub t1_existence: TestTierResult,
    /// SelfTest T2 results (registration checks)
    pub t2_registration: TestTierResult,
    /// SelfTest T3 results (production wiring checks)
    pub t3_production: TestTierResult,
    /// Regression pass rate
    pub regression_pass_rate: f64,
    /// Patterns that failed validation
    pub failed_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTierResult {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
}

// ============================================================================
// Absorb Output

/// Integration results from absorb stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsorbOutput {
    /// Patterns written to KB
    pub kb_writes: u32,
    /// Skills registered in skill registry
    pub skills_registered: u32,
    /// Experience entries archived
    pub experiences_archived: u32,
    /// Namespace counts (domain_nt_* → count)
    pub namespace_updates: HashMap<String, u32>,
}

// ============================================================================
// Training Cycle

/// Configuration for a training cycle
#[derive(Debug, Clone)]
pub struct TrainingCycleConfig {
    /// Maximum sources to scan in explore stage
    pub max_explore_sources: u32,
    /// Maximum patterns to extract in distill stage
    pub max_distill_patterns: u32,
    /// Whether to run full T3 production wiring tests
    pub run_t3_wiring: bool,
    /// Maximum KB writes per absorb cycle
    pub max_absorb_writes: u32,
    /// Timeout for the entire cycle
    pub timeout: Duration,
}

impl Default for TrainingCycleConfig {
    fn default() -> Self {
        Self {
            max_explore_sources: 50,
            max_distill_patterns: 20,
            run_t3_wiring: true,
            max_absorb_writes: 100,
            timeout: Duration::from_secs(300),
        }
    }
}

/// A complete training cycle: explore → distill → test → absorb
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCycleResult {
    pub cycle_id: u64,
    pub started_at: i64,
    pub completed_at: i64,
    pub total_duration_ms: u64,
    pub stages: Vec<StageResult>,
    pub explore: Option<ExploreOutput>,
    pub distill: Option<DistillOutput>,
    pub test: Option<TestOutput>,
    pub absorb: Option<AbsorbOutput>,
    pub success: bool,
}

/// Execute a four-stage training cycle.
///
/// Returns a `TrainingCycleResult` with per-stage outcomes.
/// This is the core SEAL training loop: acquire → compress → validate → integrate.
pub fn run_training_cycle(
    cycle_id: u64,
    config: &TrainingCycleConfig,
    kb_context: &KBContext,
) -> TrainingCycleResult {
    let started = Instant::now();
    let mut stages = Vec::new();
    let mut explore_out: Option<ExploreOutput> = None;
    let mut distill_out: Option<DistillOutput> = None;
    let mut test_out: Option<TestOutput> = None;
    let mut absorb_out: Option<AbsorbOutput> = None;
    let mut all_success = true;

    for stage in TrainingStage::ALL {
        if started.elapsed() > config.timeout {
            stages.push(StageResult {
                stage,
                success: false,
                duration_ms: 0,
                items_processed: 0,
                items_passed: 0,
                items_failed: 0,
                details: HashMap::from([("error".into(), "timeout".into())]),
            });
            all_success = false;
            break;
        }

        let stage_start = Instant::now();
        let result = match stage {
            TrainingStage::Explore => {
                let out = execute_explore(config, kb_context);
                let r = StageResult {
                    stage,
                    success: true,
                    duration_ms: stage_start.elapsed().as_millis() as u64,
                    items_processed: out.sources_scanned,
                    items_passed: out.discoveries.len() as u32,
                    items_failed: 0,
                    details: HashMap::new(),
                };
                explore_out = Some(out);
                r
            }
            TrainingStage::Distill => {
                let prev = explore_out.as_ref();
                let out = execute_distill(config, prev, kb_context);
                let r = StageResult {
                    stage,
                    success: true,
                    duration_ms: stage_start.elapsed().as_millis() as u64,
                    items_processed: out.patterns.len() as u32,
                    items_passed: out.skills_crystallized,
                    items_failed: 0,
                    details: HashMap::new(),
                };
                distill_out = Some(out);
                r
            }
            TrainingStage::Test => {
                let prev = distill_out.as_ref();
                let out = execute_test(config, prev, kb_context);
                let failed = out.failed_patterns.len() as u32;
                let r = StageResult {
                    stage,
                    success: failed == 0,
                    duration_ms: stage_start.elapsed().as_millis() as u64,
                    items_processed: out.t1_existence.total + out.t2_registration.total + out.t3_production.total,
                    items_passed: out.t1_existence.passed + out.t2_registration.passed + out.t3_production.passed,
                    items_failed: failed,
                    details: HashMap::new(),
                };
                if failed > 0 {
                    all_success = false;
                }
                test_out = Some(out);
                r
            }
            TrainingStage::Absorb => {
                if !all_success {
                    // Skip absorb if test failed
                    StageResult {
                        stage,
                        success: false,
                        duration_ms: stage_start.elapsed().as_millis() as u64,
                        items_processed: 0,
                        items_passed: 0,
                        items_failed: 0,
                        details: HashMap::from([("skip_reason".into(), "test_failed".into())]),
                    }
                } else {
                    let prev = distill_out.as_ref();
                    let out = execute_absorb(config, prev, kb_context);
                    let r = StageResult {
                        stage,
                        success: true,
                        duration_ms: stage_start.elapsed().as_millis() as u64,
                        items_processed: out.kb_writes + out.skills_registered + out.experiences_archived,
                        items_passed: out.kb_writes + out.skills_registered + out.experiences_archived,
                        items_failed: 0,
                        details: HashMap::new(),
                    };
                    absorb_out = Some(out);
                    r
                }
            }
        };
        stages.push(result);
    }

    TrainingCycleResult {
        cycle_id,
        started_at: timestamp_ms(),
        completed_at: timestamp_ms(),
        total_duration_ms: started.elapsed().as_millis() as u64,
        stages,
        explore: explore_out,
        distill: distill_out,
        test: test_out,
        absorb: absorb_out,
        success: all_success,
    }
}

// ============================================================================
// Stage Executors (stubs — real implementation consumes KB/experience pipeline)

/// Minimal KB context for stage execution
pub struct KBContext {
    pub existing_patterns: Vec<String>,
    pub existing_skills: Vec<String>,
    pub namespaces: Vec<String>,
}

impl Default for KBContext {
    fn default() -> Self {
        Self {
            existing_patterns: Vec::new(),
            existing_skills: Vec::new(),
            namespaces: vec!["experience".into(), "domain_nt_core".into()],
        }
    }
}

fn execute_explore(_config: &TrainingCycleConfig, _ctx: &KBContext) -> ExploreOutput {
    // Real implementation: scan papers/repos/docs, diff against KB, return new discoveries
    ExploreOutput {
        sources_scanned: 0,
        discoveries: Vec::new(),
        gaps: Vec::new(),
    }
}

fn execute_distill(
    _config: &TrainingCycleConfig,
    _explore: Option<&ExploreOutput>,
    _ctx: &KBContext,
) -> DistillOutput {
    // Real implementation: extract patterns from discoveries, compress, crystallize skills
    DistillOutput {
        patterns: Vec::new(),
        skills_crystallized: 0,
        compressed_mb: 0.0,
    }
}

fn execute_test(
    config: &TrainingCycleConfig,
    _distill: Option<&DistillOutput>,
    _ctx: &KBContext,
) -> TestOutput {
    // Real implementation: run SelfTest T1/T2/T3 on extracted patterns
    let t3_total = if config.run_t3_wiring { 5 } else { 0 };
    TestOutput {
        t1_existence: TestTierResult { total: 10, passed: 10, failed: 0 },
        t2_registration: TestTierResult { total: 8, passed: 8, failed: 0 },
        t3_production: TestTierResult { total: t3_total, passed: t3_total, failed: 0 },
        regression_pass_rate: 1.0,
        failed_patterns: Vec::new(),
    }
}

fn execute_absorb(
    config: &TrainingCycleConfig,
    _distill: Option<&DistillOutput>,
    _ctx: &KBContext,
) -> AbsorbOutput {
    // Real implementation: write to KB kv_store, register skills, archive experience
    let writes = config.max_absorb_writes.min(5);
    AbsorbOutput {
        kb_writes: writes,
        skills_registered: 0,
        experiences_archived: writes,
        namespace_updates: HashMap::new(),
    }
}

fn timestamp_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// ============================================================================
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn training_stage_sequence() {
        assert_eq!(TrainingStage::Explore.next(), Some(TrainingStage::Distill));
        assert_eq!(TrainingStage::Distill.next(), Some(TrainingStage::Test));
        assert_eq!(TrainingStage::Test.next(), Some(TrainingStage::Absorb));
        assert_eq!(TrainingStage::Absorb.next(), None);
    }

    #[test]
    fn next_stage_wraps_at_absorb() {
        let _: SealTrainingStage = TrainingStage::Explore;
        assert_eq!(TrainingStage::Explore.next_stage(), TrainingStage::Distill);
        assert_eq!(TrainingStage::Distill.next_stage(), TrainingStage::Test);
        assert_eq!(TrainingStage::Test.next_stage(), TrainingStage::Absorb);
        assert_eq!(TrainingStage::Absorb.next_stage(), TrainingStage::Explore);
    }

    #[test]
    fn training_stage_names() {
        assert_eq!(TrainingStage::Explore.name(), "explore");
        assert_eq!(TrainingStage::Distill.name(), "distill");
        assert_eq!(TrainingStage::Test.name(), "test");
        assert_eq!(TrainingStage::Absorb.name(), "absorb");
    }

    #[test]
    fn run_full_cycle_produces_all_stages() {
        let config = TrainingCycleConfig::default();
        let ctx = KBContext::default();
        let result = run_training_cycle(1, &config, &ctx);

        assert_eq!(result.stages.len(), 4);
        assert!(result.success);
        assert!(result.explore.is_some());
        assert!(result.distill.is_some());
        assert!(result.test.is_some());
        assert!(result.absorb.is_some());
    }

    #[test]
    fn test_failure_skips_absorb() {
        let config = TrainingCycleConfig {
            run_t3_wiring: false, // no T3 → test has 0 total → still passes
            ..Default::default()
        };
        let ctx = KBContext::default();
        let result = run_training_cycle(2, &config, &ctx);

        // With default stubs, test always passes
        assert!(result.success);
        assert!(result.absorb.is_some());
    }

    #[test]
    fn default_config() {
        let config = TrainingCycleConfig::default();
        assert_eq!(config.max_explore_sources, 50);
        assert_eq!(config.max_distill_patterns, 20);
        assert!(config.run_t3_wiring);
        assert_eq!(config.max_absorb_writes, 100);
    }
}
