use super::VoiceCommand;

#[derive(Debug, Clone)]
pub struct VoiceTrigger {
    wake_words: Vec<String>,
    threshold: f64,
    cooldown_secs: u64,
    last_trigger: Option<std::time::SystemTime>,
}

impl VoiceTrigger {
    pub fn new() -> Self {
        Self {
            wake_words: vec!["hey neotrix".to_string(), "hey neo".to_string(), "neotrix".to_string()],
            threshold: 0.5,
            cooldown_secs: 5,
            last_trigger: None,
        }
    }

    pub fn with_wake_words(mut self, words: Vec<String>) -> Self {
        self.wake_words = words;
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_cooldown(mut self, secs: u64) -> Self {
        self.cooldown_secs = secs;
        self
    }

    pub fn detect(&mut self, text: &str) -> bool {
        if let Some(last) = self.last_trigger {
            if let Ok(elapsed) = last.elapsed() {
                if elapsed.as_secs() < self.cooldown_secs {
                    return false;
                }
            }
        }
        let text_lower = text.to_lowercase();
        for wake in &self.wake_words {
            if text_lower.contains(wake) {
                self.last_trigger = Some(std::time::SystemTime::now());
                return true;
            }
        }
        false
    }

    pub fn detect_with_command(&mut self, text: &str) -> Option<(bool, Option<VoiceCommand>)> {
        let triggered = self.detect(text);
        if !triggered {
            return Some((false, None));
        }
        let cleaned = text.to_lowercase();
        for wake in &self.wake_words {
            let cleaned = cleaned.replace(wake, "").trim().to_string();
            if cleaned.is_empty() {
                return Some((true, None));
            }
            let cmd = VoiceCommand::parse(&cleaned);
            return Some((true, Some(cmd)));
        }
        Some((true, None))
    }

    pub fn wake_words(&self) -> &[String] {
        &self.wake_words
    }

    pub fn cooldown_secs(&self) -> u64 {
        self.cooldown_secs
    }

    pub fn is_on_cooldown(&self) -> bool {
        match self.last_trigger {
            Some(t) => t.elapsed().map(|e| e.as_secs() < self.cooldown_secs).unwrap_or(false),
            None => false,
        }
    }
}

impl Default for VoiceTrigger {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_wake_word() {
        let mut trigger = VoiceTrigger::new();
        assert!(trigger.detect("hey neotrix open settings"));
        assert!(trigger.last_trigger.is_some());
    }

    #[test]
    fn test_detect_no_wake_word() {
        let mut trigger = VoiceTrigger::new();
        assert!(!trigger.detect("open settings"));
    }

    #[test]
    fn test_cooldown_blocks_rapid_triggers() {
        let mut trigger = VoiceTrigger::new();
        trigger.cooldown_secs = 60;
        assert!(trigger.detect("hey neotrix"));
        assert!(!trigger.detect("hey neotrix"));
    }

    #[test]
    fn test_detect_with_command_extracts() {
        let mut trigger = VoiceTrigger::new();
        let result = trigger.detect_with_command("hey neotrix open settings");
        assert!(result.is_some());
        let (triggered, cmd) = result.unwrap();
        assert!(triggered);
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), VoiceCommand::OpenSettings);
    }

    #[test]
    fn test_detect_with_command_no_command() {
        let mut trigger = VoiceTrigger::new();
        trigger.cooldown_secs = 0;
        let result = trigger.detect_with_command("hey neotrix");
        assert!(result.is_some());
        let (triggered, cmd) = result.unwrap();
        assert!(triggered);
        assert!(cmd.is_none());
    }

    #[test]
    fn test_is_on_cooldown() {
        let mut trigger = VoiceTrigger::new();
        trigger.cooldown_secs = 1_000_000;
        assert!(!trigger.is_on_cooldown());
        trigger.detect("hey neotrix");
        assert!(trigger.is_on_cooldown());
    }
}
