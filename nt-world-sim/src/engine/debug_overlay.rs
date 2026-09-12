use crate::engine::renderer::Color;

pub struct DebugOverlay {
    pub visible: bool,
    pub fps: f32,
    pub frame_time: f32,
    pub entity_count: usize,
    pub system_count: usize,
    pub particle_count: usize,
    pub messages: Vec<DebugMessage>,
    pub max_messages: usize,
    pub show_collision: bool,
    pub show_grid: bool,
    pub show_fps: bool,
    pub show_entities: bool,
}

pub struct DebugMessage {
    pub text: String,
    pub color: Color,
    pub timer: f32,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            visible: false,
            fps: 0.0,
            frame_time: 0.0,
            entity_count: 0,
            system_count: 0,
            particle_count: 0,
            messages: Vec::new(),
            max_messages: 10,
            show_collision: false,
            show_grid: false,
            show_fps: true,
            show_entities: true,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn update(&mut self, dt: f32) {
        self.frame_time = dt;
        self.fps = 1.0 / dt.max(0.001);

        self.messages.retain_mut(|m| {
            m.timer -= dt;
            m.timer > 0.0
        });
    }

    pub fn log(&mut self, text: &str) {
        self.messages.push(DebugMessage {
            text: text.to_string(),
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            timer: 5.0,
        });
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn log_color(&mut self, text: &str, color: Color) {
        self.messages.push(DebugMessage {
            text: text.to_string(),
            color,
            timer: 5.0,
        });
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn set_entity_count(&mut self, count: usize) {
        self.entity_count = count;
    }

    pub fn set_system_count(&mut self, count: usize) {
        self.system_count = count;
    }

    pub fn set_particle_count(&mut self, count: usize) {
        self.particle_count = count;
    }

    pub fn render(&self) -> Vec<(String, f32, f32, Color)> {
        if !self.visible {
            return Vec::new();
        }

        let mut items = Vec::new();

        if self.show_fps {
            let color = if self.fps > 50.0 {
                Color {
                    r: 0.2,
                    g: 0.9,
                    b: 0.2,
                    a: 1.0,
                }
            } else if self.fps > 30.0 {
                Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.2,
                    a: 1.0,
                }
            } else {
                Color {
                    r: 0.9,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                }
            };
            items.push((
                format!("FPS: {:.1} ({:.2}ms)", self.fps, self.frame_time * 1000.0),
                8.0,
                8.0,
                color,
            ));
        }

        if self.show_entities {
            items.push((
                format!(
                    "Entities: {} | Systems: {} | Particles: {}",
                    self.entity_count, self.system_count, self.particle_count
                ),
                8.0,
                24.0,
                Color {
                    r: 0.8,
                    g: 0.8,
                    b: 0.8,
                    a: 1.0,
                },
            ));
        }

        for (i, msg) in self.messages.iter().enumerate() {
            items.push((
                msg.text.clone(),
                8.0,
                40.0 + i as f32 * 16.0,
                msg.color,
            ));
        }

        items
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let overlay = DebugOverlay::new();
        assert!(!overlay.visible);
        assert!(overlay.show_fps);
        assert!(overlay.show_entities);
        assert!(!overlay.show_collision);
        assert!(!overlay.show_grid);
    }

    #[test]
    fn test_toggle() {
        let mut overlay = DebugOverlay::new();
        overlay.toggle();
        assert!(overlay.visible);
        overlay.toggle();
        assert!(!overlay.visible);
    }

    #[test]
    fn test_log_and_expiry() {
        let mut overlay = DebugOverlay::new();
        overlay.visible = true;
        overlay.log("test message");
        assert_eq!(overlay.message_count(), 1);
        overlay.update(6.0); // past 5s timer
        assert_eq!(overlay.message_count(), 0);
    }

    #[test]
    fn test_max_messages() {
        let mut overlay = DebugOverlay::new();
        overlay.max_messages = 3;
        for i in 0..5 {
            overlay.log(&format!("msg {}", i));
        }
        assert_eq!(overlay.message_count(), 3);
    }

    #[test]
    fn test_render_empty_when_hidden() {
        let overlay = DebugOverlay::new();
        assert!(overlay.render().is_empty());
    }

    #[test]
    fn test_render_with_fps() {
        let mut overlay = DebugOverlay::new();
        overlay.visible = true;
        overlay.fps = 60.0;
        overlay.frame_time = 1.0 / 60.0;
        let items = overlay.render();
        assert!(!items.is_empty());
        assert!(items[0].0.contains("FPS"));
    }

    #[test]
    fn test_update_frame_time() {
        let mut overlay = DebugOverlay::new();
        overlay.update(0.016);
        assert!((overlay.frame_time - 0.016).abs() < 0.001);
        assert!((overlay.fps - 62.5).abs() < 1.0);
    }
}
