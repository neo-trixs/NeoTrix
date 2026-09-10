use crate::agents::memory_stream::{MemoryKind, MemoryNode, MemoryStream};
use crate::agents::planning::{Goal, GoalStatus, PlanningStack};
use crate::agents::sim_agent::AgentAction;
use crate::agents::action_awareness::ActionAwareness;
use crate::foundation::math_bridge::Vec2;

/// ThoughtGeneration — Adapter 3
///
/// Before action selection, generates a "thought" that summarizes:
/// - Current state summary
/// - Recent memory highlights
/// - Goal progress
///
/// The thought is stored in MemoryStream as MemoryKind::Thought (new variant).
/// Implements the ReAct (Reasoning + Acting) loop where agents think
/// step-by-step before acting.
///
/// Sources:
/// - ReAct Framework (Yao et al. 2023): Thought→Action→Observation loop
/// - Generative Agents: natural language reasoning traces
pub struct ThoughtGeneration {
    max_recent_memories: usize,
    thought_importance: f32,
}

impl ThoughtGeneration {
    pub fn new() -> Self {
        Self {
            max_recent_memories: 5,
            thought_importance: 7.0,
        }
    }

    /// Generate a thought summarizing current state, memory, and goals.
    /// The thought is returned as a MemoryNode to be stored in MemoryStream.
    pub fn generate_thought(
        &self,
        agent_id: &str,
        current_tick: u64,
        position: Vec2,
        energy: f32,
        health: f32,
        hunger: f32,
        planning: &PlanningStack,
        memory: &MemoryStream,
        awareness: &ActionAwareness,
    ) -> MemoryNode {
        // 1. Current state summary
        let state_summary = format!(
            "energy={:.0} health={:.0} hunger={:.0} pos=({:.0},{:.0})",
            energy, health, hunger, position.x, position.y
        );

        // 2. Goal progress
        let goal_summary = self.summarize_goals(planning);

        // 3. Recent memory highlights
        let memory_summary = self.summarize_recent_memories(memory);

        // 4. World model confidence
        let confidence = awareness.confidence();

        // 5. Compose thought
        let description = format!(
            "[Thought@{}] State: {} | Goals: {} | Memory: {} | Confidence: {:.2}",
            current_tick, state_summary, goal_summary, memory_summary, confidence
        );

        // 6. Extract keywords for retrieval
        let mut keywords = vec!["thought".to_string(), "reflection".to_string()];
        if energy < 30.0 { keywords.push("low_energy".to_string()); }
        if health < 40.0 { keywords.push("low_health".to_string()); }
        if hunger > 60.0 { keywords.push("hungry".to_string()); }
        for goal in planning.goals() {
            if goal.status == GoalStatus::Active {
                keywords.push(goal.name.clone());
            }
        }

        MemoryNode {
            id: 0,
            kind: MemoryKind::Thought,
            agent_id: agent_id.to_string(),
            created_tick: current_tick,
            last_accessed_tick: current_tick,
            description,
            importance: self.thought_importance,
            keywords,
            citations: vec![],
            embedding: None,
        }
    }

    /// Summarize current goals into a concise string.
    fn summarize_goals(&self, planning: &PlanningStack) -> String {
        let active: Vec<&Goal> = planning.goals().iter()
            .filter(|g| g.status == GoalStatus::Active)
            .collect();

        if active.is_empty() {
            "no active goals".to_string()
        } else {
            let goal_strs: Vec<String> = active.iter().map(|g| {
                format!("{}({:.1})", g.name, g.priority)
            }).collect();
            goal_strs.join(", ")
        }
    }

    /// Summarize recent memories into keywords.
    fn summarize_recent_memories(&self, memory: &MemoryStream) -> String {
        let recent = memory.recent(self.max_recent_memories);
        if recent.is_empty() {
            "no recent memories".to_string()
        } else {
            let kinds: Vec<String> = recent.iter().map(|m| format!("{:?}", m.kind)).collect();
            let unique: Vec<&str> = kinds.iter().map(|s| s.as_str()).collect::<std::collections::HashSet<_>>()
                .into_iter().collect();
            unique.join(", ")
        }
    }
}

impl Default for ThoughtGeneration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::memory_stream::{MemoryKind, MemoryNode, MemoryStream};
    use crate::agents::planning::{Goal, GoalStatus, GoalType, PlanningStack};
    use crate::agents::action_awareness::ActionAwareness;
    use crate::agents::sim_agent::AgentAction;
    use crate::foundation::math_bridge::Vec2;

    fn make_memory(kind: MemoryKind, tick: u64) -> MemoryNode {
        MemoryNode {
            id: 0,
            kind,
            agent_id: "test".into(),
            created_tick: tick,
            last_accessed_tick: tick,
            description: "test".into(),
            importance: 5.0,
            keywords: vec![],
            citations: vec![],
            embedding: None,
        }
    }

    #[test]
    fn new_default() {
        let tg = ThoughtGeneration::new();
        assert_eq!(tg.max_recent_memories, 5);
    }

    #[test]
    fn generate_thought_with_no_goals() {
        let tg = ThoughtGeneration::new();
        let ps = PlanningStack::new();
        let ms = MemoryStream::new(100);
        let aa = ActionAwareness::new();

        let thought = tg.generate_thought(
            "agent_0", 10, Vec2::new(50.0, 50.0),
            80.0, 90.0, 20.0, &ps, &ms, &aa,
        );

        assert_eq!(thought.kind, MemoryKind::Thought);
        assert!(thought.description.contains("no active goals"));
        assert!(thought.description.contains("energy=80"));
        assert!(thought.keywords.contains(&"thought".to_string()));
    }

    #[test]
    fn generate_thought_with_active_goals() {
        let tg = ThoughtGeneration::new();
        let mut ps = PlanningStack::new();
        ps.add_goal(Goal {
            name: "eat".to_string(),
            priority: 0.8,
            goal_type: GoalType::Survival,
            status: GoalStatus::Active,
            subgoals: vec![],
            created_tick: 0,
            deadline_tick: None,
            expected_actions: vec![AgentAction::Think],
        });

        let ms = MemoryStream::new(100);
        let aa = ActionAwareness::new();

        let thought = tg.generate_thought(
            "agent_0", 10, Vec2::new(50.0, 50.0),
            30.0, 70.0, 70.0, &ps, &ms, &aa,
        );

        assert!(thought.description.contains("eat"));
        assert!(thought.keywords.contains(&"eat".to_string()));
        assert!(thought.keywords.contains(&"low_energy".to_string()));
        assert!(thought.keywords.contains(&"hungry".to_string()));
    }

    #[test]
    fn generate_thought_with_memories() {
        let tg = ThoughtGeneration::new();
        let ps = PlanningStack::new();
        let mut ms = MemoryStream::new(100);
        ms.add(make_memory(MemoryKind::Observation, 5));
        ms.add(make_memory(MemoryKind::Reflection, 6));
        let aa = ActionAwareness::new();

        let thought = tg.generate_thought(
            "agent_0", 10, Vec2::new(50.0, 50.0),
            80.0, 90.0, 20.0, &ps, &ms, &aa,
        );

        assert!(!thought.description.contains("no recent memories"));
    }

    #[test]
    fn generate_thought_keywords_include_state_signals() {
        let tg = ThoughtGeneration::new();
        let ps = PlanningStack::new();
        let ms = MemoryStream::new(100);
        let aa = ActionAwareness::new();

        let thought = tg.generate_thought(
            "agent_0", 10, Vec2::new(50.0, 50.0),
            10.0, 20.0, 80.0, &ps, &ms, &aa,
        );

        assert!(thought.keywords.contains(&"low_energy".to_string()));
        assert!(thought.keywords.contains(&"low_health".to_string()));
        assert!(thought.keywords.contains(&"hungry".to_string()));
    }
}
