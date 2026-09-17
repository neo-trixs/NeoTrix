#![forbid(unsafe_code)]

//! # TradeIntelligence — 贸易智能分析引擎
//!
//! 集成客户意图识别、商机评估、风险预警与市场分析的统一智能层。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ─── Enums ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    Inquiry,
    PriceComparison,
    BulkOrder,
    SampleRequest,
    Partnership,
    Complaint,
    Information,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Immediate,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpportunityStage {
    Lead,
    Qualified,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskType {
    Payment,
    Quality,
    Delivery,
    Compliance,
    Market,
    Reputation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seasonality {
    Peak,
    OffPeak,
    Normal,
}

// ─── Input Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerData {
    pub customer_id: String,
    pub name: String,
    pub country: String,
    pub industry: String,
    pub grade: String,
    pub source: String,
    pub total_value: f64,
    pub interaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub content: String,
    pub sentiment: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    pub region: String,
    pub industry_trend: f32,
    pub competitor_activity: f32,
    pub exchange_rate: f32,
    pub seasonality: Seasonality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub policy_type: String,
    pub rules: Vec<String>,
    pub exceptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceInput {
    pub customer_data: CustomerData,
    pub interaction_history: Vec<InteractionEvent>,
    pub market_context: MarketContext,
    pub company_policies: Vec<Policy>,
}

// ─── Output Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIntent {
    pub primary_intent: IntentType,
    pub secondary_intents: Vec<IntentType>,
    pub urgency: UrgencyLevel,
    pub budget_range: Option<BudgetRange>,
    pub decision_timeline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentFactor {
    pub factor: String,
    pub impact: f32,
    pub weight: f32,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityAssessment {
    pub score: f32,
    pub stage: OpportunityStage,
    pub estimated_value: f64,
    pub conversion_probability: f32,
    pub time_to_close: Option<u32>,
    pub factors: Vec<AssessmentFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub risk_type: RiskType,
    pub severity: RiskSeverity,
    pub description: String,
    pub mitigation: String,
    pub probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub priority: Priority,
    pub expected_outcome: String,
    pub deadline: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceOutput {
    pub intent: CustomerIntent,
    pub opportunity: OpportunityAssessment,
    pub risks: Vec<RiskAlert>,
    pub recommendations: Vec<Recommendation>,
    pub confidence: f32,
}

// ─── Configuration ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    pub payment_risk: f32,
    pub delivery_risk: f32,
    pub compliance_risk: f32,
    pub market_risk: f32,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            payment_risk: 0.6,
            delivery_risk: 0.5,
            compliance_risk: 0.7,
            market_risk: 0.5,
        }
    }
}

// ─── Model Trait ──────────────────────────────────────────────────────────

pub trait IntelligenceModel: Send + Sync {
    fn model_type(&self) -> &str;
    fn predict(&self, input: &IntelligenceInput) -> IntelligenceOutput;
    fn confidence(&self) -> f32;
}

// ─── Stats ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct IntelligenceStats {
    pub total_analyses: u64,
    pub avg_confidence: f32,
    pub model_count: usize,
    pub high_risk_detections: u64,
    pub opportunity_closures: u64,
}

// ─── TradeIntelligence Engine ─────────────────────────────────────────────

pub struct TradeIntelligence {
    models: HashMap<String, Box<dyn IntelligenceModel>>,
    risk_thresholds: RiskThresholds,
    stats: IntelligenceStats,
}

impl TradeIntelligence {
    /// Create new intelligence engine
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            risk_thresholds: RiskThresholds::default(),
            stats: IntelligenceStats::default(),
        }
    }

    /// Register an intelligence model
    pub fn register_model(&mut self, name: String, model: Box<dyn IntelligenceModel>) {
        self.models.insert(name, model);
    }

    /// Update risk thresholds
    pub fn update_thresholds(&mut self, thresholds: RiskThresholds) {
        self.risk_thresholds = thresholds;
    }

    /// Analyze customer intent
    pub fn analyze_intent(&self, input: &IntelligenceInput) -> CustomerIntent {
        let cd = &input.customer_data;
        let history = &input.interaction_history;

        // Heuristic-based intent detection from interaction history
        let (primary, secondary) = self.classify_intent_from_history(history);

        let urgency = self.assess_urgency(input);
        let budget_range = self.estimate_budget(cd);
        let decision_timeline = self.estimate_timeline(input);

        CustomerIntent {
            primary_intent: primary,
            secondary_intents: secondary,
            urgency,
            budget_range,
            decision_timeline,
        }
    }

    /// Assess opportunity
    pub fn assess_opportunity(&self, input: &IntelligenceInput) -> OpportunityAssessment {
        let cd = &input.customer_data;
        let mc = &input.market_context;

        let mut factors = Vec::new();

        // Customer grade factor
        let (grade_impact, grade_weight) = match cd.grade.as_str() {
            "A" => (0.8, 0.3),
            "B" => (0.5, 0.25),
            "C" => (0.2, 0.2),
            _ => (0.0, 0.15),
        };
        factors.push(AssessmentFactor {
            factor: "customer_grade".to_string(),
            impact: grade_impact,
            weight: grade_weight,
            explanation: format!(
                "Customer grade {} contributes {} to opportunity",
                cd.grade, grade_impact
            ),
        });

        // Industry trend factor
        let trend_impact = mc.industry_trend.clamp(-1.0, 1.0);
        factors.push(AssessmentFactor {
            factor: "industry_trend".to_string(),
            impact: trend_impact,
            weight: 0.2,
            explanation: format!("Industry trend in {} is {:.1}", mc.region, trend_impact),
        });

        // Interaction depth factor
        let depth_impact = if cd.interaction_count > 10 {
            0.7
        } else if cd.interaction_count > 5 {
            0.4
        } else {
            0.1
        };
        factors.push(AssessmentFactor {
            factor: "interaction_depth".to_string(),
            impact: depth_impact,
            weight: 0.2,
            explanation: format!(
                "{} interactions indicate {} engagement",
                cd.interaction_count,
                if depth_impact > 0.5 {
                    "deep"
                } else if depth_impact > 0.3 {
                    "moderate"
                } else {
                    "shallow"
                }
            ),
        });

        // Seasonality factor
        let season_impact = match mc.seasonality {
            Seasonality::Peak => 0.6,
            Seasonality::Normal => 0.0,
            Seasonality::OffPeak => -0.4,
        };
        factors.push(AssessmentFactor {
            factor: "seasonality".to_string(),
            impact: season_impact,
            weight: 0.15,
            explanation: format!(
                "Seasonality: {:?} with impact {:.1}",
                mc.seasonality, season_impact
            ),
        });

        // Competitor activity factor
        let comp_impact = -mc.competitor_activity.clamp(-1.0, 1.0);
        factors.push(AssessmentFactor {
            factor: "competitor_activity".to_string(),
            impact: comp_impact,
            weight: 0.15,
            explanation: format!(
                "Competitor activity level {:.1} reduces opportunity",
                mc.competitor_activity
            ),
        });

        // Calculate weighted score (0.0–100.0)
        let raw_score: f32 = factors.iter().map(|f| f.impact * f.weight).sum();
        let score = ((raw_score + 1.0) / 2.0 * 100.0).clamp(0.0, 100.0);

        let stage = self.determine_stage(cd, &input.interaction_history);
        let conversion_probability = (score / 100.0).clamp(0.05, 0.95);
        let estimated_value = cd.total_value * conversion_probability as f64;
        let time_to_close = Some(match stage {
            OpportunityStage::Lead => 90,
            OpportunityStage::Qualified => 60,
            OpportunityStage::Proposal => 30,
            OpportunityStage::Negotiation => 14,
            OpportunityStage::ClosedWon | OpportunityStage::ClosedLost => 0,
        });

        OpportunityAssessment {
            score,
            stage,
            estimated_value,
            conversion_probability,
            time_to_close,
            factors,
        }
    }

    /// Detect risks
    pub fn detect_risks(&self, input: &IntelligenceInput) -> Vec<RiskAlert> {
        let mut alerts = Vec::new();
        let cd = &input.customer_data;
        let mc = &input.market_context;
        let th = &self.risk_thresholds;

        // Payment risk
        let payment_score = if cd.total_value > 100_000.0 {
            0.8
        } else if cd.total_value > 50_000.0 {
            0.5
        } else {
            0.2
        };
        if payment_score > th.payment_risk {
            alerts.push(RiskAlert {
                risk_type: RiskType::Payment,
                severity: Self::score_to_severity(payment_score),
                description: format!(
                    "High-value customer (${:.0}) exceeds payment risk threshold",
                    cd.total_value
                ),
                mitigation: "Request advance payment or letter of credit".to_string(),
                probability: payment_score,
            });
        }

        // Delivery risk based on region
        let delivery_score = if mc.region.to_lowercase().contains("africa")
            || mc.region.to_lowercase().contains("middle east")
        {
            0.7
        } else if mc.region.to_lowercase().contains("south america") {
            0.5
        } else {
            0.2
        };
        if delivery_score > th.delivery_risk {
            alerts.push(RiskAlert {
                risk_type: RiskType::Delivery,
                severity: Self::score_to_severity(delivery_score),
                description: format!("Region '{}' has elevated delivery risk", mc.region),
                mitigation: "Use established shipping routes with insurance".to_string(),
                probability: delivery_score,
            });
        }

        // Market risk from exchange rate volatility
        let market_score = mc.exchange_rate.abs().min(1.0);
        if market_score > th.market_risk {
            alerts.push(RiskAlert {
                risk_type: RiskType::Market,
                severity: Self::score_to_severity(market_score),
                description: format!(
                    "Exchange rate volatility ({:.2}) poses market risk",
                    mc.exchange_rate
                ),
                mitigation: "Hedge currency exposure or price in stable currency".to_string(),
                probability: market_score,
            });
        }

        // Compliance risk from policies
        if input
            .company_policies
            .iter()
            .any(|p| p.exceptions.len() > 2)
        {
            let compliance_score = 0.65;
            if compliance_score > th.compliance_risk {
                alerts.push(RiskAlert {
                    risk_type: RiskType::Compliance,
                    severity: Self::score_to_severity(compliance_score),
                    description: "Multiple policy exceptions detected".to_string(),
                    mitigation: "Review compliance checklist before proceeding".to_string(),
                    probability: compliance_score,
                });
            }
        }

        // Reputation risk from negative sentiment
        let avg_sentiment: f32 = if input.interaction_history.is_empty() {
            0.0
        } else {
            input
                .interaction_history
                .iter()
                .map(|e| e.sentiment)
                .sum::<f32>()
                / input.interaction_history.len() as f32
        };
        if avg_sentiment < -0.3 {
            alerts.push(RiskAlert {
                risk_type: RiskType::Reputation,
                severity: RiskSeverity::Medium,
                description: format!("Negative sentiment trend ({:.2})", avg_sentiment),
                mitigation: "Escalate to account manager for relationship repair".to_string(),
                probability: avg_sentiment.abs(),
            });
        }

        alerts
    }

    /// Generate recommendations from full analysis
    pub fn generate_recommendations(&self, output: &IntelligenceOutput) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        // Intent-based recommendations
        match output.intent.primary_intent {
            IntentType::BulkOrder => {
                recs.push(Recommendation {
                    action: "Prepare volume discount proposal".to_string(),
                    priority: Priority::High,
                    expected_outcome: "Increase order value by 15-20%".to_string(),
                    deadline: Some(now_ts() + 7 * 86400),
                });
            }
            IntentType::Partnership => {
                recs.push(Recommendation {
                    action: "Schedule partnership discussion with BD team".to_string(),
                    priority: Priority::High,
                    expected_outcome: "Establish strategic partnership framework".to_string(),
                    deadline: Some(now_ts() + 14 * 86400),
                });
            }
            IntentType::Complaint => {
                recs.push(Recommendation {
                    action: "Escalate to customer success for rapid resolution".to_string(),
                    priority: Priority::Urgent,
                    expected_outcome: "Restore customer satisfaction".to_string(),
                    deadline: Some(now_ts() + 2 * 86400),
                });
            }
            IntentType::SampleRequest => {
                recs.push(Recommendation {
                    action: "Ship sample with tracking and follow-up in 3 days".to_string(),
                    priority: Priority::Medium,
                    expected_outcome: "Convert sample to order".to_string(),
                    deadline: Some(now_ts() + 5 * 86400),
                });
            }
            IntentType::PriceComparison => {
                recs.push(Recommendation {
                    action: "Provide competitive pricing analysis with value proposition"
                        .to_string(),
                    priority: Priority::Medium,
                    expected_outcome: "Differentiate from competitors".to_string(),
                    deadline: Some(now_ts() + 3 * 86400),
                });
            }
            IntentType::Inquiry | IntentType::Information => {
                recs.push(Recommendation {
                    action: "Send detailed product catalog with case studies".to_string(),
                    priority: Priority::Low,
                    expected_outcome: "Nurture lead toward qualified stage".to_string(),
                    deadline: Some(now_ts() + 5 * 86400),
                });
            }
        }

        // Risk-based recommendations
        for risk in &output.risks {
            match risk.severity {
                RiskSeverity::Critical | RiskSeverity::High => {
                    recs.push(Recommendation {
                        action: format!("Mitigate {}: {}", risk.risk_type_str(), risk.mitigation),
                        priority: Priority::Urgent,
                        expected_outcome: "Reduce exposure".to_string(),
                        deadline: Some(now_ts() + 1 * 86400),
                    });
                }
                RiskSeverity::Medium => {
                    recs.push(Recommendation {
                        action: format!("Monitor: {}", risk.mitigation),
                        priority: Priority::Medium,
                        expected_outcome: "Early detection of escalation".to_string(),
                        deadline: Some(now_ts() + 7 * 86400),
                    });
                }
                RiskSeverity::Low => {
                    recs.push(Recommendation {
                        action: format!("Note: {}", risk.mitigation),
                        priority: Priority::Low,
                        expected_outcome: "Awareness".to_string(),
                        deadline: None,
                    });
                }
            }
        }

        // Opportunity-based recommendation
        if output.opportunity.score > 70.0 {
            recs.push(Recommendation {
                action: "Fast-track: assign senior account manager".to_string(),
                priority: Priority::High,
                expected_outcome: "Accelerate close for high-value opportunity".to_string(),
                deadline: Some(now_ts() + 3 * 86400),
            });
        }

        // Deduplicate by action prefix
        recs.dedup_by(|a, b| a.action == b.action);
        recs
    }

    /// Full intelligence analysis
    pub fn analyze(&mut self, input: &IntelligenceInput) -> IntelligenceOutput {
        let intent = self.analyze_intent(input);
        let opportunity = self.assess_opportunity(input);
        let risks = self.detect_risks(input);

        let high_risk_count = risks
            .iter()
            .filter(|r| matches!(r.severity, RiskSeverity::Critical | RiskSeverity::High))
            .count();

        let output = IntelligenceOutput {
            intent,
            opportunity,
            risks,
            recommendations: Vec::new(),
            confidence: 0.0,
        };

        let recommendations = self.generate_recommendations(&output);
        let confidence = self.compute_confidence(input);

        let final_output = IntelligenceOutput {
            recommendations,
            confidence,
            ..output
        };

        // Update stats
        self.stats.total_analyses += 1;
        self.stats.avg_confidence =
            (self.stats.avg_confidence * (self.stats.total_analyses - 1) as f32 + confidence)
                / self.stats.total_analyses as f32;
        self.stats.high_risk_detections += high_risk_count as u64;

        final_output
    }

    /// Get intelligence stats
    pub fn stats(&self) -> IntelligenceStats {
        IntelligenceStats {
            model_count: self.models.len(),
            ..self.stats.clone()
        }
    }

    // ─── Private Helpers ──────────────────────────────────────────────────

    fn classify_intent_from_history(
        &self,
        history: &[InteractionEvent],
    ) -> (IntentType, Vec<IntentType>) {
        if history.is_empty() {
            return (IntentType::Inquiry, vec![IntentType::Information]);
        }

        let mut intent_scores: HashMap<&str, f32> = HashMap::new();

        for event in history {
            let content_lower = event.content.to_lowercase();

            if content_lower.contains("price")
                || content_lower.contains("cost")
                || content_lower.contains("报价")
            {
                *intent_scores.entry("price").or_insert(0.0) += event.sentiment.abs() + 0.3;
            }
            if content_lower.contains("bulk")
                || content_lower.contains("quantity")
                || content_lower.contains("批量")
            {
                *intent_scores.entry("bulk").or_insert(0.0) += event.sentiment.abs() + 0.5;
            }
            if content_lower.contains("sample") || content_lower.contains("样品") {
                *intent_scores.entry("sample").or_insert(0.0) += event.sentiment.abs() + 0.4;
            }
            if content_lower.contains("partner") || content_lower.contains("合作") {
                *intent_scores.entry("partnership").or_insert(0.0) += event.sentiment.abs() + 0.6;
            }
            if content_lower.contains("complaint")
                || content_lower.contains("problem")
                || content_lower.contains("投诉")
            {
                *intent_scores.entry("complaint").or_insert(0.0) += event.sentiment.abs() + 0.7;
            }
        }

        let primary = if let Some((&best, _)) = intent_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        {
            match best {
                "bulk" => IntentType::BulkOrder,
                "sample" => IntentType::SampleRequest,
                "partnership" => IntentType::Partnership,
                "complaint" => IntentType::Complaint,
                "price" => IntentType::PriceComparison,
                _ => IntentType::Inquiry,
            }
        } else {
            IntentType::Inquiry
        };

        let secondary: Vec<IntentType> = intent_scores
            .iter()
            .filter(|(&k, _)| {
                k != match &primary {
                    IntentType::BulkOrder => "bulk",
                    IntentType::SampleRequest => "sample",
                    IntentType::Partnership => "partnership",
                    IntentType::Complaint => "complaint",
                    IntentType::PriceComparison => "price",
                    _ => "",
                }
            })
            .filter(|(_, &v)| v > 0.2)
            .map(|(&k, _)| match k {
                "bulk" => IntentType::BulkOrder,
                "sample" => IntentType::SampleRequest,
                "partnership" => IntentType::Partnership,
                "complaint" => IntentType::Complaint,
                "price" => IntentType::PriceComparison,
                _ => IntentType::Information,
            })
            .collect();

        (primary, secondary)
    }

    fn assess_urgency(&self, input: &IntelligenceInput) -> UrgencyLevel {
        let recent_count = input
            .interaction_history
            .iter()
            .filter(|e| e.timestamp > now_ts() - 3 * 86400)
            .count();

        if recent_count >= 5 {
            UrgencyLevel::Immediate
        } else if recent_count >= 3 {
            UrgencyLevel::High
        } else if recent_count >= 1 {
            UrgencyLevel::Medium
        } else {
            UrgencyLevel::Low
        }
    }

    fn estimate_budget(&self, cd: &CustomerData) -> Option<BudgetRange> {
        if cd.total_value <= 0.0 {
            return None;
        }
        let min = cd.total_value * 0.5;
        let max = cd.total_value * 1.5;
        Some(BudgetRange { min, max })
    }

    fn estimate_timeline(&self, input: &IntelligenceInput) -> Option<String> {
        let avg_sentiment: f32 = if input.interaction_history.is_empty() {
            0.0
        } else {
            input
                .interaction_history
                .iter()
                .map(|e| e.sentiment)
                .sum::<f32>()
                / input.interaction_history.len() as f32
        };

        Some(if avg_sentiment > 0.5 {
            "1-2 weeks".to_string()
        } else if avg_sentiment > 0.0 {
            "2-4 weeks".to_string()
        } else if avg_sentiment > -0.3 {
            "1-2 months".to_string()
        } else {
            "Uncertain — relationship repair needed".to_string()
        })
    }

    fn determine_stage(&self, cd: &CustomerData, history: &[InteractionEvent]) -> OpportunityStage {
        let has_negotiation = history.iter().any(|e| {
            let c = e.content.to_lowercase();
            c.contains("negotiat") || c.contains("contract") || c.contains("agreement")
        });
        let has_proposal = history.iter().any(|e| {
            let c = e.content.to_lowercase();
            c.contains("proposal") || c.contains("quotation") || c.contains("报价单")
        });

        if has_negotiation {
            OpportunityStage::Negotiation
        } else if has_proposal {
            OpportunityStage::Proposal
        } else if cd.interaction_count > 5 && cd.total_value > 10_000.0 {
            OpportunityStage::Qualified
        } else if cd.interaction_count > 0 {
            OpportunityStage::Lead
        } else {
            OpportunityStage::Lead
        }
    }

    fn compute_confidence(&self, input: &IntelligenceInput) -> f32 {
        let data_richness = if input.interaction_history.is_empty() {
            0.2
        } else {
            (input.interaction_history.len() as f32 / 10.0).min(1.0)
        };
        let market_richness = if input.market_context.region.is_empty() {
            0.3
        } else {
            0.7
        };
        let model_boost = if self.models.is_empty() { 0.0 } else { 0.1 };

        ((data_richness * 0.5 + market_richness * 0.3 + model_boost + 0.1).min(1.0) * 100.0).round()
            / 100.0
    }

    fn score_to_severity(score: f32) -> RiskSeverity {
        if score >= 0.8 {
            RiskSeverity::Critical
        } else if score >= 0.6 {
            RiskSeverity::High
        } else if score >= 0.4 {
            RiskSeverity::Medium
        } else {
            RiskSeverity::Low
        }
    }
}

impl Default for TradeIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskAlert {
    pub fn risk_type_str(&self) -> &str {
        match self.risk_type {
            RiskType::Payment => "Payment",
            RiskType::Quality => "Quality",
            RiskType::Delivery => "Delivery",
            RiskType::Compliance => "Compliance",
            RiskType::Market => "Market",
            RiskType::Reputation => "Reputation",
        }
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_input() -> IntelligenceInput {
        IntelligenceInput {
            customer_data: CustomerData {
                customer_id: "C001".to_string(),
                name: "Acme Corp".to_string(),
                country: "US".to_string(),
                industry: "Manufacturing".to_string(),
                grade: "A".to_string(),
                source: "Exhibition".to_string(),
                total_value: 250_000.0,
                interaction_count: 8,
            },
            interaction_history: vec![
                InteractionEvent {
                    event_type: "email".to_string(),
                    timestamp: now_ts() - 86400,
                    content: "We need bulk order pricing for 500 units".to_string(),
                    sentiment: 0.6,
                },
                InteractionEvent {
                    event_type: "call".to_string(),
                    timestamp: now_ts() - 172800,
                    content: "Can you provide a proposal with volume discount?".to_string(),
                    sentiment: 0.4,
                },
            ],
            market_context: MarketContext {
                region: "North America".to_string(),
                industry_trend: 0.3,
                competitor_activity: 0.4,
                exchange_rate: 0.05,
                seasonality: Seasonality::Peak,
            },
            company_policies: vec![Policy {
                policy_type: "payment".to_string(),
                rules: vec!["Net 30".to_string()],
                exceptions: vec![],
            }],
        }
    }

    #[test]
    fn test_new_engine() {
        let ti = TradeIntelligence::new();
        assert!(ti.models.is_empty());
        assert_eq!(ti.risk_thresholds.payment_risk, 0.6);
    }

    #[test]
    fn test_analyze_intent_bulk_order() {
        let ti = TradeIntelligence::new();
        let input = make_test_input();
        let intent = ti.analyze_intent(&input);
        assert_eq!(intent.primary_intent, IntentType::BulkOrder);
        assert_eq!(intent.urgency, UrgencyLevel::Medium);
        assert!(intent.budget_range.is_some());
        assert!(intent.decision_timeline.is_some());
    }

    #[test]
    fn test_assess_opportunity_high_value() {
        let ti = TradeIntelligence::new();
        let input = make_test_input();
        let opp = ti.assess_opportunity(&input);
        assert!(opp.score > 50.0, "score should be > 50, got {}", opp.score);
        assert_eq!(opp.stage, OpportunityStage::Proposal);
        assert!(opp.estimated_value > 0.0);
        assert!(opp.conversion_probability > 0.0 && opp.conversion_probability <= 1.0);
        assert!(!opp.factors.is_empty());
    }

    #[test]
    fn test_detect_risks_high_value_payment() {
        let ti = TradeIntelligence::new();
        let input = make_test_input();
        let risks = ti.detect_risks(&input);
        let payment_risks: Vec<_> = risks
            .iter()
            .filter(|r| r.risk_type == RiskType::Payment)
            .collect();
        assert!(
            !payment_risks.is_empty(),
            "should detect payment risk for high-value customer"
        );
    }

    #[test]
    fn test_detect_risks_negative_sentiment() {
        let ti = TradeIntelligence::new();
        let mut input = make_test_input();
        input.interaction_history = vec![
            InteractionEvent {
                event_type: "email".to_string(),
                timestamp: now_ts() - 86400,
                content: "Very disappointed with quality".to_string(),
                sentiment: -0.8,
            },
            InteractionEvent {
                event_type: "email".to_string(),
                timestamp: now_ts() - 3600,
                content: "Still waiting for response".to_string(),
                sentiment: -0.5,
            },
        ];
        let risks = ti.detect_risks(&input);
        let rep_risks: Vec<_> = risks
            .iter()
            .filter(|r| r.risk_type == RiskType::Reputation)
            .collect();
        assert!(!rep_risks.is_empty(), "should detect reputation risk");
    }

    #[test]
    fn test_generate_recommendations() {
        let ti = TradeIntelligence::new();
        let input = make_test_input();
        let mut output = ti.analyze(&input);
        // Re-generate with actual output
        output.recommendations = ti.generate_recommendations(&output);
        assert!(
            !output.recommendations.is_empty(),
            "should produce recommendations"
        );
    }

    #[test]
    fn test_full_analysis() {
        let mut ti = TradeIntelligence::new();
        let input = make_test_input();
        let output = ti.analyze(&input);
        assert!(output.confidence > 0.0);
        assert!(!output.recommendations.is_empty());
        assert!(!output.risks.is_empty());

        let stats = ti.stats();
        assert_eq!(stats.total_analyses, 1);
        assert!(stats.avg_confidence > 0.0);
    }

    #[test]
    fn test_update_thresholds() {
        let mut ti = TradeIntelligence::new();
        let new_th = RiskThresholds {
            payment_risk: 0.9,
            delivery_risk: 0.8,
            compliance_risk: 0.9,
            market_risk: 0.8,
        };
        ti.update_thresholds(new_th.clone());
        assert_eq!(ti.risk_thresholds.payment_risk, 0.9);
    }

    #[test]
    fn test_empty_history_inquiry() {
        let ti = TradeIntelligence::new();
        let mut input = make_test_input();
        input.interaction_history = vec![];
        let intent = ti.analyze_intent(&input);
        assert_eq!(intent.primary_intent, IntentType::Inquiry);
    }

    #[test]
    fn test_complaint_intent() {
        let ti = TradeIntelligence::new();
        let mut input = make_test_input();
        input.interaction_history = vec![InteractionEvent {
            event_type: "email".to_string(),
            timestamp: now_ts(),
            content: "I want to file a complaint about the defective batch".to_string(),
            sentiment: -0.9,
        }];
        let intent = ti.analyze_intent(&input);
        assert_eq!(intent.primary_intent, IntentType::Complaint);
    }

    #[test]
    fn test_score_to_severity_boundaries() {
        assert!(matches!(
            TradeIntelligence::score_to_severity(0.9),
            RiskSeverity::Critical
        ));
        assert!(matches!(
            TradeIntelligence::score_to_severity(0.7),
            RiskSeverity::High
        ));
        assert!(matches!(
            TradeIntelligence::score_to_severity(0.5),
            RiskSeverity::Medium
        ));
        assert!(matches!(
            TradeIntelligence::score_to_severity(0.1),
            RiskSeverity::Low
        ));
    }

    #[test]
    fn test_default_impl() {
        let ti = TradeIntelligence::default();
        assert!(ti.models.is_empty());
    }
}
