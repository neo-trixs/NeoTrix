use crate::world_sim::WorldSim;

pub struct UIPanel {
    pub title: String,
    pub content: String,
    pub visible: bool,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl UIPanel {
    pub fn new(title: &str, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            title: title.to_string(),
            content: String::new(),
            visible: true,
            x,
            y,
            width,
            height,
        }
    }

    pub fn render(&self) -> String {
        if !self.visible {
            return String::new();
        }

        let mut output = String::new();
        output.push_str(&format!("┌{}┐\n", "─".repeat(self.width as usize - 2)));
        output.push_str(&format!("│{:^width$}│\n", self.title, width = (self.width - 2) as usize));
        output.push_str(&format!("├{}┤\n", "─".repeat(self.width as usize - 2)));

        for line in self.content.lines() {
            output.push_str(&format!("│ {:<width$}│\n", line, width = (self.width - 3) as usize));
        }

        output.push_str(&format!("└{}┘\n", "─".repeat(self.width as usize - 2)));
        output
    }

    pub fn update_content(&mut self, content: &str) {
        self.content = content.to_string();
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

pub struct PanelManager {
    pub panels: Vec<UIPanel>,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
        }
    }

    pub fn add_panel(&mut self, panel: UIPanel) {
        self.panels.push(panel);
    }

    pub fn render_all(&self) -> String {
        let mut output = String::new();
        for panel in &self.panels {
            output.push_str(&panel.render());
        }
        output
    }

    pub fn update_panel(&mut self, title: &str, content: &str) {
        if let Some(panel) = self.panels.iter_mut().find(|p| p.title == title) {
            panel.update_content(content);
        }
    }

    pub fn toggle_panel(&mut self, title: &str) {
        if let Some(panel) = self.panels.iter_mut().find(|p| p.title == title) {
            panel.toggle();
        }
    }

    pub fn create_stats_panel(&mut self) {
        let mut panel = UIPanel::new("Stats", 0, 0, 30, 10);
        panel.update_content("Agents: 0\nTick: 0\nFPS: 0");
        self.add_panel(panel);
    }

    pub fn create_agent_panel(&mut self) {
        let mut panel = UIPanel::new("Agent", 0, 12, 30, 15);
        panel.update_content("Select an agent...");
        self.add_panel(panel);
    }

    pub fn create_controls_panel(&mut self) {
        let mut panel = UIPanel::new("Controls", 32, 0, 25, 8);
        panel.update_content("[WASD] Move\n[Space] Pause\n[Q/E] Zoom");
        self.add_panel(panel);
    }

    pub fn update_stats(&mut self, sim: &WorldSim) {
        let alive = sim.agents.iter().filter(|a| a.core.alive).count();
        let content = format!(
            "Agents: {}\nTick: {}\nAlive: {}",
            sim.agents.len(),
            sim.tick,
            alive
        );
        self.update_panel("Stats", &content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let panel = UIPanel::new("Test", 0, 0, 20, 10);
        assert_eq!(panel.title, "Test");
        assert!(panel.visible);
    }

    #[test]
    fn test_panel_render() {
        let panel = UIPanel::new("Test", 0, 0, 20, 10);
        let output = panel.render();
        assert!(output.contains("Test"));
    }

    #[test]
    fn test_panel_manager() {
        let mut manager = PanelManager::new();
        manager.add_panel(UIPanel::new("A", 0, 0, 10, 5));
        manager.add_panel(UIPanel::new("B", 10, 0, 10, 5));
        assert_eq!(manager.panels.len(), 2);
    }
}
