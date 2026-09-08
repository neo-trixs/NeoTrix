//! repo_reverse_prompt (P12, absorbed from gitreverse) — 仓库→prompt 反向工程。
//!
//! 将仓库元数据 (README/文件树/语言统计/dependencies) 合成单条 vibe-coding
//! 风格用户 prompt, 实现 "repo→prompt" 反向工程。纯函数、确定性、零网络。

use crate::core::nt_core_self_test::SelfTest;

/// 仓库元数据快照 (由上游吸收管线填充)。
#[derive(Debug, Clone, Default)]
pub struct RepoMetadata {
    pub name: String,
    pub description: String,
    pub language: String,
    pub file_tree: Vec<String>,
    pub star_count: u64,
    pub dependency_count: usize,
}

/// 合成后的单条用户 prompt 及其分解。
#[derive(Debug, Clone, Default)]
pub struct ReversePrompt {
    pub prompt: String,
    pub tokens_used: usize,
    pub sections: Vec<String>,
}

/// 反向工程配置: 文件树截断上限 + prompt 最大字符数。
#[derive(Debug, Clone)]
pub struct RepoReversePrompt {
    pub max_tree_entries: usize,
    pub max_prompt_chars: usize,
}

impl Default for RepoReversePrompt {
    fn default() -> Self {
        Self {
            max_tree_entries: 50,
            max_prompt_chars: 4000,
        }
    }
}

impl RepoReversePrompt {
    pub fn new(max_tree_entries: usize, max_prompt_chars: usize) -> Self {
        Self {
            max_tree_entries: max_tree_entries.max(1),
            max_prompt_chars: max_prompt_chars.max(64),
        }
    }

    /// 项目简介 section (name + description + language + stars)。
    fn intro_section(&self, meta: &RepoMetadata) -> String {
        let stars = meta.star_count;
        format!(
            "项目简介: {} (语言: {}, stars: {}) — {}",
            meta.name, meta.language, stars, meta.description
        )
    }

    /// 文件结构 section, 截断到 max_tree_entries。
    fn tree_section(&self, meta: &RepoMetadata) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("文件结构:".into());
        for entry in meta.file_tree.iter().take(self.max_tree_entries) {
            lines.push(format!("  {}", entry));
        }
        if meta.file_tree.len() > self.max_tree_entries {
            lines.push(format!("  … 其余 {} 项已省略", meta.file_tree.len() - self.max_tree_entries));
        }
        lines.join("\n")
    }

    /// 依赖密度 section。
    fn deps_section(&self, meta: &RepoMetadata) -> String {
        format!("依赖密度: {} 个 dependencies", meta.dependency_count)
    }

    /// 组装 sections → prompt, 超限截断并估算 tokens。
    pub fn synthesize(&self, meta: &RepoMetadata) -> ReversePrompt {
        let sections = vec![
            self.intro_section(meta),
            self.tree_section(meta),
            self.deps_section(meta),
        ];
        let joined = sections.join("\n\n");
        let prompt = truncate_chars(&joined, self.max_prompt_chars);
        let tokens_used = prompt.chars().count() / 4;
        ReversePrompt {
            prompt,
            tokens_used,
            sections,
        }
    }

    /// vibe-coding 前缀 + 截断 tree。
    pub fn vibe_prompt(&self, meta: &RepoMetadata) -> String {
        let mut body = format!("请用 vibe-coding 方式基于以下仓库重建项目: {}", meta.name);
        let tree = self.tree_section(meta);
        if !tree.is_empty() {
            body.push_str("\n\n");
            body.push_str(&tree);
        }
        truncate_chars(&body, self.max_prompt_chars)
    }

    /// 已合成 sections 数量。
    pub fn section_count(&self, rp: &ReversePrompt) -> usize {
        rp.sections.len()
    }
}

/// 按字符截断 (保留完整前缀, 省略号标记省略)。
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

impl SelfTest for RepoReversePrompt {
    fn name(&self) -> &str {
        "nt_world_absorber_repo_reverse_prompt"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures: Vec<String> = Vec::new();

        let eng = Self::default();
        let meta = RepoMetadata {
            name: "demo-repo".into(),
            description: "A demo repo".into(),
            language: "Rust".into(),
            file_tree: vec!["Cargo.toml".into(), "src/main.rs".into(), "README.md".into()],
            star_count: 42,
            dependency_count: 7,
        };
        let rp = eng.synthesize(&meta);
        if eng.section_count(&rp) != 3 {
            failures.push("section_count should be 3".into());
        }
        if !rp.prompt.contains("demo-repo") {
            failures.push("prompt should contain repo name".into());
        }
        if !rp.prompt.contains("依赖密度: 7") {
            failures.push("prompt should contain dependency count".into());
        }
        if rp.tokens_used == 0 {
            failures.push("tokens_used should be positive".into());
        }
        let vibe = eng.vibe_prompt(&meta);
        if !vibe.contains("demo-repo") {
            failures.push("vibe_prompt should contain repo name".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta_with_tree(n: usize) -> RepoMetadata {
        RepoMetadata {
            name: "demo-repo".into(),
            description: "A demo repo".into(),
            language: "Rust".into(),
            file_tree: (0..n).map(|i| format!("src/file_{}.rs", i)).collect(),
            star_count: 42,
            dependency_count: 7,
        }
    }

    #[test]
    fn synthesize_has_three_sections() {
        let eng = RepoReversePrompt::default();
        let rp = eng.synthesize(&meta_with_tree(3));
        assert_eq!(eng.section_count(&rp), 3);
        assert_eq!(rp.sections.len(), 3);
        assert!(rp.sections[0].contains("demo-repo"));
        assert!(rp.sections[0].contains("Rust"));
        assert!(rp.sections[0].contains("stars: 42"));
        assert!(rp.sections[1].contains("文件结构"));
        assert!(rp.sections[2].contains("依赖密度: 7"));
    }

    #[test]
    fn tree_truncation_limits_entries() {
        let eng = RepoReversePrompt::new(5, 4000);
        let rp = eng.synthesize(&meta_with_tree(100));
        let in_tree = rp
            .sections
            .iter()
            .filter(|s| s.contains("文件结构"))
            .next()
            .unwrap();
        assert!(in_tree.contains("src/file_0.rs"));
        assert!(in_tree.contains("src/file_4.rs"));
        assert!(!in_tree.contains("src/file_5.rs"));
        assert!(in_tree.contains("已省略"));
    }

    #[test]
    fn overlong_prompt_is_truncated() {
        let eng = RepoReversePrompt::new(10, 120);
        let rp = eng.synthesize(&meta_with_tree(10));
        assert!(rp.prompt.chars().count() <= 120);
        assert!(rp.prompt.ends_with("..."));
    }

    #[test]
    fn tokens_used_is_positive() {
        let eng = RepoReversePrompt::default();
        let rp = eng.synthesize(&meta_with_tree(3));
        assert!(rp.tokens_used > 0);
        assert_eq!(rp.tokens_used, rp.prompt.chars().count() / 4);
    }

    #[test]
    fn vibe_prompt_contains_repo_name() {
        let eng = RepoReversePrompt::default();
        let meta = meta_with_tree(3);
        let vibe = eng.vibe_prompt(&meta);
        assert!(vibe.contains("vibe-coding"));
        assert!(vibe.contains("demo-repo"));
        assert!(vibe.contains("文件结构"));
    }

    #[test]
    fn default_config_bounds() {
        let eng = RepoReversePrompt::default();
        assert_eq!(eng.max_tree_entries, 50);
        assert_eq!(eng.max_prompt_chars, 4000);
        let clamped = RepoReversePrompt::new(0, 0);
        assert!(clamped.max_tree_entries >= 1);
        assert!(clamped.max_prompt_chars >= 64);
    }

    #[test]
    fn self_test_passes() {
        let eng = RepoReversePrompt::default();
        assert!(eng.self_test().is_ok());
    }
}