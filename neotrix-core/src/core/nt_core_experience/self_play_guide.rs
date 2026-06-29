#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProblemDimension {
    Relevance,
    Cleanliness,
    Solvability,
    InformationGain,
}

impl ProblemDimension {
    pub fn variants() -> Vec<ProblemDimension> {
        vec![
            ProblemDimension::Relevance,
            ProblemDimension::Cleanliness,
            ProblemDimension::Solvability,
            ProblemDimension::InformationGain,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ProblemScore {
    pub problem: String,
    pub relevance: f64,
    pub cleanliness: f64,
    pub solvability: f64,
    pub information_gain: f64,
    pub composite: f64,
}

impl ProblemScore {
    pub fn composite(&self) -> f64 {
        0.35 * self.relevance
            + 0.25 * self.cleanliness
            + 0.25 * self.solvability
            + 0.15 * self.information_gain
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum AntiPatternType {
    PromptQuality,
    SessionHygiene,
    CodeReview,
    ToolMastery,
    ContextManagement,
}

#[derive(Debug, Clone)]
pub struct AntiPatternMatch {
    pub pattern_type: AntiPatternType,
    pub name: &'static str,
    pub description: &'static str,
    pub severity: u8,
    pub suggestion: &'static str,
}

#[derive(Debug, Clone)]
pub struct AntiPatternResult {
    pub matches: Vec<AntiPatternMatch>,
    pub overall_score: f64,
    pub critical_count: u8,
}

#[derive(Debug, Clone)]
pub struct SkillCandidate {
    pub name: String,
    pub description: String,
    pub frequency: u32,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct SelfPlayGuideStats {
    pub problems_scored: u64,
    pub anti_patterns_detected: u64,
    pub skills_discovered: u64,
    pub avg_composite_score: f64,
    pub per_type: HashMap<String, u64>,
}

impl Default for SelfPlayGuideStats {
    fn default() -> Self {
        Self {
            problems_scored: 0,
            anti_patterns_detected: 0,
            skills_discovered: 0,
            avg_composite_score: 0.0,
            per_type: HashMap::new(),
        }
    }
}

pub struct SelfPlayGuide {
    patterns: Vec<AntiPatternRule>,
    stats: SelfPlayGuideStats,
    score_history: Vec<f64>,
}

struct AntiPatternRule {
    pattern_type: AntiPatternType,
    name: &'static str,
    description: &'static str,
    severity: u8,
    suggestion: &'static str,
    detector: fn(&str) -> bool,
}

impl SelfPlayGuide {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_rules(),
            stats: SelfPlayGuideStats::default(),
            score_history: Vec::new(),
        }
    }

    fn default_rules() -> Vec<AntiPatternRule> {
        vec![
            AntiPatternRule {
                pattern_type: AntiPatternType::PromptQuality,
                name: "vague_goal",
                description: "Prompt lacks specific, measurable goal",
                severity: 3,
                suggestion: "Add concrete success criteria and output format",
                detector: |p| p.len() < 50 && !p.contains('?'),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::PromptQuality,
                name: "missing_context",
                description: "Prompt without background or constraints",
                severity: 2,
                suggestion: "Provide relevant context, constraints, and examples",
                detector: |p| !p.contains("context") && !p.contains("background") && p.len() > 100,
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::PromptQuality,
                name: "overloaded_prompt",
                description: "Multiple unrelated requests in one prompt",
                severity: 2,
                suggestion: "Split into sequential focused prompts",
                detector: |p| p.matches("and").count() > 5,
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::SessionHygiene,
                name: "message_storm",
                description: ">20 rapid messages without pause",
                severity: 2,
                suggestion: "Batch related questions and allow processing time",
                detector: |_| false,
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::SessionHygiene,
                name: "no_checkpoint",
                description: "Long session without saving progress",
                severity: 3,
                suggestion: "Create checkpoints after each completed step",
                detector: |_| false,
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::CodeReview,
                name: "no_diff_verification",
                description: "Applied change without reviewing diff",
                severity: 3,
                suggestion: "Always review diff before accepting changes",
                detector: |p| p.contains("apply") || p.contains("execute"),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::CodeReview,
                name: "blind_trust_test",
                description: "Accepted test results without manual verification",
                severity: 2,
                suggestion: "Spot-check test logic and edge cases",
                detector: |p| p.contains("pass") && p.contains("test"),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::ToolMastery,
                name: "manual_search",
                description: "Manually searching when grep/tools available",
                severity: 1,
                suggestion: "Use code search tools for faster and more accurate results",
                detector: |p| p.contains("find") && !p.contains("grep"),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::ToolMastery,
                name: "rewrite_verbose",
                description: "Rewriting code instead of using refactor tools",
                severity: 2,
                suggestion: "Use structured refactoring for safe, tracked changes",
                detector: |p| p.contains("rewrite") || p.contains("start over"),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::ContextManagement,
                name: "context_bloat",
                description: "Unnecessary large context without prioritization",
                severity: 2,
                suggestion: "Prune irrelevant files and focus on the core issue",
                detector: |p| p.len() > 2000 && !p.contains("summary"),
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::ContextManagement,
                name: "missing_instruction_file",
                description: "No AGENTS.md or CLAUDE.md found in project root",
                severity: 3,
                suggestion: "Create a project guide with conventions and patterns",
                detector: |_| false,
            },
            AntiPatternRule {
                pattern_type: AntiPatternType::PromptQuality,
                name: "binary_question",
                description: "Yes/no question that wastes an interaction",
                severity: 1,
                suggestion: "Ask open-ended questions or provide options to evaluate",
                detector: |p| p.trim().ends_with('?') && p.len() < 60,
            },
        ]
    }

    pub fn score_problem(&mut self, problem: &str, target: &str) -> ProblemScore {
        self.stats.problems_scored += 1;

        let relevance = self.compute_relevance(problem, target);
        let cleanliness = self.compute_cleanliness(problem);
        let solvability = self.compute_solvability(problem);
        let information_gain = self.compute_information_gain(problem, target);

        let score = ProblemScore {
            problem: problem.to_string(),
            relevance,
            cleanliness,
            solvability,
            information_gain,
            composite: 0.0,
        };
        let composite = score.composite();
        self.score_history.push(composite);
        let n = self.stats.problems_scored as f64;
        self.stats.avg_composite_score = if n > 1.0 {
            self.stats.avg_composite_score * ((n - 1.0) / n) + composite / n
        } else {
            composite
        };

        ProblemScore { composite, ..score }
    }

    fn compute_relevance(&self, problem: &str, target: &str) -> f64 {
        let p_lower = problem.to_lowercase();
        let t_lower = target.to_lowercase();
        let target_words: Vec<&str> = t_lower.split_whitespace().collect();
        let matches = target_words.iter().filter(|w| p_lower.contains(*w)).count();
        let ratio = matches as f64 / target_words.len().max(1) as f64;
        (ratio * 0.8 + 0.2).min(1.0)
    }

    fn compute_cleanliness(&self, problem: &str) -> f64 {
        let mut score: f64 = 1.0;
        let triggers = ["?", "how", "what", "why", "explain", "compare", "analyze"];
        let has_clear_intent = triggers.iter().any(|t| problem.to_lowercase().contains(t));
        if !has_clear_intent {
            score -= 0.2_f64;
        }
        if problem.len() < 20 {
            score -= 0.3_f64;
        }
        if problem.len() > 500 {
            score -= 0.15_f64;
        }
        let line_breaks = problem.matches('\n').count();
        if line_breaks > 5 {
            score -= 0.1_f64;
        }
        score.max(0.1_f64).min(1.0_f64)
    }

    fn compute_solvability(&self, problem: &str) -> f64 {
        let mut score: f64 = 0.7;
        if problem.contains("implement") || problem.contains("write") || problem.contains("create")
        {
            score += 0.2;
        }
        if problem.contains("debug") || problem.contains("fix") || problem.contains("error") {
            score += 0.15;
        }
        if problem.contains("?") {
            score += 0.1;
        }
        score.min(1.0)
    }

    fn compute_information_gain(&self, problem: &str, _target: &str) -> f64 {
        let mut score: f64 = 0.5;
        if problem.contains("why") || problem.contains("how") || problem.contains("compare") {
            score += 0.2;
        }
        if problem.contains("analyze") || problem.contains("evaluate") || problem.contains("assess")
        {
            score += 0.15;
        }
        if problem.contains("predict") || problem.contains("future") || problem.contains("trend") {
            score += 0.15;
        }
        score.min(1.0)
    }

    pub fn detect_anti_patterns(&mut self, session_log: &str) -> AntiPatternResult {
        let mut matches = Vec::new();
        for rule in &self.patterns {
            if (rule.detector)(session_log) {
                matches.push(AntiPatternMatch {
                    pattern_type: rule.pattern_type.clone(),
                    name: rule.name,
                    description: rule.description,
                    severity: rule.severity,
                    suggestion: rule.suggestion,
                });
                *self
                    .stats
                    .per_type
                    .entry(format!("{:?}", rule.pattern_type))
                    .or_insert(0) += 1;
            }
        }

        self.stats.anti_patterns_detected += matches.len() as u64;
        let critical_count = matches.iter().filter(|m| m.severity >= 3).count() as u8;
        let overall_score = if matches.is_empty() {
            1.0
        } else {
            let penalty = matches.iter().map(|m| m.severity as f64 * 0.1).sum::<f64>();
            (1.0 - penalty.min(0.8)).max(0.2)
        };

        AntiPatternResult {
            matches,
            overall_score,
            critical_count,
        }
    }

    pub fn discover_skills(&mut self, prompts: &[String]) -> Vec<SkillCandidate> {
        let mut freq: HashMap<String, u32> = HashMap::new();
        for p in prompts {
            let key = if p.len() < 100 {
                p.to_string()
            } else {
                p[..100].to_string()
            };
            *freq.entry(key).or_insert(0) += 1;
        }

        let mut candidates: Vec<SkillCandidate> = freq
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(name, count)| {
                let confidence = (count as f64 / prompts.len() as f64).min(1.0);
                SkillCandidate {
                    description: format!("Repeated pattern: used {} times", count),
                    name,
                    frequency: count,
                    confidence,
                }
            })
            .collect();

        candidates.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        self.stats.skills_discovered += candidates.len() as u64;
        candidates.truncate(10);
        candidates
    }

    pub fn stats(&self) -> &SelfPlayGuideStats {
        &self.stats
    }

    pub fn tick(&mut self, input: Option<(&str, &str, &[String])>) -> String {
        match input {
            Some((problem, target, session_logs)) => {
                let score = self.score_problem(problem, target);
                let ap_result = self.detect_anti_patterns(&session_logs.join("\n"));
                let skills = self.discover_skills(session_logs);
                format!(
                    "self_play_guide:tick=composite={:.3}_anti={}_skills={}_critical={}",
                    score.composite,
                    ap_result.matches.len(),
                    skills.len(),
                    ap_result.critical_count
                )
            }
            None => {
                format!(
                    "self_play_guide:tick=idle_scored={}_anti_total={}_skills_total={}_avg_score={:.3}",
                    self.stats.problems_scored,
                    self.stats.anti_patterns_detected,
                    self.stats.skills_discovered,
                    self.stats.avg_composite_score
                )
            }
        }
    }
}

impl Default for SelfPlayGuide {
    fn default() -> Self {
        Self::new()
    }
}
