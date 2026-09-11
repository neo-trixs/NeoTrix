pub mod minimap;
pub mod stats;
pub mod event_log;

pub struct Hud {
    pub show_minimap: bool,
    pub show_stats: bool,
    pub show_event_log: bool,
}

impl Hud {
    pub fn new() -> Self {
        Hud {
            show_minimap: true,
            show_stats: true,
            show_event_log: true,
        }
    }

    pub fn toggle_minimap(&mut self) {
        self.show_minimap = !self.show_minimap;
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn toggle_event_log(&mut self) {
        self.show_event_log = !self.show_event_log;
    }
}
