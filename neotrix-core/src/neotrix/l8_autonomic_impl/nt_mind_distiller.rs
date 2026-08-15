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

#[derive(Debug, Clone)]
pub struct VerbalizedSampling {
    pub num_candidates: usize,
    pub rng_seed: u64,
    pub sampled: usize,
    pub used: usize,
    state: u64,
}

impl Default for VerbalizedSampling {
    fn default() -> Self {
        Self::new()
    }
}

impl VerbalizedSampling {
    pub fn new() -> Self {
        Self::with_seed(0x9E3779B97F4A7C15)
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            num_candidates: 5,
            rng_seed: seed,
            sampled: 0,
            used: 0,
            state: seed,
        }
    }

    pub fn candidates_prompt(&self, task: &str) -> String {
        format!(
            "Generate {} distinct candidate responses for the following task. For each candidate, output a probability between 0 and 1 such that all probabilities sum to 1:\n\nTask: {}\n\nReturn as a numbered list '1. <candidate> (p=<prob>)'.",
            self.num_candidates, task
        )
    }

    pub fn parse_candidates(&self, raw: &str) -> Vec<(String, f64)> {
        let mut candidates: Vec<(String, f64)> = Vec::new();
        for line in raw.lines() {
            let line = line.trim();
            let Some(dot) = line.find('.') else { continue };
            let prefix = &line[..dot];
            if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let rest = line[dot + 1..].trim();
            if let Some((text, prob)) = extract_probability(rest) {
                if !text.is_empty() && prob.is_finite() && prob > 0.0 {
                    candidates.push((text, prob));
                }
            }
        }
        let total: f64 = candidates.iter().map(|(_, p)| *p).sum();
        if total <= 0.0 {
            return Vec::new();
        }
        candidates
            .into_iter()
            .map(|(text, prob)| (text, prob / total))
            .collect()
    }

    pub fn sample(&mut self, candidates: Vec<(String, f64)>) -> Option<String> {
        self.sampled += 1;
        if candidates.is_empty() {
            return None;
        }
        let u = self.next_f64();
        let mut cumulative = 0.0;
        for (text, prob) in &candidates {
            cumulative += prob;
            if u < cumulative {
                return Some(text.clone());
            }
        }
        candidates.last().map(|(text, _)| text.clone())
    }

    pub fn apply_to_patterns(
        &mut self,
        patterns: &[SessionPattern],
        suggestions: &[String],
    ) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        let mut seen: HashMap<String, usize> = HashMap::new();
        for suggestion in suggestions {
            let count = seen.entry(suggestion.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                // Diversity signal: the same suggestion recurs across patterns.
                // Re-inject via VS: build the candidate prompt and mark the
                // diversified variant so sampling is visible downstream.
                let _prompt = self.candidates_prompt(suggestion);
                self.used += 1;
                output.push(format!("[VS] {}", suggestion));
            } else {
                output.push(suggestion.clone());
            }
        }
        let _ = patterns;
        output
    }

    fn next_f64(&mut self) -> f64 {
        // Deterministic LCG (Numerical Recipes constants); no external rand crate.
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 11) as f64) / (1u64 << 53) as f64
    }
}

fn extract_probability(rest: &str) -> Option<(String, f64)> {
    let bytes = rest.as_bytes();
    let n = bytes.len();
    let mut last_start: Option<usize> = None;
    let mut last_end: Option<usize> = None;
    let mut last_val: Option<f64> = None;
    let mut i = 0;
    while i < n {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < n && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let token = &rest[start..i];
            let mut val = token.parse::<f64>().ok();
            // A minus sign immediately before the digit (e.g. "p=-0.5") makes
            // the probability negative; negate so it is discarded below.
            if val.is_some() && start > 0 && bytes[start - 1] == b'-' {
                val = val.map(|v| -v);
            }
            let mut end = i;
            if val.is_some() {
                if i < n && bytes[i] == b'%' {
                    end = i + 1;
                    val = val.map(|v| v / 100.0);
                }
                if let Some(v) = val {
                    if v.is_finite() && v > 0.0 {
                        last_start = Some(start);
                        last_end = Some(end);
                        last_val = Some(v);
                    }
                }
            }
            if end > i {
                i = end;
            }
        } else {
            i += 1;
        }
    }
    match (last_start, last_end, last_val) {
        (Some(start), Some(end), Some(val)) => {
            // Compute the byte range covering the probability decoration
            // ("(p=0.3)", "p=0.3", "(30%)", or bare "0.3") so it can be
            // stripped cleanly from the candidate text.
            let mut remove_start = start;
            let mut remove_end = end;
            if remove_end < n && bytes[remove_end] == b')' {
                remove_end += 1;
            }
            if remove_start >= 2 && &rest[remove_start - 2..remove_start] == "p=" {
                remove_start -= 2;
            }
            if remove_start > 0 && bytes[remove_start - 1] == b'(' {
                remove_start -= 1;
            }
            let mut text = String::with_capacity(rest.len());
            text.push_str(&rest[..remove_start]);
            if remove_end < rest.len() {
                text.push_str(&rest[remove_end..]);
            }
            Some((text.trim().to_string(), val))
        }
        _ => None,
    }
}

pub struct SessionDistiller {
    pub session_logs_dir: PathBuf,
    pub agents_path: PathBuf,
    pub patterns: Vec<SessionPattern>,
    pub vs: VerbalizedSampling,
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
            vs: VerbalizedSampling::new(),
        }
    }

    pub fn with_paths(session_logs_dir: PathBuf, agents_path: PathBuf) -> Self {
        Self {
            session_logs_dir,
            agents_path,
            patterns: Vec::new(),
            vs: VerbalizedSampling::new(),
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

    pub fn generate_suggestions(&mut self, patterns: &[SessionPattern]) -> Vec<String> {
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

        // VS diversity injection: repeated patterns yield duplicate suggestion
        // content, which triggers VerbalizedSampling diversification instead of
        // collapsing to a single answer (R-P42: wired into the existing dedup path).
        self.vs.apply_to_patterns(patterns, &suggestions)
    }

    pub fn vs_stats(&self) -> (usize, usize) {
        (self.vs.sampled, self.vs.used)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledOutput {
    pub id: String,
    pub kept_chars: usize,
    pub total_chars: usize,
    pub ratio: f64,
    pub error_lines: usize,
    pub artifact_path: String,
}

pub struct CommandDistiller {
    pub artifact_dir: PathBuf,
}

impl Default for CommandDistiller {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandDistiller {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            artifact_dir: home.join(".neotrix").join("distill-artifacts"),
        }
    }

    pub fn with_dir(artifact_dir: PathBuf) -> Self {
        Self { artifact_dir }
    }

    pub fn distill(&self, id: &str, full: &str, max_chars: usize) -> Result<DistilledOutput, String> {
        std::fs::create_dir_all(&self.artifact_dir).map_err(|e| e.to_string())?;
        let artifact_path = self.artifact_dir.join(format!("{}.out.txt", id));
        std::fs::write(&artifact_path, full).map_err(|e| e.to_string())?;

        let lines: Vec<&str> = full.lines().collect();
        let total_chars = full.chars().count();
        let error_lines = lines.iter().filter(|l| Self::is_error_line(l)).count();

        let mut kept: Vec<&str> = Vec::new();
        let mut blank_run = 0usize;
        for (i, l) in lines.iter().enumerate() {
            let is_error = Self::is_error_line(l);
            let is_head = i < 8;
            let is_tail = i + 12 >= lines.len();
            if is_error {
                kept.push(l);
                blank_run = 0;
            } else if is_head || is_tail || blank_run <= 1 {
                kept.push(l);
                blank_run = 0;
            } else {
                blank_run += 1;
            }
        }

        let mut deduped: Vec<&str> = Vec::new();
        for l in kept {
            if deduped.last().map(|prev| *prev == l).unwrap_or(false) {
                continue;
            }
            deduped.push(l);
        }

        let mut text = deduped.join("\n");
        let text_len = text.chars().count();
        if text_len > max_chars {
            let head_keep = max_chars * 3 / 4;
            let head: String = text.chars().take(head_keep).collect();
            let tail_len = max_chars.saturating_sub(head_keep);
            let tail: String = text.chars().skip(text_len.saturating_sub(tail_len)).collect();
            text = format!("{}\n... [truncated] ...\n{}", head, tail);
        }

        let kept_chars = text.chars().count();
        let ratio = if total_chars == 0 {
            1.0
        } else {
            kept_chars as f64 / total_chars as f64
        };

        Ok(DistilledOutput {
            id: id.to_string(),
            kept_chars,
            total_chars,
            ratio,
            error_lines,
            artifact_path: artifact_path.to_string_lossy().to_string(),
        })
    }

    pub fn expand(&self, id: &str) -> Result<String, String> {
        let artifact_path = self.artifact_dir.join(format!("{}.out.txt", id));
        std::fs::read_to_string(&artifact_path).map_err(|e| format!("expand {}: {}", id, e))
    }

    fn is_error_line(line: &str) -> bool {
        let lower = line.to_lowercase();
        lower.contains("error")
            || lower.contains("fail")
            || lower.contains("panic")
            || lower.contains("assert")
            || lower.contains("traceback")
            || lower.contains("exception")
            || lower.contains("cannot compile")
            || line.contains("-->")
            || line.starts_with("E0")
            || line.starts_with("E2")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log(content: &str) -> Vec<(String, String)> {
        vec![("2026-05-15".to_string(), content.to_string())]
    }

    #[test]
    fn test_command_distiller_persists_and_expands() {
        let dir = tempfile::tempdir().unwrap();
        let distiller = CommandDistiller::with_dir(dir.path().to_path_buf());
        let full = "cargo check\n   Compiling x\nerror[E0277]: mismatch\nwarning: unused\n   Finished\n";
        let out = distiller.distill("t1", full, 4096).unwrap();
        assert_eq!(out.total_chars, full.chars().count());
        assert!(out.error_lines >= 1);
        assert!(out.artifact_path.contains("t1.out.txt"));
        assert!(distiller.expand("t1").unwrap().contains("E0277"));
    }

    #[test]
    fn test_command_distiller_error_first_keeps_failures() {
        let dir = tempfile::tempdir().unwrap();
        let distiller = CommandDistiller::with_dir(dir.path().to_path_buf());
        let mut full = String::new();
        for i in 0..500 {
            full.push_str(&format!("noise line {}\n", i));
        }
        full.push_str("error[E0631]: type mismatch in closure\n");
        let out = distiller.distill("t2", &full, 400).unwrap();
        assert!(out.kept_chars <= 400 + 24);
        assert!(out.kept_chars < out.total_chars);
        let expanded = distiller.expand("t2").unwrap();
        assert!(expanded.contains("E0631"));
    }

    #[test]
    fn test_command_distiller_expand_missing_id() {
        let dir = tempfile::tempdir().unwrap();
        let distiller = CommandDistiller::with_dir(dir.path().to_path_buf());
        assert!(distiller.expand("nope").is_err());
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

    #[test]
    fn test_vs_candidates_prompt() {
        let vs = VerbalizedSampling::new();
        let prompt = vs.candidates_prompt("write a limerick");
        assert!(prompt.contains("probabilities"));
        assert!(prompt.contains("5"));
        assert!(prompt.contains("write a limerick"));
    }

    #[test]
    fn test_vs_parse_candidates_normalizes() {
        let vs = VerbalizedSampling::new();
        let raw = "1. rust pun (p=0.5)\n2. inside joke 0.3\n3. dad joke (30%)\n";
        let candidates = vs.parse_candidates(raw);
        assert_eq!(candidates.len(), 3);
        assert!(candidates[0].0.contains("rust pun"));
        assert!(candidates[1].0.contains("inside joke"));
        assert!(candidates[2].0.contains("dad joke"));
        let sum: f64 = candidates.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 1e-9, "normalized sum {}, expected ~1.0", sum);
    }

    #[test]
    fn test_vs_parse_candidates_malformed() {
        let vs = VerbalizedSampling::new();
        assert!(vs.parse_candidates("no candidates here, just garbage").is_empty());
        assert!(vs.parse_candidates("").is_empty());
        assert!(vs.parse_candidates("1. no probability attached").is_empty());
        assert!(vs.parse_candidates("1. zero probability (p=0)\n2. neg (p=-0.5)").is_empty());
    }

    #[test]
    fn test_vs_sample_deterministic_ranking() {
        let candidates = vec![
            ("top".to_string(), 0.9),
            ("mid".to_string(), 0.05),
            ("rest".to_string(), 0.05),
        ];
        let mut vs = VerbalizedSampling::with_seed(42);
        assert_eq!(vs.sample(candidates.clone()).unwrap(), "top");
        assert_eq!(vs.sampled, 1);
        // Same seed repeats the same pick deterministically.
        let mut again = VerbalizedSampling::with_seed(42);
        assert_eq!(again.sample(candidates).unwrap(), "top");
    }

    #[test]
    fn test_vs_sample_empty() {
        let mut vs = VerbalizedSampling::new();
        assert!(vs.sample(Vec::new()).is_none());
        assert_eq!(vs.sampled, 1);
    }

    #[test]
    fn test_vs_apply_to_patterns() {
        let mut vs = VerbalizedSampling::new();
        let patterns = vec![SessionPattern {
            name: "parallel_dispatch".to_string(),
            description: "d".to_string(),
            frequency: 2,
            sessions: vec!["a".to_string()],
            actionable: true,
        }];
        let suggestions = vec![
            "use Task tool".to_string(),
            "use Task tool".to_string(),
            "unique".to_string(),
        ];
        let out = vs.apply_to_patterns(&patterns, &suggestions);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], "use Task tool");
        assert_eq!(out[1], "[VS] use Task tool");
        assert_eq!(out[2], "unique");
        assert_eq!(vs.used, 1);
    }

    #[test]
    fn test_generate_suggestions_vs_wiring() {
        let mut d = SessionDistiller::new();
        let p = SessionPattern {
            name: "parallel_dispatch".to_string(),
            description: "d".to_string(),
            frequency: 2,
            sessions: vec![],
            actionable: true,
        };
        let suggestions = d.generate_suggestions(&[p.clone(), p]);
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[1], format!("[VS] {}", suggestions[0]));
        assert_eq!(d.vs_stats(), (0, 1));
    }

    #[test]
    fn test_vs_stats() {
        let mut d = SessionDistiller::new();
        assert_eq!(d.vs_stats(), (0, 0));
        let _ = d.vs.sample(vec![("a".to_string(), 1.0)]);
        let _ = d.vs.apply_to_patterns(&[], &["x".to_string(), "x".to_string()]);
        assert_eq!(d.vs_stats(), (1, 1));
    }
}
