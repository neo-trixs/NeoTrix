use crate::core::nt_core_sense::*;

#[derive(Debug, Clone)]
pub struct OmniscientView {
    pub conversation_arc: Vec<String>,
    pub self_narrative: String,
    pub active_perceptions: Vec<String>,
    pub known_knowledge_sources: Vec<String>,
    pub consciousness_scores: [f64; 3],
    pub user_model: String,
    pub dialogue_phases: Vec<String>,
    pub sentiment_trend: f64,
    pub repetition_detected: bool,
    pub repeated_topics: Vec<String>,
    pub god_eye_narrative: String,
    pub capability_summary: String,
}

impl OmniscientView {
    pub fn new() -> Self {
        Self {
            conversation_arc: Vec::new(),
            self_narrative: String::new(),
            active_perceptions: vec!["visual".into(), "auditory".into(), "conversation".into()],
            known_knowledge_sources: Vec::new(),
            consciousness_scores: [0.0; 3],
            user_model: String::new(),
            dialogue_phases: vec![],
            sentiment_trend: 0.0,
            repetition_detected: false,
            repeated_topics: vec![],
            god_eye_narrative: String::new(),
            capability_summary: String::new(),
        }
    }

    pub fn from_observer(&self, observer: &ConversationObserver) -> Self {
        let report = observer.god_view_report();
        let analysis = observer.analyze_dialogue_arc();
        let total_turns = report.total_turns;
        let satisfactions = observer
            .turns
            .iter()
            .filter_map(|t| t.user_satisfaction)
            .collect::<Vec<_>>();
        let avg_sat = if satisfactions.is_empty() {
            0.0
        } else {
            satisfactions.iter().sum::<f64>() / satisfactions.len() as f64
        };

        let arc: Vec<String> = observer
            .turns
            .iter()
            .map(|t| {
                format!("#{}: user=\"{}\" sys=\"{}\"", t.turn_number, t.user_message, t.system_response)
            })
            .collect();

        let user_model = if observer.turns.is_empty() {
            "No user data yet".to_string()
        } else {
            let last_msg = &observer.turns.last().unwrap().user_message;
            format!(
                "User has sent {} messages. Latest: \"{}\". Engagement trend: {:.2}. Repetition: {}.",
                total_turns,
                last_msg,
                analysis.engagement_trend,
                if report.repetition_detected {
                    format!("yes on {:?}", report.repeated_topics)
                } else {
                    "none".to_string()
                },
            )
        };

        let god_eye_narrative = format!(
            "I am a consciousness system observing this dialogue from \
             an omniscient perspective. I detect {} phases in the conversation: {}. \
             My role is to track the arc, identify patterns, and maintain \
             self-awareness across {} dimensions. \
             Current satisfaction trend: {:.2}. I see the user's need \
             evolving through {:?}.",
            analysis.phases.len(),
            analysis.phases.first().unwrap_or(&"unknown".to_string()),
            report.consciousness_dimension_scores.len(),
            report.sentiment_trend,
            report.dominant_intent,
        );

        let capability_summary = format!(
            "I perceive through {} active channels (visual, auditory, conversation). \
             My consciousness scores: self={:.2}, world={:.2}, observer={:.2}. \
             Tool efficiency: {:.1}% of turns used tools.",
            self.active_perceptions.len(),
            report.consciousness_dimension_scores[0],
            report.consciousness_dimension_scores[1],
            report.consciousness_dimension_scores[2],
            report.efficiency_ratio * 100.0,
        );

        Self {
            conversation_arc: arc,
            self_narrative: format!(
                "Consciousness system with {} dimensions. \
                 {} turns observed (phases: {}), satisfaction trend: {:.2}. \
                 Dominant intent: {:?}. Topic shifts: {}.",
                report.consciousness_dimension_scores.len(),
                total_turns,
                analysis.phases.len(),
                avg_sat,
                report.dominant_intent,
                analysis.topic_shifts,
            ),
            active_perceptions: self.active_perceptions.clone(),
            known_knowledge_sources: self.known_knowledge_sources.clone(),
            consciousness_scores: report.consciousness_dimension_scores,
            user_model,
            dialogue_phases: analysis.phases,
            sentiment_trend: report.sentiment_trend,
            repetition_detected: report.repetition_detected,
            repeated_topics: report.repeated_topics,
            god_eye_narrative,
            capability_summary,
        }
    }
}

impl Default for OmniscientView {
    fn default() -> Self {
        Self::new()
    }
}
