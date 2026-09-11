pub struct AgentInspector {
    pub selected: Option<u32>,
}

impl AgentInspector {
    pub fn new() -> Self {
        AgentInspector { selected: None }
    }

    pub fn select(&mut self, agent_id: u32) {
        self.selected = Some(agent_id);
    }

    pub fn deselect(&mut self) {
        self.selected = None;
    }

    pub fn get_selected(&self) -> Option<u32> {
        self.selected
    }
}
