//! DSH-Explore — 研究语料探索 (CORDIS EU 研究数据) (C1)
//!
//! 吸收 dive.antinomie.org/dsh-explore (cordis-from-dsh): 研究语料探索工具，
//! 以 CORDIS (EU 研究项目数据库) 为底层语料，提供结构化检索与结果归一。
//! C1: SelfTest T1 + 单测；search 离线 (合成语料) 验证。

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// 研究项目记录 (CORDIS 归一结构)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct _ResearchProject {
    pub cordis_id: String,
    pub title: String,
    pub acronym: String,
    pub year: u16,
}

/// 研究语料探索契约 (DSH-Explore 抽象)
pub trait _ResearchCorpusExplorer: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<_ResearchProject>, String>;
}

/// 离线/合成语料探索器 (零网络依赖；真实实现走 CORDIS API)
pub struct _CordisExplorer;

impl _ResearchCorpusExplorer for _CordisExplorer {
    fn search(&self, query: &str) -> Result<Vec<_ResearchProject>, String> {
        if query.trim().is_empty() {
            return Err("dsh_explore: empty query".into());
        }
        if query.to_lowercase().contains("graph") {
            Ok(vec![_ResearchProject {
                cordis_id: "CORDIS-101012345".into(),
                title: "Graph Neural Networks for Reasoning".into(),
                acronym: "GNN-REASON".into(),
                year: 2023,
            }])
        } else {
            Ok(vec![])
        }
    }
}

/// SelfTest (T1)
pub struct _DshExploreSelfTest;

impl SelfTest for _DshExploreSelfTest {
    fn name(&self) -> &str {
        "nt_world_dsh_explore"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let e = _CordisExplorer;
        if e.search("").is_ok() {
            return Err(vec!["dsh_explore: empty query must error".into()]);
        }
        let r = e.search("graph neural").map_err(|e| vec![e])?;
        if r.is_empty() {
            return Err(vec!["dsh_explore: expected hit for 'graph'".into()]);
        }
        if r[0].cordis_id != "CORDIS-101012345" {
            return Err(vec!["dsh_explore: unexpected record id".into()]);
        }
        Ok(())
    }
}

/// 注册 DSH-Explore SelfTest
pub fn _register_dsh_explore_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(_DshExploreSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_errors() {
        assert!(_CordisExplorer.search("").is_err());
    }

    #[test]
    fn graph_query_hits() {
        // TODO: _CordisExplorer is an offline mock. Real CORDIS API integration
        // needs network + auth. This test validates the offline mock contract,
        // NOT real CORDIS behavior. Replace with integration test when wired.
        let r = _CordisExplorer.search("graph reasoning").unwrap();
        assert!(!r.is_empty(), "offline mock should return at least one result for 'graph' keyword");
        assert_eq!(r[0].cordis_id, "CORDIS-101012345");
    }

    #[test]
    fn no_match_returns_empty() {
        let r = _CordisExplorer.search("zzz unrelated").unwrap();
        assert!(r.is_empty());
    }
}
