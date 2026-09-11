pub struct EventLogPanel {
    pub events: Vec<String>,
    pub max_events: usize,
}

impl EventLogPanel {
    pub fn new() -> Self {
        EventLogPanel {
            events: Vec::new(),
            max_events: 100,
        }
    }

    pub fn push(&mut self, msg: &str) {
        self.events.push(msg.into());
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    pub fn recent(&self, n: usize) -> &[String] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }
}
