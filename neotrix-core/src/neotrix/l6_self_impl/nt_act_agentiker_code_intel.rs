//! L6 / NT-ACT — agentiker-code-intel (github.com/IVRZ-da/agentiker-code-intel)
//! 吸收节点 (C1)。
//!
//! 源: agentiker-code-intel — Agent 代码智能提取与分类: 从代码库抽取符号/
//! 关系/语义, 分类为可被 agent 利用的代码智能。NeoTrix 视角: 符号提取 + 类别
//! 分类 trait (C1: 基于正则/关键字 stub, 真实 AST 留待 C2+)。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashSet;

/// 代码符号类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Struct,
    Trait,
    Unknown,
}

/// 提取出的符号。
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
}

/// 代码智能提取器。
pub trait CodeIntel {
    fn extract(&self, src: &str) -> Vec<Symbol>;
    fn kinds(&self, syms: &[Symbol]) -> HashSet<SymbolKind>;
}

pub struct AgentikerCodeIntel;

impl CodeIntel for AgentikerCodeIntel {
    fn extract(&self, src: &str) -> Vec<Symbol> {
        let mut out = Vec::new();
        for line in src.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("fn ") {
                if let Some(name) = rest.split(['(', ' ']).next() {
                    out.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                    });
                }
            } else if let Some(rest) = t.strip_prefix("struct ") {
                out.push(Symbol {
                    name: rest.split([' ', '{']).next().unwrap_or("").to_string(),
                    kind: SymbolKind::Struct,
                });
            } else if let Some(rest) = t.strip_prefix("trait ") {
                out.push(Symbol {
                    name: rest.split([' ', '{']).next().unwrap_or("").to_string(),
                    kind: SymbolKind::Trait,
                });
            }
        }
        out
    }

    fn kinds(&self, syms: &[Symbol]) -> HashSet<SymbolKind> {
        syms.iter().map(|s| s.kind).collect()
    }
}

#[derive(Default)]
pub struct AgentikerCodeIntelSelfTest;

impl SelfTest for AgentikerCodeIntelSelfTest {
    fn name(&self) -> &str {
        "nt_act_agentiker_code_intel"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let ci = AgentikerCodeIntel;
        let src = "fn run() {}\nstruct Foo {}\ntrait Bar {}";
        let syms = ci.extract(src);
        let mut errs = Vec::new();
        if syms.len() != 3 {
            errs.push(format!("code_intel: expected 3 symbols, got {}", syms.len()));
        }
        let kinds = ci.kinds(&syms);
        if !kinds.contains(&SymbolKind::Function)
            || !kinds.contains(&SymbolKind::Struct)
            || !kinds.contains(&SymbolKind::Trait)
        {
            errs.push("code_intel: missing expected symbol kinds".into());
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_fn_struct_trait() {
        let syms = AgentikerCodeIntel.extract("fn a() {}\nstruct B {}\ntrait C {}");
        assert_eq!(syms.len(), 3);
    }

    #[test]
    fn classifies_kinds() {
        let syms = AgentikerCodeIntel.extract("fn a() {}\nstruct B {}");
        let kinds = AgentikerCodeIntel.kinds(&syms);
        assert!(kinds.contains(&SymbolKind::Function));
        assert!(kinds.contains(&SymbolKind::Struct));
    }

    #[test]
    fn ignores_unknown() {
        let syms = AgentikerCodeIntel.extract("let x = 1;");
        assert!(syms.is_empty());
    }
}
