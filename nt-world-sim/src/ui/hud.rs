use crate::engine::renderer::Color;
use crate::game::time::GameTime;
use crate::game::weather::Weather;
use super::theme::{StardewTheme, UiRenderer, UiDrawCommand};

pub struct HUD {
    pub visible: bool,
    pub energy: u32,
    pub max_energy: u32,
    pub gold: i32,
}

impl HUD {
    pub fn new() -> Self {
        Self { visible: true, energy: 100, max_energy: 100, gold: 500 }
    }

    pub fn render(&self, time: &GameTime, weather: &Weather) -> Vec<UiDrawCommand> {
        let mut cmds = Vec::new();
        if !self.visible { return cmds; }

        let theme = StardewTheme::default();

        // Time display
        cmds.push(UiDrawCommand::Rect {
            x: 620.0, y: 8.0, width: 120.0, height: 30.0,
            color: Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
        });
        cmds.push(UiDrawCommand::Text {
            text: format!("{:02}:{:02}", time.hour, time.minute),
            x: 628.0, y: 14.0, size: 14.0, color: theme.gold_text,
        });

        // Day / season
        cmds.push(UiDrawCommand::Rect {
            x: 620.0, y: 42.0, width: 120.0, height: 24.0,
            color: Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
        });
        cmds.push(UiDrawCommand::Text {
            text: format!("Day {} - {}", time.day, time.season.season_name()),
            x: 628.0, y: 46.0, size: 11.0, color: theme.white_text,
        });

        // Weather
        cmds.push(UiDrawCommand::Rect {
            x: 620.0, y: 70.0, width: 120.0, height: 20.0,
            color: Color { r: 0.05, g: 0.05, b: 0.1, a: 0.7 },
        });
        cmds.push(UiDrawCommand::Text {
            text: format!("{} {}", weather.current.icon(), weather.current.name()),
            x: 628.0, y: 73.0, size: 10.0, color: theme.white_text,
        });

        // Energy bar
        let energy_pct = self.energy as f32 / self.max_energy as f32;
        cmds.extend(UiRenderer::draw_bar(
            &theme, 8.0, 560.0, 150.0, 20.0, energy_pct, theme.energy_green,
        ));
        cmds.push(UiDrawCommand::Text {
            text: format!("Resonance {}/{}", self.energy, self.max_energy),
            x: 16.0, y: 563.0, size: 10.0, color: theme.white_text,
        });

        // Gold
        cmds.push(UiDrawCommand::Rect {
            x: 620.0, y: 560.0, width: 120.0, height: 24.0,
            color: Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
        });
        cmds.push(UiDrawCommand::Text {
            text: format!("{} IP", self.gold),
            x: 628.0, y: 564.0, size: 12.0, color: theme.gold_text,
        });

        cmds
    }
}

impl Default for HUD {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::time::GameTime;
    use crate::game::weather::Weather;

    #[test]
    fn test_hud_elements() {
        let hud = HUD::new();
        let time = GameTime::new();
        let weather = Weather::new();
        let cmds = hud.render(&time, &weather);
        // Time rect+text, day rect+text, weather rect+text, energy bar(3)+text, gold rect+text = 12
        assert_eq!(cmds.len(), 12);
    }

    #[test]
    fn test_hud_hidden() {
        let mut hud = HUD::new();
        hud.visible = false;
        let time = GameTime::new();
        let weather = Weather::new();
        assert!(hud.render(&time, &weather).is_empty());
    }
}
