use crate::agents::memory_stream::{MemoryKind, MemoryNode, MemoryStream};
use std::collections::HashMap;

struct Theme {
    keyword: String,
    memory_ids: Vec<u64>,
    frequency: usize,
}

pub struct ReflectionEngine {
    importance_accumulator: f32,
    reflection_threshold: f32,
    max_context_memories: usize,
    insights_per_reflection: usize,
}

impl ReflectionEngine {
    pub fn new() -> Self {
        Self {
            importance_accumulator: 0.0,
            reflection_threshold: 15.0,
            max_context_memories: 20,
            insights_per_reflection: 3,
        }
    }

    pub fn on_new_memory(&mut self, importance: f32) -> bool {
        self.importance_accumulator += importance;
        self.importance_accumulator >= self.reflection_threshold
    }

    pub fn reflect(
        &mut self,
        memory: &MemoryStream,
        agent_id: &str,
        current_tick: u64,
    ) -> Vec<MemoryNode> {
        self.importance_accumulator = 0.0;

        let recent = memory.recent(self.max_context_memories);
        let themes = self.find_themes(recent);

        let mut insights = Vec::new();
        for theme in themes.iter().take(self.insights_per_reflection) {
            let insight = self.synthesize_insight(theme, recent, agent_id, current_tick);
            insights.push(insight);
        }

        insights
    }

    fn find_themes(&self, memories: &[MemoryNode]) -> Vec<Theme> {
        let mut keyword_map: HashMap<String, Vec<u64>> = HashMap::new();

        for node in memories {
            for kw in &node.keywords {
                keyword_map
                    .entry(kw.clone())
                    .or_default()
                    .push(node.id);
            }
        }

        let mut themes: Vec<Theme> = keyword_map
            .into_iter()
            .map(|(keyword, memory_ids)| {
                let frequency = memory_ids.len();
                Theme {
                    keyword,
                    memory_ids,
                    frequency,
                }
            })
            .collect();

        themes.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        themes
    }

    fn synthesize_insight(
        &self,
        theme: &Theme,
        _context: &[MemoryNode],
        agent_id: &str,
        current_tick: u64,
    ) -> MemoryNode {
        let description = format!(
            "Pattern observed: \"{}\" appeared in {} recent events",
            theme.keyword, theme.frequency
        );

        MemoryNode {
            id: 0,
            kind: MemoryKind::Reflection,
            agent_id: agent_id.to_string(),
            created_tick: current_tick,
            last_accessed_tick: current_tick,
            description,
            importance: 9.0,
            keywords: vec![theme.keyword.clone()],
            citations: theme.memory_ids.clone(),
            embedding: None,
        }
    }

    pub fn accumulator(&self) -> f32 {
        self.importance_accumulator
    }

    pub fn threshold(&self) -> f32 {
        self.reflection_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory(kind: MemoryKind, importance: f32, tick: u64, keywords: Vec<String>) -> MemoryNode {
        MemoryNode {
            id: 0,
            kind,
            agent_id: "test_agent".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: "test".into(),
            importance,
            keywords,
            citations: vec![],
            embedding: None,
        }
    }

    #[test]
    fn accumulation_triggers_at_threshold() {
        let mut engine = ReflectionEngine::new();
        assert!(!engine.on_new_memory(5.0));
        assert!(!engine.on_new_memory(5.0));
        assert!(engine.on_new_memory(5.0));
    }

    #[test]
    fn reflection_resets_accumulator() {
        let mut engine = ReflectionEngine::new();
        engine.on_new_memory(15.0);
        assert!(engine.accumulator() >= engine.threshold());

        let stream = MemoryStream::new(100);
        engine.reflect(&stream, "agent_0", 0);

        assert_eq!(engine.accumulator(), 0.0);
    }

    #[test]
    fn reflection_does_not_trigger_below_threshold() {
        let mut engine = ReflectionEngine::new();
        assert!(!engine.on_new_memory(3.0));
        assert!(!engine.on_new_memory(3.0));
        assert!(!engine.on_new_memory(3.0));
    }

    #[test]
    fn theme_finding_with_overlapping_keywords() {
        let engine = ReflectionEngine::new();
        let mut stream = MemoryStream::new(100);

        stream.add(make_memory(MemoryKind::Observation, 5.0, 0, vec!["combat".into(), "enemy".into()]));
        stream.add(make_memory(MemoryKind::Observation, 5.0, 1, vec!["combat".into(), "ally".into()]));
        stream.add(make_memory(MemoryKind::Observation, 5.0, 2, vec!["combat".into()]));

        let recent = stream.recent(10);
        let themes = engine.find_themes(recent);

        assert!(!themes.is_empty());
        assert_eq!(themes[0].keyword, "combat");
        assert_eq!(themes[0].frequency, 3);
    }

    #[test]
    fn reflection_node_has_correct_kind_and_citations() {
        let mut engine = ReflectionEngine::new();
        let mut stream = MemoryStream::new(100);

        let id1 = stream.add(make_memory(MemoryKind::Observation, 5.0, 0, vec!["combat".into()]));
        let id2 = stream.add(make_memory(MemoryKind::Observation, 5.0, 1, vec!["combat".into()]));
        let _id3 = stream.add(make_memory(MemoryKind::Observation, 5.0, 2, vec!["combat".into()]));

        let insights = engine.reflect(&stream, "agent_0", 10);

        assert_eq!(insights.len(), 1);
        let insight = &insights[0];
        assert_eq!(insight.kind, MemoryKind::Reflection);
        assert_eq!(insight.importance, 9.0);
        assert!(insight.citations.contains(&id1));
        assert!(insight.citations.contains(&id2));
        assert!(insight.description.contains("combat"));
    }

    #[test]
    fn empty_memory_stream_yields_no_insights() {
        let mut engine = ReflectionEngine::new();
        let stream = MemoryStream::new(100);

        let insights = engine.reflect(&stream, "agent_0", 0);
        assert!(insights.is_empty());
    }

    #[test]
    fn multiple_themes_sorted_by_frequency() {
        let mut engine = ReflectionEngine::new();
        let mut stream = MemoryStream::new(100);

        stream.add(make_memory(MemoryKind::Observation, 5.0, 0, vec!["combat".into(), "rare_kw".into()]));
        stream.add(make_memory(MemoryKind::Observation, 5.0, 1, vec!["combat".into()]));
        stream.add(make_memory(MemoryKind::Observation, 5.0, 2, vec!["combat".into()]));

        let recent = stream.recent(10);
        let themes = engine.find_themes(recent);

        assert!(themes.len() >= 2);
        assert_eq!(themes[0].keyword, "combat");
        assert_eq!(themes[0].frequency, 3);
    }

    #[test]
    fn insight_count_limited_by_insights_per_reflection() {
        let mut engine = ReflectionEngine::new();
        let mut stream = MemoryStream::new(100);

        for i in 0..10 {
            stream.add(make_memory(
                MemoryKind::Observation,
                5.0,
                i,
                vec![format!("kw_{}", i)],
            ));
        }

        let insights = engine.reflect(&stream, "agent_0", 10);
        assert!(insights.len() <= engine.insights_per_reflection);
    }
}
