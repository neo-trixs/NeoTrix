//! L3 经验层 — 快速积累: 情境、失败教训、成功方案

use serde::{Deserialize, Serialize};

/// 单条经验 (情境-行动-结果-反思)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: String,
    pub context: String,      // 情境描述
    pub action: String,       // 采取的行动
    pub result: String,       // 行动结果
    pub reflection: String,   // 反思: 学到了什么
    pub domain: String,       // 领域标签
    pub quality: f64,         // 质量评分 0-1
    pub timestamp: String,
}

/// 失败教训
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub failure_description: String,
    pub root_cause: String,
    pub fix_applied: String,
    pub takeaway: String,     // 核心教训
    pub category: String,     // "bug" | "design" | "process" | "tool"
    pub severity: f64,        // 严重程度 0-1
    pub timestamp: String,
}

/// 成功方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub id: String,
    pub problem: String,
    pub approach: String,
    pub result: String,
    pub reusable: bool,       // 是否可复用
    pub pattern: String,      // 抽象模式
    pub domain: String,
    pub timestamp: String,
}

/// L3 经验层 — 快速积累
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalExperience {
    pub episodes: Vec<Episode>,
    pub failures: Vec<Lesson>,
    pub successes: Vec<Solution>,
    next_id: usize,
}

impl CrystalExperience {
    pub fn new() -> Self {
        Self {
            episodes: Vec::new(),
            failures: Vec::new(),
            successes: Vec::new(),
            next_id: 1,
        }
    }

    fn gen_id(&mut self, prefix: &str) -> String {
        let id = format!("{}-{:04}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    /// 记录一条经验
    pub fn record_episode(
        &mut self,
        context: impl Into<String>,
        action: impl Into<String>,
        result: impl Into<String>,
        reflection: impl Into<String>,
        domain: impl Into<String>,
        quality: f64,
    ) -> String {
        let id = self.gen_id("EP");
        self.episodes.push(Episode {
            id: id.clone(),
            context: context.into(),
            action: action.into(),
            result: result.into(),
            reflection: reflection.into(),
            domain: domain.into(),
            quality,
            timestamp: timestamp_now(),
        });
        id
    }

    /// 记录一次失败
    pub fn record_failure(
        &mut self,
        description: impl Into<String>,
        root_cause: impl Into<String>,
        fix: impl Into<String>,
        takeaway: impl Into<String>,
        category: impl Into<String>,
        severity: f64,
    ) -> String {
        let id = self.gen_id("FL");
        self.failures.push(Lesson {
            id: id.clone(),
            failure_description: description.into(),
            root_cause: root_cause.into(),
            fix_applied: fix.into(),
            takeaway: takeaway.into(),
            category: category.into(),
            severity,
            timestamp: timestamp_now(),
        });
        id
    }

    /// 记录一次成功
    pub fn record_success(
        &mut self,
        problem: impl Into<String>,
        approach: impl Into<String>,
        result: impl Into<String>,
        reusable: bool,
        pattern: impl Into<String>,
        domain: impl Into<String>,
    ) -> String {
        let id = self.gen_id("SU");
        self.successes.push(Solution {
            id: id.clone(),
            problem: problem.into(),
            approach: approach.into(),
            result: result.into(),
            reusable,
            pattern: pattern.into(),
            domain: domain.into(),
            timestamp: timestamp_now(),
        });
        id
    }

    /// 按领域查找经验
    pub fn episodes_by_domain(&self, domain: &str) -> Vec<&Episode> {
        self.episodes.iter().filter(|e| e.domain == domain).collect()
    }

    /// 按类别查找失败
    pub fn failures_by_category(&self, category: &str) -> Vec<&Lesson> {
        self.failures.iter().filter(|f| f.category == category).collect()
    }

    /// 获取可复用的成功方案
    pub fn reusable_solutions(&self) -> Vec<&Solution> {
        self.successes.iter().filter(|s| s.reusable).collect()
    }

    /// 统计
    pub fn stats(&self) -> ExperienceStats {
        let avg_episode_quality = if self.episodes.is_empty() {
            0.0
        } else {
            self.episodes.iter().map(|e| e.quality).sum::<f64>() / self.episodes.len() as f64
        };
        let avg_failure_severity = if self.failures.is_empty() {
            0.0
        } else {
            self.failures.iter().map(|f| f.severity).sum::<f64>() / self.failures.len() as f64
        };
        ExperienceStats {
            total_episodes: self.episodes.len(),
            total_failures: self.failures.len(),
            total_successes: self.successes.len(),
            avg_episode_quality,
            avg_failure_severity,
            reusable_solutions: self.successes.iter().filter(|s| s.reusable).count(),
        }
    }
}

/// 经验统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceStats {
    pub total_episodes: usize,
    pub total_failures: usize,
    pub total_successes: usize,
    pub avg_episode_quality: f64,
    pub avg_failure_severity: f64,
    pub reusable_solutions: usize,
}

fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}
