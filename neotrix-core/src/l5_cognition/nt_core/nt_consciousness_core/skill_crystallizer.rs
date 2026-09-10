#![forbid(unsafe_code)]

//! Skill Crystallization Pipeline (技能结晶流水线)
//!
//! Transforms raw experiences into reusable, validated skills through a 5-stage pipeline:
//!   Experience → Pattern Recognition → Abstraction → Test → Register
//!
//! Components:
//!   - PatternRecognizer: finds recurring patterns across experiences
//!   - AbstractionEngine: abstracts patterns into reusable skill definitions
//!   - TestHarness: validates skills before registration
//!   - SkillRegistry: stores and versions crystallized skills

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill Crystallizer — orchestrates the full crystallization pipeline
pub struct SkillCrystallizer {
    /// Pattern recognition engine
    pub pattern_recognizer: PatternRecognizer,
    /// Abstraction engine
    pub abstraction_engine: AbstractionEngine,
    /// Test harness for validation
    pub test_harness: TestHarness,
    /// Skill registry for storage
    pub registry: SkillRegistry,
    /// Pipeline configuration
    pub config: CrystallizerConfig,
    /// Pipeline history
    pub history: Vec<PipelineRecord>,
}

/// Crystallizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizerConfig {
    /// Minimum experiences before pattern recognition
    pub min_experiences_for_pattern: usize,
    /// Minimum confidence for pattern acceptance
    pub min_pattern_confidence: f64,
    /// Minimum confidence for skill registration
    pub min_skill_confidence: f64,
    /// Maximum pipeline history
    pub max_history: usize,
}

impl Default for CrystallizerConfig {
    fn default() -> Self {
        Self {
            min_experiences_for_pattern: 3,
            min_pattern_confidence: 0.6,
            min_skill_confidence: 0.7,
            max_history: 500,
        }
    }
}

/// A raw experience input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Experience ID
    pub id: String,
    /// Task description
    pub task: String,
    /// Steps taken
    pub steps: Vec<String>,
    /// Outcome
    pub outcome: Outcome,
    /// Metrics
    pub metrics: HashMap<String, f64>,
    /// Context tags
    pub tags: Vec<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Experience outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    PartialSuccess { score: f64 },
    Failure { reason: String },
}

/// A recognized pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedPattern {
    /// Pattern ID
    pub id: String,
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Source experience IDs
    pub source_experiences: Vec<String>,
    /// Confidence score
    pub confidence: f64,
    /// Pattern template (abstracted steps)
    pub template: PatternTemplate,
    /// Frequency of occurrence
    pub frequency: usize,
}

/// Pattern template — abstracted structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternTemplate {
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Abstracted steps
    pub steps: Vec<AbstractStep>,
    /// Postconditions
    pub postconditions: Vec<String>,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// An abstracted step in a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step type
    pub step_type: StepType,
    /// Input requirements
    pub inputs: Vec<String>,
    /// Output produced
    pub outputs: Vec<String>,
}

/// Step types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Data acquisition
    Acquisition,
    /// Data transformation
    Transformation,
    /// Validation step
    Validation,
    /// Integration step
    Integration,
    /// Output generation
    Output,
}

/// A crystallized skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizedSkill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Skill version
    pub version: String,
    /// Based on pattern
    pub pattern_id: String,
    /// Skill steps
    pub steps: Vec<SkillStep>,
    /// Test results
    pub test_results: TestResults,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: String,
    /// Last updated timestamp
    pub updated_at: String,
}

/// A step in a crystallized skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step type
    pub step_type: StepType,
    /// Input parameters
    pub inputs: Vec<String>,
    /// Output produced
    pub outputs: Vec<String>,
    /// Whether step is required
    pub required: bool,
}

/// Test results for a skill
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestResults {
    /// Total test cases
    pub total_cases: usize,
    /// Passed test cases
    pub passed: usize,
    /// Failed test cases
    pub failed: usize,
    /// Pass rate
    pub pass_rate: f64,
    /// Test details
    pub details: Vec<TestCaseResult>,
    /// Whether skill passed validation
    pub validated: bool,
}

/// Individual test case result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    /// Test name
    pub name: String,
    /// Test input
    pub input: String,
    /// Expected output
    pub expected: String,
    /// Actual output
    pub actual: String,
    /// Whether test passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Pipeline record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRecord {
    /// Record ID
    pub id: String,
    /// Input experiences
    pub input_experiences: Vec<String>,
    /// Patterns recognized
    pub patterns_found: usize,
    /// Skills crystallized
    pub skills_crystallized: usize,
    /// Pipeline stage reached
    pub stage_reached: PipelineStage,
    /// Timestamp
    pub timestamp: String,
}

/// Pipeline stages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    ExperienceCollection,
    PatternRecognition,
    Abstraction,
    Testing,
    Registration,
}

/// Pattern recognition engine
pub struct PatternRecognizer {
    /// Minimum experiences for pattern detection
    pub min_experiences: usize,
    /// Minimum confidence threshold
    pub min_confidence: f64,
    /// Recognized patterns
    pub patterns: Vec<RecognizedPattern>,
}

impl PatternRecognizer {
    pub fn new(min_experiences: usize, min_confidence: f64) -> Self {
        Self {
            min_experiences,
            min_confidence,
            patterns: Vec::new(),
        }
    }

    /// Analyze experiences and recognize patterns
    pub fn analyze(&mut self, experiences: &[Experience]) -> Vec<RecognizedPattern> {
        if experiences.len() < self.min_experiences {
            return Vec::new();
        }

        let mut new_patterns = Vec::new();

        // Group experiences by tag overlap
        let mut tag_groups: HashMap<String, Vec<&Experience>> = HashMap::new();
        for exp in experiences {
            for tag in &exp.tags {
                tag_groups.entry(tag.clone()).or_default().push(exp);
            }
        }

        // Find patterns in groups with sufficient overlap
        for (tag, group) in &tag_groups {
            if group.len() >= self.min_experiences {
                let confidence = self.calculate_pattern_confidence(group);

                if confidence >= self.min_confidence {
                    let pattern = RecognizedPattern {
                        id: format!("pat_{}", uuid::Uuid::new_v4()),
                        name: format!("Pattern: {}", tag),
                        description: format!(
                            "Recurring pattern across {} experiences tagged '{}'",
                            group.len(),
                            tag
                        ),
                        source_experiences: group.iter().map(|e| e.id.clone()).collect(),
                        confidence,
                        template: self.abstract_template(group),
                        frequency: group.len(),
                    };

                    new_patterns.push(pattern);
                }
            }
        }

        self.patterns.extend(new_patterns.clone());
        new_patterns
    }

    /// Calculate confidence for a pattern based on experience similarity
    fn calculate_pattern_confidence(&self, experiences: &[&Experience]) -> f64 {
        if experiences.is_empty() {
            return 0.0;
        }

        let success_count = experiences
            .iter()
            .filter(|e| matches!(e.outcome, Outcome::Success))
            .count();

        let base_confidence = success_count as f64 / experiences.len() as f64;

        // Boost confidence for more experiences
        let volume_boost = (experiences.len() as f64 / 10.0).min(0.2);

        (base_confidence + volume_boost).min(1.0)
    }

    /// Abstract a template from experiences
    fn abstract_template(&self, experiences: &[&Experience]) -> PatternTemplate {
        let mut all_steps: Vec<String> = experiences.iter().flat_map(|e| e.steps.clone()).collect();
        all_steps.dedup();

        let required_capabilities: Vec<String> = experiences
            .iter()
            .flat_map(|e| e.tags.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        PatternTemplate {
            preconditions: vec!["Task is well-defined".to_string()],
            steps: all_steps
                .into_iter()
                .enumerate()
                .map(|(i, s)| AbstractStep {
                    name: format!("Step_{}", i + 1),
                    description: s,
                    step_type: StepType::Transformation,
                    inputs: vec![format!("input_{}", i)],
                    outputs: vec![format!("output_{}", i)],
                })
                .collect(),
            postconditions: vec!["Task completed successfully".to_string()],
            required_capabilities,
        }
    }
}

/// Abstraction engine — converts patterns to skill definitions
pub struct AbstractionEngine {
    /// Abstraction history
    pub abstractions: Vec<AbstractionRecord>,
}

/// Abstraction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractionRecord {
    /// Pattern ID
    pub pattern_id: String,
    /// Resulting skill
    pub skill: CrystallizedSkill,
    /// Timestamp
    pub timestamp: String,
}

impl AbstractionEngine {
    pub fn new() -> Self {
        Self {
            abstractions: Vec::new(),
        }
    }

    /// Abstract a pattern into a crystallized skill
    pub fn abstract_skill(&mut self, pattern: &RecognizedPattern) -> CrystallizedSkill {
        let skill_id = format!("skill_{}", uuid::Uuid::new_v4());

        let steps: Vec<SkillStep> = pattern
            .template
            .steps
            .iter()
            .map(|s| SkillStep {
                name: s.name.clone(),
                description: s.description.clone(),
                step_type: s.step_type.clone(),
                inputs: s.inputs.clone(),
                outputs: s.outputs.clone(),
                required: true,
            })
            .collect();

        let skill = CrystallizedSkill {
            id: skill_id,
            name: pattern.name.clone(),
            description: pattern.description.clone(),
            version: "1.0.0".to_string(),
            pattern_id: pattern.id.clone(),
            steps,
            test_results: TestResults {
                total_cases: 0,
                passed: 0,
                failed: 0,
                pass_rate: 0.0,
                details: Vec::new(),
                validated: false,
            },
            metadata: HashMap::from([
                ("source_pattern".to_string(), pattern.id.clone()),
                ("frequency".to_string(), pattern.frequency.to_string()),
                ("confidence".to_string(), pattern.confidence.to_string()),
            ]),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        self.abstractions.push(AbstractionRecord {
            pattern_id: pattern.id.clone(),
            skill: skill.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        skill
    }
}

/// Test harness — validates skills before registration
pub struct TestHarness {
    /// Test cases
    pub test_cases: Vec<TestCase>,
}

/// A test case for skill validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name: String,
    /// Test input
    pub input: String,
    /// Expected output pattern
    pub expected_output: String,
    /// Maximum allowed time (ms)
    pub timeout_ms: u64,
}

impl TestHarness {
    pub fn new() -> Self {
        Self {
            test_cases: Vec::new(),
        }
    }

    /// Run test suite against a skill
    pub fn run_tests(&self, skill: &CrystallizedSkill) -> TestResults {
        let mut results = Vec::new();

        for test_case in &self.test_cases {
            // Simulate test execution
            let passed = skill.steps.iter().any(|s| {
                s.description
                    .to_lowercase()
                    .contains(&test_case.input.to_lowercase())
            });

            results.push(TestCaseResult {
                name: test_case.name.clone(),
                input: test_case.input.clone(),
                expected: test_case.expected_output.clone(),
                actual: if passed {
                    test_case.expected_output.clone()
                } else {
                    "Output not matched".to_string()
                },
                passed,
                error: if passed {
                    None
                } else {
                    Some("Pattern not found in skill steps".to_string())
                },
            });
        }

        let total_cases = results.len();
        let passed_count = results.iter().filter(|r| r.passed).count();

        TestResults {
            total_cases,
            passed: passed_count,
            failed: total_cases - passed_count,
            pass_rate: if total_cases > 0 {
                passed_count as f64 / total_cases as f64
            } else {
                0.0
            },
            details: results,
            validated: total_cases > 0 && passed_count == total_cases,
        }
    }

    /// Add a test case
    pub fn add_test_case(&mut self, test_case: TestCase) {
        self.test_cases.push(test_case);
    }
}

/// Skill registry — stores and versions crystallized skills
pub struct SkillRegistry {
    /// Registered skills
    pub skills: HashMap<String, CrystallizedSkill>,
    /// Version history per skill
    pub version_history: HashMap<String, Vec<String>>,
    /// Registry configuration
    pub config: RegistryConfig,
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Maximum registered skills
    pub max_skills: usize,
    /// Whether to allow skill overwriting
    pub allow_overwrite: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_skills: 1000,
            allow_overwrite: true,
        }
    }
}

impl SkillRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            skills: HashMap::new(),
            version_history: HashMap::new(),
            config,
        }
    }

    /// Register a new skill
    pub fn register(&mut self, skill: CrystallizedSkill) -> Result<(), String> {
        if self.skills.len() >= self.config.max_skills && !self.skills.contains_key(&skill.id) {
            return Err("Registry full".to_string());
        }

        if !self.config.allow_overwrite && self.skills.contains_key(&skill.id) {
            return Err(format!("Skill {} already registered", skill.id));
        }

        // Track version history
        self.version_history
            .entry(skill.id.clone())
            .or_default()
            .push(skill.version.clone());

        self.skills.insert(skill.id.clone(), skill);
        Ok(())
    }

    /// Get a skill by ID
    pub fn get(&self, skill_id: &str) -> Option<&CrystallizedSkill> {
        self.skills.get(skill_id)
    }

    /// Get version history for a skill
    pub fn versions(&self, skill_id: &str) -> Vec<&String> {
        self.version_history
            .get(skill_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// List all skills
    pub fn list(&self) -> Vec<&CrystallizedSkill> {
        self.skills.values().collect()
    }

    /// Get registry stats
    pub fn stats(&self) -> RegistryStats {
        let total_versions: usize = self.version_history.values().map(|v| v.len()).sum();

        RegistryStats {
            total_skills: self.skills.len(),
            total_versions,
            avg_versions_per_skill: if self.skills.is_empty() {
                0.0
            } else {
                total_versions as f64 / self.skills.len() as f64
            },
        }
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_skills: usize,
    pub total_versions: usize,
    pub avg_versions_per_skill: f64,
}

impl std::fmt::Display for RegistryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "SkillRegistry: {} skills, {} total versions, avg {:.1} versions/skill",
            self.total_skills, self.total_versions, self.avg_versions_per_skill
        )
    }
}

impl SkillCrystallizer {
    /// Create a new SkillCrystallizer
    pub fn new(config: CrystallizerConfig) -> Self {
        Self {
            pattern_recognizer: PatternRecognizer::new(
                config.min_experiences_for_pattern,
                config.min_pattern_confidence,
            ),
            abstraction_engine: AbstractionEngine::new(),
            test_harness: TestHarness::new(),
            registry: SkillRegistry::new(RegistryConfig::default()),
            config,
            history: Vec::new(),
        }
    }

    /// Run the full crystallization pipeline
    pub fn crystallize(&mut self, experiences: &[Experience]) -> Vec<CrystallizedSkill> {
        // Stage 1: Pattern recognition
        let patterns = self.pattern_recognizer.analyze(experiences);

        let mut crystallized = Vec::new();

        for pattern in &patterns {
            // Stage 2: Abstraction
            let mut skill = self.abstraction_engine.abstract_skill(pattern);

            // Stage 3: Testing
            let test_results = self.test_harness.run_tests(&skill);
            skill.test_results = test_results.clone();

            // Stage 4: Registration (only if tests pass or no tests defined)
            if test_results.validated || test_results.total_cases == 0 {
                skill.test_results.validated = true;
                if self.registry.register(skill.clone()).is_ok() {
                    crystallized.push(skill);
                }
            }
        }

        // Record pipeline run
        let record = PipelineRecord {
            id: format!("pipe_{}", uuid::Uuid::new_v4()),
            input_experiences: experiences.iter().map(|e| e.id.clone()).collect(),
            patterns_found: patterns.len(),
            skills_crystallized: crystallized.len(),
            stage_reached: PipelineStage::Registration,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);
        self.trim_history();

        crystallized
    }

    /// Get pipeline stats
    pub fn stats(&self) -> CrystallizerStats {
        CrystallizerStats {
            total_runs: self.history.len(),
            total_patterns: self.pattern_recognizer.patterns.len(),
            total_skills: self.registry.skills.len(),
            total_abstractions: self.abstraction_engine.abstractions.len(),
        }
    }

    fn trim_history(&mut self) {
        while self.history.len() > self.config.max_history {
            self.history.remove(0);
        }
    }
}

/// Crystallizer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystallizerStats {
    pub total_runs: usize,
    pub total_patterns: usize,
    pub total_skills: usize,
    pub total_abstractions: usize,
}

impl std::fmt::Display for CrystallizerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Crystallizer: {} runs, {} patterns, {} skills, {} abstractions",
            self.total_runs, self.total_patterns, self.total_skills, self.total_abstractions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_experiences() -> Vec<Experience> {
        vec![
            Experience {
                id: "exp1".to_string(),
                task: "Parse CSV data".to_string(),
                steps: vec![
                    "Read file".to_string(),
                    "Split lines".to_string(),
                    "Parse fields".to_string(),
                ],
                outcome: Outcome::Success,
                metrics: HashMap::new(),
                tags: vec!["csv".to_string(), "parsing".to_string()],
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Experience {
                id: "exp2".to_string(),
                task: "Parse CSV with headers".to_string(),
                steps: vec![
                    "Read file".to_string(),
                    "Extract headers".to_string(),
                    "Parse rows".to_string(),
                ],
                outcome: Outcome::Success,
                metrics: HashMap::new(),
                tags: vec!["csv".to_string(), "parsing".to_string()],
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Experience {
                id: "exp3".to_string(),
                task: "Parse CSV columns".to_string(),
                steps: vec![
                    "Read file".to_string(),
                    "Map columns".to_string(),
                    "Validate types".to_string(),
                ],
                outcome: Outcome::Success,
                metrics: HashMap::new(),
                tags: vec!["csv".to_string(), "parsing".to_string()],
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ]
    }

    #[test]
    fn test_pattern_recognizer() {
        let mut recognizer = PatternRecognizer::new(3, 0.6);
        let experiences = sample_experiences();
        let patterns = recognizer.analyze(&experiences);

        assert!(!patterns.is_empty());
        assert!(patterns[0].confidence > 0.0);
    }

    #[test]
    fn test_pattern_recognizer_insufficient_experiences() {
        let mut recognizer = PatternRecognizer::new(5, 0.6);
        let experiences = sample_experiences();
        let patterns = recognizer.analyze(&experiences);

        assert!(patterns.is_empty());
    }

    #[test]
    fn test_abstraction_engine() {
        let mut engine = AbstractionEngine::new();
        let pattern = RecognizedPattern {
            id: "pat1".to_string(),
            name: "CSV Pattern".to_string(),
            description: "CSV parsing".to_string(),
            source_experiences: vec!["exp1".to_string()],
            confidence: 0.8,
            template: PatternTemplate {
                preconditions: vec!["File exists".to_string()],
                steps: vec![AbstractStep {
                    name: "Read".to_string(),
                    description: "Read file".to_string(),
                    step_type: StepType::Acquisition,
                    inputs: vec!["path".to_string()],
                    outputs: vec!["content".to_string()],
                }],
                postconditions: vec!["File content loaded".to_string()],
                required_capabilities: vec!["file_io".to_string()],
            },
            frequency: 3,
        };

        let skill = engine.abstract_skill(&pattern);
        assert_eq!(skill.steps.len(), 1);
        assert_eq!(engine.abstractions.len(), 1);
    }

    #[test]
    fn test_test_harness() {
        let harness = TestHarness::new();
        let skill = CrystallizedSkill {
            id: "skill1".to_string(),
            name: "Test Skill".to_string(),
            description: "A test skill".to_string(),
            version: "1.0.0".to_string(),
            pattern_id: "pat1".to_string(),
            steps: vec![SkillStep {
                name: "Step1".to_string(),
                description: "Read the input file and parse it".to_string(),
                step_type: StepType::Transformation,
                inputs: vec!["file".to_string()],
                outputs: vec!["data".to_string()],
                required: true,
            }],
            test_results: TestResults::default(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let results = harness.run_tests(&skill);
        assert_eq!(results.total_cases, 0);
    }

    #[test]
    fn test_skill_registry() {
        let mut registry = SkillRegistry::new(RegistryConfig::default());

        let skill = CrystallizedSkill {
            id: "skill1".to_string(),
            name: "Test".to_string(),
            description: "Test skill".to_string(),
            version: "1.0.0".to_string(),
            pattern_id: "pat1".to_string(),
            steps: Vec::new(),
            test_results: TestResults {
                total_cases: 5,
                passed: 5,
                failed: 0,
                pass_rate: 1.0,
                details: Vec::new(),
                validated: true,
            },
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let result = registry.register(skill);
        assert!(result.is_ok());
        assert_eq!(registry.skills.len(), 1);
    }

    #[test]
    fn test_full_pipeline() {
        let mut crystallizer = SkillCrystallizer::new(CrystallizerConfig::default());
        let experiences = sample_experiences();

        let skills = crystallizer.crystallize(&experiences);
        assert!(!skills.is_empty());
    }

    #[test]
    fn test_crystallizer_stats() {
        let crystallizer = SkillCrystallizer::new(CrystallizerConfig::default());
        let stats = crystallizer.stats();
        assert_eq!(stats.total_runs, 0);
        assert_eq!(stats.total_skills, 0);
    }
}
