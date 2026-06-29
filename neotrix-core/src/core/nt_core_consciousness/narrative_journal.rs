#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub session_id: u64,
    pub timestamp: u64,
    pub forecast: String,
    pub actual: String,
    pub pattern: String,
    pub salience: f64,
}

#[derive(Debug, Clone)]
pub struct NarrativeJournal {
    pub entries: Vec<JournalEntry>,
    pub current_session: u64,
}

impl NarrativeJournal {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_session: 0,
        }
    }

    pub fn record(&mut self, forecast: &str, actual: &str, pattern: &str, salience: f64) {
        self.entries.push(JournalEntry {
            session_id: self.current_session,
            timestamp: 0,
            forecast: forecast.to_string(),
            actual: actual.to_string(),
            pattern: pattern.to_string(),
            salience: salience.clamp(0.0, 1.0),
        });
    }

    pub fn resolve_patterns(&self) -> Vec<String> {
        let mut patterns: Vec<String> = self
            .entries
            .iter()
            .filter(|e| e.salience > 0.6)
            .map(|e| e.pattern.clone())
            .collect();
        patterns.sort();
        patterns.dedup();
        patterns
    }

    pub fn narrative_arc(&self) -> String {
        let patterns = self.resolve_patterns();
        if patterns.is_empty() {
            return "No clear narrative arc".to_string();
        }
        patterns.join(" → ")
    }
}

impl Default for NarrativeJournal {
    fn default() -> Self {
        Self::new()
    }
}
