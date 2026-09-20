use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertEntry {
    pub subdomain: String,
    pub issuer: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}

pub struct CertTransparency {
    client: Client,
}

impl CertTransparency {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn query(&self, domain: &str) -> Vec<CertEntry> {
        let url = format!("https://crt.sh/?q=%25.{domain}&output=json");
        match self.client.get(&url)
            .timeout(Duration::from_secs(15))
            .send().await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Vec<serde_json::Value>>().await {
                    Ok(entries) => {
                        let mut certs: Vec<CertEntry> = entries.iter()
                            .filter_map(|e| parse_cert_entry(e, domain))
                            .collect();
                        certs.sort_by(|a, b| b.not_before.cmp(&a.not_before));
                        certs.dedup_by(|a, b| a.subdomain == b.subdomain && a.issuer == b.issuer);
                        certs
                    }
                    Err(_) => vec![],
                }
            }
            _ => vec![],
        }
    }
}

fn parse_cert_entry(entry: &serde_json::Value, domain: &str) -> Option<CertEntry> {
    let name_value = entry["name_value"].as_str()?;
    let subdomains: Vec<&str> = name_value.split('\n')
        .map(|s| s.trim().trim_start_matches("*.").trim())
        .filter(|s| s.ends_with(domain) && !s.is_empty())
        .collect();

    let issuer = entry["issuer_name"].as_str()
        .unwrap_or("Unknown")
        .trim()
        .to_string();

    let not_before = entry["not_before"].as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now());

    let not_after = entry["not_after"].as_str()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|| Utc::now());

    subdomains.into_iter().next().map(|sub| CertEntry {
        subdomain: sub.to_string(),
        issuer,
        not_before,
        not_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cert_entry_creation() {
        let entry = CertEntry {
            subdomain: "api.example.com".into(),
            issuer: "Let's Encrypt".into(),
            not_before: Utc::now(),
            not_after: Utc::now(),
        };
        assert_eq!(entry.subdomain, "api.example.com");
        assert_eq!(entry.issuer, "Let's Encrypt");
    }

    #[test]
    fn test_parse_cert_entry_valid() {
        let json = serde_json::json!({
            "name_value": "api.example.com\nwww.example.com",
            "issuer_name": "DigiCert Inc",
            "not_before": "2024-01-01T00:00:00Z",
            "not_after": "2025-01-01T00:00:00Z",
        });
        let entry = parse_cert_entry(&json, "example.com");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert!(e.subdomain.ends_with("example.com"));
        assert_eq!(e.issuer, "DigiCert Inc");
    }

    #[test]
    fn test_parse_cert_entry_wildcard() {
        let json = serde_json::json!({
            "name_value": "*.example.com",
            "issuer_name": "Let's Encrypt",
            "not_before": "2024-06-01T00:00:00Z",
            "not_after": "2024-12-01T00:00:00Z",
        });
        let entry = parse_cert_entry(&json, "example.com");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().subdomain, "example.com");
    }

    #[test]
    fn test_parse_cert_entry_different_domain() {
        let json = serde_json::json!({
            "name_value": "api.other.com",
            "issuer_name": "Let's Encrypt",
            "not_before": "2024-01-01T00:00:00Z",
            "not_after": "2025-01-01T00:00:00Z",
        });
        let entry = parse_cert_entry(&json, "example.com");
        assert!(entry.is_none());
    }

    #[test]
    fn test_parse_cert_entry_missing_fields() {
        let json = serde_json::json!({});
        let entry = parse_cert_entry(&json, "example.com");
        assert!(entry.is_none());
    }

    #[test]
    fn test_cert_entry_serialization() {
        let entry = CertEntry {
            subdomain: "test.example.com".into(),
            issuer: "ISRG".into(),
            not_before: Utc::now(),
            not_after: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("test.example.com"));
        assert!(json.contains("ISRG"));
    }

    #[test]
    fn test_parse_cert_entry_invalid_date() {
        let json = serde_json::json!({
            "name_value": "bad.example.com",
            "issuer_name": "Unknown",
            "not_before": "not-a-date",
            "not_after": "not-a-date",
        });
        let entry = parse_cert_entry(&json, "example.com");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert!(e.subdomain.ends_with("example.com"));
    }

    #[test]
    fn test_cert_transparency_new() {
        let ct = CertTransparency::new(Client::new());
        // Client created successfully without proxy
    }
}
