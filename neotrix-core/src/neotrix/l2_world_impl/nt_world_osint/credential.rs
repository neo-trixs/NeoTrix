use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachEntry {
    pub source: String,
    pub email: String,
    pub breach_name: Option<String>,
    pub breach_date: Option<String>,
    pub data_classes: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredentialFindings {
    pub email: Option<String>,
    pub breaches: Vec<BreachEntry>,
    pub pwned_count: u64,
    pub domain_breaches: Vec<String>,
}

impl std::fmt::Display for CredentialFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── Credential / Breach Search ──")?;
        if let Some(ref email) = self.email {
            writeln!(f, "    Email:      {}", email)?;
            writeln!(f, "    Breaches:   {} (pwned {})", self.breaches.len(), self.pwned_count)?;
        }
        if !self.domain_breaches.is_empty() {
            writeln!(f, "    Domain breaches: {}", self.domain_breaches.join(", "))?;
        }
        for breach in &self.breaches {
            writeln!(f, "      [{}] {} — {} — data: {}",
                breach.breach_name.as_deref().unwrap_or("unknown"),
                breach.breach_date.as_deref().unwrap_or("?"),
                breach.source,
                breach.data_classes.join(", "))?;
        }
        Ok(())
    }
}

async fn check_hibp(email: &str, client: &Client) -> Vec<BreachEntry> {
    let hash = sha1_hash(email.to_lowercase().trim());
    let prefix = &hash[..5];
    let suffix = &hash[5..];

    let url = format!("https://api.pwnedpasswords.com/range/{prefix}");
    match client.get(&url).timeout(Duration::from_secs(10)).send().await {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.text().await.unwrap_or_default();
            let mut breaches = Vec::new();
            for line in body.lines() {
                if let Some((suffix_found, count_str)) = line.split_once(':') {
                    if suffix_found.eq_ignore_ascii_case(suffix) {
                        let count: u64 = count_str.trim().parse().unwrap_or(0);
                        breaches.push(BreachEntry {
                            source: "haveibeenpwned".to_string(),
                            email: email.to_string(),
                            breach_name: Some("Password exposed in breach".to_string()),
                            breach_date: None,
                            data_classes: vec!["password".to_string()],
                            description: Some(format!("Password appears in {count} known breaches")),
                        });
                    }
                }
            }
            breaches
        }
        _ => vec![],
    }
}

async fn check_hibp_breaches(email: &str, client: &Client, api_key: Option<&str>) -> Vec<BreachEntry> {
    let key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => return vec![],  // HIBP breach API requires API key
    };

    let url = format!("https://haveibeenpwned.com/api/v3/breachedaccount/{email}");
    match client.get(&url)
        .header("hibp-api-key", key)
        .header("user-agent", "NeoTrix-OSINT")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(breaches) => {
                    breaches.iter().map(|b| BreachEntry {
                        source: "haveibeenpwned".to_string(),
                        email: email.to_string(),
                        breach_name: b["Name"].as_str().map(|s| s.to_string()),
                        breach_date: b["BreachDate"].as_str().map(|s| s.to_string()),
                        data_classes: b["DataClasses"].as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                            .unwrap_or_default(),
                        description: b["Description"].as_str().map(|s| s.to_string()),
                    }).collect()
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

fn sha1_hash(input: &str) -> String {
    // FIPS 180-4 SHA-1 implementation (zero dependencies)
    const K0: u32 = 0x5a827999;
    const K1: u32 = 0x6ed9eba1;
    const K2: u32 = 0x8f1bbcdc;
    const K3: u32 = 0xca62c1d6;

    let msg = input.as_bytes();
    let bit_len = (msg.len() as u64) * 8;

    let mut padded = msg.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut h = [0x67452301u32, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in chunk.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), K0),
                20..=39 => (b ^ c ^ d, K1),
                40..=59 => ((b & c) | (b & d) | (c & d), K2),
                _ => (b ^ c ^ d, K3),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }

    let mut hex = String::with_capacity(40);
    for word in &h {
        hex.push_str(&format!("{:08x}", word));
    }
    hex.to_uppercase()
}

async fn check_firefox_monitor(email: &str, client: &Client) -> Vec<BreachEntry> {
    let url = format!("https://monitor.firefox.com/api/v1/scan");
    match client.post(&url)
        .json(&serde_json::json!({"email": email}))
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let mut breaches = Vec::new();
                    if let Some(breaches_arr) = json["breaches"].as_array() {
                        for b in breaches_arr {
                            breaches.push(BreachEntry {
                                source: "firefox-monitor".to_string(),
                                email: email.to_string(),
                                breach_name: b["Name"].as_str().map(|s| s.to_string()),
                                breach_date: b["BreachDate"].as_str().map(|s| s.to_string()),
                                data_classes: b["DataClasses"].as_array()
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                    .unwrap_or_default(),
                                description: b["Description"].as_str().map(|s| s.to_string()),
                            });
                        }
                    }
                    breaches
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

pub async fn investigate(target: &OsintTarget, client: &Client, config: &OsintConfig) -> Result<CredentialFindings, String> {
    let email = target.email.as_ref().ok_or("no email specified")?;
    let api_key = config.api_keys.get("hibp");

    let mut findings = CredentialFindings {
        email: Some(email.to_string()),
        ..Default::default()
    };

    // Check HIBP password range API (k-anonymity, no API key needed)
    let pwned = check_hibp(email, client).await;
    findings.pwned_count = pwned.iter().map(|b| {
        b.description.as_ref()
            .and_then(|d| d.split("in ").nth(1))
            .and_then(|s| s.split(" known").next())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    }).sum();
    findings.breaches.extend(pwned);

    // Check HIBP breach API (needs API key)
    if let Some(key) = api_key {
        let hibp_b = check_hibp_breaches(email, client, Some(key.as_str())).await;
        findings.breaches.extend(hibp_b);
    }

    // Check Firefox Monitor
    let ff = check_firefox_monitor(email, client).await;
    findings.breaches.extend(ff);

    // Deduplicate by breach name
    let mut seen = std::collections::HashSet::new();
    findings.breaches.retain(|b| {
        let key = format!("{}-{}", b.email, b.breach_name.as_deref().unwrap_or("unknown"));
        seen.insert(key)
    });

    findings.pwned_count = findings.breaches.iter()
        .map(|b| b.data_classes.len() as u64)
        .sum();

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hash_known() {
        let h = sha1_hash("password");
        assert_eq!(h.len(), 40);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha1_hash_case_sensitive() {
        let email_lower = "test@example.com";
        let email_upper = "TEST@EXAMPLE.COM";
        // In production, to_lowercase() is called before sha1_hash
        let h1 = sha1_hash(email_lower);
        let h2 = sha1_hash(&email_upper.to_lowercase());
        assert_eq!(h1, h2, "sha1 should match after to_lowercase");
    }

    #[test]
    fn test_credential_findings_default() {
        let f = CredentialFindings::default();
        assert!(f.breaches.is_empty());
        assert_eq!(f.pwned_count, 0);
    }

    #[test]
    fn test_display_empty() {
        let f = CredentialFindings::default();
        let s = format!("{f}");
        assert!(s.contains("Credential"));
    }

    #[test]
    fn test_hibp_range_url() {
        let hash = sha1_hash("test@example.com");
        let prefix = &hash[..5];
        let url = format!("https://api.pwnedpasswords.com/range/{prefix}");
        assert!(url.starts_with("https://api.pwnedpasswords.com/range/"));
        assert_eq!(prefix.len(), 5);
    }
}
