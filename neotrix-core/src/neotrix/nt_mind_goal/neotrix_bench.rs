//! NeoTrixBench — 能力向量增长的标准化测量基准
//!
//! S-07: 衡量 NeoTrix 自主进化能力的关键指标:
//!   - 编译健康分 (compile_health): 0 errors / 0 warnings → 100
//!   - 测试覆盖率 (test_coverage): test_count / total_modules
//!   - 代码卫生分 (code_hygiene): unwrap/unsafe/todo 综合评分
//!   - 自修复速度 (self_heal_speed): 从诊断到修复的平均周期数
//!   - 能力向量增长 (capability_growth): CapabilityVector 各维度变化
//!
//! 文献对齐 (2026):
//!   - Maya XP-D9: Phi threshold crossing 9k→22k 作为意识涌现度量
//!   - 核心差异: NeoTrix 用 compile health + test coverage 替代 Phi
//!     作为系统健康的主要度

use crate::core::nt_core_cap::CapabilityVector;
use crate::neotrix::nt_mind_evolution_loop::ProjectSnapshot;

/// 基准维度
#[derive(Debug, Clone)]
pub struct BenchScore {
    pub compile_health: f64,
    pub test_coverage: f64,
    pub code_hygiene: f64,
    pub composite: f64,
}

/// 基准结果 — 跨 session 可比较
#[derive(Debug, Clone)]
pub struct BenchResult {
    pub score: BenchScore,
    pub capability_vector: Option<CapabilityVector>,
    pub total_files: usize,
    pub total_lines: usize,
    pub timestamp: i64,
}

/// NeoTrix 基准套件
pub struct NeoTrixBench;

impl NeoTrixBench {
    /// 从项目快照计算基准分数
    pub fn score_snapshot(snapshot: &ProjectSnapshot) -> BenchScore {
        let compile_health =
            Self::compile_health(snapshot.compile_errors, snapshot.compile_warnings);
        let test_coverage = Self::test_coverage(snapshot.test_count, snapshot.total_files);
        let code_hygiene = Self::code_hygiene(
            snapshot.unsafe_count,
            snapshot.unwrap_count,
            snapshot.todo_count,
        );
        let composite = compile_health * 0.35 + test_coverage * 0.30 + code_hygiene * 0.35;
        BenchScore {
            compile_health,
            test_coverage,
            code_hygiene,
            composite,
        }
    }

    /// 完整基准运行
    pub fn run(snapshot: &ProjectSnapshot, cv: Option<CapabilityVector>) -> BenchResult {
        let score = Self::score_snapshot(snapshot);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        BenchResult {
            score,
            capability_vector: cv,
            total_files: snapshot.total_files,
            total_lines: snapshot.total_lines,
            timestamp,
        }
    }

    /// 编译健康: errors=0 & warnings=0 → 100
    fn compile_health(errors: usize, warnings: usize) -> f64 {
        let e_penalty = (errors as f64) * 20.0;
        let w_penalty = (warnings as f64) * 5.0;
        (100.0f64 - e_penalty - w_penalty).max(0.0) / 100.0
    }

    /// 测试覆盖率: 每文件 1 test 为基准
    fn test_coverage(tests: usize, files: usize) -> f64 {
        if files == 0 {
            return 0.0;
        }
        let ratio = tests as f64 / files.max(1) as f64;
        (ratio / 5.0).min(1.0) // 5 tests/file = 100%
    }

    /// 代码卫生: unwrap 0 + unsafe 0 + todo 0 → 100
    fn code_hygiene(unsafe_count: usize, unwrap_count: usize, todo_count: usize) -> f64 {
        let u_penalty = (unsafe_count as f64) * 10.0;
        let w_penalty = (unwrap_count as f64) * 5.0;
        let t_penalty = (todo_count as f64) * 2.0;
        (100.0f64 - u_penalty - w_penalty - t_penalty).max(0.0) / 100.0
    }

    /// 生成进化趋势报告 (两期对比)
    pub fn trend_report(prev: &BenchResult, curr: &BenchResult) -> Vec<String> {
        let mut lines = Vec::new();
        let delta = |a: f64, b: f64| -> String {
            let d = b - a;
            if d > 0.01 {
                format!("↑ {:+.1}%", d * 100.0)
            } else if d < -0.01 {
                format!("↓ {:+.1}%", d * 100.0)
            } else {
                "→ 不变".into()
            }
        };
        lines.push(format!(
            "综合: {:.1}% {}",
            curr.score.composite * 100.0,
            delta(prev.score.composite, curr.score.composite)
        ));
        lines.push(format!(
            "编译健康: {:.1}% {}",
            curr.score.compile_health * 100.0,
            delta(prev.score.compile_health, curr.score.compile_health)
        ));
        lines.push(format!(
            "测试覆盖: {:.1}% {}",
            curr.score.test_coverage * 100.0,
            delta(prev.score.test_coverage, curr.score.test_coverage)
        ));
        lines.push(format!(
            "代码卫生: {:.1}% {}",
            curr.score.code_hygiene * 100.0,
            delta(prev.score.code_hygiene, curr.score.code_hygiene)
        ));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ProjectSnapshot {
        ProjectSnapshot {
            total_files: 120,
            total_lines: 130000,
            large_files: vec![],
            modules_without_tests: vec![],
            file_unsafe_hotspots: vec![],
            unsafe_count: 0,
            unwrap_count: 0,
            todo_count: 0,
            compile_errors: 0,
            compile_warnings: 0,
            test_count: 600,
            test_failures: 0,
        }
    }

    #[test]
    fn test_perfect_snapshot_scores_100() {
        let score = NeoTrixBench::score_snapshot(&sample_snapshot());
        assert!((score.composite - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compile_errors_penalize() {
        let mut s = sample_snapshot();
        s.compile_errors = 3;
        let score = NeoTrixBench::score_snapshot(&s);
        assert!(score.compile_health < 1.0);
        assert_eq!(score.compile_health, 0.4);
    }

    #[test]
    fn test_zero_tests_zero_coverage() {
        let mut s = sample_snapshot();
        s.test_count = 0;
        let score = NeoTrixBench::score_snapshot(&s);
        assert!(score.test_coverage < 0.01);
    }

    #[test]
    fn test_code_hygiene_penalties() {
        let mut s = sample_snapshot();
        s.unwrap_count = 3;
        s.todo_count = 5;
        let score = NeoTrixBench::score_snapshot(&s);
        assert!(score.code_hygiene < 1.0);
        assert!(score.code_hygiene > 0.5);
    }

    #[test]
    fn test_trend_report_shows_change() {
        let mut s1 = sample_snapshot();
        s1.compile_errors = 5;
        let prev = NeoTrixBench::run(&s1, None);
        let curr = NeoTrixBench::run(&sample_snapshot(), None);
        let report = NeoTrixBench::trend_report(&prev, &curr);
        assert!(report.iter().any(|l| l.contains("↑") || l.contains("→")));
    }

    #[test]
    fn test_trend_report_no_change() {
        let s = sample_snapshot();
        let prev = NeoTrixBench::run(&s, None);
        let curr = NeoTrixBench::run(&s, None);
        let report = NeoTrixBench::trend_report(&prev, &curr);
        assert!(report.iter().all(|l| l.contains("→")));
    }

    #[test]
    fn test_run_includes_timestamp() {
        let result = NeoTrixBench::run(&sample_snapshot(), None);
        assert!(result.timestamp > 0);
    }

    #[test]
    fn test_bench_result_with_capability_vector() {
        let cv = CapabilityVector::default();
        let result = NeoTrixBench::run(&sample_snapshot(), Some(cv));
        assert!(result.capability_vector.is_some());
    }
}
