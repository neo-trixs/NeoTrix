use crate::game::dialogue::DialogueTree;

pub struct DialogueUi {
    pub active: bool,
    pub portrait: String,
    pub speaker: String,
    pub text: String,
    pub responses: Vec<String>,
    pub selected_response: usize,
}

impl DialogueUi {
    pub fn new() -> Self {
        Self {
            active: false,
            portrait: String::new(),
            speaker: String::new(),
            text: String::new(),
            responses: Vec::new(),
            selected_response: 0,
        }
    }

    pub fn show(&mut self, tree: &DialogueTree) {
        if let Some(line) = tree.current_line() {
            self.active = true;
            self.speaker = line.speaker.clone();
            self.text = line.text.clone();
            self.responses = line.responses.iter().map(|r| r.text.clone()).collect();
        }
    }

    pub fn hide(&mut self) {
        self.active = false;
    }

    pub fn select_response(&mut self, index: usize) {
        self.selected_response = index;
    }
}

impl Default for DialogueUi {
    fn default() -> Self { Self::new() }
}
