#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    pub next_line: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct DialogueLine {
    pub speaker: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
}

#[derive(Debug, Clone)]
pub struct DialogueTree {
    pub lines: Vec<DialogueLine>,
    pub current: usize,
}

impl DialogueTree {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current: 0,
        }
    }

    pub fn add_line(&mut self, speaker: &str, text: &str) -> usize {
        let idx = self.lines.len();
        self.lines.push(DialogueLine {
            speaker: speaker.to_string(),
            text: text.to_string(),
            responses: Vec::new(),
        });
        idx
    }

    pub fn add_response(&mut self, line_index: usize, text: &str, next_line: Option<usize>) {
        if let Some(line) = self.lines.get_mut(line_index) {
            line.responses.push(DialogueResponse {
                text: text.to_string(),
                next_line,
            });
        }
    }

    pub fn current_line(&self) -> Option<&DialogueLine> {
        self.lines.get(self.current)
    }

    pub fn select_response(&mut self, index: usize) -> bool {
        if let Some(line) = self.lines.get(self.current) {
            if let Some(resp) = line.responses.get(index) {
                if let Some(next) = resp.next_line {
                    self.current = next;
                    return true;
                }
            }
        }
        false
    }
}

impl Default for DialogueTree {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_tree() {
        let mut tree = DialogueTree::new();
        let idx = tree.add_line("Sage", "Hello, traveler.");
        tree.add_response(idx, "Hi!", Some(1));
        tree.add_line("Sage", "Welcome to the village.");

        assert!(tree.current_line().is_some());
        assert_eq!(tree.current_line().unwrap().speaker, "Sage");

        tree.select_response(0);
        assert_eq!(tree.current, 1);
    }
}
