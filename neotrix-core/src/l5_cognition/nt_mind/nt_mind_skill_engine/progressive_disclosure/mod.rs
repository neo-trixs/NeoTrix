//! Progressive Disclosure — trailofbits/skills + AnchorPromote (P4) 吸收
//! 
//! frontmatter 索引 (~30 tokens) 命中才加载全文
//! Minimal(2工具) → Standard(10工具) 阶梯

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 披露阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisclosureStage {
    Minimal = 0,
    Standard = 1,
    Full = 2,
}

impl DisclosureStage {
    pub fn tool_budget(&self) -> usize {
        match self {
            DisclosureStage::Minimal => 2,
            DisclosureStage::Standard => 10,
            DisclosureStage::Full => 100,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DisclosureStage::Minimal => "Minimal",
            DisclosureStage::Standard => "Standard",
            DisclosureStage::Full => "Full",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            DisclosureStage::Minimal => Some(DisclosureStage::Standard),
            DisclosureStage::Standard => Some(DisclosureStage::Full),
            DisclosureStage::Full => None,
        }
    }
}

/// 披露信号
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromoteSignal {
    FirstDurableCall,
    ExplicitRequest,
    Both,
}

/// 技能索引条目 (frontmatter only, ~30 tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillIndexEntry {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub category: String,
    pub priority: u8,
    pub verified: bool,
    pub estimated_tokens: usize,
}

/// 技能完整条目 (on-demand load)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFullEntry {
    pub index: SkillIndexEntry,
    pub body: String,
    pub references: Vec<String>,
    pub parent: String,
}

/// AnchorPromote 引擎
#[derive(Debug)]
pub struct AnchorPromote {
    pub stage: DisclosureStage,
    pub minimal_tools: usize,
    pub standard_tools: usize,
    pub durable_calls: u32,
    pub signal: PromoteSignal,
    pub active_skills: Vec<String>,
}

impl AnchorPromote {
    pub fn new(minimal: usize, standard: usize, signal: PromoteSignal) -> Self {
        Self {
            stage: DisclosureStage::Minimal,
            minimal_tools: minimal,
            standard_tools: standard,
            durable_calls: 0,
            signal,
            active_skills: Vec::new(),
        }
    }

    pub fn active_tool_count(&self) -> usize {
        match self.stage {
            DisclosureStage::Minimal => self.minimal_tools,
            DisclosureStage::Standard => self.standard_tools,
            DisclosureStage::Full => self.active_skills.len(),
        }
    }

    pub fn record_call(&mut self) {
        self.durable_calls += 1;
    }

    pub fn maybe_promote(&mut self) -> bool {
        match self.signal {
            PromoteSignal::FirstDurableCall => {
                if self.durable_calls >= 1 && self.stage == DisclosureStage::Minimal {
                    self.stage = DisclosureStage::Standard;
                    return true;
                }
            }
            PromoteSignal::ExplicitRequest => {
                // Handled externally
            }
            PromoteSignal::Both => {
                if self.durable_calls >= 1 && self.stage == DisclosureStage::Minimal {
                    self.stage = DisclosureStage::Standard;
                    return true;
                }
            }
        }
        false
    }

    pub fn disclosure_savings(&self) -> f64 {
        1.0 - (self.minimal_tools as f64 / self.standard_tools as f64)
    }

    /// 技能可见性过滤
    pub fn filter_skills(&self, skills: Vec<SkillIndexEntry>) -> Vec<SkillIndexEntry> {
        let budget = self.active_tool_count();
        skills.into_iter()
            .filter(|s| s.verified) // Only verified skills
            .take(budget)
            .collect()
    }

    /// 注册检查 (detect inner-register notation, state markers, etc.)
    pub fn register_check(&self, output: &str) -> Vec<String> {
        let mut findings = Vec::new();
        
        if output.contains("⇒") || output.contains("⟹") {
            findings.push("inner-register notation detected".to_string());
        }
        
        let state_markers = ["PHEW", "GRRR", "DONE", "RETRY", "WAIT"];
        for marker in &state_markers {
            if output.contains(marker) {
                findings.push("state markers detected".to_string());
                break;
            }
        }
        
        if output.to_lowercase().contains("verified") && !output.contains("coverage") {
            findings.push("verified without coverage".to_string());
        }
        
        let lines: Vec<&str> = output.lines().collect();
        for window in lines.windows(3) {
            if window[0] == window[1] && window[1] == window[2] {
                findings.push("repetition loop detected".to_string());
                break;
            }
        }
        
        findings
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let mut ap = AnchorPromote::new(2, 10, PromoteSignal::FirstDurableCall);
        
        // Test 1: Default stage
        assert_eq!(ap.stage, DisclosureStage::Minimal);
        assert_eq!(ap.active_tool_count(), 2);
        
        // Test 2: Promotion on durable call
        ap.record_call();
        assert!(ap.maybe_promote());
        assert_eq!(ap.stage, DisclosureStage::Standard);
        assert_eq!(ap.active_tool_count(), 10);
        
        // Test 3: No re-promote
        assert!(!ap.maybe_promote());
        assert_eq!(ap.stage, DisclosureStage::Standard);
        
        // Test 4: Savings calculation
        assert!((ap.disclosure_savings() - 0.8).abs() < 0.01);
        
        // Test 5: Register check
        let findings = ap.register_check("Result: A ⇒ B");
        assert!(findings.iter().any(|f| f.contains("inner-register")));
        
        let findings = ap.register_check("PHEW done");
        assert!(findings.iter().any(|f| f.contains("state markers")));
        
        Ok(())
    }
}

impl Default for AnchorPromote {
    fn default() -> Self {
        Self::new(2, 10, PromoteSignal::FirstDurableCall)
    }
}

/// 渐进披露技能加载器
#[derive(Debug)]
pub struct ProgressiveSkillLoader {
    pub index: HashMap<String, SkillIndexEntry>,
    pub loaded_bodies: HashMap<String, SkillFullEntry>,
    pub disclosure: AnchorPromote,
}

impl ProgressiveSkillLoader {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            loaded_bodies: HashMap::new(),
            disclosure: AnchorPromote::default(),
        }
    }

    /// 注册技能索引 (仅 frontmatter)
    pub fn register_index(&mut self, entry: SkillIndexEntry) {
        self.index.insert(entry.name.clone(), entry);
    }

    /// 按需加载技能全文
    pub fn load_skill(&mut self, name: &str, body: String, references: Vec<String>, parent: String) -> Option<SkillFullEntry> {
        let index = self.index.get(name)?.clone();
        let full = SkillFullEntry {
            index: index.clone(),
            body,
            references,
            parent,
        };
        self.loaded_bodies.insert(name.to_string(), full.clone());
        Some(full)
    }

    /// 获取可见技能 (受披露阶段预算限制)
    pub fn visible_skills(&self) -> Vec<SkillIndexEntry> {
        self.disclosure.filter_skills(self.index.values().cloned().collect())
    }

    /// 记录持久化调用
    pub fn record_durable_call(&mut self) {
        self.disclosure.record_call();
        self.disclosure.maybe_promote();
    }

    /// SelfTest
    pub fn self_test() -> Result<(), String> {
        let mut loader = ProgressiveSkillLoader::new();
        
        loader.register_index(SkillIndexEntry {
            name: "skill-a".to_string(),
            description: "Skill A".to_string(),
            triggers: vec!["a".to_string()],
            category: "test".to_string(),
            priority: 10,
            verified: true,
            estimated_tokens: 100,
        });
        
        loader.register_index(SkillIndexEntry {
            name: "skill-b".to_string(),
            description: "Skill B".to_string(),
            triggers: vec!["b".to_string()],
            category: "test".to_string(),
            priority: 20,
            verified: true,
            estimated_tokens: 100,
        });
        
        // Minimal stage: only 2 tools budget
        let visible = loader.visible_skills();
        assert_eq!(visible.len(), 2); // Both fit in budget of 2
        
        // Load one
        loader.load_skill("skill-a", "body a".to_string(), vec![], "".to_string());
        assert!(loader.loaded_bodies.contains_key("skill-a"));
        
        Ok(())
    }
}

impl Default for ProgressiveSkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_promote_stages() {
        let mut ap = AnchorPromote::new(2, 10, PromoteSignal::FirstDurableCall);
        assert_eq!(ap.stage, DisclosureStage::Minimal);
        assert_eq!(ap.active_tool_count(), 2);
        
        ap.record_call();
        assert!(ap.maybe_promote());
        assert_eq!(ap.stage, DisclosureStage::Standard);
        assert_eq!(ap.active_tool_count(), 10);
    }

    #[test]
    fn test_disclosure_savings() {
        let ap = AnchorPromote::new(2, 10, PromoteSignal::FirstDurableCall);
        assert!((ap.disclosure_savings() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_register_check() {
        let ap = AnchorPromote::default();
        let findings = ap.register_check("A ⇒ B");
        assert!(findings.iter().any(|f| f.contains("inner-register")));
    }

    #[test]
    fn test_progressive_loader() {
        let mut loader = ProgressiveSkillLoader::new();
        loader.register_index(SkillIndexEntry {
            name: "test".to_string(),
            description: "Test".to_string(),
            triggers: vec![],
            category: "test".to_string(),
            priority: 10,
            verified: true,
            estimated_tokens: 50,
        });
        
        let visible = loader.visible_skills();
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(AnchorPromote::self_test().is_ok());
        assert!(ProgressiveSkillLoader::self_test().is_ok());
    }
}