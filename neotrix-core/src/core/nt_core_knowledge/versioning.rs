use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StalenessLevel {
    Fresh,
    Recent,
    Aging,
    Stale,
    Obsolete,
}

impl StalenessLevel {
    pub fn from_age(age: Duration) -> Self {
        let secs = age.as_secs_f64();
        if secs < 60.0 {
            StalenessLevel::Fresh
        } else if secs < 300.0 {
            StalenessLevel::Recent
        } else if secs < 3600.0 {
            StalenessLevel::Aging
        } else if secs < 86400.0 {
            StalenessLevel::Stale
        } else {
            StalenessLevel::Obsolete
        }
    }

    pub fn retention_multiplier(&self) -> f64 {
        match self {
            StalenessLevel::Fresh => 1.0,
            StalenessLevel::Recent => 0.9,
            StalenessLevel::Aging => 0.6,
            StalenessLevel::Stale => 0.3,
            StalenessLevel::Obsolete => 0.1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            StalenessLevel::Fresh => "fresh",
            StalenessLevel::Recent => "recent",
            StalenessLevel::Aging => "aging",
            StalenessLevel::Stale => "stale",
            StalenessLevel::Obsolete => "obsolete",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeVersion {
    pub version: u64,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub change_log: Vec<String>,
    pub staleness: StalenessLevel,
    pub access_count: u64,
}

impl KnowledgeVersion {
    pub fn new(version: u64) -> Self {
        let now = Instant::now();
        Self {
            version,
            created_at: now,
            updated_at: now,
            change_log: vec![format!("v{} created", version)],
            staleness: StalenessLevel::Fresh,
            access_count: 0,
        }
    }

    pub fn update(&mut self, description: &str) {
        self.version += 1;
        self.updated_at = Instant::now();
        self.change_log
            .push(format!("v{}: {}", self.version, description));
        if self.change_log.len() > 100 {
            self.change_log.remove(0);
        }
        self.staleness = StalenessLevel::Fresh;
    }

    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.staleness = StalenessLevel::Fresh;
    }

    pub fn refresh_staleness(&mut self) {
        let age = Instant::now().duration_since(self.updated_at);
        self.staleness = StalenessLevel::from_age(age);
    }

    pub fn age(&self) -> Duration {
        Instant::now().duration_since(self.updated_at)
    }

    pub fn version_age(&self) -> Duration {
        Instant::now().duration_since(self.created_at)
    }
}

#[derive(Debug, Clone)]
pub struct VersionManager {
    pub versions: Vec<KnowledgeVersion>,
    pub max_versions: usize,
    pub auto_archive: bool,
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionManager {
    pub fn new() -> Self {
        Self {
            versions: Vec::with_capacity(10),
            max_versions: 20,
            auto_archive: true,
        }
    }

    pub fn create_version(&mut self) -> &KnowledgeVersion {
        let version = KnowledgeVersion::new(self.versions.len() as u64 + 1);
        self.versions.push(version);
        self.versions
            .last()
            .expect("create_version just pushed a version")
    }

    pub fn current_version(&self) -> Option<&KnowledgeVersion> {
        self.versions.last()
    }

    pub fn current_version_mut(&mut self) -> Option<&mut KnowledgeVersion> {
        self.versions.last_mut()
    }

    pub fn update_current(&mut self, description: &str) {
        if let Some(current) = self.current_version_mut() {
            current.update(description);
        }
    }

    pub fn get(&self, version: u64) -> Option<&KnowledgeVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    pub fn stalest(&self) -> Option<&KnowledgeVersion> {
        self.versions.iter().min_by_key(|v| v.staleness as u8)
    }

    pub fn fresh_count(&self) -> usize {
        self.versions
            .iter()
            .filter(|v| v.staleness == StalenessLevel::Fresh)
            .count()
    }

    pub fn archive_obsolete(&mut self) {
        if !self.auto_archive {
            return;
        }
        self.versions
            .retain(|v| v.staleness != StalenessLevel::Obsolete);
        while self.versions.len() > self.max_versions {
            self.versions.remove(0);
        }
    }

    pub fn refresh_all(&mut self) {
        for v in &mut self.versions {
            v.refresh_staleness();
        }
    }

    pub fn version_count(&self) -> usize {
        self.versions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_version_is_fresh() {
        let v = KnowledgeVersion::new(1);
        assert_eq!(v.staleness, StalenessLevel::Fresh);
        assert_eq!(v.version, 1);
    }

    #[test]
    fn test_update_increments_version() {
        let mut v = KnowledgeVersion::new(1);
        v.update("changed something");
        assert_eq!(v.version, 2);
        assert_eq!(v.change_log.len(), 2);
    }

    #[test]
    fn test_staleness_after_update() {
        let mut v = KnowledgeVersion::new(1);
        v.staleness = StalenessLevel::Stale;
        v.update("refresh");
        assert_eq!(v.staleness, StalenessLevel::Fresh);
    }

    #[test]
    fn test_version_manager_creation() {
        let mgr = VersionManager::new();
        assert_eq!(mgr.version_count(), 0);
    }

    #[test]
    fn test_create_and_retrieve() {
        let mut mgr = VersionManager::new();
        mgr.create_version();
        mgr.create_version();
        assert_eq!(mgr.version_count(), 2);
        let v1 = mgr.get(1);
        assert!(v1.is_some());
        assert_eq!(v1.unwrap().version, 1);
    }

    #[test]
    fn test_current_version() {
        let mut mgr = VersionManager::new();
        mgr.create_version();
        mgr.create_version();
        let current = mgr.current_version();
        assert!(current.is_some());
        assert_eq!(current.unwrap().version, 2);
    }

    #[test]
    fn test_update_current() {
        let mut mgr = VersionManager::new();
        mgr.create_version();
        mgr.update_current("patch applied");
        let current = mgr.current_version().unwrap();
        assert_eq!(current.version, 2);
        assert!(current
            .change_log
            .iter()
            .any(|l| l.contains("patch applied")));
    }

    #[test]
    fn test_staleness_level_from_age() {
        assert_eq!(
            StalenessLevel::from_age(Duration::from_secs(30)),
            StalenessLevel::Fresh
        );
        assert_eq!(
            StalenessLevel::from_age(Duration::from_secs(120)),
            StalenessLevel::Recent
        );
        assert_eq!(
            StalenessLevel::from_age(Duration::from_secs(600)),
            StalenessLevel::Aging
        );
        assert_eq!(
            StalenessLevel::from_age(Duration::from_secs(7200)),
            StalenessLevel::Stale
        );
        assert_eq!(
            StalenessLevel::from_age(Duration::from_secs(90000)),
            StalenessLevel::Obsolete
        );
    }

    #[test]
    fn test_retention_multiplier_decreases() {
        assert!((StalenessLevel::Fresh.retention_multiplier() - 1.0).abs() < 1e-9);
        assert!(
            StalenessLevel::Fresh.retention_multiplier()
                > StalenessLevel::Obsolete.retention_multiplier()
        );
    }

    #[test]
    fn test_archive_removes_obsolete() {
        let mut mgr = VersionManager::new();
        mgr.create_version();
        mgr.create_version();
        mgr.versions[0].staleness = StalenessLevel::Obsolete;
        mgr.archive_obsolete();
        assert_eq!(mgr.version_count(), 1);
    }

    #[test]
    fn test_refresh_all_updates_staleness() {
        let mut mgr = VersionManager::new();
        mgr.create_version();
        mgr.versions[0].updated_at = Instant::now() - Duration::from_secs(7200);
        mgr.refresh_all();
        assert_eq!(mgr.versions[0].staleness, StalenessLevel::Stale);
    }
}
