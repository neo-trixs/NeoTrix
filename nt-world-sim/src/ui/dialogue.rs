use crate::engine::renderer::Rect;

#[derive(Debug, Clone)]
pub struct DialogueNode {
    pub speaker: String,
    pub text: String,
    pub responses: Vec<DialogueResponse>,
    pub portrait_id: u32,
}

#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub text: String,
    pub next_node: Option<usize>,
    pub resonance_change: Option<i32>,
}

pub struct DialogueBox {
    pub visible: bool,
    pub nodes: Vec<DialogueNode>,
    pub current_node: usize,
    pub text_progress: u32,
    pub text_speed: u32,
}

impl DialogueBox {
    pub fn new() -> Self {
        Self { visible: false, nodes: Vec::new(), current_node: 0, text_progress: 0, text_speed: 2 }
    }
    
    pub fn start_dialogue(&mut self, nodes: Vec<DialogueNode>) {
        self.nodes = nodes;
        self.current_node = 0;
        self.text_progress = 0;
        self.visible = true;
    }
    
    pub fn advance(&mut self) -> bool {
        if let Some(node) = self.nodes.get(self.current_node) {
            let full_len = node.text.len() as u32;
            if self.text_progress < full_len {
                self.text_progress = full_len;
                return false;
            }
            if !node.responses.is_empty() { return false; }
            self.current_node += 1;
            self.text_progress = 0;
            if self.current_node >= self.nodes.len() {
                self.visible = false;
                return true;
            }
        }
        false
    }
    
    pub fn select_response(&mut self, index: usize) {
        if let Some(node) = self.nodes.get(self.current_node) {
            if let Some(response) = node.responses.get(index) {
                if let Some(next) = response.next_node {
                    self.current_node = next;
                    self.text_progress = 0;
                } else {
                    self.current_node += 1;
                    self.text_progress = 0;
                    if self.current_node >= self.nodes.len() {
                        self.visible = false;
                    }
                }
            }
        }
    }
    
    pub fn current_display_text(&self) -> String {
        self.nodes.get(self.current_node)
            .map(|n| {
                let end = self.text_progress as usize;
                if end >= n.text.len() { n.text.clone() }
                else { n.text[..end].to_string() }
            })
            .unwrap_or_default()
    }
    
    pub fn render(&self) -> Option<(Rect, String, String, Vec<(String, usize)>)> {
        if !self.visible { return None; }
        let node = self.nodes.get(self.current_node)?;
        let responses: Vec<(String, usize)> = node.responses.iter()
            .enumerate()
            .map(|(i, r)| (r.text.clone(), i))
            .collect();
        Some((
            Rect { x: 40.0, y: 400.0, width: 720.0, height: 180.0 },
            node.speaker.clone(),
            self.current_display_text(),
            responses,
        ))
    }
}

impl Default for DialogueBox {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dialogue() -> DialogueBox {
        let nodes = vec![
            DialogueNode {
                speaker: "Sage".into(),
                text: "Hello, traveler.".into(),
                responses: vec![
                    DialogueResponse { text: "Hi!".into(), next_node: Some(1), resonance_change: None },
                ],
                portrait_id: 0,
            },
            DialogueNode {
                speaker: "Sage".into(),
                text: "Welcome to the village.".into(),
                responses: vec![],
                portrait_id: 0,
            },
        ];
        let mut db = DialogueBox::new();
        db.start_dialogue(nodes);
        db
    }

    #[test]
    fn test_dialogue_start() {
        let db = make_dialogue();
        assert!(db.visible);
        assert_eq!(db.current_node, 0);
    }

    #[test]
    fn test_advance_shows_text() {
        let mut db = make_dialogue();
        assert!(!db.advance());
        assert_eq!(db.text_progress, "Hello, traveler.".len() as u32);
    }

    #[test]
    fn test_advance_waits_for_response() {
        let mut db = make_dialogue();
        db.text_progress = "Hello, traveler.".len() as u32;
        assert!(!db.advance());
    }

    #[test]
    fn test_select_response_advances() {
        let mut db = make_dialogue();
        db.text_progress = "Hello, traveler.".len() as u32;
        db.select_response(0);
        assert_eq!(db.current_node, 1);
        assert_eq!(db.text_progress, 0);
    }

    #[test]
    fn test_advance_to_end() {
        let mut db = make_dialogue();
        db.select_response(0);
        db.text_progress = "Welcome to the village.".len() as u32;
        assert!(db.advance());
        assert!(!db.visible);
    }

    #[test]
    fn test_render_none_when_hidden() {
        let db = DialogueBox::new();
        assert!(db.render().is_none());
    }

    #[test]
    fn test_render_some_when_visible() {
        let db = make_dialogue();
        let result = db.render();
        assert!(result.is_some());
        let (rect, speaker, text, _) = result.unwrap();
        assert_eq!(speaker, "Sage");
        assert_eq!(text, "Hello, traveler.");
        assert_eq!(rect.width, 720.0);
    }
}
