//! prompt_library — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Lightweight discovered skill (legacy compatibility).
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
}

// ────────────────────────────────────────────────────────────────
// P23: PromptLibrary (吸收 prompts.chat — 提示词资产库)
// 提示词资产持久库: 命名 + 版本 + 标签路由。供 harness / 进化 loop
// 复用工程化提示词, 替代散落的硬编码 prompt。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    pub name: String,
    pub version: u32,
    pub tags: Vec<String>,
    pub content: String,
    pub author: String,
}

impl PromptEntry {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            tags: vec![],
            content: content.into(),
            author: "neotrix".into(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptLibrary {
    prompts: Vec<PromptEntry>,
}

impl PromptLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, entry: PromptEntry) -> Result<(), String> {
        if let Some(existing) = self.prompts.iter_mut().find(|p| p.name == entry.name) {
            // 同名 → 版本递增 (prompts.chat 语义: 同名可迭代)
            existing.version += 1;
            existing.content = entry.content;
            existing.tags = entry.tags;
            return Ok(());
        }
        self.prompts.push(entry);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&PromptEntry> {
        self.prompts.iter().find(|p| p.name == name)
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&PromptEntry> {
        self.prompts.iter().filter(|p| p.tags.iter().any(|t| t == tag)).collect()
    }

    pub fn all(&self) -> &[PromptEntry] {
        &self.prompts
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }
}

impl crate::l0_substrate::nt_core_self_test::SelfTest for PromptLibrary {
    fn name(&self) -> &str {
        "nt_mind_prompt_library"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut lib = PromptLibrary::new();
        lib.register(PromptEntry::new("judge_rubric", "score 1-5").with_tags(vec!["eval".into()]))
            .map_err(|e| vec![e])?;
        if lib.len() != 1 {
            return Err(vec!["prompt library should hold 1 entry".into()]);
        }
        Ok(())
    }
}
