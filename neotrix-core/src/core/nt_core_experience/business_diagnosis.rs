#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosisPhase {
    Explore,
    Analyze,
    Recommend,
    Validate,
}

#[derive(Debug, Clone)]
pub struct KnowledgeAtom {
    pub id: String,
    pub domain: String,
    pub question: String,
    pub insight: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct DecisionState {
    pub id: u64,
    pub phase: DiagnosisPhase,
    pub context: String,
    pub chosen_path: Option<String>,
    pub alternatives: Vec<String>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct BusinessDiagnosisStats {
    pub total_diagnoses: u64,
    pub atoms_used: u64,
    pub decisions_made: u64,
    pub avg_decision_depth: f64,
    pub per_phase: HashMap<String, u64>,
}

impl Default for BusinessDiagnosisStats {
    fn default() -> Self {
        Self {
            total_diagnoses: 0,
            atoms_used: 0,
            decisions_made: 0,
            avg_decision_depth: 0.0,
            per_phase: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosisResult {
    pub problem_statement: String,
    pub findings: Vec<String>,
    pub root_causes: Vec<String>,
    pub recommendations: Vec<String>,
    pub decision_path: Vec<DecisionState>,
    pub confidence: f64,
}

pub struct BusinessDiagnosisEngine {
    atoms: Vec<KnowledgeAtom>,
    stats: BusinessDiagnosisStats,
    next_id: u64,
}

impl BusinessDiagnosisEngine {
    pub fn new() -> Self {
        Self {
            atoms: Self::default_atoms(),
            stats: BusinessDiagnosisStats::default(),
            next_id: 1,
        }
    }

    fn default_atoms() -> Vec<KnowledgeAtom> {
        vec![
            KnowledgeAtom { id: "fin_001".into(), domain: "financial".into(), question: "Is revenue growing or declining?".into(), insight: "Revenue trend reveals market position and pricing power. Declining revenue signals competitive pressure or product-market drift.".into(), confidence: 0.85 },
            KnowledgeAtom { id: "fin_002".into(), domain: "financial".into(), question: "Are margins expanding or contracting?".into(), insight: "Margin trends indicate operational efficiency. Margin compression often precedes cash flow crisis.".into(), confidence: 0.82 },
            KnowledgeAtom { id: "fin_003".into(), domain: "financial".into(), question: "Is cash flow positive?".into(), insight: "Cash flow is the lifeblood. Profitable companies can fail on cash flow alone.".into(), confidence: 0.90 },
            KnowledgeAtom { id: "fin_004".into(), domain: "financial".into(), question: "What is the debt-to-equity ratio?".into(), insight: "Leverage amplifies returns in good times and magnifies losses in downturns.".into(), confidence: 0.78 },
            KnowledgeAtom { id: "fin_005".into(), domain: "financial".into(), question: "How concentrated is revenue by customer?".into(), insight: "Customer concentration above 30% for a single client is a key-person risk for the business.".into(), confidence: 0.75 },
            KnowledgeAtom { id: "ops_001".into(), domain: "operational".into(), question: "Is capacity utilization optimal?".into(), insight: "Below 60% utilization suggests fixed cost inefficiency. Above 95% risks quality degradation.".into(), confidence: 0.80 },
            KnowledgeAtom { id: "ops_002".into(), domain: "operational".into(), question: "What is the unit economics breakdown?".into(), insight: "CAC-to-LTV ratio above 3:1 is unhealthy. Payback period > 18 months strains growth capital.".into(), confidence: 0.85 },
            KnowledgeAtom { id: "ops_003".into(), domain: "operational".into(), question: "How efficient is the supply chain?".into(), insight: "Inventory turnover under 4/year suggests overstock. Over 12/year risks stockouts.".into(), confidence: 0.77 },
            KnowledgeAtom { id: "mkt_001".into(), domain: "market".into(), question: "Is market share growing or eroding?".into(), insight: "Share loss in a growing market is more dangerous than share loss in a declining market.".into(), confidence: 0.83 },
            KnowledgeAtom { id: "mkt_002".into(), domain: "market".into(), question: "What is the competitive moat?".into(), insight: "Sustainable advantages come from network effects, switching costs, scale, or brand.".into(), confidence: 0.88 },
            KnowledgeAtom { id: "mkt_003".into(), domain: "market".into(), question: "How differentiated is the product?".into(), insight: "Commodity products compete on price. Differentiated products compete on value.".into(), confidence: 0.81 },
            KnowledgeAtom { id: "str_001".into(), domain: "strategic".into(), question: "Is the business model aligned with market trends?".into(), insight: "The best execution of a bad strategy still fails. Alignment with macro trends is foundational.".into(), confidence: 0.86 },
            KnowledgeAtom { id: "str_002".into(), domain: "strategic".into(), question: "What is the key risk in the next 12 months?".into(), insight: "Single-point-of-failure identification is the highest-leverage diagnostic question.".into(), confidence: 0.79 },
            KnowledgeAtom { id: "str_003".into(), domain: "strategic".into(), question: "Is there a clear decision-making framework?".into(), insight: "Organizations without clear decision rights make slow or contradictory choices.".into(), confidence: 0.84 },
        ]
    }

    pub fn diagnose(&mut self, problem: &str, context: &str) -> DiagnosisResult {
        self.stats.total_diagnoses += 1;
        let mut findings = Vec::new();
        let mut root_causes = Vec::new();
        let mut recommendations = Vec::new();
        let mut decision_path = Vec::new();
        let mut atoms_used = Vec::new();

        let d1 = self.enter_decision(problem, DiagnosisPhase::Explore);
        findings.push(format!("Initial assessment: {}", problem));
        for atom in &self.atoms {
            if context.contains(&atom.domain) || problem.contains(&atom.domain) {
                findings.push(format!(
                    "[{}] {} — {}",
                    atom.id, atom.question, atom.insight
                ));
                atoms_used.push(atom.id.clone());
            }
        }
        let d1_complete = DecisionState {
            id: d1.id,
            phase: d1.phase,
            context: problem.to_string(),
            chosen_path: Some("analyze".into()),
            alternatives: vec!["explore_deeper".into(), "recommend_immediately".into()],
            reasoning: format!("Explored {} relevant knowledge atoms", atoms_used.len()),
        };
        decision_path.push(d1_complete);

        let d2 = self.enter_decision(
            &format!("root_cause for: {}", problem),
            DiagnosisPhase::Analyze,
        );
        if findings.len() > 3 {
            root_causes.push(findings[0].clone());
            root_causes.push(format!(
                "Pattern: {} related findings suggest systemic issue",
                findings.len()
            ));
        } else {
            root_causes.push("Insufficient data to determine root cause".into());
        }
        let d2_complete = DecisionState {
            id: d2.id,
            phase: d2.phase,
            context: format!("findings={}", findings.len()),
            chosen_path: Some("identify".into()),
            alternatives: vec!["deep_analysis".into()],
            reasoning: format!("Identified {} root cause(s)", root_causes.len()),
        };
        decision_path.push(d2_complete);

        for rc in &root_causes {
            recommendations.push(format!("Address: {}. Recommended: structured mitigation plan with 30/60/90 day milestones.", rc));
        }
        recommendations.push("Set up monitoring KPIs for the top 3 identified risks.".into());

        let d3 = self.enter_decision(
            &format!("recommendations for: {}", problem),
            DiagnosisPhase::Recommend,
        );
        let d3_complete = DecisionState {
            id: d3.id,
            phase: d3.phase,
            context: format!("{} recommendations", recommendations.len()),
            chosen_path: Some("present".into()),
            alternatives: vec!["validate".into()],
            reasoning: format!(
                "Generated {} recommendations from {} findings",
                recommendations.len(),
                findings.len()
            ),
        };
        decision_path.push(d3_complete);

        self.stats.atoms_used += atoms_used.len() as u64;
        self.stats.decisions_made += decision_path.len() as u64;
        *self.stats.per_phase.entry("explore".into()).or_insert(0) += 1;
        *self.stats.per_phase.entry("analyze".into()).or_insert(0) += 1;
        *self.stats.per_phase.entry("recommend".into()).or_insert(0) += 1;

        DiagnosisResult {
            problem_statement: problem.to_string(),
            findings,
            root_causes,
            recommendations,
            decision_path,
            confidence: 0.72 + (atoms_used.len() as f64 * 0.02).min(0.20),
        }
    }

    fn enter_decision(&mut self, context: &str, phase: DiagnosisPhase) -> DecisionState {
        let id = self.next_id;
        self.next_id += 1;
        DecisionState {
            id,
            phase,
            context: context.to_string(),
            chosen_path: None,
            alternatives: Vec::new(),
            reasoning: String::new(),
        }
    }

    pub fn add_atom(&mut self, atom: KnowledgeAtom) {
        self.atoms.push(atom);
    }

    pub fn stats(&self) -> &BusinessDiagnosisStats {
        &self.stats
    }

    pub fn tick(&mut self, input: Option<(&str, &str)>) -> String {
        match input {
            Some((problem, context)) => {
                let result = self.diagnose(problem, context);
                format!(
                    "business_diagnosis:tick=diagnosed_findings={}_recommendations={}_confidence={:.2}",
                    result.findings.len(),
                    result.recommendations.len(),
                    result.confidence
                )
            }
            None => {
                format!(
                    "business_diagnosis:tick=idle_total={}_atoms={}_decisions={}",
                    self.stats.total_diagnoses,
                    self.atoms.len(),
                    self.stats.decisions_made
                )
            }
        }
    }
}

impl Default for BusinessDiagnosisEngine {
    fn default() -> Self {
        Self::new()
    }
}
