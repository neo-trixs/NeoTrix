use crate::world_sim::WorldSim;
use crate::agents::SimAgent;

pub struct Renderer {
    width: u32,
    height: u32,
    viewport_x: f32,
    viewport_y: f32,
    zoom: f32,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            viewport_x: 0.0,
            viewport_y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn render(&self, sim: &WorldSim) -> String {
        let mut output = String::new();

        output.push_str(&format!("\x1b[2J\x1b[H"));
        output.push_str(&format!("=== NT-WORLD-SIM Tick: {} ===\n", sim.tick));

        let alive = sim.agents.iter().filter(|a| a.core.alive).count();
        let total_energy: f32 = sim.agents.iter().filter(|a| a.core.alive).map(|a| a.core.energy).sum();
        let avg_energy = if alive > 0 { total_energy / alive as f32 } else { 0.0 };

        output.push_str(&format!("Agents: {} | Avg Energy: {:.1}\n", alive, avg_energy));
        output.push_str(&"─".repeat(self.width as usize));
        output.push('\n');

        let world_w = sim.config.world_width;
        let world_h = sim.config.world_height;

        for y in 0..self.height {
            for x in 0..self.width {
                let wx = (x as f32 / self.width as f32 * world_w as f32 + self.viewport_x) as i32;
                let wy = (y as f32 / self.height as f32 * world_h as f32 + self.viewport_y) as i32;

                if wx < 0 || wx >= world_w as i32 || wy < 0 || wy >= world_h as i32 {
                    output.push(' ');
                    continue;
                }

                let agent_here = sim.agents.iter()
                    .find(|a| a.core.alive && a.core.position.x as i32 == wx && a.core.position.y as i32 == wy);

                if let Some(agent) = agent_here {
                    let ch = if agent.personality.aggression > 0.7 {
                        'P'
                    } else if agent.personality.aggression > 0.3 {
                        'p'
                    } else {
                        'N'
                    };
                    output.push(ch);
                } else {
                    output.push('.');
                }
            }
            output.push('\n');
        }

        output.push_str(&"─".repeat(self.width as usize));
        output.push('\n');

        output.push_str("Controls: [WASD] Move | [Q] Zoom In | [E] Zoom Out | [Q] Quit\n");

        output
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.viewport_x += dx / self.zoom;
        self.viewport_y += dy / self.zoom;
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(5.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.2);
    }

    pub fn center_on_agent(&mut self, agent: &SimAgent) {
        self.viewport_x = agent.core.position.x - self.width as f32 / 2.0;
        self.viewport_y = agent.core.position.y - self.height as f32 / 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(80, 24);
        assert_eq!(renderer.width, 80);
        assert_eq!(renderer.height, 24);
    }

    #[test]
    fn test_renderer_pan() {
        let mut renderer = Renderer::new(80, 24);
        renderer.pan(10.0, 5.0);
        assert!(renderer.viewport_x > 0.0);
        assert!(renderer.viewport_y > 0.0);
    }

    #[test]
    fn test_renderer_zoom() {
        let mut renderer = Renderer::new(80, 24);
        renderer.zoom_in();
        assert!(renderer.zoom > 1.0);
        renderer.zoom_out();
        renderer.zoom_out();
        assert!(renderer.zoom < 1.0);
    }
}
