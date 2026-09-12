use super::widget::ProgressBar;
use crate::game::time::GameTime;
use crate::game::energy::Energy;

pub struct Hud {
    pub energy_bar: ProgressBar,
    pub resonance_bar: ProgressBar,
    pub gold_display: i32,
    pub time_display: String,
    pub season_display: String,
    pub day_display: String,
    pub visible: bool,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            energy_bar: ProgressBar::new(10.0, 560.0, 150.0, 15.0),
            resonance_bar: ProgressBar::new(10.0, 540.0, 150.0, 15.0),
            gold_display: 500,
            time_display: "06:00".to_string(),
            season_display: "Clarity".to_string(),
            day_display: "Day 1, Year 1".to_string(),
            visible: true,
        }
    }

    pub fn update(&mut self, time: &GameTime, energy: &Energy, gold: i32) {
        self.time_display = time.time_string();
        self.season_display = time.season.name().to_string();
        self.day_display = time.date_string();
        self.gold_display = gold;
        self.energy_bar.set_value(energy.current);
        self.energy_bar.max = energy.max;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}

impl Default for Hud {
    fn default() -> Self { Self::new() }
}
