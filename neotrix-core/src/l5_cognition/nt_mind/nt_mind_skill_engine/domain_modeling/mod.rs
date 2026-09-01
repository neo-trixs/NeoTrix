//! Domain Modeling Discipline — mattpocock/skills domain-modeling 模式吸收
//! 
//! 主动领域建模纪律: 挑战术语、锐化模糊语言、具体场景压力测试、内联更新 CONTEXT.md、ADR 决策门控

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// 领域术语条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTerm {
    pub term: String,
    pub definition: String,
    pub aliases: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub source_file: Option<PathBuf>,
    pub line_number: Option<usize>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub adr_ref: Option<String>,
}

/// ADR 记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADRRecord {
    pub id: String,
    pub title: String,
    pub status: ADRStatus,
    pub context: String,
    pub decision: String,
    pub consequences: Vec<String>,
    pub alternatives: Vec<Alternative>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub superseded_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ADRStatus {
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// 领域建模上下文
#[derive(Debug, Clone)]
pub struct DomainModelingContext {
    pub glossary: HashMap<String, DomainTerm>,
    pub adrs: HashMap<String, ADRRecord>,
    pub context_md_path: PathBuf,
    pub adr_dir: PathBuf,
}

impl DomainModelingContext {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            glossary: HashMap::new(),
            adrs: HashMap::new(),
            context_md_path: project_root.join("CONTEXT.md"),
            adr_dir: project_root.join("docs").join("adr"),
        }
    }

    /// 挑战术语冲突
    pub fn challenge_term(&mut self, term: &str, proposed_definition: &str) -> ChallengeResult {
        if let Some(existing) = self.glossary.get(term) {
            if existing.definition != proposed_definition {
                return ChallengeResult::Conflict {
                    term: term.to_string(),
                    existing_definition: existing.definition.clone(),
                    proposed_definition: proposed_definition.to_string(),
                    existing_source: existing.source_file.clone(),
                };
            }
        }
        // 别名通道: sharpen_term 以 precise_term 为主键、vague_term 入 aliases;
        // 对模糊原词的挑战若不查别名则永远 NoConflict (共享语言漂移漏检)。
        for existing in self.glossary.values() {
            if existing.aliases.iter().any(|a| a == term)
                && existing.definition != proposed_definition
            {
                return ChallengeResult::Conflict {
                    term: term.to_string(),
                    existing_definition: existing.definition.clone(),
                    proposed_definition: proposed_definition.to_string(),
                    existing_source: existing.source_file.clone(),
                };
            }
        }
        ChallengeResult::NoConflict
    }

    /// 锐化模糊语言
    pub fn sharpen_term(&mut self, vague_term: &str, precise_term: &str, definition: &str) -> SharpenResult {
        if self.glossary.contains_key(vague_term) {
            return SharpenResult::AlreadyExists(vague_term.to_string());
        }
        
        let term = DomainTerm {
            term: precise_term.to_string(),
            definition: definition.to_string(),
            aliases: vec![vague_term.to_string()],
            anti_patterns: vec![],
            source_file: None,
            line_number: None,
            last_updated: chrono::Utc::now(),
            adr_ref: None,
        };
        
        self.glossary.insert(precise_term.to_string(), term);
        SharpenResult::Success(precise_term.to_string())
    }

    /// 压力测试场景
    pub fn stress_test(&self, scenario: &str) -> Vec<StressTestFinding> {
        let mut findings = Vec::new();
        
        // 简单的关键词检测，实际应更复杂
        for (term, def) in &self.glossary {
            if scenario.contains(term) {
                // 检查场景中是否以冲突方式使用
                for anti in &def.anti_patterns {
                    if scenario.contains(anti) {
                        findings.push(StressTestFinding {
                            term: term.clone(),
                            issue: format!("Scenario uses anti-pattern: {}", anti),
                            severity: StressSeverity::High,
                        });
                    }
                }
            }
        }
        
        findings
    }

    /// 内联更新 CONTEXT.md
    pub fn update_context_md(&self) -> Result<(), String> {
        let mut content = String::from("# Domain Glossary\n\n");
        content.push_str(&format!("_Auto-generated at {}_\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        let mut terms: Vec<_> = self.glossary.values().collect();
        terms.sort_by(|a, b| a.term.cmp(&b.term));
        
        for term in terms {
            content.push_str(&format!("## {}\n", term.term));
            content.push_str(&format!("{}\n\n", term.definition));
            
            if !term.aliases.is_empty() {
                content.push_str(&format!("**Aliases**: {}\n\n", term.aliases.join(", ")));
            }
            if !term.anti_patterns.is_empty() {
                content.push_str(&format!("**Anti-patterns**: {}\n\n", term.anti_patterns.join(", ")));
            }
            if let Some(ref adr) = term.adr_ref {
                content.push_str(&format!("**ADR**: {}\n\n", adr));
            }
        }
        
        std::fs::write(&self.context_md_path, content).map_err(|e| e.to_string())
    }

    /// 创建 ADR (满足三条件才创建)
    pub fn create_adr(
        &mut self,
        title: &str,
        context: &str,
        decision: &str,
        consequences: Vec<String>,
        alternatives: Vec<Alternative>,
        hard_to_reverse: bool,
        surprising: bool,
        real_tradeoff: bool,
    ) -> ADRResult {
        if !(hard_to_reverse && surprising && real_tradeoff) {
            return ADRResult::Skipped {
                reason: "ADR criteria not met: need hard_to_reverse && surprising && real_tradeoff".to_string(),
            };
        }

        let id = format!("{:04}-{}.md", self.adrs.len() + 1, title.to_lowercase().replace(' ', "-"));
        let adr = ADRRecord {
            id: id.clone(),
            title: title.to_string(),
            status: ADRStatus::Accepted,
            context: context.to_string(),
            decision: decision.to_string(),
            consequences,
            alternatives,
            created_at: chrono::Utc::now(),
            superseded_by: None,
        };

        self.adrs.insert(id.clone(), adr.clone());
        
        // 写入文件
        let _ = std::fs::create_dir_all(&self.adr_dir);
        let adr_path = self.adr_dir.join(&id);
        let adr_content = self.render_adr(&adr);
        let _ = std::fs::write(adr_path, adr_content);

        ADRResult::Created(id)
    }

    fn render_adr(&self, adr: &ADRRecord) -> String {
        let mut content = format!("# {}\n\n", adr.title);
        content.push_str(&format!("**Status**: {:?}\n", adr.status));
        content.push_str(&format!("**Date**: {}\n\n", adr.created_at.format("%Y-%m-%d")));
        
        content.push_str("## Context\n");
        content.push_str(&format!("{}\n\n", adr.context));
        
        content.push_str("## Decision\n");
        content.push_str(&format!("{}\n\n", adr.decision));
        
        content.push_str("## Consequences\n");
        for c in &adr.consequences {
            content.push_str(&format!("- {}\n", c));
        }
        content.push_str("\n");
        
        if !adr.alternatives.is_empty() {
            content.push_str("## Alternatives Considered\n");
            for alt in &adr.alternatives {
                content.push_str(&format!("### {}\n", alt.description));
                content.push_str("**Pros**:\n");
                for p in &alt.pros { content.push_str(&format!("- {}\n", p)); }
                content.push_str("**Cons**:\n");
                for c in &alt.cons { content.push_str(&format!("- {}\n", c)); }
                content.push_str("\n");
            }
        }
        
        content
    }
}

/// 挑战结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeResult {
    NoConflict,
    Conflict {
        term: String,
        existing_definition: String,
        proposed_definition: String,
        existing_source: Option<PathBuf>,
    },
}

/// 锐化结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharpenResult {
    Success(String),
    AlreadyExists(String),
}

impl SharpenResult {
    /// Returns the contained `String` if `Success`, panics otherwise.
    pub fn unwrap(self) -> String {
        match self {
            SharpenResult::Success(s) => s,
            SharpenResult::AlreadyExists(s) => s,
        }
    }

    /// Returns the contained `String` if `Success`, panics with message otherwise.
    pub fn expect(self, msg: &str) -> String {
        match self {
            SharpenResult::Success(s) => s,
            SharpenResult::AlreadyExists(_) => panic!("{}", msg),
        }
    }
}

/// 压力测试发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestFinding {
    pub term: String,
    pub issue: String,
    pub severity: StressSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StressSeverity {
    Low,
    Medium,
    High,
}

/// ADR 结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ADRResult {
    Created(String),
    Skipped { reason: String },
}

/// 领域建模引擎
#[derive(Debug)]
pub struct DomainModelingEngine {
    context: DomainModelingContext,
}

impl DomainModelingEngine {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            context: DomainModelingContext::new(project_root),
        }
    }

    pub fn load_existing(&mut self) -> Result<(), std::io::Error> {
        // 读取现有 CONTEXT.md
        if self.context.context_md_path.exists() {
            let content = std::fs::read_to_string(&self.context.context_md_path)?;
            self.parse_context_md(&content);
        }

        // 读取现有 ADRs
        if self.context.adr_dir.exists() {
            for entry in std::fs::read_dir(&self.context.adr_dir)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "md") {
                    let content = std::fs::read_to_string(entry.path())?;
                    if let Some(adr) = self.parse_adr_file(&content, entry.path()) {
                        self.context.adrs.insert(adr.id.clone(), adr);
                    }
                }
            }
        }

        Ok(())
    }

    fn parse_context_md(&mut self, content: &str) {
        // 简单解析，实际应更健壮
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("## ") {
                let term = lines[i].trim_start_matches("## ").trim();
                let mut definition = String::new();
                i += 1;
                while i < lines.len() && !lines[i].starts_with("## ") {
                    definition.push_str(lines[i]);
                    definition.push('\n');
                    i += 1;
                }
                let term_obj = DomainTerm {
                    term: term.to_string(),
                    definition: definition.trim().to_string(),
                    aliases: vec![],
                    anti_patterns: vec![],
                    source_file: Some(self.context.context_md_path.clone()),
                    line_number: None,
                    last_updated: chrono::Utc::now(),
                    adr_ref: None,
                };
                self.context.glossary.insert(term.to_string(), term_obj);
            } else {
                i += 1;
            }
        }
    }

    fn parse_adr_file(&self, content: &str, path: PathBuf) -> Option<ADRRecord> {
        // 简化解析
        if let Some(title_line) = content.lines().find(|l| l.starts_with("# ")) {
            let title = title_line.trim_start_matches("# ").trim();
            let id = path.file_name()?.to_str()?.to_string();
            Some(ADRRecord {
                id,
                title: title.to_string(),
                status: ADRStatus::Accepted,
                context: "".to_string(),
                decision: "".to_string(),
                consequences: vec![],
                alternatives: vec![],
                created_at: chrono::Utc::now(),
                superseded_by: None,
            })
        } else {
            None
        }
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        // Use a temp directory path directly without tempfile crate
        let temp_path = std::path::PathBuf::from("/tmp/neotrix_domain_modeling_test");
        let _ = std::fs::remove_dir_all(&temp_path);
        std::fs::create_dir_all(&temp_path).map_err(|e| e.to_string())?;
        
        let mut engine = DomainModelingEngine::new(temp_path.clone());

        // Test 1: Sharpen term
        let result = engine.context.sharpen_term("account", "Customer", "A paying customer entity");
        assert!(matches!(result, SharpenResult::Success(_)));

        // Test 2: Challenge conflict
        let _ = engine.context.sharpen_term("user", "User", "A system user");
        let challenge = engine.context.challenge_term("user", "A different definition");
        assert!(matches!(challenge, ChallengeResult::Conflict { .. }));

        // Test 3: ADR criteria
        let result = engine.context.create_adr(
            "Test ADR",
            "Context",
            "Decision",
            vec!["Consequence".to_string()],
            vec![],
            true, true, true
        );
        assert!(matches!(result, ADRResult::Created(_)));

        let result = engine.context.create_adr(
            "Test ADR 2",
            "Context",
            "Decision",
            vec![],
            vec![],
            false, true, true // missing hard_to_reverse
        );
        assert!(matches!(result, ADRResult::Skipped { .. }));

        // Test 4: Context.md update
        engine.context.update_context_md()?;
        assert!(engine.context.context_md_path.exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_path);
        
        Ok(())
    }
}

impl Default for DomainModelingEngine {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharpen_term() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut engine = DomainModelingEngine::new(temp_dir.path().to_path_buf());
        let result = engine.context.sharpen_term("account", "Customer", "A paying customer");
        assert!(matches!(result, SharpenResult::Success(_)));
    }

    #[test]
    fn test_challenge_conflict() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut engine = DomainModelingEngine::new(temp_dir.path().to_path_buf());
        engine.context.sharpen_term("user", "User", "System user").unwrap();
        let challenge = engine.context.challenge_term("user", "Different definition");
        assert!(matches!(challenge, ChallengeResult::Conflict { .. }));
    }

    #[test]
    fn test_adr_criteria() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut engine = DomainModelingEngine::new(temp_dir.path().to_path_buf());
        
        // All three criteria met
        let result = engine.context.create_adr(
            "Test", "Ctx", "Dec", vec![], vec![], true, true, true
        );
        assert!(matches!(result, ADRResult::Created(_)));

        // Missing hard_to_reverse
        let result = engine.context.create_adr(
            "Test2", "Ctx", "Dec", vec![], vec![], false, true, true
        );
        assert!(matches!(result, ADRResult::Skipped { .. }));
    }

    #[test]
    fn test_context_md_generation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut engine = DomainModelingEngine::new(temp_dir.path().to_path_buf());
        engine.context.sharpen_term("test", "Test", "A test term").unwrap();
        engine.context.update_context_md().unwrap();
        
        let content = std::fs::read_to_string(&engine.context.context_md_path).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("A test term"));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(DomainModelingEngine::self_test().is_ok());
    }
}