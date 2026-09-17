#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesScript {
    pub id: String,
    pub name: String,
    pub category: ScriptCategory,
    pub trigger_conditions: Vec<Condition>,
    pub content: String,
    pub variables: Vec<String>,
    pub effectiveness_score: f32,
    pub usage_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScriptCategory {
    Opening,
    Qualification,
    Presentation,
    ObjectionHandling,
    Closing,
    FollowUp,
    Referral,
    ReEngagement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: Operator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpStrategy {
    pub id: String,
    pub name: String,
    pub customer_stage: CustomerStage,
    pub channel: CommunicationChannel,
    pub frequency: FollowUpFrequency,
    pub max_attempts: u32,
    pub templates: Vec<String>,
    pub success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CustomerStage {
    NewLead,
    Engaged,
    Qualified,
    ProposalSent,
    Negotiating,
    ClosedWon,
    ClosedLost,
    Dormant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommunicationChannel {
    Email,
    WhatsApp,
    Phone,
    Meeting,
    Social,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FollowUpFrequency {
    Daily,
    Weekly,
    BiWeekly,
    Monthly,
    Quarterly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecord {
    pub date: String,
    pub salesperson_id: String,
    pub metric: String,
    pub value: f64,
    pub target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingRequest {
    pub salesperson_id: String,
    pub customer_id: String,
    pub situation: String,
    pub history: Vec<InteractionHistory>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionHistory {
    pub channel: String,
    pub timestamp: i64,
    pub direction: Direction,
    pub content: String,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingResponse {
    pub recommended_scripts: Vec<SalesScript>,
    pub follow_up_strategy: FollowUpStrategy,
    pub talking_points: Vec<String>,
    pub objection_handlers: Vec<ObjectionHandler>,
    pub performance_prediction: PerformancePrediction,
    pub action_items: Vec<ActionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionHandler {
    pub objection: String,
    pub response: String,
    pub confidence: f32,
    pub alternative_responses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    pub conversion_probability: f32,
    pub estimated_close_date: Option<String>,
    pub estimated_value: f64,
    pub factors: Vec<PredictionFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionFactor {
    pub factor: String,
    pub impact: f32,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action: String,
    pub priority: Priority,
    pub deadline: Option<String>,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachStats {
    pub total_scripts: u32,
    pub total_strategies: u32,
    pub avg_effectiveness: f32,
    pub top_performing_scripts: Vec<String>,
}

pub struct SalesCoach {
    scripts: Vec<SalesScript>,
    strategies: Vec<FollowUpStrategy>,
    performance_history: HashMap<String, Vec<PerformanceRecord>>,
}

impl Default for SalesCoach {
    fn default() -> Self {
        Self::new()
    }
}

impl SalesCoach {
    pub fn new() -> Self {
        let mut coach = Self {
            scripts: Vec::new(),
            strategies: Vec::new(),
            performance_history: HashMap::new(),
        };
        coach.load_defaults();
        coach
    }

    pub fn coach(&self, request: &CoachingRequest) -> CoachingResponse {
        let stage = self.infer_stage(&request.history);

        let recommended_scripts = self.recommend_scripts(&request.situation, &stage);
        let follow_up_strategy = self.generate_strategy(&stage, &CommunicationChannel::Email);
        let talking_points = self.generate_talking_points(&request.situation);

        let objection_handlers: Vec<ObjectionHandler> = request
            .history
            .iter()
            .filter(|h| h.direction == Direction::Inbound)
            .map(|h| self.handle_objection(&h.content))
            .collect();

        let performance_prediction = self.predict_performance(
            &request.salesperson_id,
            self.performance_history
                .get(&request.salesperson_id)
                .cloned()
                .as_deref()
                .unwrap_or(&[]),
        );

        let mut response = CoachingResponse {
            recommended_scripts,
            follow_up_strategy,
            talking_points,
            objection_handlers,
            performance_prediction,
            action_items: Vec::new(),
        };

        response.action_items = self.generate_action_items(&response);
        response
    }

    pub fn recommend_scripts(
        &self,
        situation: &str,
        customer_stage: &CustomerStage,
    ) -> Vec<SalesScript> {
        let situation_lower = situation.to_lowercase();

        let mut scored: Vec<(f32, &SalesScript)> = self
            .scripts
            .iter()
            .filter(|s| self.matches_stage(&s.category, customer_stage))
            .map(|s| {
                let mut score = s.effectiveness_score;
                for cond in &s.trigger_conditions {
                    if self.evaluate_condition(cond, &situation_lower) {
                        score += 0.3;
                    }
                }
                (score, s)
            })
            .filter(|(score, _)| *score > 0.3)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(3).map(|(_, s)| s.clone()).collect()
    }

    pub fn generate_strategy(
        &self,
        customer_stage: &CustomerStage,
        channel: &CommunicationChannel,
    ) -> FollowUpStrategy {
        self.strategies
            .iter()
            .find(|s| &s.customer_stage == customer_stage && &s.channel == channel)
            .cloned()
            .unwrap_or_else(|| self.default_strategy(customer_stage, channel))
    }

    pub fn handle_objection(&self, objection: &str) -> ObjectionHandler {
        let lower = objection.to_lowercase();

        let known_objections: Vec<(&str, &str, Vec<&str>)> = vec![
            (
                "too expensive",
                "I understand budget is a concern. Let me show you the ROI breakdown - most clients see a return within 3 months.",
                vec![
                    "We offer flexible payment plans that can fit your budget.",
                    "Let me share a case study of a similar company that saved 40% annually.",
                ],
            ),
            (
                "not the right time",
                "Timing is important. What specific timeline would work better for your team?",
                vec![
                    "We can start with a pilot program to test when you're ready.",
                    "I'll send you a timeline we can revisit in Q2.",
                ],
            ),
            (
                "need to think about it",
                "Absolutely. What specific aspects would you like to think through? I can provide additional info.",
                vec![
                    "I'll send a summary email with key points for your review.",
                    "Would it help to schedule a brief call to discuss your questions?",
                ],
            ),
            (
                "already have a solution",
                "That's great you have something in place. How is it working for you? Are there areas for improvement?",
                vec![
                    "We often complement existing solutions rather than replace them.",
                    "Let me show you how our clients improved results after switching.",
                ],
            ),
            (
                "budget constraints",
                "Budget is always important. Let me show you how we can work within your current allocation.",
                vec![
                    "We have tiered pricing that scales with your usage.",
                    "Many clients find the cost is offset by efficiency gains.",
                ],
            ),
        ];

        for (pattern, response, alternatives) in &known_objections {
            if lower.contains(pattern) {
                return ObjectionHandler {
                    objection: objection.to_string(),
                    response: response.to_string(),
                    confidence: 0.85,
                    alternative_responses: alternatives.iter().map(|s| s.to_string()).collect(),
                };
            }
        }

        ObjectionHandler {
            objection: objection.to_string(),
            response: format!(
                "I appreciate you sharing that concern. Let me address it directly: {}",
                objection
            ),
            confidence: 0.5,
            alternative_responses: vec![
                "Could you elaborate on that concern so I can better address it?".to_string(),
                "That's a valid point. Let me share some relevant data.".to_string(),
            ],
        }
    }

    pub fn predict_performance(
        &self,
        salesperson_id: &str,
        history: &[PerformanceRecord],
    ) -> PerformancePrediction {
        let records: Vec<&PerformanceRecord> = history
            .iter()
            .filter(|r| r.salesperson_id == salesperson_id)
            .collect();

        if records.is_empty() {
            return PerformancePrediction {
                conversion_probability: 0.5,
                estimated_close_date: None,
                estimated_value: 0.0,
                factors: vec![PredictionFactor {
                    factor: "no_data".to_string(),
                    impact: 0.0,
                    explanation: "No performance history available".to_string(),
                }],
            };
        }

        let avg_ratio: f64 = records
            .iter()
            .map(|r| r.value / r.target.max(0.01))
            .sum::<f64>()
            / records.len() as f64;

        let conversion_probability = (avg_ratio as f32).clamp(0.1, 0.95);

        let high_value_records: Vec<&&PerformanceRecord> =
            records.iter().filter(|r| r.value >= r.target).collect();
        let value: f64 = records.iter().map(|r| r.value).sum::<f64>() / records.len() as f64;

        let factors = vec![
            PredictionFactor {
                factor: "historical_attainment".to_string(),
                impact: avg_ratio as f32,
                explanation: format!("Historical target attainment: {:.0}%", avg_ratio * 100.0),
            },
            PredictionFactor {
                factor: "consistency".to_string(),
                impact: high_value_records.len() as f32 / records.len().max(1) as f32,
                explanation: format!(
                    "{}/{} periods met or exceeded target",
                    high_value_records.len(),
                    records.len()
                ),
            },
        ];

        PerformancePrediction {
            conversion_probability,
            estimated_close_date: None,
            estimated_value: value,
            factors,
        }
    }

    pub fn generate_talking_points(&self, situation: &str) -> Vec<String> {
        let lower = situation.to_lowercase();
        let mut points = Vec::new();

        if lower.contains("demo") || lower.contains("demonstration") {
            points.push(
                "Focus on the 3 key features that solve their stated pain points".to_string(),
            );
            points.push("Prepare a customized workflow matching their use case".to_string());
            points.push("Have ROI data ready for their specific industry".to_string());
        }

        if lower.contains("objection") || lower.contains("concern") {
            points.push("Listen fully before responding - don't interrupt".to_string());
            points.push("Acknowledge the concern as valid before addressing".to_string());
            points.push("Use social proof from similar companies".to_string());
        }

        if lower.contains("closing") || lower.contains("close") {
            points.push("Summarize all agreed-upon value points".to_string());
            points.push("Create urgency with a time-limited offer".to_string());
            points.push("Ask directly: Shall we proceed with the agreement?".to_string());
        }

        if lower.contains("first call") || lower.contains("initial") || lower.contains("intro") {
            points.push("Research the prospect's company and role beforehand".to_string());
            points.push("Ask open-ended questions to understand their challenges".to_string());
            points
                .push("Keep the conversation focused on their needs, not your product".to_string());
            points.push("Agree on clear next steps before ending the call".to_string());
        }

        if lower.contains("proposal") || lower.contains("pricing") {
            points.push("Lead with value before discussing price".to_string());
            points.push("Present options at different price points".to_string());
            points.push("Highlight ROI and payback period".to_string());
        }

        if points.is_empty() {
            points.push("Understand the customer's core pain point first".to_string());
            points.push("Tailor your message to their specific situation".to_string());
            points.push("Always propose clear next steps".to_string());
        }

        points
    }

    pub fn generate_action_items(&self, response: &CoachingResponse) -> Vec<ActionItem> {
        let mut items = Vec::new();

        if !response.recommended_scripts.is_empty() {
            items.push(ActionItem {
                action: format!(
                    "Review and practice {} recommended script(s)",
                    response.recommended_scripts.len()
                ),
                priority: Priority::High,
                deadline: Some("before_next_call".to_string()),
                expected_outcome: "Improved conversation flow and confidence".to_string(),
            });
        }

        match response.performance_prediction.conversion_probability {
            p if p < 0.3 => {
                items.push(ActionItem {
                    action: "Schedule coaching session to address performance gaps".to_string(),
                    priority: Priority::Urgent,
                    deadline: Some("within_24h".to_string()),
                    expected_outcome: "Identified improvement areas with action plan".to_string(),
                });
            }
            p if p < 0.6 => {
                items.push(ActionItem {
                    action: "Review recent calls and identify improvement patterns".to_string(),
                    priority: Priority::Medium,
                    deadline: Some("within_week".to_string()),
                    expected_outcome: "Pattern recognition for better outcomes".to_string(),
                });
            }
            _ => {}
        }

        items.push(ActionItem {
            action: "Follow up with customer per strategy".to_string(),
            priority: Priority::High,
            deadline: Some(
                response
                    .follow_up_strategy
                    .frequency
                    .deadline_str()
                    .to_string(),
            ),
            expected_outcome: "Maintained engagement and deal progression".to_string(),
        });

        items
    }

    pub fn update_script_effectiveness(&mut self, script_id: &str, success: bool) {
        if let Some(script) = self.scripts.iter_mut().find(|s| s.id == script_id) {
            script.usage_count += 1;
            let delta = if success { 0.05 } else { -0.03 };
            script.effectiveness_score = (script.effectiveness_score + delta).clamp(0.0, 1.0);
        }
    }

    pub fn stats(&self) -> CoachStats {
        let total_scripts = self.scripts.len() as u32;
        let total_strategies = self.strategies.len() as u32;

        let avg_effectiveness = if self.scripts.is_empty() {
            0.0
        } else {
            self.scripts
                .iter()
                .map(|s| s.effectiveness_score)
                .sum::<f32>()
                / self.scripts.len() as f32
        };

        let mut top: Vec<(String, f32)> = self
            .scripts
            .iter()
            .map(|s| (s.name.clone(), s.effectiveness_score))
            .collect();
        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        CoachStats {
            total_scripts,
            total_strategies,
            avg_effectiveness,
            top_performing_scripts: top.into_iter().take(5).map(|(name, _)| name).collect(),
        }
    }

    // ── Private helpers ──

    fn load_defaults(&mut self) {
        self.scripts = vec![
            SalesScript {
                id: "scr_001".to_string(),
                name: "Discovery Opening".to_string(),
                category: ScriptCategory::Opening,
                trigger_conditions: vec![Condition {
                    field: "stage".to_string(),
                    operator: Operator::Equals,
                    value: "NewLead".to_string(),
                }],
                content: "Hi {name}, I noticed {company} is {trigger}. I'd love to learn about your priorities.".to_string(),
                variables: vec!["name".to_string(), "company".to_string(), "trigger".to_string()],
                effectiveness_score: 0.78,
                usage_count: 0,
            },
            SalesScript {
                id: "scr_002".to_string(),
                name: "Pain Point Qualifier".to_string(),
                category: ScriptCategory::Qualification,
                trigger_conditions: vec![Condition {
                    field: "stage".to_string(),
                    operator: Operator::Equals,
                    value: "Engaged".to_string(),
                }],
                content: "What's the biggest challenge you're facing with {pain_area} right now?".to_string(),
                variables: vec!["pain_area".to_string()],
                effectiveness_score: 0.82,
                usage_count: 0,
            },
            SalesScript {
                id: "scr_003".to_string(),
                name: "Value Proposition Close".to_string(),
                category: ScriptCategory::Closing,
                trigger_conditions: vec![Condition {
                    field: "stage".to_string(),
                    operator: Operator::Equals,
                    value: "Negotiating".to_string(),
                }],
                content: "Based on our discussion, you'll see {roi}% improvement in {metric} within {timeframe}. Shall we move forward?".to_string(),
                variables: vec!["roi".to_string(), "metric".to_string(), "timeframe".to_string()],
                effectiveness_score: 0.85,
                usage_count: 0,
            },
            SalesScript {
                id: "scr_004".to_string(),
                name: "Social Proof Close".to_string(),
                category: ScriptCategory::Closing,
                trigger_conditions: vec![Condition {
                    field: "objection".to_string(),
                    operator: Operator::Contains,
                    value: "risk".to_string(),
                }],
                content: "{similar_company} saw {result} within {timeframe} of implementing our solution. We can achieve the same for you.".to_string(),
                variables: vec!["similar_company".to_string(), "result".to_string(), "timeframe".to_string()],
                effectiveness_score: 0.80,
                usage_count: 0,
            },
            SalesScript {
                id: "scr_005".to_string(),
                name: "Re-engagement Check-in".to_string(),
                category: ScriptCategory::ReEngagement,
                trigger_conditions: vec![Condition {
                    field: "stage".to_string(),
                    operator: Operator::Equals,
                    value: "Dormant".to_string(),
                }],
                content: "Hi {name}, it's been a while since we last connected. {company}'s priorities may have shifted - is {solution} still relevant?".to_string(),
                variables: vec!["name".to_string(), "company".to_string(), "solution".to_string()],
                effectiveness_score: 0.65,
                usage_count: 0,
            },
            SalesScript {
                id: "scr_006".to_string(),
                name: "Referral Request".to_string(),
                category: ScriptCategory::Referral,
                trigger_conditions: vec![Condition {
                    field: "stage".to_string(),
                    operator: Operator::Equals,
                    value: "ClosedWon".to_string(),
                }],
                content: "I'm glad we could help {company} achieve {result}. Do you know anyone else who could benefit from a similar outcome?".to_string(),
                variables: vec!["company".to_string(), "result".to_string()],
                effectiveness_score: 0.72,
                usage_count: 0,
            },
        ];

        self.strategies = vec![
            FollowUpStrategy {
                id: "str_001".to_string(),
                name: "New Lead Fast Track".to_string(),
                customer_stage: CustomerStage::NewLead,
                channel: CommunicationChannel::Email,
                frequency: FollowUpFrequency::Daily,
                max_attempts: 5,
                templates: vec![
                    "Hi {name}, following up on our conversation about {topic}.".to_string(),
                    "Hi {name}, I found this resource relevant to {pain_point}.".to_string(),
                ],
                success_rate: 0.45,
            },
            FollowUpStrategy {
                id: "str_002".to_string(),
                name: "Engaged Lead Nurture".to_string(),
                customer_stage: CustomerStage::Engaged,
                channel: CommunicationChannel::Email,
                frequency: FollowUpFrequency::BiWeekly,
                max_attempts: 4,
                templates: vec![
                    "Hi {name}, I wanted to share some insights on {topic}.".to_string()
                ],
                success_rate: 0.55,
            },
            FollowUpStrategy {
                id: "str_003".to_string(),
                name: "Proposal Follow-up".to_string(),
                customer_stage: CustomerStage::ProposalSent,
                channel: CommunicationChannel::Phone,
                frequency: FollowUpFrequency::Weekly,
                max_attempts: 3,
                templates: vec![
                    "Hi {name}, I wanted to check if you had any questions about the proposal."
                        .to_string(),
                ],
                success_rate: 0.62,
            },
            FollowUpStrategy {
                id: "str_004".to_string(),
                name: "Negotiation Push".to_string(),
                customer_stage: CustomerStage::Negotiating,
                channel: CommunicationChannel::Meeting,
                frequency: FollowUpFrequency::Daily,
                max_attempts: 5,
                templates: vec!["Hi {name}, let's finalize the terms we discussed.".to_string()],
                success_rate: 0.70,
            },
            FollowUpStrategy {
                id: "str_005".to_string(),
                name: "Dormant Reactivation".to_string(),
                customer_stage: CustomerStage::Dormant,
                channel: CommunicationChannel::Email,
                frequency: FollowUpFrequency::Monthly,
                max_attempts: 3,
                templates: vec![
                    "Hi {name}, it's been a while. Things may have changed - want to reconnect?"
                        .to_string(),
                ],
                success_rate: 0.30,
            },
        ];
    }

    fn infer_stage(&self, history: &[InteractionHistory]) -> CustomerStage {
        if history.is_empty() {
            return CustomerStage::NewLead;
        }

        let outcomes: Vec<&str> = history
            .iter()
            .filter_map(|h| h.outcome.as_deref())
            .collect();

        for outcome in &outcomes {
            let lower = outcome.to_lowercase();
            if lower.contains("closed") && lower.contains("won") {
                return CustomerStage::ClosedWon;
            }
            if lower.contains("closed") && lower.contains("lost") {
                return CustomerStage::ClosedLost;
            }
            if lower.contains("proposal") {
                return CustomerStage::ProposalSent;
            }
            if lower.contains("negotiat") {
                return CustomerStage::Negotiating;
            }
            if lower.contains("qualified") {
                return CustomerStage::Qualified;
            }
        }

        if history.len() >= 3 {
            CustomerStage::Engaged
        } else {
            CustomerStage::NewLead
        }
    }

    fn matches_stage(&self, category: &ScriptCategory, stage: &CustomerStage) -> bool {
        matches!(
            (category, stage),
            (ScriptCategory::Opening, CustomerStage::NewLead)
                | (
                    ScriptCategory::Qualification,
                    CustomerStage::NewLead | CustomerStage::Engaged
                )
                | (
                    ScriptCategory::Presentation,
                    CustomerStage::Engaged | CustomerStage::Qualified
                )
                | (
                    ScriptCategory::ObjectionHandling,
                    CustomerStage::Qualified | CustomerStage::Negotiating
                )
                | (
                    ScriptCategory::Closing,
                    CustomerStage::Negotiating | CustomerStage::ProposalSent
                )
                | (
                    ScriptCategory::FollowUp,
                    CustomerStage::ProposalSent | CustomerStage::Engaged
                )
                | (ScriptCategory::Referral, CustomerStage::ClosedWon)
                | (
                    ScriptCategory::ReEngagement,
                    CustomerStage::Dormant | CustomerStage::ClosedLost
                )
        )
    }

    fn evaluate_condition(&self, condition: &Condition, context: &str) -> bool {
        match condition.operator {
            Operator::Equals => context.contains(&condition.value.to_lowercase()),
            Operator::Contains => context.contains(&condition.value.to_lowercase()),
            Operator::GreaterThan => context.parse::<f64>().map_or(false, |v| {
                condition
                    .value
                    .parse::<f64>()
                    .map_or(false, |thresh| v > thresh)
            }),
            Operator::LessThan => context.parse::<f64>().map_or(false, |v| {
                condition
                    .value
                    .parse::<f64>()
                    .map_or(false, |thresh| v < thresh)
            }),
            Operator::In => condition.value.to_lowercase().contains(context),
        }
    }

    fn default_strategy(
        &self,
        customer_stage: &CustomerStage,
        channel: &CommunicationChannel,
    ) -> FollowUpStrategy {
        let (freq, max_attempts) = match customer_stage {
            CustomerStage::NewLead => (FollowUpFrequency::Daily, 5),
            CustomerStage::Engaged => (FollowUpFrequency::BiWeekly, 4),
            CustomerStage::Qualified => (FollowUpFrequency::Weekly, 3),
            CustomerStage::ProposalSent => (FollowUpFrequency::Weekly, 3),
            CustomerStage::Negotiating => (FollowUpFrequency::Daily, 5),
            CustomerStage::ClosedWon => (FollowUpFrequency::Monthly, 2),
            CustomerStage::ClosedLost => (FollowUpFrequency::Quarterly, 2),
            CustomerStage::Dormant => (FollowUpFrequency::Monthly, 3),
        };

        let channel_name = match channel {
            CommunicationChannel::Email => "Email",
            CommunicationChannel::WhatsApp => "WhatsApp",
            CommunicationChannel::Phone => "Phone",
            CommunicationChannel::Meeting => "Meeting",
            CommunicationChannel::Social => "Social",
        };

        FollowUpStrategy {
            id: format!("str_def_{:?}_{:?}", customer_stage, channel),
            name: format!("{:?} via {}", customer_stage, channel_name),
            customer_stage: customer_stage.clone(),
            channel: channel.clone(),
            frequency: freq,
            max_attempts,
            templates: vec![
                "Following up on our recent conversation.".to_string(),
                "I wanted to check in and see if you have any questions.".to_string(),
            ],
            success_rate: 0.40,
        }
    }
}

impl FollowUpFrequency {
    fn deadline_str(&self) -> &str {
        match self {
            FollowUpFrequency::Daily => "tomorrow",
            FollowUpFrequency::Weekly => "within_1_week",
            FollowUpFrequency::BiWeekly => "within_2_weeks",
            FollowUpFrequency::Monthly => "within_1_month",
            FollowUpFrequency::Quarterly => "within_3_months",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_coach() -> SalesCoach {
        SalesCoach::new()
    }

    fn make_request(situation: &str, history_len: usize) -> CoachingRequest {
        let history: Vec<InteractionHistory> = (0..history_len)
            .map(|i| InteractionHistory {
                channel: "email".to_string(),
                timestamp: i as i64,
                direction: Direction::Outbound,
                content: format!("Interaction {}", i),
                outcome: Some("qualified".to_string()),
            })
            .collect();

        CoachingRequest {
            salesperson_id: "sp_001".to_string(),
            customer_id: "cust_001".to_string(),
            situation: situation.to_string(),
            history,
            goals: vec!["close deal".to_string()],
        }
    }

    #[test]
    fn test_new_coach_has_defaults() {
        let coach = make_coach();
        assert!(!coach.scripts.is_empty());
        assert!(!coach.strategies.is_empty());
    }

    #[test]
    fn test_recommend_scripts_returns_matches() {
        let coach = make_coach();
        let scripts = coach.recommend_scripts("new prospect call", &CustomerStage::NewLead);
        assert!(!scripts.is_empty());
    }

    #[test]
    fn test_recommend_scripts_empty_for_wrong_stage() {
        let coach = make_coach();
        let scripts = coach.recommend_scripts("closing the deal", &CustomerStage::ClosedWon);
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_generate_strategy_default() {
        let coach = make_coach();
        let strat = coach.generate_strategy(&CustomerStage::NewLead, &CommunicationChannel::Email);
        assert_eq!(strat.customer_stage, CustomerStage::NewLead);
        assert_eq!(strat.channel, CommunicationChannel::Email);
        assert!(strat.max_attempts > 0);
    }

    #[test]
    fn test_handle_objection_expensive() {
        let coach = make_coach();
        let handler = coach.handle_objection("It's too expensive");
        assert!(handler.confidence > 0.5);
        assert!(!handler.alternative_responses.is_empty());
    }

    #[test]
    fn test_handle_objection_unknown() {
        let coach = make_coach();
        let handler = coach.handle_objection("xyzzy unknown objection");
        assert!(handler.confidence <= 0.5);
    }

    #[test]
    fn test_predict_performance_no_history() {
        let coach = make_coach();
        let pred = coach.predict_performance("sp_unknown", &[]);
        assert!(pred.conversion_probability > 0.0);
        assert!(pred.factors.len() == 1);
    }

    #[test]
    fn test_predict_performance_with_history() {
        let coach = make_coach();
        let history = vec![
            PerformanceRecord {
                date: "2026-01".to_string(),
                salesperson_id: "sp_001".to_string(),
                metric: "revenue".to_string(),
                value: 120.0,
                target: 100.0,
            },
            PerformanceRecord {
                date: "2026-02".to_string(),
                salesperson_id: "sp_001".to_string(),
                metric: "revenue".to_string(),
                value: 80.0,
                target: 100.0,
            },
        ];
        let pred = coach.predict_performance("sp_001", &history);
        assert!(pred.conversion_probability > 0.0 && pred.conversion_probability <= 1.0);
        assert!(pred.estimated_value > 0.0);
    }

    #[test]
    fn test_generate_talking_points() {
        let coach = make_coach();
        let points = coach.generate_talking_points("schedule a demo for the team");
        assert!(points.len() >= 3);
    }

    #[test]
    fn test_generate_talking_points_generic() {
        let coach = make_coach();
        let points = coach.generate_talking_points("just chatting");
        assert!(!points.is_empty());
    }

    #[test]
    fn test_generate_action_items() {
        let coach = make_coach();
        let response = coach.coach(&make_request("first call with new prospect", 1));
        let items = coach.generate_action_items(&response);
        assert!(!items.is_empty());
    }

    #[test]
    fn test_update_script_effectiveness_success() {
        let mut coach = make_coach();
        let before = coach.scripts[0].effectiveness_score;
        coach.update_script_effectiveness("scr_001", true);
        assert!(coach.scripts[0].effectiveness_score > before);
        assert_eq!(coach.scripts[0].usage_count, 1);
    }

    #[test]
    fn test_update_script_effectiveness_failure() {
        let mut coach = make_coach();
        let before = coach.scripts[0].effectiveness_score;
        coach.update_script_effectiveness("scr_001", false);
        assert!(coach.scripts[0].effectiveness_score < before);
    }

    #[test]
    fn test_stats() {
        let coach = make_coach();
        let stats = coach.stats();
        assert!(stats.total_scripts > 0);
        assert!(stats.total_strategies > 0);
        assert!(stats.avg_effectiveness > 0.0);
        assert!(!stats.top_performing_scripts.is_empty());
    }

    #[test]
    fn test_coach_full_flow() {
        let coach = make_coach();
        let request = make_request("demo call with potential client", 2);
        let response = coach.coach(&request);
        assert!(!response.recommended_scripts.is_empty());
        assert!(!response.talking_points.is_empty());
        assert!(response.performance_prediction.conversion_probability > 0.0);
        assert!(!response.action_items.is_empty());
    }
}
