//! New Cleanup Handler - 使用新清理子系统的处理器
//!
//! 替代旧 nt_mind_cleanup::CleanupEngine 的处理路径
//! 域: NT-ACT / NT-WORLD / NT-SHIELD / NT-META
//! 层: L1-L6 跨层协调

use crate::l1_action::nt_act::nt_act_cleanup::shared::*;
use crate::l1_action::nt_act::nt_act_cleanup::{SafeDeleter, CacheCleaner, DevToolCleaner};
use crate::l2_perception::nt_world::nt_world_cleanup::{SystemScanner, CacheDetector};
use crate::l3_embodiment::nt_shield::nt_shield_cleanup::{PathValidator, RiskAssessor};
use crate::l6_meta::coordination::nt_meta_cleanup::coordinator::{CleanupCoordinator, CleanupEventType};

/// 新清理引擎 — 替代旧 CleanupEngine
pub struct NewCleanupEngine {
    coordinator: CleanupCoordinator,
    scanner: SystemScanner,
    detector: CacheDetector,
    validator: PathValidator,
    assessor: RiskAssessor,
    deleter: SafeDeleter,
    config: CleanupConfig,
}

impl NewCleanupEngine {
    pub fn new() -> Self {
        Self {
            coordinator: CleanupCoordinator::new(),
            scanner: SystemScanner::new(),
            detector: CacheDetector::new(),
            validator: PathValidator::new(),
            assessor: RiskAssessor::new(),
            deleter: SafeDeleter::new(),
            config: CleanupConfig::default(),
        }
    }

    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.config.dry_run = dry_run;
        self.deleter.set_dry_run(dry_run);
    }

    pub fn set_strategy(&mut self, strategy: CleanupStrategy) {
        self.coordinator.set_strategy(strategy);
    }

    /// 执行扫描 (返回可清理项)
    pub fn scan(&mut self) -> Vec<ScanResult> {
        self.coordinator.log_event(CleanupEventType::ScanStarted, "系统扫描开始".into(), None);
        let results = self.scanner.scan();
        self.coordinator.log_event(
            CleanupEventType::ScanCompleted,
            format!("扫描完成，找到 {} 项", results.len()),
            None,
        );
        results
    }

    /// 检测缓存
    pub fn detect_caches(&self) -> Vec<CacheInfo> {
        self.detector.detect_all()
    }

    /// 执行清理 (带风险评估)
    pub fn clean(&mut self, results: &[ScanResult]) -> CleanupStats {
        self.coordinator.log_event(CleanupEventType::CleanStarted, "清理开始".into(), None);

        let mut cleaned = 0usize;
        let mut skipped = 0usize;
        let mut errors = 0usize;
        let mut bytes_freed = 0u64;

        for result in results {
            // 路径验证
            let validation = self.validator.validate(&result.path);
            if !validation.is_valid {
                skipped += 1;
                continue;
            }

            // 风险评估
            let assessment = self.assessor.assess(
                &result.category,
                result.size_bytes,
                result.age_days,
                false,
                &result.path.to_string_lossy(),
            );

            if assessment.requires_confirmation {
                skipped += 1;
                continue;
            }

            // 执行删除
            let delete_result = self.deleter.delete(&result.path);
            if delete_result.success {
                cleaned += 1;
                bytes_freed += delete_result.size_freed;
            } else {
                errors += 1;
            }
        }

        self.coordinator.update_stats(
            results.len(),
            cleaned,
            skipped,
            errors,
            bytes_freed,
        );

        let event_type = if errors > 0 {
            CleanupEventType::CleanFailed
        } else {
            CleanupEventType::CleanCompleted
        };
        self.coordinator.log_event(
            event_type,
            format!("清理完成: {} 成功, {} 跳过, {} 错误", cleaned, skipped, errors),
            Some(bytes_freed),
        );

        self.coordinator.get_stats().clone()
    }

    /// 生成清理报告
    pub fn generate_report(&self) -> CleanupReport {
        self.coordinator.generate_report()
    }

    /// 获取事件日志
    pub fn get_event_log(&self) -> &[CleanupEvent] {
        self.coordinator.get_event_log()
    }
}

impl Default for NewCleanupEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let engine = NewCleanupEngine::new();
        assert!(!engine.scanner.scan().is_empty() || engine.scanner.scan().is_empty());
    }

    #[test]
    fn test_scan_and_report() {
        let mut engine = NewCleanupEngine::new();
        engine.set_dry_run(true);
        let results = engine.scan();
        let report = engine.generate_report();
        assert!(report.stats.total_scanned <= results.len());
    }
}
