//! nt_code_graph — EVO-05 语义索引＋系统图谱（cocoindex-code＋Ix 思想）。
//!
//! 有界符号调用图：symbols/calls 纯内存，callers/callees/impact 全部分片上限
//! [`MAX_SLICE`]，explain 只拼装有界文本。向量召回与持久化以后挂 trait，本文件
//! 只做同步纯逻辑，无 IO / 全局状态。

use serde::{Deserialize, Serialize};

/// 有界切片上限（查图不灌文件）。
pub const MAX_SLICE: usize = 50;

/// 符号类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Const,
}

/// 符号节点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolNode {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: u32,
}

impl SymbolNode {
    pub fn new(
        name: impl Into<String>,
        kind: SymbolKind,
        file: impl Into<String>,
        line: u32,
    ) -> Option<Self> {
        let name = name.into();
        let file = file.into();
        if name.is_empty() || file.is_empty() {
            return None;
        }
        Some(Self { name, kind, file, line })
    }
}

/// 调用边（caller → callee，按名关联）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallEdge {
    pub caller: String,
    pub callee: String,
}

impl CallEdge {
    pub fn new(caller: impl Into<String>, callee: impl Into<String>) -> Option<Self> {
        let caller = caller.into();
        let callee = callee.into();
        if caller.is_empty() || callee.is_empty() || caller == callee {
            return None;
        }
        Some(Self { caller, callee })
    }
}

/// 有界符号调用图。
#[derive(Debug, Default, Clone)]
pub struct CodeGraph {
    symbols: Vec<SymbolNode>,
    edges: Vec<CallEdge>,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 登录符号（同名同文件同行去重）。
    pub fn add_symbol(&mut self, node: SymbolNode) -> bool {
        if self
            .symbols
            .iter()
            .any(|s| s.name == node.name && s.file == node.file && s.line == node.line)
        {
            return false;
        }
        self.symbols.push(node);
        true
    }

    /// 登录调用边（全同去重）。
    pub fn add_edge(&mut self, edge: CallEdge) -> bool {
        if self.edges.iter().any(|e| e.caller == edge.caller && e.callee == edge.callee) {
            return false;
        }
        self.edges.push(edge);
        true
    }

    fn bounded(mut names: Vec<String>) -> Vec<String> {
        names.truncate(MAX_SLICE);
        names
    }

    /// 直接调用者（有界）。
    pub fn callers(&self, name: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.callee == name)
            .map(|e| e.caller.clone())
            .collect();
        out.sort();
        out.dedup();
        Self::bounded(out)
    }

    /// 直接被调用者（有界）。
    pub fn callees(&self, name: &str) -> Vec<String> {
        let mut out: Vec<String> = self
            .edges
            .iter()
            .filter(|e| e.caller == name)
            .map(|e| e.callee.clone())
            .collect();
        out.sort();
        out.dedup();
        Self::bounded(out)
    }

    /// 影响面：沿调用边双向 BFS（有界，depth 上限 4）。
    pub fn impact(&self, name: &str, depth: usize) -> Vec<String> {
        let depth = depth.min(4);
        let mut seen = vec![name.to_owned()];
        let mut frontier = vec![name.to_owned()];
        let mut d = 0;
        while d < depth && !frontier.is_empty() && seen.len() < MAX_SLICE {
            let mut next = Vec::new();
            for cur in &frontier {
                for e in &self.edges {
                    for nb in [&e.caller, &e.callee] {
                        if (e.caller == *cur || e.callee == *cur)
                            && !seen.contains(nb)
                            && seen.len() < MAX_SLICE
                        {
                            seen.push(nb.clone());
                            next.push(nb.clone());
                        }
                    }
                }
            }
            frontier = next;
            d += 1;
        }
        seen.remove(0);
        Self::bounded(seen)
    }

    /// 有界解释文本（符号＋一度调用关系）。
    pub fn explain(&self, name: &str) -> String {
        let mut out = String::from("symbol: ");
        out.push_str(name);
        if let Some(sym) = self.symbols.iter().find(|s| s.name == name) {
            out.push_str(" @ ");
            out.push_str(&sym.file);
            out.push(':');
            out.push_str(&sym.line.to_string());
        }
        let callers = self.callers(name);
        let callees = self.callees(name);
        out.push_str(" | callers(");
        out.push_str(&callers.len().min(5).to_string());
        out.push_str("):");
        for c in callers.iter().take(5) {
            out.push_str(&format!(" {c}"));
        }
        out.push_str(" | callees(");
        out.push_str(&callees.len().min(5).to_string());
        out.push_str("):");
        for c in callees.iter().take(5) {
            out.push_str(&format!(" {c}"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sym(name: &str) -> SymbolNode {
        SymbolNode::new(name, SymbolKind::Function, "a.rs", 1).unwrap_or(SymbolNode {
            name: String::new(),
            kind: SymbolKind::Function,
            file: String::new(),
            line: 0,
        })
    }

    #[test]
    fn empty_graph_returns_empty_slices() {
        let g = CodeGraph::new();
        assert!(g.callers("x").is_empty());
        assert!(g.callees("x").is_empty());
        assert!(g.impact("x", 3).is_empty());
    }

    #[test]
    fn callers_and_callees_resolve() {
        let mut g = CodeGraph::new();
        g.add_symbol(sym("a"));
        g.add_symbol(sym("b"));
        g.add_edge(CallEdge::new("a", "b").unwrap_or(CallEdge {
            caller: String::new(),
            callee: String::new(),
        }));
        assert_eq!(g.callees("a"), vec!["b".to_owned()]);
        assert_eq!(g.callers("b"), vec!["a".to_owned()]);
    }

    #[test]
    fn slices_truncate_at_max() {
        let mut g = CodeGraph::new();
        for i in 0..70 {
            let c = format!("c{i:03}");
            g.add_edge(CallEdge {
                caller: c,
                callee: "hub".to_owned(),
            });
        }
        assert_eq!(g.callers("hub").len(), MAX_SLICE);
    }

    #[test]
    fn impact_depth_bounded() {
        let mut g = CodeGraph::new();
        g.add_edge(CallEdge {
            caller: "a".to_owned(),
            callee: "b".to_owned(),
        });
        g.add_edge(CallEdge {
            caller: "b".to_owned(),
            callee: "c".to_owned(),
        });
        let d0 = g.impact("a", 0);
        assert!(d0.is_empty());
        let d2 = g.impact("a", 9);
        assert!(d2.contains(&"b".to_owned()));
        assert!(d2.contains(&"c".to_owned()));
        assert!(d2.len() <= MAX_SLICE);
    }

    #[test]
    fn explain_mentions_symbol_and_counts() {
        let mut g = CodeGraph::new();
        g.add_symbol(sym("a"));
        let text = g.explain("a");
        assert!(text.contains("symbol: a"));
        assert!(text.contains("a.rs"));
    }
}
