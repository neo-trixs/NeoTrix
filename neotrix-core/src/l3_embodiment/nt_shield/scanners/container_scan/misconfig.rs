use serde::{Deserialize, Serialize};

use super::vulnerability::Severity;

/// A Dockerfile misconfiguration check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisconfigCheck {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub description: String,
    pub remediation: String,
}

/// Checks Dockerfiles for common security misconfigurations
pub struct DockerfileChecker;

impl DockerfileChecker {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a Dockerfile for misconfigurations
    pub fn check(path: &str) -> Vec<MisconfigCheck> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        Self::check_content(&content)
    }

    /// Analyze Dockerfile content string for misconfigurations
    pub fn check_content(content: &str) -> Vec<MisconfigCheck> {
        let mut checks = Vec::new();
        let mut has_user = false;
        let mut uses_latest = false;
        let mut has_secrets = false;
        let mut run_with_curl_wget = false;

        for line in content.lines() {
            let trimmed = line.trim();
            let upper = trimmed.to_uppercase();

            if upper.starts_with("FROM ") {
                if trimmed.to_lowercase().ends_with(":latest") || !trimmed.contains(":") {
                    uses_latest = true;
                }
            }

            if upper.starts_with("USER ") {
                let user_part = trimmed[5..].trim();
                if !user_part.is_empty() && user_part.to_lowercase() != "root" {
                    has_user = true;
                }
            }

            if upper.starts_with("COPY ") || upper.starts_with("ADD ") {
                let arg = if upper.starts_with("COPY ") {
                    trimmed[5..].trim()
                } else {
                    trimmed[4..].trim()
                };
                let lower_arg = arg.to_lowercase();
                if lower_arg.contains("id_rsa")
                    || lower_arg.contains(".env")
                    || lower_arg.contains("credentials")
                    || lower_arg.contains("secret")
                    || lower_arg.contains("token")
                    || lower_arg.contains("password")
                {
                    has_secrets = true;
                }
            }

            if upper.starts_with("RUN ") {
                let cmd = trimmed[4..].to_lowercase();
                if cmd.contains("curl ") || cmd.contains("wget ") || cmd.contains("http://") {
                    run_with_curl_wget = true;
                }
            }
        }

        if !has_user {
            checks.push(MisconfigCheck {
                id: "DC-001".to_string(),
                title: "Container runs as root".to_string(),
                severity: Severity::High,
                description: "No USER instruction found. Container will run as root by default."
                    .to_string(),
                remediation: "Add a non-root USER instruction after package installation."
                    .to_string(),
            });
        }

        if uses_latest {
            checks.push(MisconfigCheck {
                id: "DC-002".to_string(),
                title: "Using latest or untagged base image".to_string(),
                severity: Severity::Medium,
                description:
                    "Base image uses :latest tag or no tag, causing non-reproducible builds."
                        .to_string(),
                remediation: "Pin base images to specific version tags or digests.".to_string(),
            });
        }

        if has_secrets {
            checks.push(MisconfigCheck {
                id: "DC-003".to_string(),
                title: "Secrets or credentials copied into image".to_string(),
                severity: Severity::Critical,
                description: "COPY/ADD instruction references files that may contain secrets.".to_string(),
                remediation: "Use multi-stage builds or runtime secret injection (e.g. Docker secrets, env vars).".to_string(),
            });
        }

        if run_with_curl_wget {
            checks.push(MisconfigCheck {
                id: "DC-004".to_string(),
                title: "Downloading files during build via curl/wget".to_string(),
                severity: Severity::Low,
                description: "RUN instruction uses curl/wget to download files over HTTP."
                    .to_string(),
                remediation: "Verify download integrity with checksums and prefer HTTPS."
                    .to_string(),
            });
        }

        checks
    }
}

impl Default for DockerfileChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_dockerfile() {
        // Stale: the fixture carried `RUN curl … -o /app`, which legitimately
        // trips DC-004 (build-time download). Removed so the fixture is clean.
        let content = r#"FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y curl
USER nobody
EXPOSE 8080
CMD ["/app"]
"#;
        let checks = DockerfileChecker::check_content(content);
        assert!(
            checks.is_empty(),
            "Clean dockerfile should have no misconfigs: {:?}",
            checks
        );
    }

    #[test]
    fn test_root_user() {
        let content = r#"FROM ubuntu:22.04
RUN apt-get update
CMD ["/bin/bash"]
"#;
        let checks = DockerfileChecker::check_content(content);
        assert!(checks.iter().any(|c| c.id == "DC-001"));
    }

    #[test]
    fn test_latest_tag() {
        let content = r#"FROM python
RUN pip install flask
USER app
CMD ["python", "app.py"]
"#;
        let checks = DockerfileChecker::check_content(content);
        assert!(checks.iter().any(|c| c.id == "DC-002"));
    }

    #[test]
    fn test_secrets_in_copy() {
        let content = r#"FROM node:20-alpine
COPY .env /app/.env
COPY id_rsa /root/.ssh/id_rsa
USER node
CMD ["node", "server.js"]
"#;
        let checks = DockerfileChecker::check_content(content);
        assert!(checks.iter().any(|c| c.id == "DC-003"));
    }

    #[test]
    fn test_curl_in_run() {
        let content = r#"FROM alpine:3.19
RUN curl -fsSL https://example.com/script.sh | sh
USER root
"#;
        let checks = DockerfileChecker::check_content(content);
        assert!(checks.iter().any(|c| c.id == "DC-004"));
    }
}
