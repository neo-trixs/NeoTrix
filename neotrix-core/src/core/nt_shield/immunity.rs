#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureClass {
    ToolSelection,
    Execution,
    Context,
    FileSystem,
    Language,
    Dependency,
    Logic,
    Architecture,
}

impl FailureClass {
    pub fn all() -> [FailureClass; 8] {
        use FailureClass::*;
        [ToolSelection, Execution, Context, FileSystem, Language, Dependency, Logic, Architecture]
    }
}

#[derive(Debug, Clone)]
pub struct AntiPattern {
    pub id: String,
    pub class: FailureClass,
    pub symptom: String,
    pub root_cause: String,
    pub prevention: String,
    pub detection_hint: String,
    pub occurrences: u32,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct Vaccine {
    pub id: String,
    pub anti_pattern_id: String,
    pub pre_check: String,
    pub guard_condition: String,
    pub effectiveness: f64,
}

impl Vaccine {
    pub fn check(&self, context: &str) -> bool {
        context.contains(&self.guard_condition)
    }
}

#[derive(Debug, Clone)]
pub struct ImmuneMemory {
    patterns: HashMap<String, AntiPattern>,
    vaccines: HashMap<String, Vaccine>,
    by_class: HashMap<FailureClass, Vec<String>>,
}

impl ImmuneMemory {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            vaccines: HashMap::new(),
            by_class: HashMap::new(),
        }
    }

    pub fn register_pattern(&mut self, pattern: AntiPattern) -> String {
        let id = pattern.id.clone();
        self.by_class
            .entry(pattern.class)
            .or_default()
            .push(id.clone());
        self.patterns.insert(id.clone(), pattern);
        id
    }

    pub fn register_vaccine(&mut self, vaccine: Vaccine) {
        self.vaccines.insert(vaccine.id.clone(), vaccine);
    }

    pub fn lookup(&self, class: FailureClass) -> Vec<&AntiPattern> {
        self.by_class
            .get(&class)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.patterns.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn find_matching(&self, description: &str) -> Option<&AntiPattern> {
        let desc_lower = description.to_lowercase();
        self.patterns.values().find(|&pattern| desc_lower.contains(&pattern.symptom.to_lowercase())
                || desc_lower.contains(&pattern.detection_hint.to_lowercase())).map(|v| v as _)
    }

    pub fn all_patterns(&self) -> Vec<&AntiPattern> {
        self.patterns.values().collect()
    }

    pub fn patterns_by_class(&self) -> &HashMap<FailureClass, Vec<String>> {
        &self.by_class
    }
}

impl Default for ImmuneMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AdversarialReview {
    pub diff: String,
    pub approved: bool,
    pub issues: Vec<String>,
    pub confidence: f64,
}

impl AdversarialReview {
    pub fn review(code_diff: &str, _context: &str) -> Self {
        let mut issues = Vec::new();
        let red_flags = [
            ("unsafe", "uses unsafe block"),
            (".unwrap()", "uses unwrap without safety"),
            ("panic!", "uses panic macro"),
            ("todo-macro", "uses unimplemented todo"),
            ("expect(", "uses expect without reason"),
            ("as *mut", "unsafe pointer cast"),
            ("std::mem::transmute", "uses transmute"),
        ];

        for (flag, desc) in &red_flags {
            if code_diff.contains(flag) {
                issues.push(desc.to_string());
            }
        }

        let approved = issues.is_empty();
        let confidence = if approved {
            0.85
        } else {
            (1.0 - (issues.len() as f64 * 0.15)).max(0.2)
        };

        Self {
            diff: code_diff.to_string(),
            approved,
            issues,
            confidence,
        }
    }

    pub fn is_passed(&self) -> bool {
        self.approved && self.confidence > 0.6
    }
}

#[derive(Debug, Clone)]
pub struct ImmuneStats {
    pub total_patterns: u32,
    pub total_vaccines: u32,
    pub total_reviews: u32,
    pub immunity_count: u32,
    pub coverage: HashMap<FailureClass, u32>,
}

#[derive(Debug, Clone)]
pub struct ImmuneSystem {
    pub memory: ImmuneMemory,
    pub reviews: VecDeque<AdversarialReview>,
    pub immunity_count: u32,
}

impl ImmuneSystem {
    pub fn new() -> Self {
        Self {
            memory: ImmuneMemory::new(),
            reviews: VecDeque::new(),
            immunity_count: 0,
        }
    }

    pub fn record_failure(
        &mut self,
        class: FailureClass,
        symptom: impl Into<String>,
        root_cause: impl Into<String>,
        prevention: impl Into<String>,
    ) {
        let id = format!("AP-{:04}", self.memory.all_patterns().len() + 1);
        let symptom = symptom.into();
        let pattern = AntiPattern {
            id: id.clone(),
            class,
            symptom: symptom.clone(),
            root_cause: root_cause.into(),
            prevention: prevention.into(),
            detection_hint: symptom.clone(),
            occurrences: 1,
            created_at: 0,
        };
        let ap_id = self.memory.register_pattern(pattern);
        let vaccine = Vaccine {
            id: format!("VAC-{:04}", self.memory.vaccines.len() + 1),
            anti_pattern_id: ap_id,
            pre_check: format!("check for {}", symptom),
            guard_condition: symptom,
            effectiveness: 0.8,
        };
        self.memory.register_vaccine(vaccine);
        self.immunity_count += 1;
    }

    pub fn check_action(&self, action_desc: &str) -> Option<&AntiPattern> {
        self.memory.find_matching(action_desc)
    }

    pub fn review_diff(&mut self, diff: &str, context: &str) -> AdversarialReview {
        let review = AdversarialReview::review(diff, context);
        self.reviews.push_back(review.clone());
        if self.reviews.len() > 100 {
            self.reviews.pop_front();
        }
        review
    }

    pub fn stats(&self) -> ImmuneStats {
        let mut coverage: HashMap<FailureClass, u32> = HashMap::new();
        for class in FailureClass::all() {
            let count = self.memory.lookup(class).len() as u32;
            if count > 0 {
                coverage.insert(class, count);
            }
        }
        ImmuneStats {
            total_patterns: self.memory.all_patterns().len() as u32,
            total_vaccines: self.memory.vaccines.len() as u32,
            total_reviews: self.reviews.len() as u32,
            immunity_count: self.immunity_count,
            coverage,
        }
    }
}

impl Default for ImmuneSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut memory = ImmuneMemory::new();
        let pattern = AntiPattern {
            id: "AP-0001".to_string(),
            class: FailureClass::ToolSelection,
            symptom: "wrong tool used".to_string(),
            root_cause: "ambiguous task description".to_string(),
            prevention: "verify tool matches intent".to_string(),
            detection_hint: "tool mismatch".to_string(),
            occurrences: 1,
            created_at: 0,
        };
        memory.register_pattern(pattern);
        let results = memory.lookup(FailureClass::ToolSelection);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vaccine_trigger() {
        let vaccine = Vaccine {
            id: "VAC-0001".to_string(),
            anti_pattern_id: "AP-0001".to_string(),
            pre_check: "check".to_string(),
            guard_condition: "panic".to_string(),
            effectiveness: 0.9,
        };
        assert!(vaccine.check("this will panic if nil"));
        assert!(!vaccine.check("safe operation"));
    }

    #[test]
    fn test_immune_system_stats() {
        let mut system = ImmuneSystem::new();
        system.record_failure(FailureClass::Logic, "infinite loop", "missing break condition", "always add loop bounds");
        let stats = system.stats();
        assert_eq!(stats.total_patterns, 1);
        assert_eq!(stats.total_vaccines, 1);
        assert_eq!(stats.immunity_count, 1);
    }

    #[test]
    fn test_adversarial_review_pass_fail() {
        let clean = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let review = AdversarialReview::review(clean, "");
        assert!(review.is_passed());

        let unsafe_code = "fn dangerous() { unsafe { *ptr = 42; } }";
        let review2 = AdversarialReview::review(unsafe_code, "");
        assert!(!review2.is_passed());
    }

    #[test]
    fn test_check_action_matches() {
        let mut system = ImmuneSystem::new();
        system.record_failure(FailureClass::Execution, "timeout error", "no retry logic", "add exponential backoff");
        let result = system.check_action("got timeout error again");
        assert!(result.is_some());
        let result2 = system.check_action("unrelated action");
        assert!(result2.is_none());
    }
}
