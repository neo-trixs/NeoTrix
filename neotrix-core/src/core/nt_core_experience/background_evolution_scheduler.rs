use std::collections::VecDeque;

/// 背景进化调度器 — yoyo-evolve 风格的自进化定时任务
///
/// 以硬件后台任务的模式运行 (而非仅 consciousness tick):
/// - 每 N 周期触发一次 "源审计" 任务
/// - 在 EvolutionTaskSystem 中创建 SourceAudit 类型任务
/// - 跟踪已执行的审计记录
/// - 可选: 自动触发自修改提案
#[derive(Debug)]
pub struct BackgroundEvolutionScheduler {
    /// 基础间隔 (cycle)
    pub base_interval: u64,
    /// 上次触发 cycle
    pub last_run_cycle: u64,
    /// 已执行的审计次数
    pub audit_count: u64,
    /// 已生成的自修改提案数
    pub self_modify_proposals: u64,
    /// 审计日志 (最近的)
    pub audit_log: VecDeque<AuditRecord>,
    /// 配置参数
    pub config: BESConfig,
}

/// 一次审计记录
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub cycle: u64,
    pub audit_type: AuditType,
    pub task_id: u64,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditType {
    SourceStructure,
    CompileHealth,
    DependencyFreshness,
    DeadCodeScan,
}

impl AuditType {
    pub fn name(&self) -> &'static str {
        match self {
            AuditType::SourceStructure => "source_structure",
            AuditType::CompileHealth => "compile_health",
            AuditType::DependencyFreshness => "dependency_freshness",
            AuditType::DeadCodeScan => "dead_code_scan",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BESConfig {
    /// 源审计间隔 (cycle)
    pub source_audit_interval: u64,
    /// 编译健康审计间隔
    pub compile_audit_interval: u64,
    /// 依赖新鲜度审计间隔
    pub dep_audit_interval: u64,
    /// 死代码扫描间隔
    pub dead_code_interval: u64,
    /// 最大审计日志保留数
    pub max_audit_log: usize,
    /// 自修改冷却周期 (每个 source 审计后至少等待 N cycle 才能提议自修改)
    pub self_modify_cooldown: u64,
}

impl Default for BESConfig {
    fn default() -> Self {
        Self {
            source_audit_interval: 100,
            compile_audit_interval: 200,
            dep_audit_interval: 500,
            dead_code_interval: 300,
            max_audit_log: 50,
            self_modify_cooldown: 50,
        }
    }
}

impl BackgroundEvolutionScheduler {
    pub fn new(base_interval: u64) -> Self {
        Self {
            base_interval,
            last_run_cycle: 0,
            audit_count: 0,
            self_modify_proposals: 0,
            audit_log: VecDeque::with_capacity(50),
            config: BESConfig::default(),
        }
    }

    pub fn with_config(base_interval: u64, config: BESConfig) -> Self {
        Self {
            base_interval,
            last_run_cycle: 0,
            audit_count: 0,
            self_modify_proposals: 0,
            audit_log: VecDeque::with_capacity(config.max_audit_log),
            config,
        }
    }

    /// 评估当前 cycle 应执行何种审计，返回要创建的任务描述
    ///
    /// 返回值: Vec<(AuditType, title, description, priority, impact)>
    pub fn evaluate(
        &mut self,
        cycle: u64,
        recent_mutations: usize,
        current_meta_acc: f64,
    ) -> Vec<(AuditType, String, String, u8, f64)> {
        if cycle < self.last_run_cycle + self.base_interval && !self.is_milestone(cycle) {
            return Vec::new();
        }

        let mut tasks = Vec::new();

        // 源结构审计 (每 base_interval cycle)
        if cycle >= self.last_run_cycle + self.config.source_audit_interval {
            let priority = (8u8).min(5 + (recent_mutations / 5) as u8);
            let impact = 0.5 + (recent_mutations as f64 * 0.01).min(0.4);
            tasks.push((
                AuditType::SourceStructure,
                format!("Background source structure audit (cycle {})", cycle),
                format!(
                    "autonomous source audit triggered every {} cycles. recent_mutations={} meta_acc={:.3}",
                    self.config.source_audit_interval, recent_mutations, current_meta_acc
                ),
                priority,
                impact,
            ));
            self.last_run_cycle = cycle;
        }

        // 编译健康审计 (每 compile_audit_interval cycle)
        if self.config.compile_audit_interval > 0 && cycle % self.config.compile_audit_interval == 0
        {
            tasks.push((
                AuditType::CompileHealth,
                format!("Background compile health check (cycle {})", cycle),
                "autonomous compile health scan: check for new compile errors, dependency issues, and type-level regressions".to_string(),
                7,
                0.6,
            ));
        }

        // 依赖新鲜度审计
        if self.config.dep_audit_interval > 0 && cycle % self.config.dep_audit_interval == 0 {
            tasks.push((
                AuditType::DependencyFreshness,
                format!("Dependency freshness audit (cycle {})", cycle),
                "check workspace Cargo.toml for outdated or unused dependencies".to_string(),
                5,
                0.3,
            ));
        }

        // 死代码扫描
        if self.config.dead_code_interval > 0 && cycle % self.config.dead_code_interval == 0 {
            tasks.push((
                AuditType::DeadCodeScan,
                format!("Dead code scan (cycle {})", cycle),
                "scan for modules that exist on disk but are not registered in mod.rs".to_string(),
                7,
                0.5,
            ));
        }

        tasks
    }

    /// 检查 cycle 是否为某个审计间隔的倍数
    fn is_milestone(&self, cycle: u64) -> bool {
        if cycle == 0 {
            return false;
        }
        cycle % self.config.source_audit_interval == 0
            || (self.config.compile_audit_interval > 0
                && cycle % self.config.compile_audit_interval == 0)
            || (self.config.dep_audit_interval > 0 && cycle % self.config.dep_audit_interval == 0)
            || (self.config.dead_code_interval > 0 && cycle % self.config.dead_code_interval == 0)
    }

    /// 记录一次已执行的审计
    pub fn record_audit(
        &mut self,
        cycle: u64,
        audit_type: AuditType,
        task_id: u64,
        description: &str,
    ) {
        self.audit_count += 1;
        if self.audit_log.len() >= self.config.max_audit_log {
            self.audit_log.pop_front();
        }
        self.audit_log.push_back(AuditRecord {
            cycle,
            audit_type,
            task_id,
            description: description.to_string(),
        });
    }

    /// 判断是否可以触发自修改 (非冷却期)
    pub fn can_self_modify(&self, cycle: u64) -> bool {
        cycle > self.last_run_cycle + self.config.self_modify_cooldown
    }

    /// 统计数据
    pub fn stats(&self) -> BESStats {
        let mut by_type: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for r in &self.audit_log {
            *by_type.entry(r.audit_type.name().to_string()).or_insert(0) += 1;
        }
        BESStats {
            total_audits: self.audit_count,
            audits_by_type: by_type,
            self_modify_proposals: self.self_modify_proposals,
            last_run_cycle: self.last_run_cycle,
        }
    }

    pub fn summary(&self) -> String {
        let s = self.stats();
        format!(
            "bg_scheduler: {} audits (types: {:?}) proposals={} last_run={} interval={}",
            s.total_audits,
            s.audits_by_type,
            s.self_modify_proposals,
            s.last_run_cycle,
            self.base_interval
        )
    }
}

#[derive(Debug, Clone)]
pub struct BESStats {
    pub total_audits: u64,
    pub audits_by_type: std::collections::HashMap<String, usize>,
    pub self_modify_proposals: u64,
    pub last_run_cycle: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_returns_nothing_before_interval() {
        let mut s = BackgroundEvolutionScheduler::new(100);
        s.last_run_cycle = 50;
        let tasks = s.evaluate(60, 0, 0.8);
        assert!(tasks.is_empty(), "should not create tasks before interval");
    }

    #[test]
    fn test_evaluate_returns_source_audit_at_interval() {
        let mut s = BackgroundEvolutionScheduler::new(100);
        s.last_run_cycle = 0;
        let tasks = s.evaluate(100, 10, 0.8);
        assert!(!tasks.is_empty(), "should create source audit at interval");
        let has_source = tasks
            .iter()
            .any(|(t, _, _, _, _)| *t == AuditType::SourceStructure);
        assert!(has_source, "should include source structure audit");
    }

    #[test]
    fn test_compile_health_at_milestone() {
        let mut s = BackgroundEvolutionScheduler::with_config(
            100,
            BESConfig {
                compile_audit_interval: 200,
                source_audit_interval: 10000, // disable source audit
                ..Default::default()
            },
        );
        let tasks = s.evaluate(200, 0, 0.8);
        assert!(
            !tasks.is_empty(),
            "should create compile health audit at milestone"
        );
        let has_compile = tasks
            .iter()
            .any(|(t, _, _, _, _)| *t == AuditType::CompileHealth);
        assert!(has_compile, "should include compile health check");
    }

    #[test]
    fn test_record_audit_increments_count() {
        let mut s = BackgroundEvolutionScheduler::new(100);
        assert_eq!(s.audit_count, 0);
        s.record_audit(100, AuditType::SourceStructure, 1, "test audit");
        assert_eq!(s.audit_count, 1);
    }

    #[test]
    fn test_can_self_modify_respects_cooldown() {
        let mut s = BackgroundEvolutionScheduler::with_config(
            100,
            BESConfig {
                self_modify_cooldown: 10,
                ..Default::default()
            },
        );
        s.last_run_cycle = 100;
        assert!(
            !s.can_self_modify(105),
            "should not allow self-modify during cooldown"
        );
        assert!(
            s.can_self_modify(111),
            "should allow self-modify after cooldown"
        );
    }

    #[test]
    fn test_stats_tracking() {
        let mut s = BackgroundEvolutionScheduler::new(100);
        s.record_audit(100, AuditType::SourceStructure, 1, "audit 1");
        s.record_audit(200, AuditType::CompileHealth, 2, "audit 2");
        let stats = s.stats();
        assert_eq!(stats.total_audits, 2);
        assert_eq!(
            *stats.audits_by_type.get("source_structure").unwrap_or(&0),
            1
        );
        assert_eq!(*stats.audits_by_type.get("compile_health").unwrap_or(&0), 1);
    }

    #[test]
    fn test_dead_code_scan_at_interval() {
        let mut s = BackgroundEvolutionScheduler::with_config(
            100,
            BESConfig {
                dead_code_interval: 300,
                source_audit_interval: 10000, // disable
                compile_audit_interval: 0,    // disable
                dep_audit_interval: 0,        // disable
                ..Default::default()
            },
        );
        let tasks = s.evaluate(300, 0, 0.8);
        assert!(
            !tasks.is_empty(),
            "should create dead code scan at interval"
        );
        let has_dead = tasks
            .iter()
            .any(|(t, _, _, _, _)| *t == AuditType::DeadCodeScan);
        assert!(has_dead, "should include dead code scan");
    }

    #[test]
    fn test_summary_format() {
        let s = BackgroundEvolutionScheduler::new(100);
        let summary = s.summary();
        assert!(
            summary.contains("bg_scheduler:"),
            "summary should contain prefix"
        );
        assert!(summary.contains("audits=0"), "summary should show 0 audits");
    }

    #[test]
    fn test_audit_type_name() {
        assert_eq!(AuditType::SourceStructure.name(), "source_structure");
        assert_eq!(AuditType::CompileHealth.name(), "compile_health");
        assert_eq!(
            AuditType::DependencyFreshness.name(),
            "dependency_freshness"
        );
        assert_eq!(AuditType::DeadCodeScan.name(), "dead_code_scan");
    }
}
