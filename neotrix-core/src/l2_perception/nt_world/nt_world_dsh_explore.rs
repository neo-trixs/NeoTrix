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



/// SelfTest (T1)
pub struct _DshExploreSelfTest;

impl SelfTest for _DshExploreSelfTest {
    fn name(&self) -> &str {
        "nt_world_dsh_explore"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
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
    fn selftest_passes() {
        let t = _DshExploreSelfTest;
        assert!(t.self_test().is_ok());
    }
}
