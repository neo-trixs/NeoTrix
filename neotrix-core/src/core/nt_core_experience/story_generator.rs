use std::collections::VecDeque;

const MAX_EVENTS: usize = 20;

pub struct StoryGenerator {
    pub max_length: usize,
    pub recent_events: VecDeque<String>,
    pub narrative_style: String,
}

#[derive(Debug)]
pub struct StoryStats {
    pub event_count: usize,
    pub narrative_style: String,
    pub templates_available: usize,
}

impl StoryGenerator {
    pub fn new(max_length: usize, narrative_style: &str) -> Self {
        Self {
            max_length,
            recent_events: VecDeque::with_capacity(MAX_EVENTS),
            narrative_style: narrative_style.to_string(),
        }
    }

    pub fn add_event(&mut self, event: String) {
        if self.recent_events.len() >= MAX_EVENTS {
            self.recent_events.pop_front();
        }
        self.recent_events.push_back(event);
    }

    pub fn stats(&self) -> StoryStats {
        StoryStats {
            event_count: self.recent_events.len(),
            narrative_style: self.narrative_style.clone(),
            templates_available: 5,
        }
    }

    pub fn generate_story(&self, prompt: &str) -> String {
        let template_idx = (prompt.len() + self.recent_events.len()) % 5;
        let events: Vec<&str> = self.recent_events.iter().map(|s| s.as_str()).collect();
        let event_summary = if events.is_empty() {
            "No recent events recorded.".to_string()
        } else {
            events.join("; ")
        };

        let story = match template_idx {
            0 => format!(
                "Beginning. {}\n\nEvents unfolded: {}\n\nConclusion. The narrative continues.",
                prompt, event_summary
            ),
            1 => format!(
                "A sequence of observations: {}\n\nContext: {}\n\nOutcome: The pattern suggests ongoing evolution.",
                event_summary, prompt
            ),
            2 => format!(
                "Once, in the flow of cognition:\n- {}\n\nPrompt: {}\n\nThe threads weave together into meaning.",
                event_summary, prompt
            ),
            3 => format!(
                "Reflecting on recent experience:\n{}\n\nGuided by: {}\n\nA new synthesis emerges from the fragments.",
                event_summary, prompt
            ),
            _ => format!(
                "Narrative stream:\nEvents: {}\n\nIntent: {}\n\nStory coherence maintained.",
                event_summary, prompt
            ),
        };

        if story.len() > self.max_length {
            let truncated: String = story.chars().take(self.max_length).collect();
            format!("{}...[truncated]", truncated.trim_end())
        } else {
            story
        }
    }
}

impl Default for StoryGenerator {
    fn default() -> Self {
        Self::new(1024, "concise")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_story_generator() {
        let sg = StoryGenerator::new(512, "detailed");
        assert_eq!(sg.max_length, 512);
        assert_eq!(sg.narrative_style, "detailed");
        assert_eq!(sg.recent_events.len(), 0);
    }

    #[test]
    fn test_add_event() {
        let mut sg = StoryGenerator::default();
        sg.add_event("User asked a question about VSA".into());
        assert_eq!(sg.recent_events.len(), 1);
    }

    #[test]
    fn test_max_events_capped() {
        let mut sg = StoryGenerator::default();
        for i in 0..30 {
            sg.add_event(format!("Event {}", i));
        }
        assert_eq!(sg.recent_events.len(), MAX_EVENTS);
        assert_eq!(sg.recent_events.front().unwrap(), "Event 10");
    }

    #[test]
    fn test_generate_story_with_events() {
        let mut sg = StoryGenerator::default();
        sg.add_event("System initialized".into());
        sg.add_event("User requested analysis".into());
        let story = sg.generate_story("Tell me what happened");
        assert!(!story.is_empty());
        assert!(story.contains("System initialized"));
        assert!(story.contains("User requested analysis"));
    }

    #[test]
    fn test_generate_story_empty_events() {
        let sg = StoryGenerator::default();
        let story = sg.generate_story("Hello");
        assert!(story.contains("No recent events"));
    }

    #[test]
    fn test_stats() {
        let mut sg = StoryGenerator::new(100, "poetic");
        sg.add_event("A star was born".into());
        let s = sg.stats();
        assert_eq!(s.event_count, 1);
        assert_eq!(s.narrative_style, "poetic");
        assert_eq!(s.templates_available, 5);
    }

    #[test]
    fn test_truncation() {
        let sg = StoryGenerator::new(20, "concise");
        let story = sg.generate_story("A very long prompt that should definitely be truncated by the max length setting in the story generator");
        assert!(story.len() <= 20 + "...[truncated]".len());
        assert!(story.ends_with("...[truncated]"));
    }

    #[test]
    fn test_different_templates() {
        let mut sg = StoryGenerator::default();
        sg.add_event("Event A".into());
        let s0 = sg.generate_story("P");
        let s1 = sg.generate_story("Q");
        let s2 = sg.generate_story("R");
        let s3 = sg.generate_story("S");
        let s4 = sg.generate_story("T");
        assert_ne!(s0, s1);
        assert_ne!(s1, s2);
        assert_ne!(s2, s3);
        assert_ne!(s3, s4);
    }

    #[test]
    fn test_default() {
        let sg = StoryGenerator::default();
        assert_eq!(sg.max_length, 1024);
        assert_eq!(sg.narrative_style, "concise");
    }
}
