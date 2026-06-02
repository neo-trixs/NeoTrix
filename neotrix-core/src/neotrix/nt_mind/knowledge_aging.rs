use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeFreshness {
    Fresh,
    Recent,
    Stale,
    Expired,
}

impl KnowledgeFreshness {
    pub fn label(&self) -> &str {
        match self {
            KnowledgeFreshness::Fresh => "fresh",
            KnowledgeFreshness::Recent => "recent",
            KnowledgeFreshness::Stale => "stale",
            KnowledgeFreshness::Expired => "expired",
        }
    }

    pub fn should_rescan(&self) -> bool {
        matches!(self, KnowledgeFreshness::Stale | KnowledgeFreshness::Expired)
    }
}

pub struct KnowledgeAgingEntry {
    pub source_url: String,
    pub source_name: String,
    pub domain: String,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub access_count: u64,
    pub confidence: f64,
    pub half_life_days: f64,
}

pub struct KnowledgeAging {
    pub entries: Vec<KnowledgeAgingEntry>,
    pub max_entries: usize,
    pub rescans_triggered: u64,
    pub expired_entries: u64,
}

impl KnowledgeAging {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 500,
            rescans_triggered: 0,
            expired_entries: 0,
        }
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    pub fn register(&mut self, url: &str, name: &str, domain: &str, confidence: f64) {
        let now = Self::now();
        self.entries.push(KnowledgeAgingEntry {
            source_url: url.to_string(),
            source_name: name.to_string(),
            domain: domain.to_string(),
            created_at: now,
            last_accessed_at: now,
            access_count: 1,
            confidence,
            half_life_days: 30.0,
        });

        if self.entries.len() > self.max_entries {
            self.entries.sort_by(|a, b| b.last_accessed_at.cmp(&a.last_accessed_at));
            self.entries.truncate(self.max_entries);
        }
    }

    pub fn access(&mut self, url: &str) {
        let now = Self::now();
        if let Some(entry) = self.entries.iter_mut().find(|e| e.source_url == url) {
            entry.last_accessed_at = now;
            entry.access_count += 1;
        }
    }

    pub fn freshness(&self, entry: &KnowledgeAgingEntry) -> KnowledgeFreshness {
        let now = Self::now();
        let age_days = (now - entry.created_at) as f64 / 86400.0;
        let half_life = entry.half_life_days;

        if age_days < half_life * 0.3 {
            KnowledgeFreshness::Fresh
        } else if age_days < half_life * 0.7 {
            KnowledgeFreshness::Recent
        } else if age_days < half_life * 1.5 {
            KnowledgeFreshness::Stale
        } else {
            KnowledgeFreshness::Expired
        }
    }

    pub fn stale_and_expired(&self) -> Vec<&KnowledgeAgingEntry> {
        self.entries.iter()
            .filter(|e| self.freshness(e).should_rescan())
            .collect()
    }

    pub fn run_aging_cycle(&mut self) -> AgingReport {
        let now = Self::now();
        let mut stale_count = 0;
        let mut expired_count = 0;
        let mut total_age_days = 0.0;
        let mut rescans = Vec::new();

        let mut surviving = Vec::with_capacity(self.entries.len());

        let entries = std::mem::take(&mut self.entries);
        for entry in entries {
            let age_days = (now - entry.created_at) as f64 / 86400.0;
            total_age_days += age_days;
            let half_life = entry.half_life_days;
            let freshness = if age_days < half_life * 0.3 {
                KnowledgeFreshness::Fresh
            } else if age_days < half_life * 0.7 {
                KnowledgeFreshness::Recent
            } else if age_days < half_life * 1.5 {
                KnowledgeFreshness::Stale
            } else {
                KnowledgeFreshness::Expired
            };

            match freshness {
                KnowledgeFreshness::Expired => {
                    expired_count += 1;
                    self.expired_entries += 1;
                }
                KnowledgeFreshness::Stale => {
                    stale_count += 1;
                    rescans.push(entry.source_url.clone());
                    surviving.push(entry);
                }
                _ => {
                    surviving.push(entry);
                }
            }
        }

        self.entries = surviving;
        self.rescans_triggered += rescans.len() as u64;

        AgingReport {
            total_entries: self.entries.len() + stale_count + expired_count,
            surviving_entries: self.entries.len(),
            stale_count,
            expired_count,
            avg_age_days: if self.entries.is_empty() { 0.0 } else { total_age_days / (self.entries.len() + stale_count + expired_count) as f64 },
            rescans_needed: rescans,
        }
    }

    pub fn summary(&self) -> String {
        let stale = self.stale_and_expired();
        format!(
            "KnowledgeAging: {} entries | {} stale/expired | {} rescans triggered | {} total expired",
            self.entries.len(),
            stale.len(),
            self.rescans_triggered,
            self.expired_entries,
        )
    }
}

impl Default for KnowledgeAging {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct AgingReport {
    pub total_entries: usize,
    pub surviving_entries: usize,
    pub stale_count: usize,
    pub expired_count: usize,
    pub avg_age_days: f64,
    pub rescans_needed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let aging = KnowledgeAging::new();
        assert!(aging.entries.is_empty());
    }

    #[test]
    fn test_register_adds_entry() {
        let mut aging = KnowledgeAging::new();
        aging.register("https://example.com", "test", "general", 0.8);
        assert_eq!(aging.entries.len(), 1);
    }

    #[test]
    fn test_freshness_fresh() {
        let mut aging = KnowledgeAging::new();
        aging.register("https://example.com", "test", "general", 0.8);
        let entry = &aging.entries[0];
        assert_eq!(aging.freshness(entry), KnowledgeFreshness::Fresh);
    }

    #[test]
    fn test_access_updates_timestamp() {
        let mut aging = KnowledgeAging::new();
        aging.register("https://example.com", "test", "general", 0.8);
        let ts_before = aging.entries[0].last_accessed_at;
        aging.access("https://example.com");
        assert!(aging.entries[0].last_accessed_at >= ts_before);
    }

    #[test]
    fn test_aging_cycle_removes_expired() {
        let mut aging = KnowledgeAging::new();
        aging.register("https://example.com", "test", "general", 0.8);
        aging.entries[0].created_at = 1000;
        let report = aging.run_aging_cycle();
        assert!(report.expired_count > 0 || report.stale_count > 0);
    }

    #[test]
    fn test_aging_cycle_preserves_fresh() {
        let mut aging = KnowledgeAging::new();
        aging.register("https://example.com", "test", "general", 0.8);
        let report = aging.run_aging_cycle();
        assert!(report.surviving_entries >= 1);
    }
}
