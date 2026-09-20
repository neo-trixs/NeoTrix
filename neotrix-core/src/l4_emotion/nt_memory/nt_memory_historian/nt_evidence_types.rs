use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfidenceTier {
    T1,
    T2,
    T3,
    T4,
    T5,
}

/// scansci-pi 证据优先门 — 证据不足时显式声明, 而非静默产出置信度。
/// 每个声明必须能追溯其依据 (reasons), 无依据即不通过 (不编造)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EvidenceSufficiency {
    /// 证据充分, 可作断言依据
    Sufficient,
    /// 证据不足, 显式声明并列出缺口
    Insufficient { reasons: Vec<String> },
}

impl EvidenceSufficiency {
    pub fn is_sufficient(&self) -> bool {
        matches!(self, EvidenceSufficiency::Sufficient)
    }

    pub fn reasons(&self) -> &[String] {
        match self {
            EvidenceSufficiency::Sufficient => &[],
            EvidenceSufficiency::Insufficient { reasons } => reasons,
        }
    }
}

impl ConfidenceTier {
    pub fn label(&self) -> &str {
        match self {
            ConfidenceTier::T1 => "T1 双重确认",
            ConfidenceTier::T2 => "T2 单源确认",
            ConfidenceTier::T3 => "T3 初步可考",
            ConfidenceTier::T4 => "T4 存疑",
            ConfidenceTier::T5 => "T5 判定伪造",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            ConfidenceTier::T1 => "#4AE86A",
            ConfidenceTier::T2 => "#4A8AFF",
            ConfidenceTier::T3 => "#E8C84A",
            ConfidenceTier::T4 => "#E8884A",
            ConfidenceTier::T5 => "#E84A4A",
        }
    }

    pub fn from_score(score: f64, forgery_risk: f64) -> Self {
        if forgery_risk > 0.8 {
            return ConfidenceTier::T5;
        }
        match score {
            s if s >= 0.75 => ConfidenceTier::T1,
            s if s >= 0.55 => ConfidenceTier::T2,
            s if s >= 0.35 => ConfidenceTier::T3,
            s if s >= 0.15 => ConfidenceTier::T4,
            _ => ConfidenceTier::T5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatingMethodMeta {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub range_min_bp: u64,
    pub range_max_bp: u64,
}

pub fn builtin_dating_methods() -> Vec<DatingMethodMeta> {
    vec![
        DatingMethodMeta { code: "c14".into(), name: "C14 (AMS)".into(), weight: 0.90, range_min_bp: 0, range_max_bp: 50000 },
        DatingMethodMeta { code: "tl".into(), name: "热释光 (TL)".into(), weight: 0.75, range_min_bp: 0, range_max_bp: 500000 },
        DatingMethodMeta { code: "osl".into(), name: "光释光 (OSL)".into(), weight: 0.80, range_min_bp: 0, range_max_bp: 200000 },
        DatingMethodMeta { code: "paleomag".into(), name: "古地磁".into(), weight: 0.60, range_min_bp: 0, range_max_bp: 5_000_000 },
        DatingMethodMeta { code: "esr".into(), name: "电子自旋共振 (ESR)".into(), weight: 0.65, range_min_bp: 0, range_max_bp: 5_000_000 },
        DatingMethodMeta { code: "useries".into(), name: "铀系法".into(), weight: 0.85, range_min_bp: 1000, range_max_bp: 500000 },
        DatingMethodMeta { code: "dendro".into(), name: "树轮校正".into(), weight: 1.00, range_min_bp: 0, range_max_bp: 12000 },
        DatingMethodMeta { code: "strat".into(), name: "地层层位".into(), weight: 0.50, range_min_bp: 0, range_max_bp: 5_000_000 },
        DatingMethodMeta { code: "dna".into(), name: "古DNA".into(), weight: 0.85, range_min_bp: 0, range_max_bp: 1_000_000 },
        DatingMethodMeta { code: "gcms".into(), name: "GC-MS化学分析".into(), weight: 0.85, range_min_bp: 0, range_max_bp: 50000 },
        DatingMethodMeta { code: "inaa".into(), name: "INAA中子活化".into(), weight: 0.85, range_min_bp: 0, range_max_bp: 50000 },
        DatingMethodMeta { code: "srs".into(), name: "同步辐射 (SRS)".into(), weight: 0.90, range_min_bp: 0, range_max_bp: 50000 },
        DatingMethodMeta { code: "lithic".into(), name: "石器技术分析".into(), weight: 0.60, range_min_bp: 0, range_max_bp: 3_000_000 },
        DatingMethodMeta { code: "glyph".into(), name: "字体/铭文分析".into(), weight: 0.55, range_min_bp: 0, range_max_bp: 6000 },
        DatingMethodMeta { code: "archaeoastronomy".into(), name: "天文考古".into(), weight: 0.60, range_min_bp: 0, range_max_bp: 12000 },
    ]
}

pub fn method_weight(code: &str) -> f64 {
    builtin_dating_methods().into_iter().find(|m| m.code == code).map(|m| m.weight).unwrap_or(0.5)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForgeryRisk {
    pub provenance_gap: f64,
    pub anachronism_index: f64,
    pub motivation_score: f64,
    pub verification_gap: f64,
}

impl ForgeryRisk {
    pub fn new() -> Self {
        Self { provenance_gap: 0.0, anachronism_index: 0.0, motivation_score: 0.0, verification_gap: 0.0 }
    }

    pub fn total(&self) -> f64 {
        (self.provenance_gap + self.anachronism_index + self.motivation_score + self.verification_gap) / 4.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub era: String,
    pub category: String,
    pub description: String,
    pub dating_methods: Vec<String>,
    pub context_clarity: f64,
    pub publication_level: f64,
    pub independent_replications: u32,
    pub provenance_gap: f64,
    pub anachronism_index: f64,
    pub motivation_score: f64,
    pub verification_gap: f64,
    pub references: String,
    pub connections: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl EvidenceRecord {
    pub fn forgery_risk(&self) -> ForgeryRisk {
        ForgeryRisk {
            provenance_gap: self.provenance_gap,
            anachronism_index: self.anachronism_index,
            motivation_score: self.motivation_score,
            verification_gap: self.verification_gap,
        }
    }

    pub fn raw_confidence(&self) -> f64 {
        let methods = &self.dating_methods;
        if methods.is_empty() {
            return 0.2;
        }
        let total_weight: f64 = methods.iter().map(|m| method_weight(m)).sum();
        let n = methods.len() as f64;
        if n == 0.0 {
            return 0.2;
        }
        let base = total_weight / n;
        let ctx = self.context_clarity.max(0.0);
        let pub_level = self.publication_level.max(0.0);
        let indep = (self.independent_replications as f64 * 0.3).min(1.0);
        base * ctx * (pub_level + 0.3) * (0.5 + indep)
    }

    pub fn effective_confidence(&self) -> f64 {
        let raw = self.raw_confidence();
        let risk = self.forgery_risk().total();
        raw * (1.0 - risk * 0.5)
    }

    pub fn tier(&self) -> ConfidenceTier {
        ConfidenceTier::from_score(self.effective_confidence(), self.forgery_risk().total())
    }

    /// scansci-pi 证据优先门: 显式判断证据是否足以支撑断言。
    /// 证据不足时必须返回 Insufficient 并列出缺口 (不编造、不静默通过)。
    pub fn sufficiency(&self) -> EvidenceSufficiency {
        let mut reasons: Vec<String> = Vec::new();
        if self.dating_methods.is_empty() {
            reasons.push("无任何定年方法 (dating_methods empty)".into());
        }
        if self.provenance_gap > 0.7 {
            reasons.push(format!("溯源缺口过大 (provenance_gap={:.2})", self.provenance_gap));
        }
        if self.verification_gap > 0.7 {
            reasons.push(format!("核验缺口过大 (verification_gap={:.2})", self.verification_gap));
        }
        if self.independent_replications == 0 {
            reasons.push("无独立复现 (independent_replications=0)".into());
        }
        if self.effective_confidence() < 0.35 {
            reasons.push(format!("有效置信度过低 (effective_confidence={:.2})", self.effective_confidence()));
        }
        if reasons.is_empty() {
            EvidenceSufficiency::Sufficient
        } else {
            EvidenceSufficiency::Insufficient { reasons }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianLink {
    pub from: String,
    pub to: String,
    pub probability: f64,
    pub distance_km: f64,
    pub temporal_overlap: f64,
    pub shared_dating_methods: usize,
    pub same_category: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCluster {
    pub id: String,
    pub members: Vec<String>,
    pub member_count: usize,
    pub avg_confidence: f64,
    pub internal_links: usize,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationResult {
    pub evidence_count: usize,
    pub links_found: usize,
    pub clusters_found: usize,
    pub tier_changes: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    pub total: usize,
    pub t1_count: usize,
    pub t2_count: usize,
    pub t3_count: usize,
    pub t4_count: usize,
    pub t5_count: usize,
    pub links: usize,
    pub clusters: usize,
}

/// Contradiction between two evidence records identified during arbitration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceContradiction {
    pub evidence_a_id: String,
    pub evidence_b_id: String,
    pub category: ContradictionCategory,
    pub severity: f64,
    pub description: String,
    pub resolution: Option<ConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContradictionCategory {
    Spatial,      // different locations for same claimed event
    Temporal,     // incompatible eras
    Method,       // same method gives different results
    Attribution,  // same artifact attributed to different sources
    Provenance,   // conflicting chain of custody
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub favored_id: String,
    pub reason: String,
    pub new_score: f64,
}

/// Snapshot of the full evidence table for serialization/checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTableSnapshot {
    pub version: u32,
    pub timestamp: i64,
    pub records: Vec<EvidenceRecord>,
    pub links: Vec<BayesianLink>,
    pub clusters: Vec<EvidenceCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryGateConfig {
    pub min_confidence: f64,
    pub require_peer_review: bool,
    pub max_contradictions_allowed: usize,
    pub require_method_replication: bool,
}

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    r * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

fn parse_era_center(era: &str) -> f64 {
    let s = era.replace(',', "");
    if s.contains("BP") {
        return s.split_whitespace().next().and_then(|w| w.parse::<f64>().ok()).unwrap_or(0.0);
    }
    if s.contains("CE") {
        let n: f64 = s.split_whitespace().next().and_then(|w| w.parse().ok()).unwrap_or(0.0);
        return 2026.0 - n;
    }
    if s.contains("BCE") {
        let n: f64 = s.split_whitespace().next().and_then(|w| w.parse().ok()).unwrap_or(0.0);
        return 2026.0 + n;
    }
    0.0
}

pub fn era_center(era: &str) -> f64 {
    let c = parse_era_center(era);
    if c.is_nan() || c.is_infinite() { 0.0 } else { c }
}
