use crate::core::nt_core_self::skill_crystal::SkillCrystal;
use crate::neotrix::nt_mind::self_edit::MicroEdit;

/// A tool execution trace entry.
#[derive(Debug, Clone)]
pub struct ToolTrace {
    pub tool_name: String,
    pub args: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: u64,
}

/// A proposed new skill discovered from mining action sequences.
#[derive(Debug, Clone)]
pub struct SkillProposal {
    pub name: String,
    pub confidence: f64,
    pub steps: Vec<String>,
    pub source_sequences: Vec<Vec<String>>,
}

impl ToolTrace {
    pub fn from_raw(raw: &(String, u64, bool), timestamp: u64) -> Self {
        Self {
            tool_name: raw.0.clone(),
            args: Vec::new(),
            success: raw.2,
            duration_ms: raw.1,
            timestamp,
        }
    }
}

/// Mines repeated action sequences from tool execution traces.
pub struct ActionSequenceMiner {
    pub min_sequence_length: usize,
    pub min_frequency: usize,
}

impl Default for ActionSequenceMiner {
    fn default() -> Self {
        Self {
            min_sequence_length: 2,
            min_frequency: 2,
        }
    }
}

impl ActionSequenceMiner {
    pub fn new(min_sequence_length: usize, min_frequency: usize) -> Self {
        Self {
            min_sequence_length,
            min_frequency,
        }
    }

    /// Mine repeated tool call sequences from trace data.
    pub fn mine(&self, traces: &[ToolTrace]) -> Vec<SkillProposal> {
        if traces.len() < self.min_sequence_length {
            return Vec::new();
        }

        let tool_names: Vec<&str> = traces.iter().map(|t| t.tool_name.as_str()).collect();
        let sequences = self.extract_sequences(&tool_names);
        let clustered = self.cluster_sequences(&sequences);
        let mut proposals = Vec::new();

        for (cluster_idx, cluster) in clustered.iter().enumerate() {
            if cluster.len() < self.min_frequency {
                continue;
            }
            let confidence = (cluster.len() as f64 / traces.len() as f64).min(1.0);
            let steps: Vec<String> = cluster[0].iter().map(|s| (*s).to_string()).collect();
            let name = format!("sequence_{}", cluster_idx);
            proposals.push(SkillProposal {
                name,
                confidence,
                steps,
                source_sequences: cluster
                    .iter()
                    .map(|seq| seq.iter().map(|s| (*s).to_string()).collect())
                    .collect(),
            });
        }

        proposals
    }

    /// Extract all subsequences of length >= min_sequence_length.
    fn extract_sequences<'a>(&self, tool_names: &[&'a str]) -> Vec<Vec<&'a str>> {
        let mut sequences = Vec::new();
        let mut i = 0;
        while i + self.min_sequence_length <= tool_names.len() {
            let seq: Vec<&str> = tool_names[i..i + self.min_sequence_length].to_vec();
            sequences.push(seq);
            i += 1;
        }
        sequences
    }

    /// Cluster identical sequences together.
    pub fn cluster_sequences<'a>(&self, sequences: &[Vec<&'a str>]) -> Vec<Vec<Vec<&'a str>>> {
        let mut clusters: Vec<Vec<Vec<&str>>> = Vec::new();
        for seq in sequences {
            let mut found = false;
            for cluster in &mut clusters {
                if !cluster.is_empty() && cluster[0] == *seq {
                    cluster.push(seq.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                clusters.push(vec![seq.clone()]);
            }
        }
        clusters
    }
}

/// Diagnosis result for a skill.
#[derive(Debug, Clone)]
pub enum SkillIssue {
    LowConfidence,
    MissingStep,
    IncorrectOrder,
    OutdatedPattern,
    UserRejected,
}

#[derive(Debug, Clone)]
pub struct SkillDiagnosis {
    pub needs_repair: bool,
    pub issue_type: SkillIssue,
    pub severity: f64,
    pub suggested_fix: String,
}

/// Diagnoses skill health from user feedback and LLM evaluation.
pub struct SkillDiagnoser {
    pub llm_diagnosis_enabled: bool,
}

impl Default for SkillDiagnoser {
    fn default() -> Self {
        Self {
            llm_diagnosis_enabled: true,
        }
    }
}

impl SkillDiagnoser {
    pub fn new(llm_diagnosis_enabled: bool) -> Self {
        Self {
            llm_diagnosis_enabled,
        }
    }

    /// Diagnose a skill's health.
    pub fn diagnose(
        &self,
        skill: &SkillCrystal,
        user_feedback: Option<f64>,
        llm_eval: Option<&str>,
    ) -> SkillDiagnosis {
        if let Some(fb) = user_feedback {
            if fb < 0.3 {
                return SkillDiagnosis {
                    needs_repair: true,
                    issue_type: SkillIssue::UserRejected,
                    severity: 1.0 - fb,
                    suggested_fix: "Review and rewrite skill steps based on user rejection"
                        .to_string(),
                };
            }
        }

        if skill.effectiveness < 0.4 {
            return SkillDiagnosis {
                needs_repair: true,
                issue_type: SkillIssue::LowConfidence,
                severity: 0.5 - skill.effectiveness,
                suggested_fix: "Increase confidence through repeated successful application"
                    .to_string(),
            };
        }

        if let Some(eval) = llm_eval {
            let lower = eval.to_lowercase();
            if lower.contains("missing") || lower.contains("incomplete") {
                return SkillDiagnosis {
                    needs_repair: true,
                    issue_type: SkillIssue::MissingStep,
                    severity: 0.6,
                    suggested_fix: format!("Add missing steps: {}", eval),
                };
            }
            if lower.contains("order") || lower.contains("sequence") {
                return SkillDiagnosis {
                    needs_repair: true,
                    issue_type: SkillIssue::IncorrectOrder,
                    severity: 0.5,
                    suggested_fix: format!("Reorder steps: {}", eval),
                };
            }
            if lower.contains("outdated") || lower.contains("deprecated") {
                return SkillDiagnosis {
                    needs_repair: true,
                    issue_type: SkillIssue::OutdatedPattern,
                    severity: 0.7,
                    suggested_fix: format!("Update pattern: {}", eval),
                };
            }
        }

        SkillDiagnosis {
            needs_repair: false,
            issue_type: SkillIssue::LowConfidence,
            severity: 0.0,
            suggested_fix: String::new(),
        }
    }
}

/// Repairs a skill based on diagnosis, producing a sequence of MicroEdits.
pub struct SkillRepairer;

impl SkillRepairer {
    pub fn new() -> Self {
        Self
    }

    /// Generate a repair plan (Vec<MicroEdit>) from a diagnosis.
    pub fn repair(&self, skill: &SkillCrystal, diagnosis: &SkillDiagnosis) -> Vec<MicroEdit> {
        if !diagnosis.needs_repair {
            return Vec::new();
        }

        match diagnosis.issue_type {
            SkillIssue::LowConfidence => {
                vec![
                    MicroEdit::AdjustDimension("skill_confidence".to_string(), 0.1),
                    MicroEdit::UpdateLearningRate(0.02),
                ]
            }
            SkillIssue::MissingStep => {
                vec![
                    MicroEdit::AddedDimension(skill.name.clone(), 0.5),
                    MicroEdit::AdjustDimension("skill_completeness".to_string(), 0.3),
                ]
            }
            SkillIssue::IncorrectOrder => {
                vec![MicroEdit::BatchAdjust(vec![
                    ("skill_order".to_string(), -0.2),
                    ("skill_clarity".to_string(), 0.2),
                ])]
            }
            SkillIssue::OutdatedPattern => {
                vec![
                    MicroEdit::ModifiedDimension(
                        "skill_pattern".to_string(),
                        skill.effectiveness,
                        0.3,
                    ),
                    MicroEdit::UpdateLearningRate(0.05),
                ]
            }
            SkillIssue::UserRejected => {
                vec![
                    MicroEdit::AdjustDimension("skill_acceptance".to_string(), -0.3),
                    MicroEdit::NormalizeVector,
                ]
            }
        }
    }
}

/// Full skill evolution pipeline: mine → diagnose → repair.
pub struct SkillEvolver {
    pub miner: ActionSequenceMiner,
    pub diagnoser: SkillDiagnoser,
    pub repairer: SkillRepairer,
    pub last_delta: f64,
    pub total_repaired: u64,
    pub total_proposed: u64,
}

impl Default for SkillEvolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillEvolver {
    pub fn new() -> Self {
        Self {
            miner: ActionSequenceMiner::default(),
            diagnoser: SkillDiagnoser::default(),
            repairer: SkillRepairer::new(),
            last_delta: 0.0,
            total_repaired: 0,
            total_proposed: 0,
        }
    }

    /// Mine proposals from raw tool traces.
    pub fn mine_and_propose(
        &mut self,
        raw_traces: &[(String, u64, bool)],
        timestamp: u64,
    ) -> Vec<SkillProposal> {
        let traces: Vec<ToolTrace> = raw_traces
            .iter()
            .map(|r| ToolTrace::from_raw(r, timestamp))
            .collect();
        let proposals = self.miner.mine(&traces);
        self.total_proposed += proposals.len() as u64;
        proposals
    }

    /// Diagnose a skill using optional user feedback and LLM evaluation.
    pub fn diagnose_skill(
        &self,
        skill: &SkillCrystal,
        user_feedback: Option<f64>,
        llm_eval: Option<&str>,
    ) -> SkillDiagnosis {
        self.diagnoser.diagnose(skill, user_feedback, llm_eval)
    }

    /// Repair a skill and track the operation.
    pub fn repair_skill(
        &mut self,
        skill: &SkillCrystal,
        diagnosis: &SkillDiagnosis,
    ) -> Vec<MicroEdit> {
        let edits = self.repairer.repair(skill, diagnosis);
        if !edits.is_empty() {
            self.total_repaired += 1;
        }
        edits
    }

    /// Run one full evolution cycle: mine → diagnose all → repair all.
    pub fn evolve(
        &mut self,
        crystals: &[SkillCrystal],
        raw_traces: &[(String, u64, bool)],
        timestamp: u64,
        user_feedback: Option<f64>,
    ) -> Vec<MicroEdit> {
        let _proposals = self.mine_and_propose(raw_traces, timestamp);

        let mut all_edits = Vec::new();
        for crystal in crystals {
            let diagnosis = self.diagnose_skill(crystal, user_feedback, None);
            if diagnosis.needs_repair {
                let edits = self.repair_skill(crystal, &diagnosis);
                all_edits.extend(edits);
            }
        }

        self.last_delta = all_edits.len() as f64;
        all_edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::attention_head::AttentionDomain;
    use crate::core::nt_core_self::reasoning_strategy::StrategyKind;

    #[test]
    fn test_action_sequence_miner_empty() {
        let miner = ActionSequenceMiner::new(2, 2);
        let proposals = miner.mine(&[]);
        assert!(proposals.is_empty());
    }

    #[test]
    fn test_action_sequence_miner_pattern() {
        let miner = ActionSequenceMiner::new(2, 2);
        let traces: Vec<ToolTrace> = vec![
            ToolTrace {
                tool_name: "read".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 1,
            },
            ToolTrace {
                tool_name: "edit".into(),
                args: vec![],
                success: true,
                duration_ms: 20,
                timestamp: 2,
            },
            ToolTrace {
                tool_name: "read".into(),
                args: vec![],
                success: true,
                duration_ms: 10,
                timestamp: 3,
            },
            ToolTrace {
                tool_name: "edit".into(),
                args: vec![],
                success: true,
                duration_ms: 20,
                timestamp: 4,
            },
        ];
        let proposals = miner.mine(&traces);
        assert!(!proposals.is_empty(), "should find at least one proposal");
        assert!(proposals[0].confidence > 0.0);
    }

    #[test]
    fn test_skill_diagnoser_user_feedback() {
        let diagnoser = SkillDiagnoser::new(true);
        let crystal = SkillCrystal::new(
            0,
            "test_skill",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        );
        let result = diagnoser.diagnose(&crystal, Some(0.2), None);
        assert!(result.needs_repair);
        assert!(matches!(result.issue_type, SkillIssue::UserRejected));
    }

    #[test]
    fn test_skill_diagnoser_low_confidence() {
        let diagnoser = SkillDiagnoser::new(true);
        let mut crystal = SkillCrystal::new(
            0,
            "weak",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        );
        crystal.effectiveness = 0.3;
        let result = diagnoser.diagnose(&crystal, None, None);
        assert!(result.needs_repair);
        assert!(matches!(result.issue_type, SkillIssue::LowConfidence));
    }

    #[test]
    fn test_skill_diagnoser_llm_eval() {
        let diagnoser = SkillDiagnoser::new(true);
        let crystal =
            SkillCrystal::new(0, "s", "p", StrategyKind::Direct, AttentionDomain::Code, 1);
        let result = diagnoser.diagnose(&crystal, None, Some("missing important validation step"));
        assert!(result.needs_repair);
        assert!(matches!(result.issue_type, SkillIssue::MissingStep));
    }

    #[test]
    fn test_skill_diagnoser_no_issue() {
        let diagnoser = SkillDiagnoser::new(true);
        let mut crystal = SkillCrystal::new(
            0,
            "good",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        );
        crystal.effectiveness = 0.9;
        let result = diagnoser.diagnose(&crystal, Some(0.9), Some("looks correct"));
        assert!(!result.needs_repair);
    }

    #[test]
    fn test_skill_repairer_low_confidence() {
        let repairer = SkillRepairer::new();
        let crystal = SkillCrystal::new(
            0,
            "test",
            "p",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        );
        let diagnosis = SkillDiagnosis {
            needs_repair: true,
            issue_type: SkillIssue::LowConfidence,
            severity: 0.3,
            suggested_fix: "increase confidence".to_string(),
        };
        let edits = repairer.repair(&crystal, &diagnosis);
        assert!(!edits.is_empty());
        assert!(matches!(edits[0], MicroEdit::AdjustDimension(_, _)));
    }

    #[test]
    fn test_skill_repairer_no_repair_needed() {
        let repairer = SkillRepairer::new();
        let crystal =
            SkillCrystal::new(0, "ok", "p", StrategyKind::Direct, AttentionDomain::Code, 1);
        let diagnosis = SkillDiagnosis {
            needs_repair: false,
            issue_type: SkillIssue::LowConfidence,
            severity: 0.0,
            suggested_fix: String::new(),
        };
        let edits = repairer.repair(&crystal, &diagnosis);
        assert!(edits.is_empty());
    }

    #[test]
    fn test_skill_evolver_full_cycle() {
        let mut evolver = SkillEvolver::new();
        let mut crystal = SkillCrystal::new(
            0,
            "main",
            "pattern",
            StrategyKind::Direct,
            AttentionDomain::Code,
            1,
        );
        crystal.effectiveness = 0.3;

        let raw: Vec<(String, u64, bool)> =
            vec![("read".into(), 10, true), ("edit".into(), 20, true)];

        let edits = evolver.evolve(&[crystal], &raw, 1, Some(0.2));
        assert!(!edits.is_empty(), "should produce repair edits");
        assert!(evolver.total_proposed > 0 || evolver.total_repaired > 0);
    }

    #[test]
    fn test_tool_trace_from_raw() {
        let raw = ("read_file".into(), 100, true);
        let trace = ToolTrace::from_raw(&raw, 42);
        assert_eq!(trace.tool_name, "read_file");
        assert_eq!(trace.duration_ms, 100);
        assert!(trace.success);
        assert_eq!(trace.timestamp, 42);
    }

    #[test]
    fn test_cluster_sequences() {
        let miner = ActionSequenceMiner::new(2, 2);
        let sequences = vec![vec!["a", "b"], vec!["a", "b"], vec!["c", "d"]];
        let clustered = miner.cluster_sequences(&sequences);
        assert_eq!(clustered.len(), 2);
        assert_eq!(clustered[0].len(), 2);
        assert_eq!(clustered[1].len(), 1);
    }
}
