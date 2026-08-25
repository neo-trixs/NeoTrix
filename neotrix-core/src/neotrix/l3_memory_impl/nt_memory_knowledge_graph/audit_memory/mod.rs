//! Audit Memory — cloudflare/security-audit-skill 跨运行记忆吸收
//! 
//! 读取 prior findings.json，跳过已知、定位缺口、解决冲突
//! 单次运行约 50% 覆盖率，多次运行收敛

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 单次审计运行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRun {
    pub run_id: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub findings: Vec<AuditFindingSummary>,
    pub coverage_estimate: f64, // 0.0-1.0
    pub focus_areas: Vec<String>,
}

/// 审计发现摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFindingSummary {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub attack_class: AttackClass,
    pub file_path: String,
    pub status: FindingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttackClass {
    Injection,
    AccessControl,
    ResourceFileHandling,
    CryptographySecrets,
    BusinessLogic,
    FeatureAbuseDataLeakage,
    ChainedAttacksTrustBoundaries,
    Wildcard,
    ObviousThings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingStatus {
    New,
    Known,
    Resolved,
    Disputed,
}

/// 缺口分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub uncovered_classes: Vec<AttackClass>,
    pub under_explored_subsystems: Vec<String>,
    pub conflicting_verdicts: Vec<ConflictVerdict>,
    pub recommended_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictVerdict {
    pub finding_id: String,
    pub run_a: usize,
    pub run_b: usize,
    pub verdict_a: String,
    pub verdict_b: String,
}

/// 审计记忆引擎
#[derive(Debug)]
pub struct AuditMemory {
    runs: Vec<AuditRun>,
    project_root: PathBuf,
}

impl AuditMemory {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            runs: Vec::new(),
            project_root,
        }
    }

    /// 加载历史运行
    pub fn load_history(&mut self) -> Result<(), std::io::Error> {
        let audit_dir = self.project_root.join("security-audit-skill");
        if !audit_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&audit_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let findings_file = entry.path().join("findings.json");
                if findings_file.exists() {
                    let content = std::fs::read_to_string(&findings_file)?;
                    if let Ok(run) = serde_json::from_str::<AuditRun>(&content) {
                        self.runs.push(run);
                    }
                }
            }
        }
        
        self.runs.sort_by_key(|r| r.run_id);
        Ok(())
    }

    /// 获取已知发现 (用于 Phase 2 跳过)
    pub fn get_known_findings(&self) -> Vec<AuditFindingSummary> {
        self.runs.iter()
            .flat_map(|r| r.findings.iter().filter(|f| f.status == FindingStatus::Known || f.status == FindingStatus::Resolved))
            .cloned()
            .collect()
    }

    /// 执行缺口分析
    pub fn analyze_gaps(&self) -> GapAnalysis {
        let mut uncovered_classes = vec![
            AttackClass::Injection,
            AttackClass::AccessControl,
            AttackClass::ResourceFileHandling,
            AttackClass::CryptographySecrets,
            AttackClass::BusinessLogic,
            AttackClass::FeatureAbuseDataLeakage,
            AttackClass::ChainedAttacksTrustBoundaries,
            AttackClass::Wildcard,
            AttackClass::ObviousThings,
        ];
        
        let mut covered_classes = std::collections::HashSet::new();
        let mut subsystem_coverage: HashMap<String, usize> = HashMap::new();
        let mut conflicts = Vec::new();

        for run in &self.runs {
            for finding in &run.findings {
                covered_classes.insert(finding.attack_class);
                *subsystem_coverage.entry(finding.file_path.clone()).or_insert(0) += 1;
            }
        }

        // 移除已覆盖的类别
        uncovered_classes.retain(|c| !covered_classes.contains(c));

        // 找出探索不足的子系统
        let mut under_explored: Vec<_> = subsystem_coverage.into_iter().collect();
        under_explored.sort_by_key(|(_, count)| *count);
        let under_explored_subsystems = under_explored.into_iter()
            .take(5)
            .map(|(sys, _)| sys)
            .collect();

        // 检测冲突判决 (简化)
        // 实际应比对同一 finding 在不同 run 中的 verdict

        GapAnalysis {
            uncovered_classes,
            under_explored_subsystems,
            conflicting_verdicts: conflicts,
            recommended_focus: uncovered_classes.iter()
                .map(|c| format!("{:?}", c))
                .collect(),
        }
    }

    /// 记录新运行
    pub fn record_run(&mut self, findings: Vec<AuditFindingSummary>, focus_areas: Vec<String>) -> Result<(), std::io::Error> {
        let run_id = self.runs.len() + 1;
        let coverage_estimate = self.estimate_coverage(&findings);
        
        let run = AuditRun {
            run_id,
            timestamp: chrono::Utc::now(),
            findings,
            coverage_estimate,
            focus_areas,
        };
        
        self.runs.push(run);
        self.save_latest_run()?;
        
        Ok(())
    }

    fn estimate_coverage(&self, findings: &[AuditFindingSummary]) -> f64 {
        // 简化估算: 基于发现的攻击类别多样性
        let classes: std::collections::HashSet<_> = findings.iter().map(|f| f.attack_class).collect();
        classes.len() as f64 / 9.0 // 9 个攻击类别
    }

    fn save_latest_run(&self) -> Result<(), std::io::Error> {
        let audit_dir = self.project_root.join("security-audit-skill").join(format!("run-{}", self.runs.len()));
        std::fs::create_dir_all(&audit_dir)?;
        
        if let Some(latest) = self.runs.last() {
            let content = serde_json::to_string_pretty(latest)?;
            std::fs::write(audit_dir.join("findings.json"), content)?;
        }
        
        Ok(())
    }

    /// 获取推荐下一轮重点
    pub fn next_focus(&self) -> Vec<String> {
        let gaps = self.analyze_gaps();
        gaps.recommended_focus
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let temp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;
        let mut memory = AuditMemory::new(temp_dir.path().to_path_buf());
        
        // Test 1: Empty history
        let gaps = memory.analyze_gaps();
        assert_eq!(gaps.uncovered_classes.len(), 9);
        
        // Test 2: Record run
        let findings = vec![
            AuditFindingSummary {
                id: "F1".to_string(),
                title: "SQLi".to_string(),
                severity: Severity::High,
                attack_class: AttackClass::Injection,
                file_path: "src/api.rs".to_string(),
                status: FindingStatus::New,
            }
        ];
        memory.record_run(findings, vec!["api".to_string()]).map_err(|e| e.to_string())?;
        
        let gaps = memory.analyze_gaps();
        assert!(!gaps.uncovered_classes.contains(&AttackClass::Injection));
        
        // Test 3: Next focus
        let focus = memory.next_focus();
        assert!(!focus.is_empty());
        assert!(!focus.contains(&"Injection".to_string()));
        
        Ok(())
    }
}

impl Default for AuditMemory {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_memory_creation() {
        let memory = AuditMemory::new(PathBuf::from("."));
        assert_eq!(memory.runs.len(), 0);
    }

    #[test]
    fn test_gap_analysis_empty() {
        let memory = AuditMemory::new(PathBuf::from("."));
        let gaps = memory.analyze_gaps();
        assert_eq!(gaps.uncovered_classes.len(), 9);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(AuditMemory::self_test().is_ok());
    }
}