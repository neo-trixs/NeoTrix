use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceType {
    GitHubRepo,
    GitHubFile,
    Url,
    Pdf,
    LocalPath,
}

impl fmt::Display for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::GitHubRepo => write!(f, "GitHub Repository"),
            SourceType::GitHubFile => write!(f, "GitHub File"),
            SourceType::Url => write!(f, "URL"),
            SourceType::Pdf => write!(f, "PDF"),
            SourceType::LocalPath => write!(f, "Local Path"),
        }
    }
}

pub trait Accessor: Send + Sync {
    fn source_type(&self) -> SourceType;
    fn identifier(&self) -> &str;
    fn fetch(&self) -> Result<String, String>;
    fn analyze(&self, content: &str) -> Result<AccessionReport, String>;
}

#[derive(Debug, Clone)]
pub struct AccessionReport {
    pub title: String,
    pub summary: String,
    pub domains: Vec<String>,
    pub estimated_impact: f64,
    pub source_type: SourceType,
    pub suggested_name: Option<String>,
}

pub struct UrlAccessor {
    url: String,
}

impl UrlAccessor {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string() }
    }
}

impl Accessor for UrlAccessor {
    fn source_type(&self) -> SourceType {
        SourceType::Url
    }

    fn identifier(&self) -> &str {
        &self.url
    }

    fn fetch(&self) -> Result<String, String> {
        #[cfg(feature = "full")]
        {
            let resp = reqwest::blocking::get(&self.url)
                .map_err(|e| format!("Failed to fetch URL: {}", e))?;
            resp.text().map_err(|e| format!("Failed to read response: {}", e))
        }
        #[cfg(not(feature = "full"))]
        {
            Err("reqwest not available (feature 'full' not enabled)".to_string())
        }
    }

    fn analyze(&self, content: &str) -> Result<AccessionReport, String> {
        let title = content
            .lines()
            .find(|l| l.starts_with("# ") || l.to_lowercase().contains("title"))
            .map(|l| l.trim_start_matches("# ").trim_start_matches("title: ").trim().to_string())
            .unwrap_or_else(|| "Unknown Source".to_string());

        let has_code_blocks = content.contains("```");
        let has_markdown_hdrs = content.lines().any(|l| l.starts_with("##"));
        let word_count = content.split_whitespace().count();

        let mut domains = Vec::new();
        if has_code_blocks {
            domains.push("code".to_string());
        }
        if has_markdown_hdrs {
            domains.push("documentation".to_string());
        }
        if word_count < 100 {
            domains.push("minimal".to_string());
        }

        let estimated_impact = if domains.contains(&"documentation".to_string()) && word_count > 200 {
            0.7
        } else if domains.contains(&"code".to_string()) {
            0.6
        } else {
            0.3
        };

        let summary = format!(
            "{} words, {} code blocks, source type: {}",
            word_count,
            content.matches("```").count() / 2,
            self.source_type()
        );

        Ok(AccessionReport {
            title,
            summary,
            domains,
            estimated_impact,
            source_type: self.source_type(),
            suggested_name: None,
        })
    }
}

/// Accessor for GitHub repositories (clones via git)
pub struct GitHubRepoAccessor {
    pub url: String,
    pub local_path: Option<PathBuf>,
}

impl GitHubRepoAccessor {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), local_path: None }
    }
}

impl Accessor for GitHubRepoAccessor {
    fn source_type(&self) -> SourceType {
        SourceType::GitHubRepo
    }

    fn identifier(&self) -> &str {
        &self.url
    }

    fn fetch(&self) -> Result<String, String> {
        let repo_name = self.url.split('/').next_back().unwrap_or("repo");
        let tmp_dir = std::env::temp_dir().join(format!("neotrix_compare_{}", repo_name));
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
        }
        let status = std::process::Command::new("git")
            .args(["clone", "--depth=1", &self.url, tmp_dir.to_str().expect("result")])
            .status()
            .map_err(|e| format!("git clone failed: {}", e))?;
        if !status.success() {
            return Err("git clone returned non-zero exit".to_string());
        }
        let readme_path = tmp_dir.join("README.md");
        if readme_path.exists() {
            std::fs::read_to_string(&readme_path).map_err(|e| e.to_string())
        } else {
            let readme = tmp_dir.join("README");
            if readme.exists() {
                std::fs::read_to_string(&readme).map_err(|e| e.to_string())
            } else {
                Ok(format!("Cloned {} but no README found", repo_name))
            }
        }
    }

    fn analyze(&self, content: &str) -> Result<AccessionReport, String> {
        let title = content.lines()
            .find(|l| l.starts_with('#'))
            .map(|l| l.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| self.url.clone());
        let word_count = content.split_whitespace().count();
        let domains = vec!["code".to_string(), "documentation".to_string(), "comparison".to_string()];
        let estimated_impact = (word_count as f64 / 500.0).min(1.0) * 0.8;
        Ok(AccessionReport {
            title,
            summary: format!("GitHub repository at {} ({} words)", self.url, word_count),
            domains,
            estimated_impact,
            source_type: SourceType::GitHubRepo,
            suggested_name: Some(self.url.split('/').next_back().unwrap_or("repo").to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_display() {
        assert_eq!(SourceType::Url.to_string(), "URL");
        assert_eq!(SourceType::GitHubRepo.to_string(), "GitHub Repository");
    }

    #[test]
    fn test_url_accessor_identifier() {
        let a = UrlAccessor::new("https://example.com");
        assert_eq!(a.identifier(), "https://example.com");
        assert_eq!(a.source_type(), SourceType::Url);
    }

    #[test]
    fn test_analyze_markdown_content() {
        let a = UrlAccessor::new("https://example.com");
        let content = "# My Project\n\n## Overview\nThis is a cool project.\n\n```rust\nfn main() {}\n```";
        let report = a.analyze(content).expect("value should be ok in test");
        assert_eq!(report.title, "My Project");
        assert!(report.estimated_impact > 0.5);
        assert!(report.domains.contains(&"documentation".to_string()));
    }

    #[test]
    fn test_analyze_short_content_low_impact() {
        let a = UrlAccessor::new("https://example.com");
        let report = a.analyze("hello world").expect("value should be ok in test");
        assert!(report.estimated_impact < 0.5);
        assert!(report.domains.contains(&"minimal".to_string()));
    }

    #[test]
    fn test_accession_report_clone() {
        let r = AccessionReport {
            title: "Test".into(),
            summary: "sum".into(),
            domains: vec!["a".into()],
            estimated_impact: 0.8,
            source_type: SourceType::Pdf,
            suggested_name: Some("test".into()),
        };
        let r2 = r.clone();
        assert_eq!(r.title, r2.title);
        assert_eq!(r.estimated_impact, r2.estimated_impact);
    }
}
