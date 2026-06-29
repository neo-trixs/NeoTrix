use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimePeriod {
    pub id: String,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub themes: Vec<String>,
    pub emotional_valence: f64,
    pub significance: f64,
    pub general_events: Vec<GeneralEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralEvent {
    pub id: String,
    pub summary: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub event_type: String,
    pub importance: f64,
    pub specific_details: Vec<EventDetail>,
    pub vsa_fingerprint: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDetail {
    pub id: String,
    pub description: String,
    pub timestamp: u64,
    pub confidence: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalMemory {
    pub lifetime_periods: VecDeque<LifetimePeriod>,
    pub max_periods: usize,
    pub current_period_id: Option<String>,
}

impl Default for HierarchicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalMemory {
    pub fn new() -> Self {
        Self {
            lifetime_periods: VecDeque::new(),
            max_periods: 20,
            current_period_id: None,
        }
    }

    pub fn start_period(&mut self, name: &str, themes: Vec<String>) {
        if let Some(ref current_id) = self.current_period_id {
            if let Some(period) = self
                .lifetime_periods
                .iter_mut()
                .find(|p| p.id == *current_id)
            {
                period.end_time = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                );
            }
        }
        let id = format!(
            "period_{}_{}",
            name.to_lowercase().replace(' ', "_"),
            self.lifetime_periods.len()
        );
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let period = LifetimePeriod {
            id: id.clone(),
            name: name.to_string(),
            start_time: now,
            end_time: None,
            themes,
            emotional_valence: 0.0,
            significance: 0.5,
            general_events: Vec::new(),
        };
        self.lifetime_periods.push_back(period);
        if self.lifetime_periods.len() > self.max_periods {
            self.lifetime_periods.pop_front();
        }
        self.current_period_id = Some(id);
    }

    pub fn end_current_period(&mut self) {
        if let Some(ref current_id) = self.current_period_id.clone() {
            if let Some(period) = self
                .lifetime_periods
                .iter_mut()
                .find(|p| p.id == *current_id)
            {
                period.end_time = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                );
            }
            self.current_period_id = None;
        }
    }

    pub fn add_event(
        &mut self,
        period_id: &str,
        summary: &str,
        event_type: &str,
        importance: f64,
        details: Vec<EventDetail>,
    ) {
        if let Some(period) = self.lifetime_periods.iter_mut().find(|p| p.id == period_id) {
            let event_id = format!(
                "evt_{}_{}",
                summary.to_lowercase().replace(' ', "_"),
                period.general_events.len()
            );
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let vsa_fingerprint = format!("vsa_{}", event_id).into_bytes();
            let event = GeneralEvent {
                id: event_id,
                summary: summary.to_string(),
                timestamp: now,
                duration_ms: 0,
                event_type: event_type.to_string(),
                importance: importance.clamp(0.0, 1.0),
                specific_details: details,
                vsa_fingerprint,
            };
            period.general_events.push(event);
            period.significance = period
                .general_events
                .iter()
                .map(|e| e.importance)
                .sum::<f64>()
                / period.general_events.len() as f64;
        }
    }

    pub fn add_detail(&mut self, event_id: &str, description: &str, tags: Vec<String>) {
        for period in &mut self.lifetime_periods {
            if let Some(event) = period.general_events.iter_mut().find(|e| e.id == event_id) {
                let detail_id = format!("det_{}_{}", event_id, event.specific_details.len());
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let detail = EventDetail {
                    id: detail_id,
                    description: description.to_string(),
                    timestamp: now,
                    confidence: 0.5,
                    tags,
                };
                event.specific_details.push(detail);
                return;
            }
        }
    }

    pub fn query_by_time_range(&self, start: u64, end: u64) -> Vec<&GeneralEvent> {
        self.lifetime_periods
            .iter()
            .flat_map(|p| &p.general_events)
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    pub fn query_by_theme(&self, theme: &str) -> Vec<&GeneralEvent> {
        let theme_lower = theme.to_lowercase();
        self.lifetime_periods
            .iter()
            .flat_map(|p| &p.general_events)
            .filter(|e| {
                e.event_type.to_lowercase() == theme_lower
                    || e.summary.to_lowercase().contains(&theme_lower)
            })
            .collect()
    }

    pub fn query_by_emotional_valence(
        &self,
        min_valence: f64,
        max_valence: f64,
    ) -> Vec<&GeneralEvent> {
        self.lifetime_periods
            .iter()
            .filter(|p| p.emotional_valence >= min_valence && p.emotional_valence <= max_valence)
            .flat_map(|p| &p.general_events)
            .collect()
    }

    pub fn top_level_summary(&self) -> String {
        if self.lifetime_periods.is_empty() {
            return "No lifetime periods recorded.".to_string();
        }
        let parts: Vec<String> = self
            .lifetime_periods
            .iter()
            .map(|p| {
                let end_str = match p.end_time {
                    Some(_) => "completed",
                    None => "ongoing",
                };
                let event_count = p.general_events.len();
                format!(
                    "Period '{}': {} events, valence={:.2}, significance={:.2} ({})",
                    p.name, event_count, p.emotional_valence, p.significance, end_str
                )
            })
            .collect();
        parts.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAttentionConfig {
    pub short_term_window_ms: u64,
    pub medium_term_window_ms: u64,
    pub long_term_window_ms: u64,
    pub short_term_weight: f64,
    pub medium_term_weight: f64,
    pub long_term_weight: f64,
}

impl Default for TemporalAttentionConfig {
    fn default() -> Self {
        Self {
            short_term_window_ms: 300_000,
            medium_term_window_ms: 3_600_000,
            long_term_window_ms: 86_400_000,
            short_term_weight: 0.6,
            medium_term_weight: 0.3,
            long_term_weight: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalAttentionResult {
    pub short_term_items: Vec<(String, f64)>,
    pub medium_term_items: Vec<(String, f64)>,
    pub long_term_items: Vec<(String, f64)>,
    pub fused_attention: Vec<(String, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiResolutionTemporalAttention {
    pub config: TemporalAttentionConfig,
    pub memory: HierarchicalMemory,
    pub recent_access: VecDeque<(u64, String)>,
}

impl MultiResolutionTemporalAttention {
    pub fn new(memory: HierarchicalMemory) -> Self {
        Self {
            config: TemporalAttentionConfig::default(),
            memory,
            recent_access: VecDeque::new(),
        }
    }

    pub fn attend(&self, now: u64) -> TemporalAttentionResult {
        let all_events: Vec<&GeneralEvent> = self
            .memory
            .lifetime_periods
            .iter()
            .flat_map(|p| &p.general_events)
            .collect();

        let short_term: Vec<(String, f64)> = all_events
            .iter()
            .filter(|e| now.saturating_sub(e.timestamp) <= self.config.short_term_window_ms)
            .map(|e| {
                let age = now.saturating_sub(e.timestamp);
                let recency = Self::decay_function(age, self.config.short_term_window_ms / 2);
                (e.summary.clone(), e.importance * recency)
            })
            .collect();

        let medium_term: Vec<(String, f64)> = all_events
            .iter()
            .filter(|e| {
                let age = now.saturating_sub(e.timestamp);
                age <= self.config.medium_term_window_ms && age > self.config.short_term_window_ms
            })
            .map(|e| {
                let age = now.saturating_sub(e.timestamp);
                let recency = Self::decay_function(age, self.config.medium_term_window_ms / 2);
                (e.summary.clone(), e.importance * recency)
            })
            .collect();

        let long_term: Vec<(String, f64)> = all_events
            .iter()
            .filter(|e| {
                let age = now.saturating_sub(e.timestamp);
                age > self.config.medium_term_window_ms
            })
            .map(|e| {
                let period = self
                    .memory
                    .lifetime_periods
                    .iter()
                    .find(|p| p.general_events.iter().any(|ge| ge.id == e.id));
                let period_valence = period.map(|p| p.emotional_valence).unwrap_or(0.0);
                let period_sig = period.map(|p| p.significance).unwrap_or(0.5);
                let score = e.importance * period_sig * (period_valence.abs() + 0.5);
                (e.summary.clone(), score)
            })
            .collect();

        let fused = self.fuse_attention(&short_term, &medium_term, &long_term);

        TemporalAttentionResult {
            short_term_items: short_term,
            medium_term_items: medium_term,
            long_term_items: long_term,
            fused_attention: fused,
        }
    }

    fn fuse_attention(
        &self,
        short: &[(String, f64)],
        medium: &[(String, f64)],
        long: &[(String, f64)],
    ) -> Vec<(String, f64)> {
        let mut score_map: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for (summary, score) in short {
            *score_map.entry(summary.clone()).or_insert(0.0) +=
                score * self.config.short_term_weight;
        }
        for (summary, score) in medium {
            *score_map.entry(summary.clone()).or_insert(0.0) +=
                score * self.config.medium_term_weight;
        }
        for (summary, score) in long {
            *score_map.entry(summary.clone()).or_insert(0.0) +=
                score * self.config.long_term_weight;
        }
        let mut items: Vec<(String, f64)> = score_map.into_iter().collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    pub fn record_access(&mut self, event_id: &str, timestamp: u64) {
        self.recent_access
            .push_back((timestamp, event_id.to_string()));
        if self.recent_access.len() > 1000 {
            self.recent_access.pop_front();
        }
    }

    pub fn decay_function(age_ms: u64, half_life_ms: u64) -> f64 {
        if half_life_ms == 0 {
            return 0.0;
        }
        (-(age_ms as f64) / (half_life_ms as f64)) * std::f64::consts::LN_2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory_with_events() -> HierarchicalMemory {
        let mut mem = HierarchicalMemory::new();
        mem.start_period(
            "Test Period",
            vec!["testing".to_string(), "debugging".to_string()],
        );
        let pid = mem.current_period_id.clone().unwrap();
        mem.add_event(&pid, "Fixed critical bug", "debugging", 0.9, vec![]);
        mem.add_event(&pid, "Designed new feature", "planning", 0.7, vec![]);
        mem.add_event(&pid, "Reviewed code", "reflection", 0.3, vec![]);
        mem
    }

    #[test]
    fn test_start_end_period() {
        let mut mem = HierarchicalMemory::new();
        assert!(mem.current_period_id.is_none());
        mem.start_period("Research Phase", vec!["analysis".to_string()]);
        assert!(mem.current_period_id.is_some());
        assert_eq!(mem.lifetime_periods.len(), 1);
        mem.end_current_period();
        assert!(mem.current_period_id.is_none());
        if let Some(period) = mem.lifetime_periods.back() {
            assert!(period.end_time.is_some());
        }
    }

    #[test]
    fn test_add_event_to_period() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Coding Session", vec!["rust".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        mem.add_event(&pid, "Implemented trait", "coding", 0.8, vec![]);
        let period = mem.lifetime_periods.back().unwrap();
        assert_eq!(period.general_events.len(), 1);
        assert_eq!(period.general_events[0].summary, "Implemented trait");
    }

    #[test]
    fn test_add_detail_to_event() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Debugging", vec!["fix".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        mem.add_event(&pid, "Fixed segfault", "debugging", 0.9, vec![]);
        let eid = mem.lifetime_periods.back().unwrap().general_events[0]
            .id
            .clone();
        mem.add_detail(
            &eid,
            "Null pointer in renderer",
            vec!["pointer".to_string()],
        );
        let event = &mem.lifetime_periods.back().unwrap().general_events[0];
        assert_eq!(event.specific_details.len(), 1);
        assert_eq!(
            event.specific_details[0].description,
            "Null pointer in renderer"
        );
    }

    #[test]
    fn test_query_by_time_range() {
        let mem = make_memory_with_events();
        let all: Vec<&GeneralEvent> = mem
            .lifetime_periods
            .iter()
            .flat_map(|p| &p.general_events)
            .collect();
        let min_ts = all.iter().map(|e| e.timestamp).min().unwrap_or(0);
        let max_ts = all.iter().map(|e| e.timestamp).max().unwrap_or(0);
        let results = mem.query_by_time_range(min_ts, max_ts);
        assert_eq!(results.len(), all.len());
        let empty = mem.query_by_time_range(0, 1);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_query_by_theme() {
        let mem = make_memory_with_events();
        let debugging = mem.query_by_theme("debugging");
        assert_eq!(debugging.len(), 1);
        assert_eq!(debugging[0].summary, "Fixed critical bug");
        let reflection = mem.query_by_theme("reflection");
        assert_eq!(reflection.len(), 1);
    }

    #[test]
    fn test_query_by_emotional_valence() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Happy", vec!["joy".to_string()]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            p.emotional_valence = 0.8;
        }
        mem.start_period("Sad", vec!["grief".to_string()]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            p.emotional_valence = -0.5;
        }
        let pid_happy = "happy_0".to_string();
        let pid_sad = "sad_1".to_string();
        mem.add_event(&pid_happy, "Good times", "joy", 0.8, vec![]);
        mem.add_event(&pid_sad, "Bad times", "grief", 0.6, vec![]);
        let high = mem.query_by_emotional_valence(0.5, 1.0);
        assert_eq!(high.len(), 1);
        let low = mem.query_by_emotional_valence(-1.0, 0.0);
        assert_eq!(low.len(), 1);
    }

    #[test]
    fn test_top_level_summary() {
        let empty = HierarchicalMemory::new();
        assert!(empty.top_level_summary().contains("No lifetime periods"));
        let mem = make_memory_with_events();
        let summary = mem.top_level_summary();
        assert!(summary.contains("Test Period"));
        assert!(summary.contains("3 events"));
    }

    #[test]
    fn test_temporal_attention_short_term() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Recent", vec!["now".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        let now = 100_000u64;
        mem.add_event(&pid, "Just happened", "instant", 0.9, vec![]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            if let Some(e) = p.general_events.last_mut() {
                e.timestamp = now;
            }
        }
        let attn = MultiResolutionTemporalAttention::new(mem);
        let result = attn.attend(now);
        assert_eq!(result.short_term_items.len(), 1);
        assert!((result.short_term_items[0].1 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_temporal_attention_medium_term() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Mid", vec!["middle".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        let now = 100_000u64;
        mem.add_event(&pid, "Medium event", "mid", 0.7, vec![]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            if let Some(e) = p.general_events.last_mut() {
                e.timestamp = now - 400_000;
            }
        }
        let attn = MultiResolutionTemporalAttention::new(mem);
        let result = attn.attend(now);
        assert_eq!(result.short_term_items.len(), 0);
        assert_eq!(result.medium_term_items.len(), 1);
    }

    #[test]
    fn test_temporal_attention_long_term() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Old", vec!["ancient".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        let now = 100_000u64;
        mem.add_event(&pid, "Ancient event", "old", 0.5, vec![]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            if let Some(e) = p.general_events.last_mut() {
                e.timestamp = now - 10_000_000;
            }
            p.emotional_valence = 0.9;
            p.significance = 0.8;
        }
        let attn = MultiResolutionTemporalAttention::new(mem);
        let result = attn.attend(now);
        assert_eq!(result.long_term_items.len(), 1);
        let expected = 0.5 * 0.8 * (0.9 + 0.5);
        assert!((result.long_term_items[0].1 - expected).abs() < 0.01);
    }

    #[test]
    fn test_fused_attention_weighted() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("Mixed", vec!["all".to_string()]);
        let pid = mem.current_period_id.clone().unwrap();
        let now = 100_000u64;
        mem.add_event(&pid, "Recent item", "short", 1.0, vec![]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            if let Some(e) = p.general_events.last_mut() {
                e.timestamp = now;
            }
        }
        mem.add_event(&pid, "Old item", "long", 0.8, vec![]);
        if let Some(p) = mem.lifetime_periods.back_mut() {
            if let Some(e) = p.general_events.last_mut() {
                e.timestamp = now - 10_000_000;
            }
            p.emotional_valence = 0.5;
            p.significance = 0.7;
        }
        let attn = MultiResolutionTemporalAttention::new(mem);
        let result = attn.attend(now);
        assert!(result.short_term_items.len() >= 1);
        assert!(result.long_term_items.len() >= 1);
        assert_eq!(result.fused_attention.len(), 2);
        let fused_recent = result
            .fused_attention
            .iter()
            .find(|(s, _)| s == "Recent item")
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        let fused_old = result
            .fused_attention
            .iter()
            .find(|(s, _)| s == "Old item")
            .map(|(_, s)| *s)
            .unwrap_or(0.0);
        assert!(fused_recent > 0.0);
        assert!(fused_old > 0.0);
        let short_weight = attn.config.short_term_weight;
        let long_weight = attn.config.long_term_weight;
        let expected_recent = 1.0 * short_weight;
        assert!((fused_recent - expected_recent).abs() < 0.01);
        let expected_old = 0.8 * 0.7 * (0.5 + 0.5) * long_weight;
        assert!((fused_old - expected_old).abs() < 0.01);
    }

    #[test]
    fn test_decay_function() {
        let result = MultiResolutionTemporalAttention::decay_function(0, 1000);
        assert!((result - 0.0).abs() < 1e-9);
        let result_full = MultiResolutionTemporalAttention::decay_function(1000, 1000);
        assert!((result_full - (-std::f64::consts::LN_2)).abs() < 1e-9);
        let result_half = MultiResolutionTemporalAttention::decay_function(500, 1000);
        assert!((result_half - (-0.5 * std::f64::consts::LN_2)).abs() < 1e-9);
    }

    #[test]
    fn test_record_access() {
        let mem = HierarchicalMemory::new();
        let mut attn = MultiResolutionTemporalAttention::new(mem);
        attn.record_access("evt_1", 100);
        attn.record_access("evt_2", 200);
        assert_eq!(attn.recent_access.len(), 2);
        assert_eq!(attn.recent_access[0].1, "evt_1");
        assert_eq!(attn.recent_access[1].1, "evt_2");
    }

    #[test]
    fn test_empty_memory_attention() {
        let mem = HierarchicalMemory::new();
        let attn = MultiResolutionTemporalAttention::new(mem);
        let result = attn.attend(1000);
        assert!(result.short_term_items.is_empty());
        assert!(result.medium_term_items.is_empty());
        assert!(result.long_term_items.is_empty());
        assert!(result.fused_attention.is_empty());
    }

    #[test]
    fn test_multiple_periods_summary() {
        let mut mem = HierarchicalMemory::new();
        mem.start_period("First", vec!["a".to_string()]);
        let pid1 = mem.current_period_id.clone().unwrap();
        mem.end_current_period();
        mem.start_period("Second", vec!["b".to_string()]);
        let pid2 = mem.current_period_id.clone().unwrap();
        mem.add_event(&pid1, "Event in first", "type_a", 0.5, vec![]);
        mem.add_event(&pid2, "Event in second", "type_b", 0.8, vec![]);
        let summary = mem.top_level_summary();
        assert!(summary.contains("First"));
        assert!(summary.contains("Second"));
    }

    #[test]
    fn test_max_periods_enforced() {
        let mut mem = HierarchicalMemory::new();
        mem.max_periods = 3;
        for i in 0..10 {
            mem.start_period(&format!("Period {}", i), vec![]);
        }
        assert_eq!(mem.lifetime_periods.len(), 3);
    }
}
