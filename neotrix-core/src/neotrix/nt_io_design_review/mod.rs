//! CSS/UI design anti-pattern auditor.
//! Unrelated to `core::nt_core_consciousness::inner_critic` (VSA thought quality gate).
//! This module audits HTML/CSS content for accessibility, color, typography, and motion violations.
//! Moved from `nt_shield::inner_critic` — kept as shim in nt_shield for backward compat.

mod critic;
mod detectors;
mod types;

pub use critic::*;
pub use detectors::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_detectors_loaded() {
        let critic = InnerCritic::new();
        assert_eq!(critic.detector_count(), 5);
    }

    #[test]
    fn test_empty_content_no_violations() {
        let critic = InnerCritic::new();
        let violations = critic.audit("");
        assert!(violations.is_empty());
    }

    #[test]
    fn test_gray_text_on_colored_bg() {
        let critic = InnerCritic::new();
        let content = r#".header { background-color: #1a73e8; color: #999; }"#;
        let violations = critic.audit(content);
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.name == "gray-text-on-colored-bg"));
    }

    #[test]
    fn test_pure_black_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { color: #000000; font-size: 16px; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "pure-black-gray"));
    }

    #[test]
    fn test_giant_heading_detected() {
        let critic = InnerCritic::new();
        let content = r#".hero-title { font-size: 96px; font-weight: 700; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "giant-heading"));
    }

    #[test]
    fn test_thin_font_detected() {
        let critic = InnerCritic::new();
        let content = r#"p { font-weight: 200; color: #333; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "thin-light-font"));
    }

    #[test]
    fn test_missing_alt_text_detected() {
        let critic = InnerCritic::new();
        let content = r#"<img src="photo.jpg">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "missing-alt-text"));
    }

    #[test]
    fn test_image_with_alt_passes() {
        let critic = InnerCritic::new();
        let content = r#"<img src="photo.jpg" alt="A sunny landscape">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().all(|v| v.name != "missing-alt-text"));
    }

    #[test]
    fn test_missing_lang_attr_detected() {
        let critic = InnerCritic::new();
        let content = r#"<html><head><title>Test</title></head><body></body></html>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "missing-lang-attr"));
    }

    #[test]
    fn test_auto_play_video_detected() {
        let critic = InnerCritic::new();
        let content = r#"<video autoplay muted><source src="vid.mp4"></video>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "auto-play-video"));
    }

    #[test]
    fn test_skipped_heading_level_detected() {
        let critic = InnerCritic::new();
        let content = r#"<h1>Title</h1><h3>Section</h3>"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "skipped-heading-level"));
    }

    #[test]
    fn test_fixed_header_without_skip_detected() {
        let critic = InnerCritic::new();
        let content = r#"
            .nav { position: fixed; top: 0; width: 100%; }
            .nav a { padding: 10px; }
        "#;
        let violations = critic.audit(content);
        assert!(violations
            .iter()
            .any(|v| v.name == "fixed-header-without-skip"));
    }

    #[test]
    fn test_inter_only_font_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { font-family: 'Inter'; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "inter-only-font"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(CriticSeverity::Critical > CriticSeverity::High);
        assert!(CriticSeverity::High > CriticSeverity::Medium);
        assert!(CriticSeverity::Medium > CriticSeverity::Low);
        assert!(CriticSeverity::Low > CriticSeverity::Info);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", CriticSeverity::Info), "info");
        assert_eq!(format!("{}", CriticSeverity::Critical), "critical");
    }

    #[test]
    fn test_design_violation_clone() {
        let v = DesignViolation {
            id: "test".into(),
            name: "test-name".into(),
            severity: CriticSeverity::Medium,
            description: "desc".into(),
            location: "loc".into(),
            suggestion: "sugg".into(),
        };
        let v2 = v.clone();
        assert_eq!(v.id, v2.id);
        assert_eq!(v.severity, v2.severity);
    }

    #[test]
    fn test_uppercase_body_detected() {
        let critic = InnerCritic::new();
        let content = r#"body { text-transform: uppercase; font-size: 14px; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "uppercase-body"));
    }

    #[test]
    fn test_focus_visible_removed_detected() {
        let critic = InnerCritic::new();
        let content = r#"button:focus { outline: none; }"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "focus-visible-removed"));
    }

    #[test]
    fn test_form_without_label_detected() {
        let critic = InnerCritic::new();
        let content = r#"<input type="text" name="email">"#;
        let violations = critic.audit(content);
        assert!(violations.iter().any(|v| v.name == "form-without-label"));
    }

    #[test]
    fn test_missing_grid_detected() {
        let critic = InnerCritic::new();
        let content =
            "a very long CSS file with no grid or flexbox mentioned whatsoever ".repeat(50);
        let violations = critic.audit(&content);
        assert!(violations.iter().any(|v| v.name == "missing-grid"));
    }

    #[test]
    fn test_no_false_positives_on_clean_html() {
        let critic = InnerCritic::new();
        let content = r##"<!DOCTYPE html><html lang="en"><head><title>Clean</title></head><body><h1>Title</h1><p>Clean text</p><img src="a.jpg" alt="A"><a href="#">link</a></body></html>"##;
        let violations = critic.audit(content);
        let critical: Vec<&DesignViolation> = violations
            .iter()
            .filter(|v| v.severity >= CriticSeverity::High)
            .collect();
        assert!(
            critical.is_empty(),
            "Clean HTML should not produce high/critical violations: {:?}",
            critical
        );
    }

    #[test]
    fn test_audit_sorts_by_severity() {
        let critic = InnerCritic::new();
        let content = concat!(
            r#"<img src="x.jpg">"#,
            r#".card { border-radius: 4px; }"#,
            r#"<html><title>T</title></html>"#,
        );
        let violations = critic.audit(content);
        for i in 1..violations.len() {
            assert!(
                violations[i - 1].severity >= violations[i].severity,
                "Violations should be sorted by severity descending"
            );
        }
    }
}
