use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReviewCategory {
    Structural,
    Integrity,
    TestCoverage,
    Security,
    ContentQuality,
    Convention,
    Cost,
}

impl ReviewCategory {
    pub fn label(&self) -> &'static str {
        match self {
            ReviewCategory::Structural => "Structural Discipline",
            ReviewCategory::Integrity => "Code Integrity",
            ReviewCategory::TestCoverage => "Test Coverage",
            ReviewCategory::Security => "Security",
            ReviewCategory::ContentQuality => "Content Quality",
            ReviewCategory::Convention => "Convention",
            ReviewCategory::Cost => "Cost & Complexity",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub category: ReviewCategory,
    pub score: f64,
    pub issues: Vec<String>,
    pub passed: bool,
}

impl ReviewResult {
    pub fn new(category: ReviewCategory, score: f64, issues: Vec<String>) -> Self {
        let passed = score >= 0.6;
        Self { category, score, issues, passed }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewReport {
    pub results: Vec<ReviewResult>,
    pub overall_score: f64,
    pub all_passed: bool,
}

pub trait SkillReview {
    fn review_code(&self, code: &str, context: &str) -> Vec<ReviewResult>;
    fn report(&self, code: &str, context: &str) -> ReviewReport {
        let results = self.review_code(code, context);
        let overall = if results.is_empty() {
            1.0
        } else {
            results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64
        };
        let all_passed = results.iter().all(|r| r.passed);
        ReviewReport { results, overall_score: overall, all_passed }
    }
}

fn check_structural(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let lines: Vec<&str> = code.lines().collect();
    let mut score: f64 = 1.0;

    if code.len() > 5000 {
        issues.push("Code exceeds 5000 characters, consider splitting".to_string());
        score -= 0.2;
    }
    if lines.iter().any(|l| l.len() > 200) {
        issues.push("Lines exceed 200 characters".to_string());
        score -= 0.15;
    }
    if code.matches('{').count() != code.matches('}').count() {
        issues.push("Unbalanced braces".to_string());
        score -= 0.3;
    }
    if code.matches('(').count() != code.matches(')').count() {
        issues.push("Unbalanced parentheses".to_string());
        score -= 0.2;
    }

    ReviewResult::new(ReviewCategory::Structural, score.max(0.0), issues)
}

fn check_integrity(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if code.contains("unwrap()") {
        issues.push("Contains unwrap() calls".to_string());
        score -= 0.5;
    }
    if code.contains("expect(") {
        issues.push("Contains expect() calls".to_string());
        score -= 0.1;
    }
    if code.contains("todo!()") || code.contains("todo!(") {
        issues.push("Contains todo!() macros".to_string());
        score -= 0.2;
    }
    if code.contains("unsafe ") || code.contains("unsafe{") {
        issues.push("Contains unsafe code".to_string());
        score -= 0.3;
    }

    ReviewResult::new(ReviewCategory::Integrity, score.max(0.0), issues)
}

fn check_test_coverage(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if !code.contains("#[cfg(test)]") {
        issues.push("No test module found".to_string());
        score -= 0.4;
    } else {
        let test_count = code.matches("#[test]").count();
        if test_count < 2 {
            issues.push(format!("Only {} test(s) found, recommend at least 2", test_count));
            score -= 0.2;
        }
    }

    ReviewResult::new(ReviewCategory::TestCoverage, score.max(0.0), issues)
}

fn check_security(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if code.contains("std::process::Command") || code.contains("std::process::command") {
        issues.push("Uses process execution".to_string());
        score -= 0.2;
    }
    if code.contains("eval(") || code.contains("eval (") {
        issues.push("Uses eval() which can execute arbitrary code".to_string());
        score -= 0.4;
    }
    if code.contains("password") || code.contains("secret") || code.contains("token") {
        issues.push("Contains credential-related identifiers, verify safe usage".to_string());
        score -= 0.1;
    }
    if code.contains("injection") || code.contains("sql ") {
        issues.push("Potential injection concerns, validate input sanitization".to_string());
        score -= 0.15;
    }

    ReviewResult::new(ReviewCategory::Security, score.max(0.0), issues)
}

fn check_content_quality(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if code.contains("XXXXX") || code.contains("TODO") || code.contains("FIXME") {
        issues.push("Contains placeholder markers (XXXXX/TODO/FIXME)".to_string());
        score -= 0.15;
    }
    if code.contains("println!") || code.contains("dbg!") {
        issues.push("Contains debug output (println!/dbg!), consider logging".to_string());
        score -= 0.1;
    }
    if code.contains("panic!(") {
        issues.push("Contains panic!() calls in production code".to_string());
        score -= 0.2;
    }

    ReviewResult::new(ReviewCategory::ContentQuality, score.max(0.0), issues)
}

fn check_convention(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;

    if code.contains('\t') {
        issues.push("Uses tabs for indentation, project uses spaces".to_string());
        score -= 0.15;
    }
    if code.lines().any(|l| l.len() > 100) {
        issues.push("Lines exceed 100 character convention limit".to_string());
        score -= 0.1;
    }

    ReviewResult::new(ReviewCategory::Convention, score.max(0.0), issues)
}

fn check_cost(code: &str) -> ReviewResult {
    let mut issues = Vec::new();
    let mut score: f64 = 1.0;
    let line_count = code.lines().count();

    if line_count > 300 {
        issues.push(format!("{} lines — consider splitting into smaller modules", line_count));
        score -= 0.2;
    }
    if code.matches("clone()").count() > 5 {
        issues.push("Excessive clone() calls, consider borrowing".to_string());
        score -= 0.1;
    }
    if code.matches("for ").count() > 10 {
        issues.push("Excessive loops, consider iterator combinators".to_string());
        score -= 0.1;
    }

    ReviewResult::new(ReviewCategory::Cost, score.max(0.0), issues)
}

pub fn review_code(code: &str, _context: &str) -> Vec<ReviewResult> {
    vec![
        check_structural(code),
        check_integrity(code),
        check_test_coverage(code),
        check_security(code),
        check_content_quality(code),
        check_convention(code),
        check_cost(code),
    ]
}

pub struct DefaultSkillReview;

impl SkillReview for DefaultSkillReview {
    fn review_code(&self, code: &str, context: &str) -> Vec<ReviewResult> {
        review_code(code, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_category_labels() {
        assert_eq!(ReviewCategory::Structural.label(), "Structural Discipline");
        assert_eq!(ReviewCategory::Security.label(), "Security");
        assert_eq!(ReviewCategory::Cost.label(), "Cost & Complexity");
    }

    #[test]
    fn test_review_clean_code() {
        let code = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_foo() { assert_eq!(2 + 2, 4); }

    #[test]
    fn test_bar() { assert_eq!(3 * 3, 9); }
}

pub fn add(a: i32, b: i32) -> i32 { a + b }
"#;
        let results = review_code(code, "unit test");
        assert!(results.iter().any(|r| r.category == ReviewCategory::TestCoverage));
        let tc = results.iter().find(|r| r.category == ReviewCategory::TestCoverage).unwrap();
        assert!(tc.passed);
    }

    #[test]
    fn test_review_detects_unwrap() {
        let code = "fn main() { let x = some_result.unwrap(); }";
        let results = review_code(code, "test");
        let integrity = results.iter().find(|r| r.category == ReviewCategory::Integrity).unwrap();
        assert!(!integrity.passed);
        assert!(integrity.issues.iter().any(|i| i.contains("unwrap")));
    }

    #[test]
    fn test_review_detects_unsafe() {
        let code = "unsafe { std::ptr::read(ptr) }";
        let results = review_code(code, "test");
        let integrity = results.iter().find(|r| r.category == ReviewCategory::Integrity).unwrap();
        assert!(integrity.issues.iter().any(|i| i.contains("unsafe")));
    }

    #[test]
    fn test_skill_review_trait() {
        let reviewer = DefaultSkillReview;
        let report = reviewer.report("fn test() {}", "context");
        assert_eq!(report.results.len(), 7);
        // Clean code passes all categories
        assert!(report.all_passed);
    }

    #[test]
    fn test_review_all_categories_present() {
        let code = "fn main() {}";
        let results = review_code(code, "context");
        let categories: Vec<ReviewCategory> = results.iter().map(|r| r.category).collect();
        assert!(categories.contains(&ReviewCategory::Structural));
        assert!(categories.contains(&ReviewCategory::Integrity));
        assert!(categories.contains(&ReviewCategory::TestCoverage));
        assert!(categories.contains(&ReviewCategory::Security));
        assert!(categories.contains(&ReviewCategory::ContentQuality));
        assert!(categories.contains(&ReviewCategory::Convention));
        assert!(categories.contains(&ReviewCategory::Cost));
    }
}
