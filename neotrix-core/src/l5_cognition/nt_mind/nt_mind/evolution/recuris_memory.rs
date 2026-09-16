#[derive(Debug, Clone)]
pub struct Pattern {
    pub text: String,
    pub success_rate: f64,
    pub uses: u32,
}

pub struct RecurisMemory {
    patterns: Vec<Pattern>,
}

impl RecurisMemory {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn store(&mut self, text: &str) {
        if let Some(p) = self.patterns.iter_mut().find(|p| p.text == text) {
            p.uses += 1;
        } else {
            self.patterns.push(Pattern {
                text: text.to_string(),
                success_rate: 0.5,
                uses: 1,
            });
        }
    }

    pub fn update(&mut self, text: &str, success: bool) {
        if let Some(p) = self.patterns.iter_mut().find(|p| p.text == text) {
            let n = p.uses as f64;
            p.success_rate = p.success_rate * ((n - 1.0) / n) + if success { 1.0 / n } else { 0.0 };
        }
    }

    pub fn query(&self, kw: &str) -> Vec<&Pattern> {
        self.patterns
            .iter()
            .filter(|p| p.text.contains(kw))
            .collect()
    }

    pub fn best(&self) -> Option<&Pattern> {
        self.patterns.iter().max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count(&self) -> usize {
        self.patterns.len()
    }
}

impl Default for RecurisMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store() {
        let mut m = RecurisMemory::new();
        m.store("rust-ownership");
        assert_eq!(m.count(), 1);
    }

    #[test]
    fn test_query() {
        let mut m = RecurisMemory::new();
        m.store("rust-ownership");
        m.store("python-gil");
        assert_eq!(m.query("rust").len(), 1);
    }
}
