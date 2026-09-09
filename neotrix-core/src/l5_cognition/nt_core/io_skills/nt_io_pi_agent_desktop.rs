//! Pi 编程智能体桌面客户端 (NT-IO)
//!
//! 吸收源: github.com/Chasen-Liao/pi-agent-desktop
//! 成熟度: C1 (unit-tested stub, 无 Electron 运行时集成)
//!
//! 核心能力: 把编程智能体编排为桌面客户端形态 ——
//! 会话树 (session tree)、双轨分支 (dual-track branch)、CodeGraph MCP 接口。
//! 本 stub 负责会话/分支模型校验与 CodeGraph 查询规格生成。

use crate::core::nt_core_self_test::SelfTest;

/// 会话节点: 一棵会话树中的一次对话/任务单元。
#[derive(Debug, Clone, PartialEq)]
pub struct SessionNode {
    pub id: String,
    pub parent_id: Option<String>,
    pub label: String,
}

/// 双轨分支: 主轨 (main) 与实验轨 (track), 可合并回主轨。
#[derive(Debug, Clone, PartialEq)]
pub struct DualTrack {
    pub main: String,
    pub track: String,
}

/// 智能体桌面客户端 trait — 会话树 + 双轨分支 + CodeGraph 接口 stub。
pub trait AgentDesktopClient: Send + Sync {
    /// 校验会话树合法: 空树否, 每个非根节点有合法 parent, 无环。
    fn is_valid_tree(&self, nodes: &[SessionNode]) -> bool;
    /// 生成 CodeGraph MCP 查询规格: 给定符号名返回查询串, 空名返回 None。
    fn codegraph_query(&self, symbol: &str) -> Option<String>;
    /// 校验双轨可合并 (主轨与实验轨均非空)。
    fn can_merge_track(&self, track: &DualTrack) -> bool;
}

/// 默认实现。
#[derive(Default)]
pub struct PiAgentDesktop;

impl AgentDesktopClient for PiAgentDesktop {
    fn is_valid_tree(&self, nodes: &[SessionNode]) -> bool {
        if nodes.is_empty() {
            return false;
        }
        let ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        if ids.len() != nodes.len() {
            return false; // duplicate id
        }
        for n in nodes {
            if let Some(p) = &n.parent_id {
                if !ids.contains(p.as_str()) {
                    return false; // dangling parent
                }
            }
        }
        // 单根: 至多 1 个无 parent 的节点
        let roots = nodes.iter().filter(|n| n.parent_id.is_none()).count();
        if roots != 1 {
            return false;
        }
        // 环检测: 每个节点沿父链回溯必到根
        for n in nodes {
            let mut cur = n.parent_id.as_deref();
            let mut guard = 0usize;
            while let Some(p) = cur {
                if guard > nodes.len() {
                    return false; // cycle
                }
                guard += 1;
                cur = nodes.iter().find(|x| x.id == p).and_then(|x| x.parent_id.as_deref());
            }
        }
        true
    }

    fn codegraph_query(&self, symbol: &str) -> Option<String> {
        if symbol.trim().is_empty() {
            None
        } else {
            Some(format!("codegraph://symbol/{}", symbol))
        }
    }

    fn can_merge_track(&self, track: &DualTrack) -> bool {
        !track.main.trim().is_empty() && !track.track.trim().is_empty()
    }
}

/// T1 SelfTest: 验证会话树校验与 CodeGraph 接口存在且生效。
#[derive(Default)]
pub struct PiAgentDesktopSelfTest;

impl SelfTest for PiAgentDesktopSelfTest {
    fn name(&self) -> &str {
        "nt_io_pi_agent_desktop"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let a = PiAgentDesktop;
        let tree = vec![
            SessionNode { id: "root".into(), parent_id: None, label: "root".into() },
            SessionNode { id: "c1".into(), parent_id: Some("root".into()), label: "child".into() },
        ];
        if !a.is_valid_tree(&tree) {
            return Err(vec!["nt_io_pi_agent_desktop: valid tree rejected".into()]);
        }
        match a.codegraph_query("Foo::bar") {
            Some(q) if q.contains("Foo::bar") => Ok(()),
            _ => Err(vec!["nt_io_pi_agent_desktop: codegraph query failed".into()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tree() {
        let a = PiAgentDesktop;
        let tree = vec![
            SessionNode { id: "r".into(), parent_id: None, label: "r".into() },
            SessionNode { id: "a".into(), parent_id: Some("r".into()), label: "a".into() },
            SessionNode { id: "b".into(), parent_id: Some("a".into()), label: "b".into() },
        ];
        assert!(a.is_valid_tree(&tree));
    }

    #[test]
    fn test_rejects_dangling_and_duplicate() {
        let a = PiAgentDesktop;
        let dangling = vec![
            SessionNode { id: "r".into(), parent_id: None, label: "r".into() },
            SessionNode { id: "x".into(), parent_id: Some("ghost".into()), label: "x".into() },
        ];
        assert!(!a.is_valid_tree(&dangling));
        let dup = vec![
            SessionNode { id: "r".into(), parent_id: None, label: "r".into() },
            SessionNode { id: "r".into(), parent_id: None, label: "r2".into() },
        ];
        assert!(!a.is_valid_tree(&dup));
    }

    #[test]
    fn test_codegraph_and_merge() {
        let a = PiAgentDesktop;
        assert_eq!(a.codegraph_query(""), None);
        assert!(a.codegraph_query("MyType").unwrap().starts_with("codegraph://"));
        assert!(a.can_merge_track(&DualTrack { main: "m".into(), track: "t".into() }));
        assert!(!a.can_merge_track(&DualTrack { main: "".into(), track: "t".into() }));
    }
}
