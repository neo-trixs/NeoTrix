use crate::neotrix::nt_mind::core::CapabilityVector;
use crate::neotrix::nt_mind::self_edit::MicroEdit;

#[derive(Debug, Clone, PartialEq)]
pub enum UxSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UxCategory {
    Accessibility,
    VisualHierarchy,
    Consistency,
    Feedback,
    Affordance,
    ErrorPrevention,
    Flexibility,
    AestheticDesign,
    Recognition,
    HelpDocumentation,
}

#[derive(Debug, Clone)]
pub struct UxIssue {
    pub category: UxCategory,
    pub severity: UxSeverity,
    pub heuristic: String,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub struct UxReviewReport {
    pub component: String,
    pub issues: Vec<UxIssue>,
    pub accessibility_score: f64,
    pub visual_score: f64,
    pub consistency_score: f64,
    pub overall_score: f64,
}

impl UxReviewReport {
    pub fn total(&self) -> usize {
        self.issues.len()
    }

    pub fn by_category(&self, cat: UxCategory) -> Vec<&UxIssue> {
        self.issues.iter().filter(|i| i.category == cat).collect()
    }

    pub fn by_severity(&self, sev: UxSeverity) -> Vec<&UxIssue> {
        self.issues.iter().filter(|i| i.severity == sev).collect()
    }
}

pub struct UxReviewEngine {
    pub capability: CapabilityVector,
}

impl UxReviewEngine {
    pub fn new(capability: CapabilityVector) -> Self {
        Self { capability }
    }

    /// Full UX review of a component description
    pub fn review(&self, component: &str, description: &str) -> UxReviewReport {
        let mut issues = Vec::new();
        issues.extend(self.check_accessibility(description));
        issues.extend(self.check_visual_hierarchy(description));
        issues.extend(self.check_consistency(description));
        issues.extend(self.check_nielsen_heuristic(description));

        let a11y_count = issues.iter().filter(|i| i.category == UxCategory::Accessibility).count() as f64;
        let visual_count = issues.iter().filter(|i| i.category == UxCategory::VisualHierarchy).count() as f64;
        let consistency_count = issues.iter().filter(|i| i.category == UxCategory::Consistency).count() as f64;

        let a11y_score = (1.0 - (a11y_count * 0.15).min(1.0)).max(0.0);
        let visual_score = (1.0 - (visual_count * 0.15).min(1.0)).max(0.0);
        let consistency_score = (1.0 - (consistency_count * 0.15).min(1.0)).max(0.0);
        let overall = (a11y_score * 0.4 + visual_score * 0.35 + consistency_score * 0.25).clamp(0.0, 1.0);

        UxReviewReport {
            component: component.to_string(),
            issues,
            accessibility_score: a11y_score,
            visual_score,
            consistency_score,
            overall_score: overall,
        }
    }

    /// Check WCAG accessibility issues
    pub fn check_accessibility(&self, desc: &str) -> Vec<UxIssue> {
        let mut issues = Vec::new();
        let lower = desc.to_lowercase();

        if !lower.contains("alt") && !lower.contains("aria-label") && !lower.contains("aria-labelledby") {
            issues.push(UxIssue {
                category: UxCategory::Accessibility,
                severity: UxSeverity::High,
                heuristic: "WCAG 1.1.1 Non-text Content".to_string(),
                description: "Images or icons missing alt text or ARIA labels".to_string(),
                suggestion: "Add alt attributes to all images or aria-label to icon buttons".to_string(),
            });
        }

        if !lower.contains("role") && !lower.contains("aria-") {
            issues.push(UxIssue {
                category: UxCategory::Accessibility,
                severity: UxSeverity::Medium,
                heuristic: "WCAG 4.1.2 Name, Role, Value".to_string(),
                description: "Interactive elements should have explicit ARIA roles".to_string(),
                suggestion: "Add role attributes to custom interactive components".to_string(),
            });
        }

        if !lower.contains("focus") && !lower.contains("tabindex") && !lower.contains("keyboard") {
            issues.push(UxIssue {
                category: UxCategory::Accessibility,
                severity: UxSeverity::High,
                heuristic: "WCAG 2.1.1 Keyboard".to_string(),
                description: "Keyboard navigation support not detected".to_string(),
                suggestion: "Ensure all interactive elements are keyboard accessible with visible focus indicators".to_string(),
            });
        }

        if !lower.contains("contrast") && !lower.contains("color ") {
            issues.push(UxIssue {
                category: UxCategory::Accessibility,
                severity: UxSeverity::Medium,
                heuristic: "WCAG 1.4.3 Contrast Minimum".to_string(),
                description: "Color contrast requirements not mentioned".to_string(),
                suggestion: "Ensure text/background contrast ratio of at least 4.5:1 for normal text".to_string(),
            });
        }

        issues
    }

    /// Check visual hierarchy
    pub fn check_visual_hierarchy(&self, desc: &str) -> Vec<UxIssue> {
        let mut issues = Vec::new();
        let lower = desc.to_lowercase();

        if !lower.contains("heading") && !lower.contains("h1") && !lower.contains("h2") && !lower.contains("h3") {
            issues.push(UxIssue {
                category: UxCategory::VisualHierarchy,
                severity: UxSeverity::Medium,
                heuristic: "Clear visual hierarchy".to_string(),
                description: "No heading structure detected".to_string(),
                suggestion: "Use a clear heading hierarchy (h1 → h2 → h3) to establish content structure".to_string(),
            });
        }

        if !lower.contains("spacing") && !lower.contains("padding") && !lower.contains("margin") && !lower.contains("gap") {
            issues.push(UxIssue {
                category: UxCategory::VisualHierarchy,
                severity: UxSeverity::Low,
                heuristic: "Consistent spacing".to_string(),
                description: "Spacing system not explicitly referenced".to_string(),
                suggestion: "Define a consistent spacing scale (4px/8px/16px/24px/32px) for visual rhythm".to_string(),
            });
        }

        issues
    }

    /// Check visual consistency
    pub fn check_consistency(&self, desc: &str) -> Vec<UxIssue> {
        let mut issues = Vec::new();
        let lower = desc.to_lowercase();

        if !lower.contains("design system") && !lower.contains("style guide") && !lower.contains("token") {
            issues.push(UxIssue {
                category: UxCategory::Consistency,
                severity: UxSeverity::Medium,
                heuristic: "Consistency and standards (Nielsen #4)".to_string(),
                description: "No design system or style guide reference detected".to_string(),
                suggestion: "Reference or create a design system for consistent colors, typography, and component behavior".to_string(),
            });
        }

        if !lower.contains("button") && !lower.contains("icon") && !lower.contains("input") {
            issues.push(UxIssue {
                category: UxCategory::Consistency,
                severity: UxSeverity::Low,
                heuristic: "Platform conventions".to_string(),
                description: "Platform-specific UI patterns not detected in description".to_string(),
                suggestion: "Follow platform conventions (iOS HIG / Material Design) for native-feeling interactions".to_string(),
            });
        }

        issues
    }

    /// Check Nielsen's 10 heuristics (simplified implementation)
    pub fn check_nielsen_heuristic(&self, desc: &str) -> Vec<UxIssue> {
        let mut issues = Vec::new();
        let lower = desc.to_lowercase();

        // #1: Visibility of system status
        if !lower.contains("loading") && !lower.contains("progress") && !lower.contains("status") && !lower.contains("spinner") {
            issues.push(UxIssue {
                category: UxCategory::Feedback,
                severity: UxSeverity::Medium,
                heuristic: "Nielsen #1: Visibility of system status".to_string(),
                description: "Loading/processing states not described".to_string(),
                suggestion: "Provide visual feedback for all user actions (loading spinners, progress bars, confirmation messages)".to_string(),
            });
        }

        // #5: Error prevention
        if !lower.contains("validation") && !lower.contains("confirm") && !lower.contains("undo") {
            issues.push(UxIssue {
                category: UxCategory::ErrorPrevention,
                severity: UxSeverity::Medium,
                heuristic: "Nielsen #5: Error prevention".to_string(),
                description: "Error prevention mechanisms not described".to_string(),
                suggestion: "Add input validation, confirmation dialogs for destructive actions, and undo support".to_string(),
            });
        }

        // #6: Recognition rather than recall
        if !lower.contains("menu") && !lower.contains("navigation") && !lower.contains("breadcrumb") {
            issues.push(UxIssue {
                category: UxCategory::Recognition,
                severity: UxSeverity::Low,
                heuristic: "Nielsen #6: Recognition rather than recall".to_string(),
                description: "Navigation or wayfinding elements not described".to_string(),
                suggestion: "Make navigation options visible and use breadcrumb trails so users don't have to remember where they are".to_string(),
            });
        }

        // #9: Help users recognize, diagnose, and recover from errors
        if !lower.contains("error message") && !lower.contains("error state") && !lower.contains("try again") {
            issues.push(UxIssue {
                category: UxCategory::HelpDocumentation,
                severity: UxSeverity::Medium,
                heuristic: "Nielsen #9: Help users recognize and recover from errors".to_string(),
                description: "Error recovery guidance not described".to_string(),
                suggestion: "Provide clear error messages with actionable steps to resolve the issue".to_string(),
            });
        }

        issues
    }

    /// Convert UX review issues to MicroEdit vector adjustments for SEAL loop
    pub fn issues_to_micro_edits(&self, report: &UxReviewReport) -> Vec<MicroEdit> {
        let mut edits = Vec::new();

        // Adjust accessibility dimension based on score
        let a11y_delta = (report.accessibility_score - 0.5) * 0.1;
        edits.push(MicroEdit::AdjustDimension("accessibility".to_string(), a11y_delta));

        // If overall score is low, apply a general adjustment
        if report.overall_score < 0.5 {
            edits.push(MicroEdit::AdjustDimension("quality_gates".to_string(), -0.05));
        }

        edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_mind::core::CapabilityVector;

    fn make_engine() -> UxReviewEngine {
        UxReviewEngine::new(CapabilityVector::default())
    }

    #[test]
    fn test_ux_review_empty_description() {
        let engine = make_engine();
        let report = engine.review("test", "");
        assert!(report.total() > 0);
        assert!(report.overall_score < 1.0);
    }

    #[test]
    fn test_ux_review_good_description() {
        let engine = make_engine();
        let desc = "The component uses proper heading hierarchy (h1, h2, h3), all images have alt attributes, aria-labels on interactive elements, keyboard navigation with visible focus, proper color contrast, loading states with spinners, input validation, and consistent spacing with a defined design system.";
        let report = engine.review("good-component", desc);
        assert!(report.total() < engine.review("test", "").total());
    }

    #[test]
    fn test_accessibility_check() {
        let engine = make_engine();
        let issues = engine.check_accessibility("A simple button");
        let has_a11y = issues.iter().any(|i| i.category == UxCategory::Accessibility);
        assert!(has_a11y);
    }

    #[test]
    fn test_accessibility_check_with_aria() {
        let engine = make_engine();
        let issues = engine.check_accessibility("Button with aria-label and keyboard focus");
        assert!(issues.len() < 4); // At least some issues should be resolved
    }

    #[test]
    fn test_visual_hierarchy_check() {
        let engine = make_engine();
        let issues = engine.check_visual_hierarchy("A component");
        assert!(issues.iter().any(|i| i.category == UxCategory::VisualHierarchy));
    }

    #[test]
    fn test_consistency_check() {
        let engine = make_engine();
        let issues = engine.check_consistency("A component");
        assert!(issues.iter().any(|i| i.category == UxCategory::Consistency));
    }

    #[test]
    fn test_nielsen_heuristics_check() {
        let engine = make_engine();
        let issues = engine.check_nielsen_heuristic("A simple form with a button");
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_report_by_category() {
        let engine = make_engine();
        let report = engine.review("test", "");
        let a11y = report.by_category(UxCategory::Accessibility);
        assert!(!a11y.is_empty());
    }

    #[test]
    fn test_report_by_severity() {
        let engine = make_engine();
        let report = engine.review("test", "");
        let critical = report.by_severity(UxSeverity::Critical);
        let high = report.by_severity(UxSeverity::High);
        // Should be some high severity issues
        assert!(!high.is_empty() || !critical.is_empty());
    }

    #[test]
    fn test_issues_to_micro_edits() {
        let engine = make_engine();
        let report = engine.review("test", "");
        let edits = engine.issues_to_micro_edits(&report);
        assert!(!edits.is_empty());
    }
}
