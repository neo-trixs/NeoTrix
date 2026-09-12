use crate::engine::renderer::{Color, Vec2};
use super::theme::{StardewTheme, UiDrawCommand};

pub struct Minimap {
    pub x: f32, pub y: f32,
    pub size: f32,
    pub visible: bool,
    pub player_pos: Vec2,
    pub world_size: (f32, f32),
    pub npc_positions: Vec<(String, Vec2, Color)>,
    pub zone_boundaries: Vec<(String, f32, f32, f32, f32)>,
}

impl Minimap {
    pub fn new(x: f32, y: f32, size: f32) -> Self {
        Self {
            x, y, size, visible: true,
            player_pos: Vec2 { x: 0.0, y: 0.0 },
            world_size: (3200.0, 2400.0),
            npc_positions: Vec::new(),
            zone_boundaries: Vec::new(),
        }
    }

    pub fn render(&self, theme: &StardewTheme) -> Vec<UiDrawCommand> {
        if !self.visible { return Vec::new(); }

        let mut cmds = Vec::new();

        cmds.push(UiDrawCommand::Rect {
            x: self.x, y: self.y,
            width: self.size, height: self.size,
            color: Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
        });

        cmds.push(UiDrawCommand::RectBorder {
            x: self.x, y: self.y,
            width: self.size, height: self.size,
            color: theme.wood_light, thickness: 2.0,
        });

        for (_name, zx, zy, zw, zh) in &self.zone_boundaries {
            let mx = self.x + (zx / self.world_size.0) * self.size;
            let my = self.y + (zy / self.world_size.1) * self.size;
            let mw = (zw / self.world_size.0) * self.size;
            let mh = (zh / self.world_size.1) * self.size;
            cmds.push(UiDrawCommand::RectBorder {
                x: mx, y: my, width: mw, height: mh,
                color: Color { r: 0.4, g: 0.4, b: 0.4, a: 0.5 }, thickness: 1.0,
            });
        }

        for (_name, pos, color) in &self.npc_positions {
            let nx = self.x + (pos.x / self.world_size.0) * self.size;
            let ny = self.y + (pos.y / self.world_size.1) * self.size;
            cmds.push(UiDrawCommand::Circle {
                x: nx, y: ny, radius: 3.0, color: *color,
            });
        }

        let px = self.x + (self.player_pos.x / self.world_size.0) * self.size;
        let py = self.y + (self.player_pos.y / self.world_size.1) * self.size;
        cmds.push(UiDrawCommand::Circle {
            x: px, y: py, radius: 4.0,
            color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        });

        cmds
    }
}

impl Default for Minimap {
    fn default() -> Self { Self::new(8.0, 8.0, 120.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_new() {
        let m = Minimap::new(10.0, 20.0, 150.0);
        assert_eq!(m.x, 10.0);
        assert_eq!(m.y, 20.0);
        assert_eq!(m.size, 150.0);
        assert!(m.visible);
        assert!(m.npc_positions.is_empty());
    }

    #[test]
    fn test_minimap_hidden_returns_empty() {
        let mut m = Minimap::new(0.0, 0.0, 100.0);
        m.visible = false;
        let theme = StardewTheme::default();
        assert!(m.render(&theme).is_empty());
    }

    #[test]
    fn test_minimap_renders_base_elements() {
        let m = Minimap::new(0.0, 0.0, 100.0);
        let theme = StardewTheme::default();
        let cmds = m.render(&theme);
        assert!(cmds.len() >= 3);
    }

    #[test]
    fn test_minimap_with_npcs() {
        let mut m = Minimap::new(0.0, 0.0, 100.0);
        m.npc_positions.push(("Sage".into(), Vec2 { x: 100.0, y: 100.0 }, Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }));
        let theme = StardewTheme::default();
        let cmds = m.render(&theme);
        assert!(cmds.len() >= 4);
    }

    #[test]
    fn test_minimap_with_zones() {
        let mut m = Minimap::new(0.0, 0.0, 100.0);
        m.zone_boundaries.push(("Farm".into(), 0.0, 0.0, 800.0, 600.0));
        let theme = StardewTheme::default();
        let cmds = m.render(&theme);
        assert!(cmds.len() >= 4);
    }
}
