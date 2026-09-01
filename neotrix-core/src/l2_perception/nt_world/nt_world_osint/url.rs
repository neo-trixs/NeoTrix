use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{OsintConfig, OsintTarget};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSnapshot {
    pub url: String,
    pub timestamp: String,
    pub status: Option<u16>,
    pub mimetype: Option<String>,
    pub length: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UrlHistoryFindings {
    pub snapshots: Vec<UrlSnapshot>,
    pub domain: String,
}

impl std::fmt::Display for UrlHistoryFindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── URL History (Wayback Machine) ──")?;
        writeln!(f, "    Domain:     {}", self.domain)?;
        writeln!(f, "    Snapshots:  {}", self.snapshots.len())?;
        if !self.snapshots.is_empty() {
            let years: std::collections::BTreeSet<String> = self.snapshots.iter()
                .filter_map(|s| s.timestamp.get(..4))
                .map(|y| y.to_string())
                .collect();
            writeln!(f, "    Years:      {}", years.iter().cloned().collect::<Vec<_>>().join(", "))?;
            for snap in self.snapshots.iter().take(10) {
                writeln!(f, "      {} [{}] {}", snap.timestamp, snap.status.unwrap_or(0), snap.url)?;
            }
            if self.snapshots.len() > 10 {
                writeln!(f, "      ... and {} more", self.snapshots.len() - 10)?;
            }
        }
        Ok(())
    }
}

async fn query_cdx(domain: &str, client: &Client, from: &str, to: &str) -> Vec<UrlSnapshot> {
    // Use Wayback Machine CDX API: https://github.com/internetarchive/wayback/tree/master/wayback-cdx-server
    let url = format!(
        "https://web.archive.org/cdx/search/cdx?url={domain}/*&output=json&from={from}&to={to}&limit=5000"
    );
    match client.get(&url).timeout(Duration::from_secs(30)).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Vec<Vec<String>>>().await {
                Ok(rows) => {
                    let mut snapshots = Vec::new();
                    // First row is headers; skip it
                    for row in rows.iter().skip(1) {
                        if row.len() >= 6 {
                            let raw_url = row.first().cloned().unwrap_or_default();
                            let timestamp = row.get(1).cloned().unwrap_or_default();
                            let status = row.get(4).and_then(|s| s.parse::<u16>().ok());
                            let mimetype = row.get(3).cloned();
                            let length = row.get(5).and_then(|s| s.parse::<u64>().ok());
                            let archived_url = format!("https://web.archive.org/web/{timestamp}/{raw_url}");
                            snapshots.push(UrlSnapshot {
                                url: if raw_url.is_empty() { archived_url } else { raw_url },
                                timestamp: if timestamp.len() == 14 {
                                    format!("{}-{}-{} {}:{}:{}",
                                        &timestamp[..4], &timestamp[4..6], &timestamp[6..8],
                                        &timestamp[8..10], &timestamp[10..12], &timestamp[12..14])
                                } else { timestamp },
                                status,
                                mimetype,
                                length,
                            });
                        }
                    }
                    // Sort by timestamp descending (most recent first)
                    snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    // Remove duplicates
                    snapshots.dedup_by(|a, b| a.url == b.url && a.timestamp == b.timestamp);
                    snapshots
                }
                Err(_) => vec![],
            }
        }
        _ => vec![],
    }
}

pub async fn investigate(target: &OsintTarget, client: &Client, _config: &OsintConfig) -> Result<UrlHistoryFindings, String> {
    let domain = target.domain.as_ref().ok_or("no domain specified")?;
    let mut findings = UrlHistoryFindings {
        domain: domain.to_string(),
        ..Default::default()
    };

    // Query by decade for large sites
    let decades = ["1996", "2000", "2005", "2010", "2015", "2020", "2025"];
    for decade in &decades {
        let from = decade;
        let to = match *decade {
            "1996" => "1999",
            "2000" => "2004",
            "2005" => "2009",
            "2010" => "2014",
            "2015" => "2019",
            "2020" => "2024",
            "2025" => "2026",
            _ => "2026",
        };
        let snapshots = query_cdx(domain, client, from, to).await;
        findings.snapshots.extend(snapshots);
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    findings.snapshots.retain(|s| seen.insert((s.url.clone(), s.timestamp.clone())));

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_findings_default() {
        let f = UrlHistoryFindings::default();
        assert_eq!(f.domain, "");
        assert!(f.snapshots.is_empty());
    }

    #[test]
    fn test_url_findings_display_empty() {
        let f = UrlHistoryFindings::default();
        let s = format!("{f}");
        assert!(s.contains("URL History"), "output: {s}");
        assert!(s.contains("Snapshots:"), "output: {s}");
        assert!(s.len() > 20, "output: {s}");
    }

    #[test]
    fn test_timestamp_formatting() {
        let raw = "20230720123456";
        let formatted = format!("{}-{}-{} {}:{}:{}",
            &raw[..4], &raw[4..6], &raw[6..8],
            &raw[8..10], &raw[10..12], &raw[12..14]);
        assert_eq!(formatted, "2023-07-20 12:34:56");
    }

    #[test]
    fn test_cdx_url_format() {
        let domain = "example.com";
        let url = format!("https://web.archive.org/cdx/search/cdx?url={domain}/*&output=json&from=2020&to=2024&limit=5000");
        assert!(url.contains("example.com"));
        assert!(url.contains("output=json"));
    }
}
