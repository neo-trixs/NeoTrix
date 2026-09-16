use std::path::Path;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub category: IssueCategory,
    pub severity: IssueSeverity,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueCategory {
    MissingFields,
    InvalidFormat,
    InconsistentData,
    OrphanedReference,
    BrokenLink,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Critical,
    Major,
    Minor,
    Info,
}

pub struct ArtifactValidator;

impl ArtifactValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_json(&self, content: &str) -> ValidationResult {
        let mut issues = Vec::new();
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(val) => {
                if val.is_null() {
                    issues.push(ValidationIssue {
                        category: IssueCategory::InvalidFormat,
                        severity: IssueSeverity::Major,
                        message: "JSON is null".into(),
                        location: None,
                    });
                }
                if let Some(obj) = val.as_object() {
                    for (k, v) in obj {
                        if v.is_null() {
                            issues.push(ValidationIssue {
                                category: IssueCategory::MissingFields,
                                severity: IssueSeverity::Minor,
                                message: format!("Field '{}' is null", k),
                                location: Some(k.clone()),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                issues.push(ValidationIssue {
                    category: IssueCategory::InvalidFormat,
                    severity: IssueSeverity::Critical,
                    message: format!("Invalid JSON: {}", e),
                    location: None,
                });
            }
        }
        let score = if issues.is_empty() {
            100.0
        } else {
            let total_deduction: f64 = issues
                .iter()
                .map(|i| match i.severity {
                    IssueSeverity::Critical => 30.0,
                    IssueSeverity::Major => 15.0,
                    IssueSeverity::Minor => 5.0,
                    IssueSeverity::Info => 1.0,
                })
                .sum();
            (100.0 - total_deduction).max(0.0)
        };
        ValidationResult {
            valid: issues.iter().all(|i| i.severity != IssueSeverity::Critical),
            issues,
            score,
        }
    }

    pub fn validate_markdown(&self, content: &str) -> ValidationResult {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            issues.push(ValidationIssue {
                category: IssueCategory::MissingFields,
                severity: IssueSeverity::Major,
                message: "Empty document".into(),
                location: None,
            });
        }
        let has_header = lines.iter().any(|l| l.starts_with("# "));
        if !has_header {
            issues.push(ValidationIssue {
                category: IssueCategory::MissingFields,
                severity: IssueSeverity::Minor,
                message: "No header found".into(),
                location: None,
            });
        }
        let broken_links: Vec<String> = lines
            .iter()
            .filter_map(|l| {
                let link = l.strip_prefix('[')?.strip_suffix(']')?;
                Some(link.to_string())
            })
            .filter(|link| !link.starts_with("http") && !link.starts_with('#'))
            .collect();
        for link in &broken_links {
            issues.push(ValidationIssue {
                category: IssueCategory::BrokenLink,
                severity: IssueSeverity::Minor,
                message: format!("Potentially broken link: {}", link),
                location: None,
            });
        }
        let score = if issues.is_empty() {
            100.0
        } else {
            (100.0 - issues.len() as f64 * 5.0).max(0.0)
        };
        ValidationResult {
            valid: true,
            issues,
            score,
        }
    }

    pub fn validate_file(
        &self,
        path: &Path,
    ) -> Result<ValidationResult, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Ok(match ext {
            "json" | "jsonc" => self.validate_json(&content),
            "md" | "markdown" => self.validate_markdown(&content),
            _ => ValidationResult {
                valid: true,
                issues: vec![],
                score: 100.0,
            },
        })
    }
}

impl Default for ArtifactValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let v = ArtifactValidator::new();
        let r = v.validate_json(r#"{"name":"test","value":42}"#);
        assert!(r.valid);
        assert!(r.score > 90.0);
    }

    #[test]
    fn test_invalid_json() {
        let v = ArtifactValidator::new();
        let r = v.validate_json("not json");
        assert!(!r.valid);
        assert!(r.score < 50.0);
    }

    #[test]
    fn test_null_field() {
        let v = ArtifactValidator::new();
        let r = v.validate_json(r#"{"name":null}"#);
        assert!(r.valid);
        assert_eq!(r.issues.len(), 1);
    }

    #[test]
    fn test_markdown_no_header() {
        let v = ArtifactValidator::new();
        let r = v.validate_markdown("Just some text\nNo header here");
        assert!(!r.issues.is_empty());
    }
}
