use std::collections::HashMap;

use super::sensory_types::{
    ConversationObserver, ConversationTurn, DialogueArcAnalysis, GodViewReport, SensoryEvent,
    SensoryEventKind, SensoryMemory,
};

impl SensoryMemory {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 100,
        }
    }

    pub fn with_capacity(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    pub fn push(&mut self, event: SensoryEvent) {
        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn latest(&self, n: usize) -> Vec<&SensoryEvent> {
        self.events.iter().rev().take(n).collect()
    }

    pub fn by_kind(&self, kind: &SensoryEventKind) -> Vec<&SensoryEvent> {
        self.events.iter().filter(|e| &e.kind == kind).collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for SensoryMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationObserver {
    pub fn new(max_turns: usize) -> Self {
        Self {
            turns: Vec::new(),
            current_topic: None,
            topic_shifts: 0,
            max_turns,
        }
    }

    pub fn record_turn(&mut self, turn: ConversationTurn) {
        if self.turns.len() >= self.max_turns {
            self.turns.remove(0);
        }
        self.turns.push(turn);
    }

    pub fn god_view_report(&self) -> GodViewReport {
        let total_turns = self.turns.len();
        let analysis = self.analyze_dialogue_arc();
        let topic_shift_count = self.topic_shifts + analysis.topic_shifts;

        let satisfactions: Vec<f64> = self
            .turns
            .iter()
            .filter_map(|t| t.user_satisfaction)
            .collect();
        let avg_user_satisfaction = if satisfactions.is_empty() {
            None
        } else {
            Some(satisfactions.iter().sum::<f64>() / satisfactions.len() as f64)
        };

        let dominant_intent = {
            let mut counts: HashMap<&str, usize> = HashMap::new();
            for turn in &self.turns {
                if let Some(ref intent) = turn.intent_label {
                    *counts.entry(intent.as_str()).or_insert(0) += 1;
                }
            }
            counts
                .into_iter()
                .max_by_key(|&(_, c)| c)
                .map(|(k, _)| k.to_string())
        };

        let turns_with_tools = self
            .turns
            .iter()
            .filter(|t| !t.tools_used.is_empty())
            .count();
        let efficiency_ratio = if total_turns > 0 {
            turns_with_tools as f64 / total_turns as f64
        } else {
            0.0
        };

        let patterns_detected = Vec::new();
        let meta_insight = String::new();
        let consciousness_dimension_scores = [0.0f64; 3];

        let dialogue_arc = analysis.phases.clone();
        let sentiment_trend = analysis.sentiment_trend;
        let repeated_topics: Vec<String> = analysis
            .repetitions
            .iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(topic, _)| topic.clone())
            .collect();
        let repetition_detected = !repeated_topics.is_empty();

        GodViewReport {
            total_turns,
            topic_shift_count,
            avg_user_satisfaction,
            dominant_intent,
            efficiency_ratio,
            patterns_detected,
            meta_insight,
            consciousness_dimension_scores,
            dialogue_arc,
            sentiment_trend,
            repetition_detected,
            repeated_topics,
        }
    }

    pub fn analyze_dialogue_arc(&self) -> DialogueArcAnalysis {
        let total = self.turns.len();
        if total == 0 {
            return DialogueArcAnalysis {
                phases: vec![],
                sentiment_trend: 0.0,
                topic_shifts: 0,
                repetitions: vec![],
                tool_density: vec![],
                engagement_trend: 0.0,
                quality_per_phase: vec![],
            };
        }

        let phases: Vec<String> = self
            .turns
            .iter()
            .map(|t| {
                let msg = t.user_message.to_lowercase();
                if msg.len() < 10 {
                    "greeting".to_string()
                } else if msg.contains("what")
                    || msg.contains("how")
                    || msg.contains("why")
                    || msg.contains("explain")
                    || msg.contains("?")
                {
                    "query".to_string()
                } else if msg.contains("fix")
                    || msg.contains("bug")
                    || msg.contains("error")
                    || msg.contains("wrong")
                    || msg.contains("broken")
                {
                    "debug".to_string()
                } else if msg.contains("write")
                    || msg.contains("create")
                    || msg.contains("make")
                    || msg.contains("implement")
                    || msg.contains("add")
                {
                    "build".to_string()
                } else if msg.contains("review") || msg.contains("check") || msg.contains("test") {
                    "verify".to_string()
                } else if msg.len() < 30 {
                    "short_instruction".to_string()
                } else {
                    "complex_request".to_string()
                }
            })
            .collect();

        let topic_shifts = if total > 1 {
            phases.windows(2).filter(|w| w[0] != w[1]).count()
        } else {
            0
        };

        let scores: Vec<f64> = self
            .turns
            .iter()
            .filter_map(|t| t.user_satisfaction)
            .collect();
        let sentiment_trend = if scores.len() >= 2 {
            let n = scores.len() as f64;
            let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
            let sum_y: f64 = scores.iter().sum();
            let sum_xy: f64 = scores.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
            let sum_xx: f64 = (0..scores.len()).map(|i| (i as f64) * (i as f64)).sum();
            let denom = n * sum_xx - sum_x * sum_x;
            if denom.abs() > 1e-10 {
                (n * sum_xy - sum_x * sum_y) / denom
            } else {
                0.0
            }
        } else {
            0.0
        };

        let mut repetitions: Vec<(String, usize)> = vec![];
        let mut seen: HashMap<String, usize> = HashMap::new();
        for turn in &self.turns {
            let key = turn.user_message.chars().take(30).collect::<String>();
            *seen.entry(key).or_insert(0) += 1;
        }
        for (msg, count) in seen {
            if count >= 2 {
                repetitions.push((msg, count));
            }
        }

        let mut tool_density = Vec::new();
        let mut quality_per_phase = Vec::new();
        let unique_phases: std::collections::HashSet<&str> =
            phases.iter().map(|s| s.as_str()).collect();
        for phase in &unique_phases {
            let phase_turns: Vec<usize> = phases
                .iter()
                .enumerate()
                .filter(|(_, p)| *p == phase)
                .map(|(i, _)| i)
                .collect();
            let count = phase_turns.len();
            if count > 0 {
                let tools = phase_turns
                    .iter()
                    .filter(|&&i| !self.turns[i].tools_used.is_empty())
                    .count();
                tool_density.push(tools as f64 / count as f64);
                let sat_sum: f64 = phase_turns
                    .iter()
                    .filter_map(|&i| self.turns[i].user_satisfaction)
                    .sum();
                quality_per_phase.push(sat_sum / count as f64);
            }
        }

        DialogueArcAnalysis {
            phases,
            sentiment_trend,
            topic_shifts,
            repetitions,
            tool_density,
            engagement_trend: sentiment_trend,
            quality_per_phase,
        }
    }

    pub fn detect_topic_shifts(&self) -> Vec<usize> {
        if self.topic_shifts == 0 || self.turns.len() < 2 {
            return Vec::new();
        }
        let spacing = self.turns.len() / (self.topic_shifts + 1);
        (1..=self.topic_shifts)
            .filter_map(|i| {
                let idx = i * spacing;
                if idx < self.turns.len() {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn detect_user_satisfaction_trend(&self) -> f64 {
        let scores: Vec<f64> = self
            .turns
            .iter()
            .filter_map(|t| t.user_satisfaction)
            .collect();
        if scores.len() < 2 {
            return 0.0;
        }
        let n = scores.len() as f64;
        let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
        let sum_y: f64 = scores.iter().sum();
        let sum_xy: f64 = scores.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_xx: f64 = (0..scores.len()).map(|i| (i as f64) * (i as f64)).sum();
        (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
    }
}

impl Default for ConversationObserver {
    fn default() -> Self {
        Self::new(100)
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
