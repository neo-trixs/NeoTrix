//! `nt_rules` — 四层规则编排 (first-match-wins).
//!
//! 对标 `internal/config/rules/system_rules.go:265-347`:
//! `Custom > Project > Global > System`, 每层 glob → 规则文本;
//! `resolve(path)` 返回命中的第一层. 文本哈希与过滤分离
//! (CanonicalConfig 只哈希文本, `rules explain` 做规则调试).

use serde::{Deserialize, Serialize};

/// 规则层 (优先级从高到低).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Custom,
    Project,
    Global,
    System,
}

impl Layer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Project => "project",
            Self::Global => "global",
            Self::System => "system",
        }
    }
}

/// 一条路径规则.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRule {
    pub pattern: String,
    pub rule: String,
}

/// 解析结果.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRule {
    pub layer: Layer,
    pub pattern: String,
    pub text: String,
}

/// 四层解析器 — 每层 first-match-wins, 层间高优先胜出.
#[derive(Debug, Default)]
pub struct RuleResolver {
    custom: Vec<PathRule>,
    project: Vec<PathRule>,
    global: Vec<PathRule>,
    system: Vec<PathRule>,
}

impl RuleResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, layer: Layer, pattern: &str, rule: &str) {
        let entry = PathRule {
            pattern: pattern.to_owned(),
            rule: rule.to_owned(),
        };
        match layer {
            Layer::Custom => self.custom.push(entry),
            Layer::Project => self.project.push(entry),
            Layer::Global => self.global.push(entry),
            Layer::System => self.system.push(entry),
        }
    }

    /// 解析路径 → 命中的规则 (层优先, 层内首个 glob 命中).
    /// glob 语法与 doublestar 同语义子集 (`*`/`**`/`?`), 经 `glob` crate.
    pub fn resolve(&self, path: &str) -> Option<ResolvedRule> {
        for (layer, rules) in [
            (Layer::Custom, &self.custom),
            (Layer::Project, &self.project),
            (Layer::Global, &self.global),
            (Layer::System, &self.system),
        ] {
            for entry in rules {
                if glob_match(&entry.pattern, path) {
                    return Some(ResolvedRule {
                        layer,
                        pattern: entry.pattern.clone(),
                        text: entry.rule.clone(),
                    });
                }
            }
        }
        None
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    glob::Pattern::new(pattern)
        .map(|matcher| matcher.matches(path))
        .unwrap_or(false)
}

/// 内置系统规则 (OCR `system_rules.json` 核心子集 + neotrix 层律).
/// `**/*.rs` 规则文本指向 rev-officer Rust 维度 (unwrap/unsafe/分层).
pub fn builtin_system_rules() -> Vec<PathRule> {
    vec![
        PathRule {
            pattern: "**/*.rs".to_owned(),
            rule: include_str!("../rules/rust.md").to_owned(),
        },
        PathRule {
            pattern: "**/Cargo.toml".to_owned(),
            rule: include_str!("../rules/cargo_toml.md").to_owned(),
        },
        PathRule {
            pattern: "**/*.ts,**/*.tsx,**/*.js".to_owned(),
            rule: include_str!("../rules/ts.md").to_owned(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{Layer, RuleResolver};

    #[test]
    fn layer_priority_first_match() {
        let mut resolver = RuleResolver::new();
        resolver.add(Layer::System, "**/*.rs", "sys");
        resolver.add(Layer::Project, "src/*.rs", "proj");
        let hit = resolver.resolve("src/a.rs").expect("hit");
        assert_eq!(hit.layer, Layer::Project);
        assert_eq!(hit.text, "proj");
        let sys = resolver.resolve("other/b.rs").expect("hit");
        assert_eq!(sys.layer, Layer::System);
        assert!(resolver.resolve("x.py").is_none());
    }

    #[test]
    fn builtin_rules_cover_rs() {
        let mut resolver = RuleResolver::new();
        for rule in super::builtin_system_rules() {
            resolver.add(Layer::System, &rule.pattern, &rule.rule);
        }
        assert!(resolver.resolve("crates/x/src/lib.rs").is_some());
    }
}
