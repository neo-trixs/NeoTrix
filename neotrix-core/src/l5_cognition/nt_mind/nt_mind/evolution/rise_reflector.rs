#[derive(Debug, Clone)]
pub struct Reflection {
    pub situation: String,
    pub action: String,
    pub outcome: String,
    pub lesson: String,
    pub confidence: f64,
}

pub struct RISE {
    reflections: Vec<Reflection>,
}

impl RISE {
    pub fn new() -> Self {
        Self {
            reflections: Vec::new(),
        }
    }

    pub fn reflect(
        &mut self,
        situation: &str,
        action: &str,
        outcome: &str,
        lesson: &str,
    ) -> &Reflection {
        let confidence = if outcome.contains("success") || outcome.contains("pass") {
            0.9
        } else if outcome.contains("fail") || outcome.contains("error") {
            0.3
        } else {
            0.5
        };
        self.reflections.push(Reflection {
            situation: situation.to_string(),
            action: action.to_string(),
            outcome: outcome.to_string(),
            lesson: lesson.to_string(),
            confidence,
        });
        self.reflections.last().unwrap()
    }

    pub fn query(&self, keyword: &str) -> Vec<&Reflection> {
        self.reflections
            .iter()
            .filter(|r| r.lesson.contains(keyword) || r.situation.contains(keyword))
            .collect()
    }

    pub fn high_confidence(&self) -> Vec<&Reflection> {
        self.reflections
            .iter()
            .filter(|r| r.confidence >= 0.7)
            .collect()
    }
    pub fn count(&self) -> usize {
        self.reflections.len()
    }
    pub fn avg_confidence(&self) -> f64 {
        if self.reflections.is_empty() {
            0.0
        } else {
            self.reflections.iter().map(|r| r.confidence).sum::<f64>()
                / self.reflections.len() as f64
        }
    }
}
impl Default for RISE {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_reflect() {
        let mut r = RISE::new();
        r.reflect("bug found", "fixed it", "success", "always check types");
        assert_eq!(r.count(), 1);
        assert!(r.avg_confidence() > 0.5);
    }
    #[test]
    fn test_query() {
        let mut r = RISE::new();
        r.reflect("a", "b", "c", "rust ownership");
        r.reflect("x", "y", "z", "python gil");
        let q = r.query("rust");
        assert_eq!(q.len(), 1);
    }
}
