use crate::agents::SimAgent;

pub struct AgentInspector {
    pub selected_id: Option<String>,
    pub show_path: bool,
    pub show_memory: bool,
    pub show_relationships: bool,
}

impl AgentInspector {
    pub fn new() -> Self {
        Self {
            selected_id: None,
            show_path: false,
            show_memory: false,
            show_relationships: false,
        }
    }

    pub fn select(&mut self, agent_id: &str) {
        self.selected_id = Some(agent_id.to_string());
    }

    pub fn deselect(&mut self) {
        self.selected_id = None;
    }

    pub fn is_selected(&self, agent_id: &str) -> bool {
        self.selected_id.as_deref() == Some(agent_id)
    }

    pub fn render_info(&self, agent: &SimAgent) -> String {
        let mut info = format!("=== Agent #{} ===\n", agent.core.id);
        info.push_str(&format!("Position: ({:.1}, {:.1})\n", agent.core.position.x, agent.core.position.y));
        info.push_str(&format!("Energy: {:.1}\n", agent.core.energy));
        info.push_str(&format!("Health: {:.1}\n", agent.core.health));
        info.push_str(&format!("Hunger: {:.1}\n", agent.core.hunger));
        info.push_str(&format!("Age: {}\n", agent.core.age));
        info.push_str(&format!("Alive: {}\n", agent.core.alive));
        info.push_str(&format!("Phi: {:.3}\n", agent.phi));
        
        info.push_str("\n--- Needs (Maslow) ---\n");
        if agent.needs.len() >= 5 {
            info.push_str(&format!("Physiological: {:.2}\n", agent.needs[0]));
            info.push_str(&format!("Safety: {:.2}\n", agent.needs[1]));
            info.push_str(&format!("Love: {:.2}\n", agent.needs[2]));
            info.push_str(&format!("Esteem: {:.2}\n", agent.needs[3]));
            info.push_str(&format!("Self-Actualization: {:.2}\n", agent.needs[4]));
        }
        
        info.push_str("\n--- Personality ---\n");
        info.push_str(&format!("Openness: {:.2}\n", agent.personality.openness));
        info.push_str(&format!("Sociability: {:.2}\n", agent.personality.sociability));
        info.push_str(&format!("Aggression: {:.2}\n", agent.personality.aggression));
        info.push_str(&format!("Cooperativeness: {:.2}\n", agent.personality.cooperativeness));
        info.push_str(&format!("Curiosity: {:.2}\n", agent.personality.curiosity));
        
        info.push_str(&format!("\n--- Skills ({}) ---\n", agent.skills.len()));
        if !agent.skills.is_empty() {
            info.push_str(&agent.skills.join(", "));
            info.push('\n');
        }
        
        info
    }

    pub fn render_compact(&self, agent: &SimAgent) -> String {
        format!(
            "#{} | ({:.0},{:.0}) | E:{:.0} | H:{:.1} | Phi:{:.2}",
            agent.core.id,
            agent.core.position.x,
            agent.core.position.y,
            agent.core.energy,
            agent.core.hunger,
            agent.phi,
        )
    }

    pub fn render_list(&self, agents: &[SimAgent]) -> String {
        let mut output = String::new();
        output.push_str("=== Agent List ===\n");
        for agent in agents.iter().filter(|a| a.core.alive) {
            output.push_str(&self.render_compact(agent));
            output.push('\n');
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    #[test]
    fn test_inspector_creation() {
        let inspector = AgentInspector::new();
        assert!(inspector.selected_id.is_none());
    }

    #[test]
    fn test_inspector_select() {
        let mut inspector = AgentInspector::new();
        inspector.select("agent-42");
        assert!(inspector.is_selected("agent-42"));
        assert!(!inspector.is_selected("agent-43"));
    }

    #[test]
    fn test_inspector_render() {
        let inspector = AgentInspector::new();
        let agent = SimAgent::new(0, Vec2::new(5.0, 10.0));
        let info = inspector.render_info(&agent);
        assert!(info.contains("Agent #0"));
        assert!(info.contains("5.0"));
    }
}
