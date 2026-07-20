use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnEntry {
    pub id: String,
    pub summary: String,
    pub severity: Option<String>,
    pub cvss_score: Option<f64>,
    pub published: Option<String>,
    pub affected: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VulnFindings {
    pub vulnerabilities: Vec<VulnEntry>,
    pub advisories: Vec<String>,
    pub domain: String,
}

impl std::fmt::Display for VulnFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Vulnerability Research ──")?;
        writeln!(f, "    Domain:     {}", self.domain)?;
        writeln!(f, "    Vulns:      {}", self.vulnerabilities.len())?;
        for v in &self.vulnerabilities {
            let severity_str = v.severity.as_deref().unwrap_or("N/A");
            let score_str = v.cvss_score.map(|s| format!(" ({s})")).unwrap_or_default();
            writeln!(f, "      {:<20} [{}{score_str}] {}", v.id, severity_str, v.summary)?;
        }
        Ok(())
    }
}

async fn query_osv(ecosystem: &str, package: &str, client: &Client) -> Vec<VulnEntry> {
    let url = "https://api.osv.dev/v1/query";
    let body = serde_json::json!({
        "package": {
            "ecosystem": ecosystem,
            "name": package
        }
    });
    match client.post(url)
        .json(&body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let mut vulns = Vec::new();
                    if let Some(vulns_arr) = json["vulns"].as_array() {
                        for v in vulns_arr {
                            let id = v["id"].as_str().unwrap_or("unknown").to_string();
                            let summary = v["summary"].as_str().unwrap_or("no summary").to_string();
                            let severity = v["severity"].as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|s| s["type"].as_str())
                                .map(|s| s.to_string());
                            let cvss = v["severity"].as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|s| s["score"].as_str())
                                .and_then(|s| s.parse::<f64>().ok());
                            let published = v["published"].as_str().map(|s| s.to_string());
                            let affected: Vec<String> = v["affected"].as_array()
                                .map(|arr| {
                                    arr.iter().filter_map(|a| {
                                        a["package"]["name"].as_str().map(|s| s.to_string())
                                    }).collect()
                                })
                                .unwrap_or_default();
                            vulns.push(VulnEntry {
                                id,
                                summary,
                                severity,
                                cvss_score: cvss,
                                published,
                                affected,
                                source: "osv.dev".to_string(),
                            });
                        }
                    }
                    vulns
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

async fn query_nvd(cpe: &str, client: &Client) -> Vec<VulnEntry> {
    let url = format!("https://services.nvd.nist.gov/rest/json/cves/2.0?cpeName={cpe}&resultsPerPage=20");
    match client.get(&url)
        .timeout(Duration::from_secs(15))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let mut vulns = Vec::new();
                    if let Some(vulns_arr) = json["vulnerabilities"].as_array() {
                        for v in vulns_arr {
                            let cve = v["cve"].clone();
                            let id = cve["id"].as_str().unwrap_or("unknown").to_string();
                            let descriptions = cve["descriptions"].as_array();
                            let summary = descriptions
                                .and_then(|d| d.iter().find(|d| d["lang"].as_str() == Some("en")))
                                .and_then(|d| d["value"].as_str())
                                .unwrap_or("no description")
                                .to_string();
                            let metrics = cve["metrics"].as_object();
                            let cvss_score = metrics
                                .and_then(|m| m.values().next())
                                .and_then(|v| v.as_array())
                                .and_then(|v| v.first())
                                .and_then(|v| v["cvssData"]["baseScore"].as_f64());
                            let severity = metrics
                                .and_then(|m| m.values().next())
                                .and_then(|v| v.as_array())
                                .and_then(|v| v.first())
                                .and_then(|v| v["cvssData"]["baseSeverity"].as_str())
                                .map(|s| s.to_string());
                            let published = cve["published"].as_str().map(|s| s.to_string());
                            vulns.push(VulnEntry {
                                id,
                                summary,
                                severity,
                                cvss_score,
                                published,
                                affected: vec![cpe.to_string()],
                                source: "nvd.nist.gov".to_string(),
                            });
                        }
                    }
                    vulns
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

fn guess_ecosystem(domain: &str, _tech_stack: &[String]) -> Vec<(String, String)> {
    let mut queries = Vec::new();
    let domain_lower = domain.to_lowercase();
    // Try npm for JS-heavy domains
    if domain_lower.contains("node") || domain_lower.contains("js") || domain_lower.contains("react") {
        queries.push(("npm".to_string(), domain.to_string()));
    }
    // Try PyPI for Python
    if domain_lower.contains("python") || domain_lower.contains("django") || domain_lower.contains("flask") {
        queries.push(("PyPI".to_string(), domain.to_string()));
    }
    // Try crates.io for Rust
    if domain_lower.contains("rust") || domain_lower.contains("cargo") {
        queries.push(("crates.io".to_string(), domain.to_string()));
    }
    // Try Go
    if domain_lower.contains("go") || domain_lower.contains("golang") {
        queries.push(("Go".to_string(), domain.to_string()));
    }
    // Try Maven for Java
    if domain_lower.contains("java") || domain_lower.contains("maven") || domain_lower.contains("spring") {
        queries.push(("Maven".to_string(), domain.to_string()));
    }
    // Also try the domain name itself as a package name
    let pkg_name = domain.split('.').next().unwrap_or(domain);
    queries.push(("npm".to_string(), pkg_name.to_string()));
    queries.push(("PyPI".to_string(), pkg_name.to_string()));
    queries
}

pub async fn investigate(target: &OsintTarget, client: &Client, _config: &OsintConfig) -> Result<VulnFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = VulnFindings {
        domain: domain.to_string(),
        ..Default::default()
    };

    let ecosystems = guess_ecosystem(domain, &[]);
    for (ecosystem, package) in &ecosystems {
        let vulns = query_osv(ecosystem, package, client).await;
        findings.vulnerabilities.extend(vulns);
    }

    // Try CPE search
    let pkg_name = domain.split('.').next().unwrap_or(domain);
    let cpe = format!("cpe:2.3:a:*:{pkg_name}:*:*:*:*:*:*:*");
    let nvd_vulns = query_nvd(&cpe, client).await;
    findings.vulnerabilities.extend(nvd_vulns);

    // Deduplicate by CVE ID
    let mut seen = std::collections::HashSet::new();
    findings.vulnerabilities.retain(|v| seen.insert(v.id.clone()));

    // Sort by CVSS score descending
    findings.vulnerabilities.sort_by(|a, b| {
        b.cvss_score.unwrap_or(0.0).partial_cmp(&a.cvss_score.unwrap_or(0.0)).unwrap()
    });

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vuln_findings_default() {
        let f = VulnFindings::default();
        assert!(f.vulnerabilities.is_empty());
        assert_eq!(f.domain, "");
    }

    #[test]
    fn test_vuln_findings_display_empty() {
        let f = VulnFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Vulnerability Research"));
    }

    #[test]
    fn test_guess_ecosystem_npm() {
        let tech = vec!["React".to_string(), "Node.js".to_string()];
        let results = guess_ecosystem("my-react-app.com", &tech);
        assert!(results.iter().any(|(e, _)| e == "npm"));
    }

    #[test]
    fn test_guess_ecosystem_pypi() {
        let tech = vec!["Django".to_string(), "Python".to_string()];
        let results = guess_ecosystem("my-django-app.com", &tech);
        assert!(results.iter().any(|(e, _)| e == "PyPI"));
    }

    #[test]
    fn test_guess_ecosystem_pkg_extraction() {
        let results = guess_ecosystem("expressjs.com", &[]);
        assert!(results.iter().any(|(_, p)| p == "expressjs"));
    }

    #[test]
    fn test_vuln_entry_new() {
        let v = VulnEntry {
            id: "CVE-2024-1234".to_string(),
            summary: "Test vuln".to_string(),
            severity: Some("CRITICAL".to_string()),
            cvss_score: Some(9.8),
            published: Some("2024-01-01".to_string()),
            affected: vec!["pkg".to_string()],
            source: "test".to_string(),
        };
        assert_eq!(v.id, "CVE-2024-1234");
    }

    #[test]
    fn test_osv_url_format() {
        let body = serde_json::json!({"package": {"ecosystem": "npm", "name": "express"}});
        assert_eq!(body["package"]["name"], "express");
    }

    #[test]
    fn test_cpe_format() {
        let pkg = "express";
        let cpe = format!("cpe:2.3:a:*:{pkg}:*:*:*:*:*:*:*");
        assert!(cpe.contains("express"));
    }
}
