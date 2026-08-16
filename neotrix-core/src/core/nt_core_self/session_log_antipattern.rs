//! 会话日志反模式检测 (Session Log Antipattern Scanner)
//!
//! 吸收 microsoft/AI-Engineering-Coach 的会话日志质量机制:
//! - 反模式规则 (本节点内置 6 条核心反模式, 对应源项目 45 条规则内核子集)
//! - 重复 prompt -> 技能发现 (discover_skills)
//! - context health 评分 (fragmentation + severity-weighted score)
//! - 数据不出机器: 全部为本地内存分析, 不落盘、不上报

use std::collections::{HashMap, HashSet};

const EVIDENCE_MAX: usize = 80;

#[derive(Debug, Clone)]
pub struct Antipattern {
    pub id: u32,
    pub name: &'static str,
    pub description: &'static str,
    pub severity: u8,
}

#[derive(Debug, Clone)]
pub struct AntipatternMatch {
    pub pattern_id: u32,
    pub location: String,
    pub evidence: String,
}

#[derive(Debug, Clone)]
pub struct ContextHealth {
    pub score: f64,
    pub fragmentation: f64,
    pub repeated_prompt_count: usize,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SessionLogScanner {
    pub patterns: Vec<Antipattern>,
}

impl Default for SessionLogScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLogScanner {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                Antipattern {
                    id: 1,
                    name: "context_overflow",
                    description: "会话行数超过 500, 上下文压力过高",
                    severity: 3,
                },
                Antipattern {
                    id: 2,
                    name: "repeated_prompt",
                    description: "相同 prompt 出现 >= 3 次, 重复占用上下文",
                    severity: 2,
                },
                Antipattern {
                    id: 3,
                    name: "dead_planning",
                    description: "存在 TODO/plan 但无 done, 计划未闭环",
                    severity: 2,
                },
                Antipattern {
                    id: 4,
                    name: "unchecked_error",
                    description: "生产路径使用 unwrap/expect 且无错误处理",
                    severity: 3,
                },
                Antipattern {
                    id: 5,
                    name: "silent_failure",
                    description: "空 panic 或静默 catch, 错误不可观测",
                    severity: 3,
                },
                Antipattern {
                    id: 6,
                    name: "scope_creep",
                    description: "also/in addition 频繁出现, 需求范围蔓延",
                    severity: 1,
                },
            ],
        }
    }

    pub fn scan(&self, log_lines: &[String]) -> Vec<AntipatternMatch> {
        let mut matches = Vec::new();
        let rows = log_lines.len();

        // context_overflow: 行数 > 500
        if rows > 500 {
            matches.push(AntipatternMatch {
                pattern_id: 1,
                location: format!("{}", 500),
                evidence: truncate(&log_lines[500]),
            });
        }

        // repeated_prompt: 完全相同行 >= 3 次
        let mut freq: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, line) in log_lines.iter().enumerate() {
            freq.entry(line.as_str()).or_default().push(i);
        }
        for (prompt, idxs) in &freq {
            if idxs.len() >= 3 {
                matches.push(AntipatternMatch {
                    pattern_id: 2,
                    location: format!("{}", idxs[0]),
                    evidence: truncate(prompt),
                });
            }
        }

        // dead_planning: 有 TODO/plan: 但全文无 done
        let has_done = log_lines.iter().any(|l| l.to_lowercase().contains("done"));
        if let Some(idx) = log_lines.iter().position(|l| {
            let lc = l.to_lowercase();
            lc.contains("todo") || lc.contains("plan:")
        }) {
            if !has_done {
                matches.push(AntipatternMatch {
                    pattern_id: 3,
                    location: format!("{}", idx),
                    evidence: truncate(&log_lines[idx]),
                });
            }
        }

        // unchecked_error: 有 unwrap(/expect( 且全文无错误处理证据
        let has_error_handling = log_lines.iter().any(|l| {
            let lc = l.to_lowercase();
            lc.contains("result")
                || lc.contains("err(")
                || lc.contains("if let err")
                || lc.contains("catch")
        });
        if !has_error_handling {
            for (i, line) in log_lines.iter().enumerate() {
                if line.contains("unwrap(") || line.contains("expect(") {
                    matches.push(AntipatternMatch {
                        pattern_id: 4,
                        location: format!("{}", i),
                        evidence: truncate(line),
                    });
                }
            }
        }

        // silent_failure: 空 panic!() 或静默 catch (全文无 err 字样)
        for (i, line) in log_lines.iter().enumerate() {
            if line.contains("panic!()") || line.contains("panic!(\"\")") {
                matches.push(AntipatternMatch {
                    pattern_id: 5,
                    location: format!("{}", i),
                    evidence: truncate(line),
                });
            }
        }
        let has_error_word = log_lines.iter().any(|l| l.to_lowercase().contains("err"));
        if !has_error_word {
            if let Some(idx) = log_lines.iter().position(|l| l.contains("catch")) {
                matches.push(AntipatternMatch {
                    pattern_id: 5,
                    location: format!("{}", idx),
                    evidence: truncate(&log_lines[idx]),
                });
            }
        }

        // scope_creep: also / in addition 频繁出现 (>= 3 行)
        let creep_idx: Vec<usize> = log_lines
            .iter()
            .enumerate()
            .filter(|(_, l)| {
                let lc = l.to_lowercase();
                lc.contains("also") || lc.contains("in addition")
            })
            .map(|(i, _)| i)
            .collect();
        if creep_idx.len() >= 3 {
            matches.push(AntipatternMatch {
                pattern_id: 6,
                location: format!("{}", creep_idx[0]),
                evidence: truncate(&log_lines[creep_idx[0]]),
            });
        }

        matches
    }

    pub fn context_health(&self, log_lines: &[String]) -> ContextHealth {
        let rows = log_lines.len().max(1) as f64;
        let matches = self.scan(log_lines);

        let repeated_prompt_count = matches
            .iter()
            .filter(|m| m.pattern_id == 2)
            .count();
        let fragmentation = (repeated_prompt_count as f64 / rows).min(1.0);

        let severity_weighted_hits: f64 = matches
            .iter()
            .map(|m| {
                self.patterns
                    .iter()
                    .find(|p| p.id == m.pattern_id)
                    .map(|p| p.severity as f64)
                    .unwrap_or(0.0)
            })
            .sum();
        let score = (1.0 - severity_weighted_hits / rows).max(0.0).min(1.0);

        let mut seen = HashSet::new();
        let mut suggestions = Vec::new();
        for m in &matches {
            if seen.insert(m.pattern_id) {
                if let Some(p) = self.patterns.iter().find(|p| p.id == m.pattern_id) {
                    suggestions.push(suggestion_for(p.name));
                }
            }
        }

        ContextHealth {
            score,
            fragmentation,
            repeated_prompt_count,
            suggestions,
        }
    }

    pub fn discover_skills(&self, log_lines: &[String]) -> Vec<String> {
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for line in log_lines {
            *freq.entry(line.as_str()).or_insert(0) += 1;
        }
        let mut seen = HashSet::new();
        let mut skills = Vec::new();
        for (prompt, count) in &freq {
            if *count >= 3 {
                let prefix: Vec<&str> = prompt.split_whitespace().take(3).collect();
                if prefix.is_empty() {
                    continue;
                }
                let candidate = prefix.join(" ");
                if seen.insert(candidate.clone()) {
                    skills.push(candidate);
                }
            }
        }
        skills
    }
}

fn truncate(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.chars().count() > EVIDENCE_MAX {
        trimmed.chars().take(EVIDENCE_MAX).collect()
    } else {
        trimmed.to_string()
    }
}

fn suggestion_for(name: &str) -> String {
    match name {
        "context_overflow" => {
            "会话上下文已溢出: 建议压缩历史轮次或启用摘要截断。".to_string()
        }
        "repeated_prompt" => {
            "检测到重复 prompt: 建议将高频请求沉淀为可复用技能, 减少上下文浪费。".to_string()
        }
        "dead_planning" => "检测到未闭环计划: 建议补齐 done 标记或清理过期 TODO/plan。".to_string(),
        "unchecked_error" => {
            "检测到无保护的 unwrap/expect: 建议改用 Result/? 传播, 生产路径禁止 panic。".to_string()
        }
        "silent_failure" => "检测到静默失败: 建议显式记录错误, 保证失败可观测。".to_string(),
        "scope_creep" => "检测到需求范围蔓延: 建议收敛任务边界并冻结需求。".to_string(),
        _ => "未知反模式: 建议审查会话日志。".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self_test::SelfTest;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_context_overflow_detected() {
        let scanner = SessionLogScanner::new();
        let mut log: Vec<String> = (0..501).map(|i| format!("line {}", i)).collect();
        log.push("done".to_string());
        let matches = scanner.scan(&log);
        assert!(matches.iter().any(|m| m.pattern_id == 1));
        assert!(matches.len() == 1);
    }

    #[test]
    fn test_repeated_prompt_skill_discovery() {
        let scanner = SessionLogScanner::new();
        let log = lines(&[
            "user: refactor module A",
            "user: refactor module A",
            "user: refactor module A",
            "user: summarize results",
            "done",
        ]);
        let skills = scanner.discover_skills(&log);
        assert_eq!(skills, vec!["user: refactor module".to_string()]);
        let matches = scanner.scan(&log);
        assert!(matches.iter().any(|m| m.pattern_id == 2));
    }

    #[test]
    fn test_health_score_in_range() {
        let scanner = SessionLogScanner::new();
        let clean = lines(&[
            "open session",
            "refactor completed",
            "done",
            "tests pass",
            "Result: ok",
        ]);
        let health = scanner.context_health(&clean);
        assert!((0.0..=1.0).contains(&health.score));
        assert!((0.0..=1.0).contains(&health.fragmentation));
        assert_eq!(health.repeated_prompt_count, 0);
        assert!(health.suggestions.is_empty());

        let unhealthy = lines(&[
            "user: run pipeline",
            "user: run pipeline",
            "user: run pipeline",
            "TODO: add tests",
            "unwrap()",
            "panic!()",
            "also improve caching",
            "also add metrics",
            "in addition wire alerts",
        ]);
        let health = scanner.context_health(&unhealthy);
        assert!((0.0..=1.0).contains(&health.score));
        assert!(health.fragmentation > 0.0);
        assert!(health.repeated_prompt_count >= 1);
        assert!(!health.suggestions.is_empty());
    }

    #[test]
    fn test_silent_failure_detected() {
        let scanner = SessionLogScanner::new();
        let log = lines(&["user: run pipeline", "panic!()", "done"]);
        let matches = scanner.scan(&log);
        assert!(matches.iter().any(|m| m.pattern_id == 5));
        assert!(matches.iter().any(|m| m.evidence.contains("panic!()")));
    }

    #[test]
    fn test_dead_planning_detected() {
        let scanner = SessionLogScanner::new();
        let log = lines(&["plan: implement feature X", "user: what next"]);
        let matches = scanner.scan(&log);
        assert!(matches.iter().any(|m| m.pattern_id == 3));

        let closed = lines(&["plan: implement feature X", "done: feature X shipped"]);
        let matches = scanner.scan(&closed);
        assert!(matches.iter().all(|m| m.pattern_id != 3));
    }

    #[test]
    fn test_selftest_ok() {
        let scanner = SessionLogScanner::new();
        let result = SelfTest::self_test(&scanner);
        assert!(result.is_ok());
    }
}

impl crate::core::nt_core_self_test::SelfTest for SessionLogScanner {
    fn name(&self) -> &str {
        "nt_core_self_session_log_antipattern"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let sample: Vec<String> = [
            "prompt: refactor module A",
            "prompt: refactor module A",
            "prompt: refactor module A",
            "TODO: add tests",
            "unwrap()",
            "panic!()",
            "also improve caching",
            "also add metrics",
            "in addition wire alerts",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let matches = self.scan(&sample);
        let mut failures = Vec::new();
        if matches.len() < 2 {
            failures.push(format!(
                "nt_core_self_session_log_antipattern: scan detected {} antipatterns, expected >= 2",
                matches.len()
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}