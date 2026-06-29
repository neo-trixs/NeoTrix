//! Selector fingerprint persistence (G301.3)
//!
//! Stores and retrieves AdaptiveSelector fingerprints for URL-pattern-based
//! re-identification across sessions.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Serializable version of StructuralFingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableFingerprint {
    pub tag_path: String,
    pub text_density: f64,
    pub child_count: usize,
    pub depth: usize,
    pub sibling_index: usize,
    pub similar_sibling_count: usize,
    pub attr_keys: Vec<String>,
    pub classes: Vec<String>,
    pub id: Option<String>,
    pub text_sig: u64,
    pub has_links: bool,
    pub has_images: bool,
    pub has_tables: bool,
}

/// Persisted selector fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorRecord {
    pub id: String,
    pub url_pattern: String,
    pub css_selector: String,
    pub fingerprint: SerializableFingerprint,
    pub threshold: f64,
    pub use_count: u64,
    pub success_count: u64,
    pub created_at: String,
    pub last_used: String,
    pub tags: Vec<String>,
}

/// In-memory registry with URL-pattern-based lookup
pub struct SelectorRegistry {
    pub records: Vec<SelectorRecord>,
    pub max_records: usize,
}

/// Summary statistics for the registry
pub struct RegistryStats {
    pub total: usize,
    pub total_uses: u64,
    pub avg_success_rate: f64,
    pub unique_url_patterns: usize,
}

impl SelectorRegistry {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Vec::with_capacity(max_records.min(128)),
            max_records,
        }
    }

    pub fn store(&mut self, record: SelectorRecord) {
        if self.records.len() >= self.max_records {
            // Evict oldest by last_used
            let oldest_idx = self
                .records
                .iter()
                .enumerate()
                .min_by_key(|(_, r)| r.last_used.clone())
                .map(|(i, _)| i);
            if let Some(idx) = oldest_idx {
                self.records.swap_remove(idx);
            }
        }
        self.records.push(record);
    }

    fn url_matches_pattern(url: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if let Some(found) = url[pos..].find(part) {
                    pos += found + part.len();
                } else {
                    return false;
                }
            }
            true
        } else {
            url.contains(pattern)
        }
    }

    pub fn lookup(&self, url: &str, _label: &str) -> Option<&SelectorRecord> {
        self.records
            .iter()
            .filter(|r| Self::url_matches_pattern(url, &r.url_pattern))
            .max_by(|a, b| {
                let a_score = a.success_count as f64 / a.use_count.max(1) as f64;
                let b_score = b.success_count as f64 / b.use_count.max(1) as f64;
                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn lookup_all(&self, url: &str) -> Vec<&SelectorRecord> {
        self.records
            .iter()
            .filter(|r| Self::url_matches_pattern(url, &r.url_pattern))
            .collect()
    }

    pub fn update_stats(&mut self, id: &str, success: bool) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == id) {
            record.use_count += 1;
            if success {
                record.success_count += 1;
            }
            record.last_used = chrono_now();
        }
    }

    pub fn prune_old(&mut self, max_age_days: u64) -> usize {
        let before = self.records.len();
        // Simple heuristic: records whose last_used is not parseable or extremely old
        // We use a string-based heuristic: if created_at < cutoff_by_days
        let cutoff_secs = max_age_days * 86400;
        let now_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.records.retain(|r| {
            let created_ts = parse_iso_timestamp(&r.created_at).unwrap_or(0);
            now_ts.saturating_sub(created_ts) < cutoff_secs || {
                let used_ts = parse_iso_timestamp(&r.last_used).unwrap_or(0);
                now_ts.saturating_sub(used_ts) < cutoff_secs / 2
            }
        });

        before - self.records.len()
    }

    pub fn prune_low_confidence(&mut self, min_success_rate: f64) -> usize {
        let before = self.records.len();
        self.records.retain(|r| {
            let rate = r.success_count as f64 / r.use_count.max(1) as f64;
            rate >= min_success_rate
        });
        before - self.records.len()
    }

    pub fn export_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.records)
            .map_err(|e| format!("Serialization error: {}", e))
    }

    pub fn import_json(&mut self, json: &str) -> Result<usize, String> {
        let records: Vec<SelectorRecord> =
            serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))?;
        let count = records.len();
        for record in records {
            if self.records.len() < self.max_records {
                self.records.push(record);
            } else {
                break;
            }
        }
        Ok(count.min(self.max_records))
    }

    pub fn stats(&self) -> RegistryStats {
        let total = self.records.len();
        let total_uses: u64 = self.records.iter().map(|r| r.use_count).sum();
        let avg_success_rate = if total == 0 {
            0.0
        } else {
            let total_success: u64 = self.records.iter().map(|r| r.success_count).sum();
            total_success as f64 / total_uses.max(1) as f64
        };
        let unique_url_patterns = self
            .records
            .iter()
            .map(|r| &r.url_pattern)
            .collect::<std::collections::HashSet<_>>()
            .len();

        RegistryStats {
            total,
            total_uses,
            avg_success_rate,
            unique_url_patterns,
        }
    }
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let nanos = now.subsec_nanos();
    // Build ISO 8601 timestamp manually to avoid chrono crate dependency
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Simplistic date from days since epoch (works for 1970-2106 range)
    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        year, month, day, hours, minutes, seconds, nanos / 1000
    )
}

fn days_to_date(days: u64) -> (u64, u64, u64) {
    let mut y = 1970i64;
    let mut d = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }
    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1u64;
    for &md in months_days {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }
    (y as u64, m, (d + 1) as u64)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn parse_iso_timestamp(ts: &str) -> Option<u64> {
    // Parse "YYYY-MM-DDTHH:MM:SS.ffffffZ" or similar
    if ts.len() < 19 {
        return None;
    }
    let year: u64 = ts[0..4].parse().ok()?;
    let month: u64 = ts[5..7].parse().ok()?;
    let day: u64 = ts[8..10].parse().ok()?;
    let hour: u64 = ts[11..13].parse().ok()?;
    let min: u64 = ts[14..16].parse().ok()?;
    let sec: u64 = ts[17..19].parse().ok()?;

    let days_from_epoch = days_from_ymd(year, month, day);
    Some(days_from_epoch * 86400 + hour * 3600 + min * 60 + sec)
}

fn days_from_ymd(year: u64, month: u64, day: u64) -> u64 {
    let y = year as i64;
    let mut total = 0i64;
    // Days from 1970 to year-1
    for yr in 1970..y {
        total += if is_leap(yr) { 366 } else { 365 };
    }
    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    for (i, &md) in months_days.iter().enumerate() {
        if (i as u64) < month - 1 {
            total += md;
        }
    }
    total += day as i64 - 1;
    total as u64
}

impl Default for SelectorRegistry {
    fn default() -> Self {
        Self::new(1000)
    }
}

// ============================================================================
// Conversion from/to AdaptiveSelector
// ============================================================================

pub fn fingerprint_to_serializable(
    tag_path: &str,
    text_density: f64,
    child_count: usize,
    depth: usize,
    sibling_index: usize,
    similar_sibling_count: usize,
    attr_keys: Vec<String>,
    classes: Vec<String>,
    id: Option<String>,
    text_sig: u64,
    has_links: bool,
    has_images: bool,
    has_tables: bool,
) -> SerializableFingerprint {
    SerializableFingerprint {
        tag_path: tag_path.to_string(),
        text_density,
        child_count,
        depth,
        sibling_index,
        similar_sibling_count,
        attr_keys,
        classes,
        id,
        text_sig,
        has_links,
        has_images,
        has_tables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(id: &str, pattern: &str, use_count: u64, success_count: u64) -> SelectorRecord {
        SelectorRecord {
            id: id.to_string(),
            url_pattern: pattern.to_string(),
            css_selector: "div.content".to_string(),
            fingerprint: SerializableFingerprint {
                tag_path: "html/body/div".to_string(),
                text_density: 0.5,
                child_count: 3,
                depth: 2,
                sibling_index: 1,
                similar_sibling_count: 2,
                attr_keys: vec!["class".to_string()],
                classes: vec!["content".to_string()],
                id: None,
                text_sig: 12345,
                has_links: false,
                has_images: false,
                has_tables: false,
            },
            threshold: 0.55,
            use_count,
            success_count,
            created_at: chrono_now(),
            last_used: chrono_now(),
            tags: vec![],
        }
    }

    #[test]
    fn test_registry_store_and_lookup() {
        let mut registry = SelectorRegistry::new(100);
        let record = make_record("sel-1", "example.com/article", 10, 8);
        registry.store(record);
        assert_eq!(registry.records.len(), 1);
    }

    #[test]
    fn test_registry_lookup_by_url_pattern() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("sel-1", "example.com/article", 10, 9));
        registry.store(make_record("sel-2", "example.com/blog", 5, 2));

        let found = registry.lookup("https://example.com/article/123", "title");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "sel-1");
    }

    #[test]
    fn test_registry_lookup_all() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("sel-1", "example.com", 10, 8));
        registry.store(make_record("sel-2", "example.com", 5, 4));

        let all = registry.lookup_all("https://example.com/page");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_registry_update_stats() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("sel-1", "example.com", 5, 3));
        registry.update_stats("sel-1", true);
        let rec = registry.records.first().unwrap();
        assert_eq!(rec.use_count, 6);
        assert_eq!(rec.success_count, 4);
    }

    #[test]
    fn test_registry_prune_old() {
        let mut registry = SelectorRegistry::new(100);
        let mut record = make_record("old", "old.example.com", 1, 0);
        // Set a very old timestamp
        record.created_at = "2020-01-01T00:00:00.000000Z".to_string();
        record.last_used = "2020-01-01T00:00:00.000000Z".to_string();
        registry.store(record);
        registry.store(make_record("new", "new.example.com", 1, 1));

        let pruned = registry.prune_old(1); // 1 day max age
        assert_eq!(pruned, 1);
        assert_eq!(registry.records.len(), 1);
        assert_eq!(registry.records[0].id, "new");
    }

    #[test]
    fn test_registry_prune_low_confidence() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("good", "a.com", 10, 9));  // 90%
        registry.store(make_record("bad", "b.com", 10, 1));   // 10%
        registry.store(make_record("ok", "c.com", 10, 5));    // 50%

        let pruned = registry.prune_low_confidence(0.5); // min 50%
        assert_eq!(pruned, 1); // only "bad" pruned
        assert_eq!(registry.records.len(), 2);
    }

    #[test]
    fn test_registry_export_import_json() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("sel-1", "example.com", 5, 4));
        registry.store(make_record("sel-2", "test.com", 3, 2));

        let json = registry.export_json().unwrap();
        assert!(json.contains("sel-1"));
        assert!(json.contains("sel-2"));

        let mut imported = SelectorRegistry::new(100);
        let count = imported.import_json(&json).unwrap();
        assert_eq!(count, 2);
        assert_eq!(imported.records.len(), 2);
    }

    #[test]
    fn test_registry_stats() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("a", "a.com", 10, 8));
        registry.store(make_record("b", "b.com", 5, 3));
        registry.store(make_record("c", "a.com", 2, 1));

        let stats = registry.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.total_uses, 17);
        assert_eq!(stats.unique_url_patterns, 2);
        assert!((stats.avg_success_rate - 12.0 / 17.0).abs() < 0.01);
    }

    #[test]
    fn test_registry_max_records_enforced() {
        let mut registry = SelectorRegistry::new(3);
        for i in 0..10 {
            let mut r = make_record(&format!("sel-{}", i), &format!("{}.com", i), 1, 1);
            r.last_used = format!("2026-06-{:02}T12:00:00.000000Z", 20 + i);
            registry.store(r);
        }
        assert_eq!(registry.records.len(), 3);
    }

    #[test]
    fn test_serializable_fingerprint_roundtrip() {
        let fp = SerializableFingerprint {
            tag_path: "html/body/div/article".to_string(),
            text_density: 0.35,
            child_count: 5,
            depth: 4,
            sibling_index: 2,
            similar_sibling_count: 3,
            attr_keys: vec!["class".to_string(), "data-id".to_string()],
            classes: vec!["post".to_string(), "entry".to_string()],
            id: Some("main".to_string()),
            text_sig: 99999,
            has_links: true,
            has_images: false,
            has_tables: true,
        };
        let json = serde_json::to_string(&fp).unwrap();
        let back: SerializableFingerprint = serde_json::from_str(&json).unwrap();
        assert_eq!(back.tag_path, "html/body/div/article");
        assert_eq!(back.id, Some("main".to_string()));
        assert!((back.text_density - 0.35).abs() < 0.001);
        assert_eq!(back.attr_keys.len(), 2);
        assert!(back.has_links);
        assert!(back.has_tables);
    }

    #[test]
    fn test_url_matches_pattern() {
        assert!(SelectorRegistry::url_matches_pattern(
            "https://example.com/article/123",
            "example.com/article"
        ));
        assert!(SelectorRegistry::url_matches_pattern(
            "https://blog.example.com/post/abc",
            "example.com/*/post"
        ));
        assert!(!SelectorRegistry::url_matches_pattern(
            "https://other.com/page",
            "example.com"
        ));
    }

    #[test]
    fn test_lookup_uses_best_confidence() {
        let mut registry = SelectorRegistry::new(100);
        registry.store(make_record("bad", "example.com", 10, 1));   // 10% success
        registry.store(make_record("good", "example.com", 10, 9));  // 90% success
        let found = registry.lookup("https://example.com/page", "title");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "good");
    }

    #[test]
    fn test_empty_registry() {
        let registry = SelectorRegistry::new(100);
        assert!(registry.lookup("https://example.com", "x").is_none());
        assert!(registry.lookup_all("https://example.com").is_empty());
        let stats = registry.stats();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.total_uses, 0);
        assert!((stats.avg_success_rate - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_import_invalid_json() {
        let mut registry = SelectorRegistry::new(100);
        let result = registry.import_json("not valid json");
        assert!(result.is_err());
    }
}
