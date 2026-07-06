use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPattern {
    pub name: String,
    pub description: String,
    pub frequency: u32,
    pub sessions: Vec<String>,
    pub actionable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationReport {
    pub session_count: u32,
    pub patterns: Vec<SessionPattern>,
    pub suggestions: Vec<String>,
    pub generated_at: String,
}

pub struct SessionDistiller {
    pub session_logs_dir: PathBuf,
    pub agents_path: PathBuf,
    pub patterns: Vec<SessionPattern>,
}

impl Default for SessionDistiller {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionDistiller {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            session_logs_dir: home.join(".neotrix").join("session-logs"),
            agents_path: PathBuf::from("AGENTS.md"),
            patterns: Vec::new(),
        }
    }

    pub fn with_paths(session_logs_dir: PathBuf, agents_path: PathBuf) -> Self {
        Self {
            session_logs_dir,
            agents_path,
            patterns: Vec::new(),
        }
    }

    pub fn load_session_logs(&self) -> Vec<(String, String)> {
        let mut logs = Vec::new();
        if !self.session_logs_dir.exists() {
            return logs;
        }
        if let Ok(entries) = std::fs::read_dir(&self.session_logs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        logs.push((name, content));
                    }
                }
            }
        }
        logs.sort_by(|a, b| a.0.cmp(&b.0));
        logs
    }

    pub fn extract_patterns(&mut self, logs: &[(String, String)]) -> Vec<SessionPattern> {
        let mut pattern_map: HashMap<String, (String, Vec<String>, bool)> = HashMap::new();

        // Pattern: parallel execution ("同步执行")
        for (session_id, content) in logs {
            let lower = content.to_lowercase();

            // Detect "同步执行" pattern
            if lower.contains("同步执行") || lower.contains("parallel") {
                pattern_map.entry("parallel_dispatch".to_string())
                    .or_insert(("用户使用「同步执行」模式，要求并行分派独立任务".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }

            // Detect "还有需要进化的路线吗" pattern
            if lower.contains("还有需要进化的路线吗") || lower.contains("need evolution") {
                pattern_map.entry("bottleneck_analysis".to_string())
                    .or_insert(("用户周期性询问瓶颈分析，期望结构化路线图".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }

            // Detect architecture-first pattern
            if lower.contains("架构") || lower.contains("architecture") || lower.contains("全景") {
                pattern_map.entry("architecture_first".to_string())
                    .or_insert(("用户期望先收到架构全景/流程图的宏观概述，再进入执行".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }

            // Detect table preference
            if content.contains("|------|") || content.contains("| 等级 |") {
                pattern_map.entry("table_format".to_string())
                    .or_insert(("沟通输出偏好表格格式，便于快速对比优先级".to_string(), Vec::new(), false))
                    .1.push(session_id.clone());
            }

            // Detect batch approval pattern
            if lower.contains("继续完善") || lower.contains("全部") || lower.contains("all") {
                pattern_map.entry("batch_approval".to_string())
                    .or_insert(("用户偏好一次性批准全部路线图，而非逐个选择".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }

            // Detect "stand back" review pattern
            if lower.contains("上帝视角") || lower.contains("stand back") || lower.contains("审视") {
                pattern_map.entry("god_view_review".to_string())
                    .or_insert(("用户定期要求从上帝视角全流程审视，识别系统级缺口".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }

            // Detect GoalLoop / auto-goal usage
            if lower.contains("auto_goal") || lower.contains("自动制定") || lower.contains("pursue_auto") {
                pattern_map.entry("auto_goal_loop".to_string())
                    .or_insert(("核心工作模式：GoalLoop 自动目标追求 + BackgroundLoop".to_string(), Vec::new(), true))
                    .1.push(session_id.clone());
            }
        }

        let mut patterns: Vec<SessionPattern> = pattern_map
            .into_iter()
            .map(|(name, (desc, sessions, actionable))| SessionPattern {
                name,
                description: desc,
                frequency: sessions.len() as u32,
                sessions,
                actionable,
            })
            .collect();

        patterns.sort_by_key(|b| std::cmp::Reverse(b.frequency));
        self.patterns = patterns.clone();
        patterns
    }

    pub fn generate_suggestions(&self, patterns: &[SessionPattern]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for p in patterns {
            if !p.actionable {
                continue;
            }
            match p.name.as_str() {
                "parallel_dispatch" => {
                    suggestions.push("用户说「同步执行」时，立即用 Task tool 并行分派独立任务，不等确认。".to_string());
                }
                "bottleneck_analysis" => {
                    suggestions.push("每次完成一轮后主动提供「还有需要进化的路线吗」结构的路线图。".to_string());
                }
                "architecture_first" => {
                    suggestions.push("执行前先输出架构全景图/流程图，让用户确认方向。".to_string());
                }
                "batch_approval" => {
                    suggestions.push("给多选项时一次性列出全部路线，用户会选「同步执行」。".to_string());
                }
                "god_view_review" => {
                    suggestions.push("定期执行全流程审视：cargo check → 列出缺口 → 按等级排优先级。".to_string());
                }
                _ => {}
            }
        }
        suggestions
    }

    pub fn generate_distillation_report(&mut self) -> DistillationReport {
        let logs = self.load_session_logs();
        let patterns = self.extract_patterns(&logs);
        let suggestions = self.generate_suggestions(&patterns);

        DistillationReport {
            session_count: logs.len() as u32,
            patterns,
            suggestions,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn update_agents_md(&self) -> Result<(), String> {
        use std::io::Write;

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut content = String::new();
        content.push_str("# AGENTS.md — NeoTrix\n\n");
        content.push_str("_Auto-generated from session distillation — ");
        content.push_str(&format!("unix/{}", now_secs));
        content.push_str("_\n\n");

        if self.patterns.is_empty() {
            content.push_str("_No patterns distilled yet._\n");
        } else {
            content.push_str("## Behavioral Patterns\n\n");
            for pattern in &self.patterns {
                content.push_str(&format!(
                    "- **{}**: {} (frequency: {}, actionable: {})\n",
                    pattern.name,
                    pattern.description,
                    pattern.frequency,
                    pattern.actionable
                ));
            }
        }

        content.push_str("\n## Guidelines\n\n");
        content.push_str("- Follow existing code conventions (snake_case for Rust, camelCase for TS)\n");
        content.push_str("- Run `cargo check --lib` after each change\n");
        content.push_str("- Keep tests passing; add tests for new functionality\n");
        content.push_str("- Update TODO.md when completing items\n");

        let mut file = std::fs::File::create(&self.agents_path)
            .map_err(|e| format!("Failed to write {}: {}", self.agents_path.display(), e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write content: {}", e))?;

        Ok(())
    }

    pub fn distill_and_write(&mut self) -> Result<DistillationReport, String> {
        let report = self.generate_distillation_report();
        self.patterns = report.patterns.clone();
        self.update_agents_md()?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log(content: &str) -> Vec<(String, String)> {
        vec![("2026-05-15".to_string(), content.to_string())]
    }

    #[test]
    fn test_extract_parallel_pattern() {
        let mut d = SessionDistiller::new();
        let logs = sample_log("用户说：同步执行所有路线");
        let patterns = d.extract_patterns(&logs);
        assert!(patterns.iter().any(|p| p.name == "parallel_dispatch"));
    }

    #[test]
    fn test_extract_bottleneck_pattern() {
        let mut d = SessionDistiller::new();
        let logs = sample_log("还有需要进化的路线吗？");
        let patterns = d.extract_patterns(&logs);
        assert!(patterns.iter().any(|p| p.name == "bottleneck_analysis"));
    }

    #[test]
    fn test_generate_suggestions() {
        let mut d = SessionDistiller::new();
        let logs = sample_log("同步执行");
        let patterns = d.extract_patterns(&logs);
        let suggestions = d.generate_suggestions(&patterns);
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_empty_logs() {
        let mut d = SessionDistiller::new();
        let patterns = d.extract_patterns(&[]);
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_distillation_report() {
        let mut d = SessionDistiller::new();
        let logs = sample_log("同步执行\n还有需要进化的路线吗\n上帝视角审视");
        let patterns = d.extract_patterns(&logs);
        assert!(patterns.len() >= 3, "expected >=3 patterns, got {}", patterns.len());
        assert!(patterns.iter().any(|p| p.name == "parallel_dispatch"));
        assert!(patterns.iter().any(|p| p.name == "bottleneck_analysis"));
        assert!(patterns.iter().any(|p| p.name == "god_view_review"));
    }
}
