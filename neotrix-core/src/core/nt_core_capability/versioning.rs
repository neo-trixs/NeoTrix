//! 能力版本管理
//!
//! 支持能力版本控制、升级、回滚

use std::collections::HashMap;
use std::time::Instant;

/// 版本号
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// 解析版本字符串
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;
        Some(Self { major, minor, patch })
    }

    /// 检查是否兼容
    pub fn is_compatible(&self, other: &SemanticVersion) -> bool {
        self.major == other.major
    }

    /// 升级版本
    pub fn bump_major(&self) -> Self {
        Self { major: self.major + 1, minor: 0, patch: 0 }
    }

    pub fn bump_minor(&self) -> Self {
        Self { major: self.major, minor: self.minor + 1, patch: 0 }
    }

    pub fn bump_patch(&self) -> Self {
        Self { major: self.major, minor: self.minor, patch: self.patch + 1 }
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// 能力版本
#[derive(Debug, Clone)]
pub struct CapabilityVersion {
    /// 版本号
    pub version: SemanticVersion,
    /// 版本发布者
    pub publisher: String,
    /// 发布时间
    pub published_at: Instant,
    /// 版本说明
    pub changelog: String,
    /// 版本状态
    pub status: VersionStatus,
    /// 版本哈希
    pub hash: String,
    /// 版本大小
    pub size_bytes: u64,
}

/// 版本状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionStatus {
    Active,
    Deprecated,
    Archived,
    Beta,
}

/// 版本历史
#[derive(Debug, Clone)]
pub struct VersionHistory {
    /// 能力ID
    pub capability_id: String,
    /// 所有版本
    pub versions: Vec<CapabilityVersion>,
    /// 当前版本
    pub current: SemanticVersion,
    /// 最后更新时间
    pub last_updated: Instant,
}

/// 版本管理器
pub struct VersionManager {
    /// 版本历史
    histories: HashMap<String, VersionHistory>,
    /// 最大版本数
    max_versions: usize,
    /// 自动清理
    auto_cleanup: bool,
}

impl VersionManager {
    /// 创建新的版本管理器
    pub fn new() -> Self {
        Self {
            histories: HashMap::new(),
            max_versions: 10,
            auto_cleanup: true,
        }
    }

    /// 注册能力版本
    pub fn register_version(
        &mut self,
        capability_id: &str,
        version: SemanticVersion,
        publisher: &str,
        changelog: &str,
    ) {
        let history = self.histories.entry(capability_id.to_string()).or_insert_with(|| VersionHistory {
            capability_id: capability_id.to_string(),
            versions: Vec::new(),
            current: version.clone(),
            last_updated: Instant::now(),
        });

        // 检查版本是否已存在
        if history.versions.iter().any(|v| v.version == version) {
            return;
        }

        let cap_version = CapabilityVersion {
            version: version.clone(),
            publisher: publisher.to_string(),
            published_at: Instant::now(),
            changelog: changelog.to_string(),
            status: VersionStatus::Active,
            hash: format!("hash_{}", chrono::Utc::now().timestamp()),
            size_bytes: 0,
        };

        history.versions.push(cap_version);
        history.last_updated = Instant::now();

        // 自动清理旧版本
        if self.auto_cleanup && history.versions.len() > self.max_versions {
            history.versions.remove(0);
        }
    }

    /// 获取当前版本
    pub fn get_current_version(&self, capability_id: &str) -> Option<&SemanticVersion> {
        self.histories.get(capability_id).map(|h| &h.current)
    }

    /// 获取版本历史
    pub fn get_history(&self, capability_id: &str) -> Option<&VersionHistory> {
        self.histories.get(capability_id)
    }

    /// 检查版本兼容性
    pub fn check_compatibility(&self, capability_id: &str, required: &SemanticVersion) -> bool {
        if let Some(history) = self.histories.get(capability_id) {
            history.versions.iter().any(|v| v.version.is_compatible(required))
        } else {
            false
        }
    }

    /// 升级版本
    pub fn upgrade(
        &mut self,
        capability_id: &str,
        upgrade_type: UpgradeType,
    ) -> Option<SemanticVersion> {
        if let Some(history) = self.histories.get_mut(capability_id) {
            let new_version = match upgrade_type {
                UpgradeType::Major => history.current.bump_major(),
                UpgradeType::Minor => history.current.bump_minor(),
                UpgradeType::Patch => history.current.bump_patch(),
            };

            history.current = new_version.clone();
            history.last_updated = Instant::now();

            Some(new_version)
        } else {
            None
        }
    }

    /// 回滚版本
    pub fn rollback(&mut self, capability_id: &str) -> Option<SemanticVersion> {
        if let Some(history) = self.histories.get_mut(capability_id) {
            if history.versions.len() > 1 {
                history.versions.pop();
                if let Some(prev) = history.versions.last() {
                    let version = prev.version.clone();
                    history.current = version.clone();
                    history.last_updated = Instant::now();
                    Some(version)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 废弃版本
    pub fn deprecate(&mut self, capability_id: &str, version: &SemanticVersion) -> bool {
        if let Some(history) = self.histories.get_mut(capability_id) {
            if let Some(v) = history.versions.iter_mut().find(|v| v.version == *version) {
                v.status = VersionStatus::Deprecated;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 获取所有活跃版本
    pub fn get_active_versions(&self, capability_id: &str) -> Vec<&CapabilityVersion> {
        if let Some(history) = self.histories.get(capability_id) {
            history.versions.iter()
                .filter(|v| v.status == VersionStatus::Active)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 清理旧版本
    pub fn cleanup(&mut self, capability_id: &str, max_versions: usize) {
        if let Some(history) = self.histories.get_mut(capability_id) {
            while history.versions.len() > max_versions {
                history.versions.remove(0);
            }
        }
    }

    /// 获取版本统计
    pub fn get_stats(&self) -> VersionStats {
        let mut stats = VersionStats::default();
        for history in self.histories.values() {
            stats.total_capabilities += 1;
            stats.total_versions += history.versions.len();
            for version in &history.versions {
                match version.status {
                    VersionStatus::Active => stats.active_versions += 1,
                    VersionStatus::Deprecated => stats.deprecated_versions += 1,
                    VersionStatus::Archived => stats.archived_versions += 1,
                    VersionStatus::Beta => stats.beta_versions += 1,
                }
            }
        }
        stats
    }
}

/// 升级类型
#[derive(Debug, Clone)]
pub enum UpgradeType {
    Major,
    Minor,
    Patch,
}

/// 版本统计
#[derive(Debug, Clone, Default)]
pub struct VersionStats {
    pub total_capabilities: usize,
    pub total_versions: usize,
    pub active_versions: usize,
    pub deprecated_versions: usize,
    pub archived_versions: usize,
    pub beta_versions: usize,
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_version_parse() {
        let v = SemanticVersion::parse("1.2.3");
        assert!(v.is_some());
        let v = v.unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn version_compatibility() {
        let v1 = SemanticVersion::new(1, 2, 3);
        let v2 = SemanticVersion::new(1, 3, 0);
        assert!(v1.is_compatible(&v2));

        let v3 = SemanticVersion::new(2, 0, 0);
        assert!(!v1.is_compatible(&v3));
    }

    #[test]
    fn version_bump() {
        let v = SemanticVersion::new(1, 2, 3);
        assert_eq!(v.bump_patch().to_string(), "1.2.4");
        assert_eq!(v.bump_minor().to_string(), "1.3.0");
        assert_eq!(v.bump_major().to_string(), "2.0.0");
    }

    #[test]
    fn version_manager() {
        let mut manager = VersionManager::new();
        let version = SemanticVersion::new(1, 0, 0);

        manager.register_version("test", version, "test", "Initial version");

        let current = manager.get_current_version("test");
        assert!(current.is_some());
        assert_eq!(current.unwrap().to_string(), "1.0.0");
    }

    #[test]
    fn upgrade() {
        let mut manager = VersionManager::new();
        let version = SemanticVersion::new(1, 0, 0);

        manager.register_version("test", version, "test", "Initial version");
        let new_version = manager.upgrade("test", UpgradeType::Minor);

        assert!(new_version.is_some());
        assert_eq!(new_version.unwrap().to_string(), "1.1.0");
    }
}
