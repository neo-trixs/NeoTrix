use std::time::Duration;

use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::super::OsintConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailSource {
    WebSearch,
    Dns,
    Whois,
    Social,
}

impl std::fmt::Display for EmailSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailSource::WebSearch => write!(f, "web_search"),
            EmailSource::Dns => write!(f, "dns"),
            EmailSource::Whois => write!(f, "whois"),
            EmailSource::Social => write!(f, "social"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub email: String,
    pub source: EmailSource,
    pub confidence: f64,
}

impl EmailResult {
    pub fn new(email: String, source: EmailSource, confidence: f64) -> Self {
        Self { email, source, confidence: confidence.clamp(0.0, 1.0) }
    }
}

pub struct EmailHarvester {
    client: Client,
    email_regex: Regex,
}

impl EmailHarvester {
    pub fn new(client: Client) -> Self {
        let email_regex = Regex::new(
            r"(?i)[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}"
        ).expect("valid email regex");
        Self { client, email_regex }
    }

    pub async fn harvest(&self, domain: &str, _config: &OsintConfig) -> Vec<EmailResult> {
        let mut results = Vec::new();

        let mut dns_emails = self.harvest_from_dns(domain).await;
        results.append(&mut dns_emails);

        let mut web_emails = self.harvest_from_web(domain).await;
        results.append(&mut web_emails);

        let mut social_emails = self.harvest_from_social(domain).await;
        results.append(&mut social_emails);

        let mut whois_emails = self.harvest_from_whois(domain).await;
        results.append(&mut whois_emails);

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results.dedup_by(|a, b| a.email == b.email);
        results
    }

    async fn harvest_from_dns(&self, domain: &str) -> Vec<EmailResult> {
        let mut results = Vec::new();
        let url = format!("https://dns.google/resolve?name={domain}&type=TXT");
        if let Ok(resp) = self.client.get(&url)
            .header("accept", "application/dns-json")
            .timeout(Duration::from_secs(10))
            .send().await
        {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(answers) = json["Answer"].as_array() {
                    for ans in answers {
                        if let Some(data) = ans["data"].as_str() {
                            for cap in self.email_regex.find_iter(data) {
                                let email = cap.as_str().to_lowercase();
                                if email.ends_with(domain) {
                                    results.push(EmailResult::new(
                                        email, EmailSource::Dns, 0.85,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }

    async fn harvest_from_web(&self, domain: &str) -> Vec<EmailResult> {
        let mut results = Vec::new();
        let urls = [
            format!("https://{domain}"),
            format!("https://www.{domain}/about"),
            format!("https://www.{domain}/contact"),
        ];
        for url in &urls {
            if let Ok(resp) = self.client.get(url)
                .timeout(Duration::from_secs(10))
                .send().await
            {
                if let Ok(body) = resp.text().await {
                    for cap in self.email_regex.find_iter(&body) {
                        let email = cap.as_str().to_lowercase();
                        if email.ends_with(domain) && !results.iter().any(|r: &EmailResult| r.email == email) {
                            results.push(EmailResult::new(
                                email, EmailSource::WebSearch, 0.70,
                            ));
                        }
                    }
                }
            }
        }
        results
    }

    async fn harvest_from_social(&self, domain: &str) -> Vec<EmailResult> {
        let mut results = Vec::new();
        let query = format!("site:linkedin.com \"{domain}\" email");
        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(&query));
        if let Ok(resp) = self.client.get(&url)
            .timeout(Duration::from_secs(10))
            .send().await
        {
            if let Ok(body) = resp.text().await {
                for cap in self.email_regex.find_iter(&body) {
                    let email = cap.as_str().to_lowercase();
                    if email.ends_with(domain) {
                        results.push(EmailResult::new(
                            email, EmailSource::Social, 0.55,
                        ));
                    }
                }
            }
        }
        results
    }

    async fn harvest_from_whois(&self, domain: &str) -> Vec<EmailResult> {
        let mut results = Vec::new();
        let url = format!("https://rdap.verisign.com/com/v1/domain/{}", domain);
        if let Ok(resp) = self.client.get(&url)
            .timeout(Duration::from_secs(10))
            .send().await
        {
            if let Ok(body) = resp.text().await {
                for cap in self.email_regex.find_iter(&body) {
                    let email = cap.as_str().to_lowercase();
                    results.push(EmailResult::new(
                        email, EmailSource::Whois, 0.60,
                    ));
                }
            }
        }
        results
    }

    pub fn extract_emails_from_text(&self, text: &str) -> Vec<String> {
        let mut emails: Vec<String> = self.email_regex
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect();
        emails.sort();
        emails.dedup();
        emails
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_regex_basic() {
        let h = EmailHarvester::new(Client::new());
        let emails = h.extract_emails_from_text("Contact admin@example.com or info@test.org");
        assert_eq!(emails.len(), 2);
        assert!(emails.contains(&"admin@example.com".to_string()));
        assert!(emails.contains(&"info@test.org".to_string()));
    }

    #[test]
    fn test_email_regex_no_match() {
        let h = EmailHarvester::new(Client::new());
        let emails = h.extract_emails_from_text("no emails here");
        assert!(emails.is_empty());
    }

    #[test]
    fn test_email_regex_dedup() {
        let h = EmailHarvester::new(Client::new());
        let emails = h.extract_emails_from_text("a@b.com a@b.com a@b.com");
        assert_eq!(emails.len(), 1);
    }

    #[test]
    fn test_email_regex_case_insensitive() {
        let h = EmailHarvester::new(Client::new());
        let emails = h.extract_emails_from_text("User@Example.COM");
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0], "user@example.com");
    }

    #[test]
    fn test_email_result_confidence_clamped() {
        let r = EmailResult::new("x@y.com".into(), EmailSource::Dns, 1.5);
        assert!((r.confidence - 1.0).abs() < 1e-6);
        let r2 = EmailResult::new("x@y.com".into(), EmailSource::Dns, -0.5);
        assert!((r2.confidence - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_email_result_new() {
        let r = EmailResult::new("test@domain.com".into(), EmailSource::Whois, 0.75);
        assert_eq!(r.email, "test@domain.com");
        assert_eq!(r.source, EmailSource::Whois);
        assert!((r.confidence - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_email_source_display() {
        assert_eq!(format!("{}", EmailSource::WebSearch), "web_search");
        assert_eq!(format!("{}", EmailSource::Dns), "dns");
        assert_eq!(format!("{}", EmailSource::Whois), "whois");
        assert_eq!(format!("{}", EmailSource::Social), "social");
    }

    #[test]
    fn test_harvester_new() {
        let h = EmailHarvester::new(Client::new());
        assert!(!h.email_regex.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_harvest_empty_domain() {
        let h = EmailHarvester::new(Client::new());
        let config = OsintConfig::default();
        let results = h.harvest("nonexistent-domain-zzz.invalid", &config).await;
        assert!(results.is_empty());
    }
}
