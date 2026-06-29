use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityDimension {
    Completeness,
    Consistency,
    Conformity,
    Accuracy,
    Uniqueness,
    Integrity,
}

impl fmt::Display for QualityDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QualityDimension::Completeness => write!(f, "Completeness"),
            QualityDimension::Consistency => write!(f, "Consistency"),
            QualityDimension::Conformity => write!(f, "Conformity"),
            QualityDimension::Accuracy => write!(f, "Accuracy"),
            QualityDimension::Uniqueness => write!(f, "Uniqueness"),
            QualityDimension::Integrity => write!(f, "Integrity"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleType {
    // Completeness (5)
    MissingValue,
    NullRate { max_pct: f64 },
    EmptyField,
    RequiredFieldMissing,
    PartialRecord,
    // Consistency (5)
    CrossFieldConflict { fields: Vec<String> },
    TypeMismatch,
    UnitInconsistency,
    DuplicateRecord,
    ReferenceViolation,
    // Conformity (5)
    FormatMismatch { expected: String },
    PatternViolation { regex: String },
    OutOfRange { min: f64, max: f64 },
    EnumViolation { allowed: Vec<String> },
    LengthViolation { min: usize, max: usize },
    // Accuracy (5)
    OutlierNumeric { z_score: f64 },
    DriftDetected { baseline: f64, threshold: f64 },
    NegativeWherePositive,
    SumDiscrepancy { expected: f64 },
    TemporalInconsistency,
    // Uniqueness (5)
    DuplicateKey,
    NearDuplicate { similarity_threshold: f64 },
    CompositeKeyViolation,
    SurrogateKeyGap,
    HashCollision,
    // Integrity (6)
    OrphanReference,
    CircularDependency,
    LevelMismatch,
    AggregateDiscrepancy,
    ConstraintViolation { constraint: String },
    CascadeFailure { depth: usize },
}

impl RuleType {
    pub fn dimension(&self) -> QualityDimension {
        match self {
            RuleType::MissingValue
            | RuleType::NullRate { .. }
            | RuleType::EmptyField
            | RuleType::RequiredFieldMissing
            | RuleType::PartialRecord => QualityDimension::Completeness,
            RuleType::CrossFieldConflict { .. }
            | RuleType::TypeMismatch
            | RuleType::UnitInconsistency
            | RuleType::DuplicateRecord
            | RuleType::ReferenceViolation => QualityDimension::Consistency,
            RuleType::FormatMismatch { .. }
            | RuleType::PatternViolation { .. }
            | RuleType::OutOfRange { .. }
            | RuleType::EnumViolation { .. }
            | RuleType::LengthViolation { .. } => QualityDimension::Conformity,
            RuleType::OutlierNumeric { .. }
            | RuleType::DriftDetected { .. }
            | RuleType::NegativeWherePositive
            | RuleType::SumDiscrepancy { .. }
            | RuleType::TemporalInconsistency => QualityDimension::Accuracy,
            RuleType::DuplicateKey
            | RuleType::NearDuplicate { .. }
            | RuleType::CompositeKeyViolation
            | RuleType::SurrogateKeyGap
            | RuleType::HashCollision => QualityDimension::Uniqueness,
            RuleType::OrphanReference
            | RuleType::CircularDependency
            | RuleType::LevelMismatch
            | RuleType::AggregateDiscrepancy
            | RuleType::ConstraintViolation { .. }
            | RuleType::CascadeFailure { .. } => QualityDimension::Integrity,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Severity {
    Low(u8),
    Medium(u8),
    High(u8),
    Critical(u8),
}

impl Severity {
    pub fn count(&self) -> u8 {
        match self {
            Severity::Low(c) | Severity::Medium(c) | Severity::High(c) | Severity::Critical(c) => {
                *c
            }
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Severity::Low(_) => 0.25,
            Severity::Medium(_) => 0.50,
            Severity::High(_) => 0.75,
            Severity::Critical(_) => 1.00,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QualityIssue {
    pub id: u64,
    pub dimension: QualityDimension,
    pub rule: RuleType,
    pub severity: Severity,
    pub description: String,
    pub source: String,
    pub detected_at: Instant,
    pub resolved: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub issue_id: u64,
    pub root_cause: String,
    pub impact: String,
    pub suggested_fix: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum ActionType {
    RejectRecord,
    TransformField {
        field: String,
        transform: String,
    },
    Reprocess,
    FlagForReview,
    AutoCorrect {
        old_value: String,
        new_value: String,
    },
    DropRecord,
    CreateTicket,
}

#[derive(Debug, Clone)]
pub struct RemediationAction {
    pub action_id: u64,
    pub issue_id: u64,
    pub action_type: ActionType,
    pub applied: bool,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct MonitorSnapshot {
    pub timestamp: Instant,
    pub total_records: usize,
    pub issues_found: usize,
    pub dimension_breakdown: HashMap<QualityDimension, usize>,
    pub overall_quality_score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Degrading,
    Stable,
    InsufficientData,
}

#[derive(Debug, Clone)]
pub struct QualityTrend {
    pub dimension: QualityDimension,
    pub scores: VecDeque<f64>,
    pub direction: TrendDirection,
}

impl QualityTrend {
    pub fn new(dimension: QualityDimension) -> Self {
        Self {
            dimension,
            scores: VecDeque::with_capacity(100),
            direction: TrendDirection::InsufficientData,
        }
    }

    pub fn push(&mut self, score: f64) {
        if self.scores.len() >= 100 {
            self.scores.pop_front();
        }
        self.scores.push_back(score);
        self.recompute_direction();
    }

    fn recompute_direction(&mut self) {
        if self.scores.len() < 3 {
            self.direction = TrendDirection::InsufficientData;
            return;
        }
        let n = self.scores.len();
        let half = n / 2;
        let first_half: VecDeque<f64> = self.scores.iter().take(half).copied().collect();
        let second_half: VecDeque<f64> = self.scores.iter().skip(half).copied().collect();
        let avg1: f64 = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let avg2: f64 = second_half.iter().sum::<f64>() / second_half.len() as f64;
        let diff = avg2 - avg1;
        if diff.abs() < 0.02 {
            self.direction = TrendDirection::Stable;
        } else if diff > 0.0 {
            self.direction = TrendDirection::Improving;
        } else {
            self.direction = TrendDirection::Degrading;
        }
    }
}

#[derive(Debug, Clone)]
pub struct DQConfig {
    pub max_issues: usize,
    pub auto_remediate: bool,
    pub snapshot_interval_cycles: usize,
    pub z_score_threshold: f64,
    pub similarity_threshold: f64,
    pub learning_enabled: bool,
}

impl Default for DQConfig {
    fn default() -> Self {
        Self {
            max_issues: 10000,
            auto_remediate: false,
            snapshot_interval_cycles: 30,
            z_score_threshold: 3.0,
            similarity_threshold: 0.95,
            learning_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataQualityPipeline {
    pub config: DQConfig,
    pub issues: Vec<QualityIssue>,
    pub diagnostics: HashMap<u64, DiagnosticReport>,
    pub remediations: Vec<RemediationAction>,
    pub snapshots: VecDeque<MonitorSnapshot>,
    pub trends: HashMap<QualityDimension, QualityTrend>,
    pub next_id: u64,
    pub rule_registry: Vec<RuleType>,
    pub learned_patterns: Vec<(RuleType, String, String)>,
    cycle_counter: usize,
}

impl DataQualityPipeline {
    pub fn new(config: DQConfig) -> Self {
        let rule_registry = vec![
            RuleType::MissingValue,
            RuleType::NullRate { max_pct: 0.05 },
            RuleType::EmptyField,
            RuleType::RequiredFieldMissing,
            RuleType::PartialRecord,
            RuleType::CrossFieldConflict { fields: vec![] },
            RuleType::TypeMismatch,
            RuleType::UnitInconsistency,
            RuleType::DuplicateRecord,
            RuleType::ReferenceViolation,
            RuleType::FormatMismatch {
                expected: String::new(),
            },
            RuleType::PatternViolation {
                regex: String::new(),
            },
            RuleType::OutOfRange { min: 0.0, max: 0.0 },
            RuleType::EnumViolation { allowed: vec![] },
            RuleType::LengthViolation {
                min: 0,
                max: usize::MAX,
            },
            RuleType::OutlierNumeric { z_score: 3.0 },
            RuleType::DriftDetected {
                baseline: 0.0,
                threshold: 0.1,
            },
            RuleType::NegativeWherePositive,
            RuleType::SumDiscrepancy { expected: 0.0 },
            RuleType::TemporalInconsistency,
            RuleType::DuplicateKey,
            RuleType::NearDuplicate {
                similarity_threshold: 0.95,
            },
            RuleType::CompositeKeyViolation,
            RuleType::SurrogateKeyGap,
            RuleType::HashCollision,
            RuleType::OrphanReference,
            RuleType::CircularDependency,
            RuleType::LevelMismatch,
            RuleType::AggregateDiscrepancy,
            RuleType::ConstraintViolation {
                constraint: String::new(),
            },
            RuleType::CascadeFailure { depth: 3 },
        ];
        let mut trends = HashMap::new();
        for dim in &[
            QualityDimension::Completeness,
            QualityDimension::Consistency,
            QualityDimension::Conformity,
            QualityDimension::Accuracy,
            QualityDimension::Uniqueness,
            QualityDimension::Integrity,
        ] {
            trends.insert(*dim, QualityTrend::new(*dim));
        }
        Self {
            config,
            issues: Vec::with_capacity(1000),
            diagnostics: HashMap::new(),
            remediations: Vec::new(),
            snapshots: VecDeque::with_capacity(100),
            trends,
            next_id: 1,
            rule_registry,
            learned_patterns: Vec::new(),
            cycle_counter: 0,
        }
    }

    pub fn monitor(&mut self, records: &[HashMap<String, String>], source: &str) {
        self.cycle_counter += 1;
        if records.is_empty() {
            return;
        }
        let mut dimension_counts: HashMap<QualityDimension, usize> = HashMap::new();
        let start_len = self.issues.len();

        for record in records.iter().take(100) {
            for (field, value) in record {
                if value.trim().is_empty() {
                    self.issues.push(QualityIssue {
                        id: self.next_id,
                        dimension: QualityDimension::Completeness,
                        rule: RuleType::EmptyField,
                        severity: Severity::Medium(1),
                        description: format!("Empty field: {}", field),
                        source: source.to_string(),
                        detected_at: Instant::now(),
                        resolved: false,
                        resolution: None,
                    });
                    *dimension_counts
                        .entry(QualityDimension::Completeness)
                        .or_insert(0) += 1;
                    self.next_id += 1;
                    if self.issues.len() >= self.config.max_issues {
                        break;
                    }
                }
                if value.len() < 2 {
                    continue;
                }
                if let Ok(v) = value.parse::<f64>() {
                    if v < 0.0 {
                        self.issues.push(QualityIssue {
                            id: self.next_id,
                            dimension: QualityDimension::Accuracy,
                            rule: RuleType::NegativeWherePositive,
                            severity: Severity::Medium(1),
                            description: format!("Negative value for field '{}': {}", field, v),
                            source: source.to_string(),
                            detected_at: Instant::now(),
                            resolved: false,
                            resolution: None,
                        });
                        *dimension_counts
                            .entry(QualityDimension::Accuracy)
                            .or_insert(0) += 1;
                        self.next_id += 1;
                        if self.issues.len() >= self.config.max_issues {
                            break;
                        }
                    }
                }
            }
            if self.issues.len() >= self.config.max_issues {
                break;
            }
        }

        let new_issues = self.issues.len() - start_len;
        let total = records.len();
        let mut resolved_count = 0;
        let mut total_weight = 0.0;
        for issue in &self.issues {
            if issue.resolved {
                resolved_count += 1;
            }
            total_weight += issue.severity.weight();
        }
        let total_issues = self.issues.len();
        let score = if total_issues > 0 {
            resolved_count as f64 / total_issues.max(1) as f64
                * (1.0 - total_weight / total_issues.max(1) as f64).max(0.0)
        } else {
            1.0
        };

        let snapshot = MonitorSnapshot {
            timestamp: Instant::now(),
            total_records: total,
            issues_found: new_issues,
            dimension_breakdown: dimension_counts,
            overall_quality_score: score,
        };
        if self.snapshots.len() >= 100 {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);

        let dims = [
            QualityDimension::Completeness,
            QualityDimension::Consistency,
            QualityDimension::Conformity,
            QualityDimension::Accuracy,
            QualityDimension::Uniqueness,
            QualityDimension::Integrity,
        ];
        for dim in &dims {
            let dim_score = self.dimension_score(*dim);
            if let Some(trend) = self.trends.get_mut(dim) {
                trend.push(dim_score);
            }
        }
    }

    pub fn detect(&self, dimension: Option<QualityDimension>) -> Vec<&QualityIssue> {
        self.issues
            .iter()
            .filter(|issue| {
                if let Some(d) = dimension {
                    issue.dimension == d
                } else {
                    true
                }
            })
            .collect()
    }

    pub fn diagnose(&mut self, issue_id: u64) -> Option<DiagnosticReport> {
        let issue = self.issues.iter().find(|i| i.id == issue_id)?;
        let (root_cause, impact, suggested_fix) = match &issue.rule {
            RuleType::EmptyField | RuleType::MissingValue | RuleType::RequiredFieldMissing => (
                "Source data missing required values".to_string(),
                "Downstream processing may produce incomplete results".to_string(),
                "Populate missing fields from authoritative source or apply default".to_string(),
            ),
            RuleType::NullRate { max_pct } => (
                format!("Null rate exceeds threshold of {}%", max_pct * 100.0),
                "High null rate degrades statistical validity".to_string(),
                "Review source pipeline for data loss or apply imputation".to_string(),
            ),
            RuleType::OutOfRange { min, max } => (
                format!("Value outside expected range [{}, {}]", min, max),
                "Out-of-range values cause computation errors".to_string(),
                "Clip or reject values outside valid range".to_string(),
            ),
            RuleType::DuplicateKey | RuleType::DuplicateRecord => (
                "Duplicate entries in source data".to_string(),
                "Duplicates inflate counts and break referential integrity".to_string(),
                "Deduplicate by composite key before ingestion".to_string(),
            ),
            RuleType::OutlierNumeric { z_score } => (
                format!("Statistical outlier detected at z={}", z_score),
                "Outliers skew aggregate statistics and model training".to_string(),
                "Investigate source or apply winsorization".to_string(),
            ),
            RuleType::NegativeWherePositive => (
                "Negative value in field that expects non-negative".to_string(),
                "May indicate data corruption or unit conversion error".to_string(),
                "Set negative values to zero or flag for review".to_string(),
            ),
            _ => (
                "Unknown pattern detected".to_string(),
                "Potential data quality degradation".to_string(),
                "Review and classify for automated remediation".to_string(),
            ),
        };
        let report = DiagnosticReport {
            issue_id,
            root_cause,
            impact,
            suggested_fix,
            confidence: 0.75,
        };
        self.diagnostics.insert(issue_id, report.clone());
        Some(report)
    }

    pub fn remediate(&mut self, issue_id: u64, action: ActionType) -> bool {
        if let Some(issue) = self.issues.iter_mut().find(|i| i.id == issue_id) {
            issue.resolved = true;
            issue.resolution = Some(format!("{:?}", action));
        }
        let action_id = self.next_id;
        self.next_id += 1;
        self.remediations.push(RemediationAction {
            action_id,
            issue_id,
            action_type: action,
            applied: true,
            verified: false,
        });
        true
    }

    pub fn verify(&mut self, action_id: u64) -> bool {
        if let Some(action) = self
            .remediations
            .iter_mut()
            .find(|a| a.action_id == action_id)
        {
            action.verified = true;
            true
        } else {
            false
        }
    }

    pub fn learn(&mut self, rule: &RuleType, fix: &str, outcome: &str) {
        if !self.config.learning_enabled {
            return;
        }
        self.learned_patterns
            .push((rule.clone(), fix.to_string(), outcome.to_string()));
    }

    pub fn quality_score(&self) -> f64 {
        let total = self.issues.len();
        if total == 0 {
            return 1.0;
        }
        let resolved = self.issues.iter().filter(|i| i.resolved).count();
        let total_weight: f64 = self.issues.iter().map(|i| i.severity.weight()).sum();
        let resolved_weight: f64 = self
            .issues
            .iter()
            .filter(|i| i.resolved)
            .map(|i| i.severity.weight())
            .sum();
        let severity_factor = if total_weight > 0.0 {
            resolved_weight / total_weight
        } else {
            1.0
        };
        (resolved as f64 / total as f64) * 0.5 + severity_factor * 0.5
    }

    fn dimension_score(&self, dim: QualityDimension) -> f64 {
        let dim_issues: Vec<&QualityIssue> =
            self.issues.iter().filter(|i| i.dimension == dim).collect();
        let total = dim_issues.len();
        if total == 0 {
            return 1.0;
        }
        let resolved = dim_issues.iter().filter(|i| i.resolved).count();
        resolved as f64 / total as f64
    }

    pub fn trend(&self, dimension: QualityDimension) -> TrendDirection {
        self.trends
            .get(&dimension)
            .map(|t| t.direction.clone())
            .unwrap_or(TrendDirection::InsufficientData)
    }

    pub fn summary(&self) -> String {
        let dims = [
            QualityDimension::Completeness,
            QualityDimension::Consistency,
            QualityDimension::Conformity,
            QualityDimension::Accuracy,
            QualityDimension::Uniqueness,
            QualityDimension::Integrity,
        ];
        let mut lines = vec![
            format!("=== DataQualityPipeline Summary ==="),
            format!("Total issues: {}", self.issues.len()),
            format!(
                "Resolved: {}",
                self.issues.iter().filter(|i| i.resolved).count()
            ),
            format!("Overall quality score: {:.4}", self.quality_score()),
            format!("Remediations applied: {}", self.remediations.len()),
            format!("Learned patterns: {}", self.learned_patterns.len()),
            format!("Diagnostics cached: {}", self.diagnostics.len()),
            String::new(),
            format!("--- Per-Dimension Scores ---"),
        ];
        for dim in &dims {
            let score = self.dimension_score(*dim);
            let trend = self.trend(*dim);
            let count = self.issues.iter().filter(|i| i.dimension == *dim).count();
            lines.push(format!(
                "  {:16} score={:.4} trend={:?} issues={}",
                dim.to_string(),
                score,
                trend,
                count
            ));
        }
        if let Some(snap) = self.snapshots.back() {
            lines.push(String::new());
            lines.push(format!("--- Last Snapshot ---"));
            lines.push(format!("  Records: {}", snap.total_records));
            lines.push(format!("  Issues found: {}", snap.issues_found));
            lines.push(format!(
                "  Quality score: {:.4}",
                snap.overall_quality_score
            ));
        }
        lines.join("\n")
    }

    pub fn check_field(records: &[HashMap<String, String>], field: &str) -> (usize, usize) {
        let total = records.len();
        let missing = records
            .iter()
            .filter(|r| !r.contains_key(field) || r[field].trim().is_empty())
            .count();
        (missing, total)
    }

    pub fn check_duplicates(records: &[HashMap<String, String>], key_field: &str) -> Vec<usize> {
        let mut seen = HashMap::new();
        let mut dup_indices = Vec::new();
        for (i, rec) in records.iter().enumerate() {
            if let Some(val) = rec.get(key_field) {
                if seen.contains_key(val) {
                    dup_indices.push(i);
                } else {
                    seen.insert(val.clone(), i);
                }
            }
        }
        dup_indices
    }

    pub fn check_numeric_outliers(values: &[f64], z_threshold: f64) -> Vec<(usize, f64)> {
        if values.is_empty() {
            return vec![];
        }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        let stddev = variance.sqrt();
        if stddev < 1e-12 {
            return vec![];
        }
        values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let z = (v - mean).abs() / stddev;
                if z > z_threshold {
                    Some((i, z))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pipeline() {
        let config = DQConfig::default();
        let pipeline = DataQualityPipeline::new(config);
        assert_eq!(pipeline.issues.len(), 0);
        assert_eq!(pipeline.snapshots.len(), 0);
        assert_eq!(pipeline.next_id, 1);
        assert_eq!(pipeline.rule_registry.len(), 31);
        assert_eq!(pipeline.trends.len(), 6);
    }

    #[test]
    fn test_monitor_adds_issues() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![
            HashMap::from([
                ("name".to_string(), "Alice".to_string()),
                ("age".to_string(), "-5".to_string()),
            ]),
            HashMap::from([
                ("name".to_string(), "".to_string()),
                ("age".to_string(), "30".to_string()),
            ]),
        ];
        pipeline.monitor(&rec, "test_source");
        assert!(pipeline.issues.len() >= 2);
    }

    #[test]
    fn test_monitor_creates_snapshot() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([
            ("name".to_string(), "Bob".to_string()),
            ("age".to_string(), "25".to_string()),
        ])];
        pipeline.monitor(&rec, "src");
        assert_eq!(pipeline.snapshots.len(), 1);
        let snap = pipeline.snapshots.back().unwrap();
        assert_eq!(snap.total_records, 1);
    }

    #[test]
    fn test_detect_filters_by_dimension() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([
            ("name".to_string(), "".to_string()),
            ("age".to_string(), "-1".to_string()),
        ])];
        pipeline.monitor(&rec, "src");
        let completeness_issues = pipeline.detect(Some(QualityDimension::Completeness));
        let accuracy_issues = pipeline.detect(Some(QualityDimension::Accuracy));
        assert!(!completeness_issues.is_empty());
        assert!(!accuracy_issues.is_empty());
        let all = pipeline.detect(None);
        assert_eq!(all.len(), pipeline.issues.len());
    }

    #[test]
    fn test_diagnose_generates_report() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([("name".to_string(), "".to_string())])];
        pipeline.monitor(&rec, "src");
        let issue_id = pipeline.issues[0].id;
        let report = pipeline.diagnose(issue_id);
        assert!(report.is_some());
        let r = report.unwrap();
        assert_eq!(r.issue_id, issue_id);
        assert!(r.confidence > 0.0);
        assert!(r.confidence <= 1.0);
    }

    #[test]
    fn test_remediate_and_verify() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([("name".to_string(), "".to_string())])];
        pipeline.monitor(&rec, "src");
        let issue_id = pipeline.issues[0].id;
        assert!(!pipeline.issues[0].resolved);
        let ok = pipeline.remediate(issue_id, ActionType::FlagForReview);
        assert!(ok);
        assert!(pipeline.issues[0].resolved);
        assert_eq!(pipeline.remediations.len(), 1);
        let action_id = pipeline.remediations[0].action_id;
        assert!(pipeline.verify(action_id));
        assert!(pipeline.remediations[0].verified);
    }

    #[test]
    fn test_learn_adds_pattern() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        pipeline.learn(&RuleType::EmptyField, "fill_with_default", "success");
        assert_eq!(pipeline.learned_patterns.len(), 1);
        assert_eq!(pipeline.learned_patterns[0].1, "fill_with_default");

        let mut no_learn = DataQualityPipeline::new(DQConfig {
            learning_enabled: false,
            ..Default::default()
        });
        no_learn.learn(&RuleType::EmptyField, "fill", "ok");
        assert_eq!(no_learn.learned_patterns.len(), 0);
    }

    #[test]
    fn test_quality_score_perfect() {
        let pipeline = DataQualityPipeline::new(DQConfig::default());
        assert!((pipeline.quality_score() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_quality_score_with_issues() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([
            ("name".to_string(), "".to_string()),
            ("age".to_string(), "-1".to_string()),
        ])];
        pipeline.monitor(&rec, "src");
        let score = pipeline.quality_score();
        assert!(score < 1.0);
        assert!(score >= 0.0);
    }

    #[test]
    fn test_trend_insufficient_data() {
        let pipeline = DataQualityPipeline::new(DQConfig::default());
        let t = pipeline.trend(QualityDimension::Accuracy);
        assert_eq!(t, TrendDirection::InsufficientData);
    }

    #[test]
    fn test_trend_with_scores() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        if let Some(trend) = pipeline.trends.get_mut(&QualityDimension::Accuracy) {
            trend.push(0.5);
            trend.push(0.6);
            trend.push(0.7);
            trend.push(0.8);
        }
        let t = pipeline.trend(QualityDimension::Accuracy);
        assert!(t == TrendDirection::Improving || t == TrendDirection::InsufficientData);
    }

    #[test]
    fn test_summary_contains_info() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![HashMap::from([("name".to_string(), "".to_string())])];
        pipeline.monitor(&rec, "src");
        let s = pipeline.summary();
        assert!(s.contains("DataQualityPipeline"));
        assert!(s.contains("Total issues"));
        assert!(s.contains("Completeness"));
    }

    #[test]
    fn test_check_field_missing() {
        let records = vec![
            HashMap::from([("name".to_string(), "Alice".to_string())]),
            HashMap::from([("name".to_string(), "".to_string())]),
            HashMap::new(),
        ];
        let (missing, total) = DataQualityPipeline::check_field(&records, "name");
        assert_eq!(missing, 2);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_check_field_complete() {
        let records = vec![
            HashMap::from([("a".to_string(), "1".to_string())]),
            HashMap::from([("a".to_string(), "2".to_string())]),
        ];
        let (missing, total) = DataQualityPipeline::check_field(&records, "a");
        assert_eq!(missing, 0);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_check_duplicates() {
        let records = vec![
            HashMap::from([("id".to_string(), "1".to_string())]),
            HashMap::from([("id".to_string(), "2".to_string())]),
            HashMap::from([("id".to_string(), "1".to_string())]),
            HashMap::from([("id".to_string(), "3".to_string())]),
        ];
        let dups = DataQualityPipeline::check_duplicates(&records, "id");
        assert_eq!(dups, vec![2]);
    }

    #[test]
    fn test_check_numeric_outliers() {
        let vals = vec![10.0, 10.5, 9.8, 10.2, 100.0, 10.1, 9.9];
        let outliers = DataQualityPipeline::check_numeric_outliers(&vals, 3.0);
        assert!(!outliers.is_empty());
        let (idx, _) = outliers[0];
        assert_eq!(idx, 4);
    }

    #[test]
    fn test_quality_score_resolved_improves() {
        let mut pipeline = DataQualityPipeline::new(DQConfig::default());
        let rec = vec![
            HashMap::from([("name".to_string(), "".to_string())]),
            HashMap::from([("name".to_string(), "".to_string())]),
        ];
        pipeline.monitor(&rec, "src");
        let before = pipeline.quality_score();
        let issue_id = pipeline.issues[0].id;
        pipeline.remediate(
            issue_id,
            ActionType::AutoCorrect {
                old_value: "".to_string(),
                new_value: "unknown".to_string(),
            },
        );
        let after = pipeline.quality_score();
        assert!(after >= before);
    }
}
