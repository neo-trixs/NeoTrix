//! nt_skill_route — EVO-06 技能路由＋评审三件套（reverse-skill＋MuseAI-Skills 思想）。
//!
//! `RouteTable` 是路由单源真理：规则按 priority 升序命中首个；规则上限
//! [`MAX_RULES`]，超限拒绝新增（返回 Err，不 panic）。`SkillManifest`
//! 做三件套（name/version/permissions/acceptance）缺项检查。同步纯逻辑。

use serde::{Deserialize, Serialize};

/// 路由规则上限。
pub const MAX_RULES: usize = 64;

/// 路由错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteError {
    RegistryFull,
    EmptyPattern,
    EmptyId,
}

/// 单条路由规则（pattern 为小写子串，priority 越小越优先）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteRule {
    pub id: String,
    pub pattern: String,
    pub priority: u32,
}

impl RouteRule {
    pub fn new(id: impl Into<String>, pattern: impl Into<String>, priority: u32) -> Result<Self, RouteError> {
        let id = id.into();
        let pattern = pattern.into();
        if id.is_empty() {
            return Err(RouteError::EmptyId);
        }
        if pattern.is_empty() {
            return Err(RouteError::EmptyPattern);
        }
        Ok(Self { id, pattern: pattern.to_lowercase(), priority })
    }

    fn hits(&self, input: &str) -> bool {
        input.to_lowercase().contains(&self.pattern)
    }
}

/// 路由表（单源真理）。
#[derive(Debug, Default, Clone)]
pub struct RouteTable {
    rules: Vec<RouteRule>,
}

impl RouteTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// 新增规则（按 priority 保持升序；同 id 替换；超限拒绝）。
    pub fn add_rule(&mut self, rule: RouteRule) -> Result<(), RouteError> {
        if let Some(pos) = self.rules.iter().position(|r| r.id == rule.id) {
            self.rules[pos] = rule;
            self.rules.sort_by_key(|r| r.priority);
            return Ok(());
        }
        if self.rules.len() >= MAX_RULES {
            return Err(RouteError::RegistryFull);
        }
        self.rules.push(rule);
        self.rules.sort_by_key(|r| r.priority);
        Ok(())
    }

    /// 解析：首个命中的规则 id。
    pub fn resolve(&self, input: &str) -> Option<&str> {
        self.rules.iter().find(|r| r.hits(input)).map(|r| r.id.as_str())
    }

    /// 审计行（rule_id＋输入摘要前 24 字）。
    pub fn audit_line(rule_id: &str, input: &str) -> String {
        let mut chars = input.chars();
        let summary: String = chars.by_ref().take(24).collect();
        let more = if chars.next().is_some() { "…" } else { "" };
        format!("route={rule_id} input=\"{summary}{more}\"")
    }
}

/// 技能三件套清单。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub permissions: Vec<String>,
    pub acceptance: Vec<String>,
}

impl SkillManifest {
    /// 缺项检查：返回缺失的件名（空＝齐）。
    pub fn missing(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.name.is_empty() {
            out.push("name");
        }
        if self.version.is_empty() {
            out.push("version");
        }
        if self.permissions.is_empty() {
            out.push("permissions");
        }
        if self.acceptance.is_empty() {
            out.push("acceptance");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_wins_over_insertion_order() {
        let mut t = RouteTable::new();
        t.add_rule(RouteRule::new("low", "x", 90).unwrap_or(RouteRule {
            id: String::new(),
            pattern: String::new(),
            priority: 0,
        }))
        .ok();
        t.add_rule(RouteRule::new("high", "x", 5).unwrap_or(RouteRule {
            id: String::new(),
            pattern: String::new(),
            priority: 0,
        }))
        .ok();
        assert_eq!(t.resolve("X marks"), Some("high"));
    }

    #[test]
    fn registry_full_rejects_new_rule() {
        let mut t = RouteTable::new();
        for i in 0..MAX_RULES {
            t.add_rule(RouteRule {
                id: format!("r{i}"),
                pattern: format!("p{i}"),
                priority: i as u32,
            })
            .ok();
        }
        let err = t.add_rule(RouteRule {
            id: "over".to_owned(),
            pattern: "over".to_owned(),
            priority: 0,
        });
        assert_eq!(err, Err(RouteError::RegistryFull));
        // 同 id 替换不受上限影响
        assert!(t
            .add_rule(RouteRule {
                id: "r0".to_owned(),
                pattern: "p0x".to_owned(),
                priority: 1,
            })
            .is_ok());
    }

    #[test]
    fn no_match_returns_none_and_empty_rules() {
        let t = RouteTable::new();
        assert!(t.is_empty());
        assert_eq!(t.resolve("anything"), None);
    }

    #[test]
    fn manifest_missing_reports_parts() {
        let m = SkillManifest {
            name: String::new(),
            version: "1.0".to_owned(),
            permissions: Vec::new(),
            acceptance: vec!["eval".to_owned()],
        };
        let missing = m.missing();
        assert!(missing.contains(&"name"));
        assert!(missing.contains(&"permissions"));
        assert!(!missing.contains(&"acceptance"));
    }

    #[test]
    fn audit_line_truncates_long_input() {
        let line = RouteTable::audit_line("r1", "abcdefghijklmnopqrstuvwxyz");
        assert!(line.starts_with("route=r1"));
        assert!(line.contains("…"));
    }
}
