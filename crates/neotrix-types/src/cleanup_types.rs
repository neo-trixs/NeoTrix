use serde::{Deserialize, Serialize};

/// 风险等级 — 清理子系统共享 (单一事实源)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CleanupRiskLevel {
    Safe,
    Moderate,
    Risky,
    Protected,
}

impl CleanupRiskLevel {
    pub fn score(&self) -> u8 {
        match self {
            CleanupRiskLevel::Safe => 10,
            CleanupRiskLevel::Moderate => 30,
            CleanupRiskLevel::Risky => 60,
            CleanupRiskLevel::Protected => 90,
        }
    }
}

/// 扫描类别 — 清理子系统共享
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ScanCategory {
    SystemCache,
    SystemLog,
    TempFile,
    UserCache,
    UserLog,
    BrowserCache,
    DeveloperCache,
    ApplicationSupport,
    LargeFile,
    OldFile,
    BuildArtifacts,
    BackupFiles,
    TempFiles,
    OldLogs,
    Other,
}

/// 风险评估结果 — 清理子系统共享
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub level: CleanupRiskLevel,
    pub score: u8,
    pub reasons: Vec<String>,
    pub requires_confirmation: bool,
    pub recommendation: String,
}
