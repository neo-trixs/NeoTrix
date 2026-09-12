use crate::engine::renderer::{Color, Rect};
use crate::game::time::GameTime;
use crate::game::weather::Weather;

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
    
    pub fn render(&self, time: &GameTime, weather: &Weather) -> Vec<(Rect, Color, Option<String>)> {
        let mut elements = Vec::new();
        if !self.visible { return elements; }
        
        elements.push((
            Rect { x: 620.0, y: 8.0, width: 120.0, height: 30.0 },
            Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
            Some(format!("{:02}:{:02}", time.hour, time.minute)),
        ));
        
        elements.push((
            Rect { x: 620.0, y: 42.0, width: 120.0, height: 24.0 },
            Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
            Some(format!("Day {} - {}", time.day, time.season.season_name())),
        ));
        
        elements.push((
            Rect { x: 620.0, y: 70.0, width: 120.0, height: 20.0 },
            Color { r: 0.05, g: 0.05, b: 0.1, a: 0.7 },
            Some(format!("{} {}", weather.current.icon(), weather.current.name())),
        ));
        
        let energy_pct = self.energy as f32 / self.max_energy as f32;
        elements.push((
            Rect { x: 8.0, y: 560.0, width: 150.0, height: 20.0 },
            Color { r: 0.2, g: 0.2, b: 0.2, a: 0.8 },
            Some(format!("Resonance {}/{}", self.energy, self.max_energy)),
        ));
        elements.push((
            Rect { x: 8.0, y: 560.0, width: 150.0 * energy_pct, height: 20.0 },
            Color { r: 0.2, g: 0.7, b: 0.3, a: 0.9 },
            None,
        ));
        
        elements.push((
            Rect { x: 620.0, y: 560.0, width: 120.0, height: 24.0 },
            Color { r: 0.1, g: 0.1, b: 0.15, a: 0.8 },
            Some(format!("{} IP", self.gold)),
        ));
        
        elements
    }
}

impl Default for HUD {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::time::Season;

    #[test]
    fn test_hud_elements() {
        let hud = HUD::new();
        let time = GameTime::new();
        let weather = Weather::new();
        let elements = hud.render(&time, &weather);
        assert_eq!(elements.len(), 6);
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
