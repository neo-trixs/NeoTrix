//! NT-MIND / NT-CORE — gasp 标准吸收 (github.com/yologdev/gasp, Batch2 #11, C2 集成).
//!
//! GASP 标准 (Generative Agent Standard Protocol): 将自进化智能体的全部状态归约为
//! 五个一等公民维度 — **identity / skills / memory / journal / lineage**, 全部入仓
//! (clone 即唤醒)。NeoTrix 采纳其为自进化 schema, 补强 `ConsciousnessTree` 的
//! lineage / journal 机制 (原 ConsciousnessTree 已有 cycle / current_contract / fruits
//! 但缺显式 journal 与 lineage 维度)。
//!
//! 本模块为 **C2 集成**: 定义 `GaspRepo` 五维度结构体 + 与 `ConsciousnessTree`
//! 字段的对应表 (文档注释), 作为 NeoTrix 自进化 schema 的采纳落点。
//!
//! # GASP 五维度 → ConsciousnessTree 字段 对应表
//! | GASP 维度   | ConsciousnessTree 字段                          | 补充语义                       |
//! |-------------|------------------------------------------------|-------------------------------|
//! | identity    | `awakened: bool` + `SystemIdentity`            | 唤醒标志 = clone 即唤醒        |
//! | skills      | `branches: HashMap<BranchKind, CapabilityBranch>` + `atoms` | 36 原子能力 = 技能单元 |
//! | memory      | `soil: DataFoundation` (kb_node_count 等) + `kb: Option<KB>` | KB 持久 = 跨会话记忆 |
//! | journal     | (NEW) 建议注入 `leaves`/`fruits` 之外的审计轨迹 | 补强原缺失的 journal 维度     |
//! | lineage     | `cycle: u64` + `current_contract: EvolutionContract` | 代际推进 = cycle/contract 血缘 |

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// GASP identity 维度 — 映射 `awakened` + `SystemIdentity`。
#[derive(Debug, Clone, PartialEq)]
pub struct GaspIdentity {
    pub id: String,
    pub awakened: bool,
    pub species: String,
}

/// GASP skills 维度 — 映射 `branches` / `atoms` (36 原子能力)。
#[derive(Debug, Clone, PartialEq)]
pub struct GaspSkill {
    pub name: String,
    pub promoted: bool,
}

/// GASP memory 维度 — 映射 `soil` (kb_node_count) + `kb` handle。
#[derive(Debug, Clone, PartialEq)]
pub struct GaspMemory {
    pub kb_node_count: u64,
    pub experience_cycles: u64,
}

/// GASP journal 维度 — **补强 ConsciousnessTree 原缺的显式审计轨迹**。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GaspJournal {
    pub entries: Vec<String>,
}

impl GaspJournal {
    pub fn append(&mut self, stage: &str, detail: &str) {
        self.entries.push(format!("[{}] {}", stage, detail));
    }
}

/// GASP lineage 维度 — 映射 `cycle` / `current_contract` 代际血缘。
#[derive(Debug, Clone, PartialEq)]
pub struct GaspLineage {
    pub parent_id: Option<String>,
    pub generation: u64,
    pub cycle: u64,
}

/// GASP 标准仓库 — 自进化 schema 的五维度采纳结构体。
///
/// 直接对齐 ConsciousnessTree 字段 (见模块级对应表), 作为 NeoTrix 把 GASP 标准
/// 收编为自进化 schema 的单一事实源节点 (R-P42: 强化现有节点, 禁止平行适配器)。
pub struct GaspRepo {
    pub identity: GaspIdentity,
    pub skills: HashMap<String, GaspSkill>,
    pub memory: GaspMemory,
    pub journal: GaspJournal,
    pub lineage: GaspLineage,
}

impl GaspRepo {
    /// 构造一个完整 GASP 仓库 (五维度全部初始化)。
    pub fn new(id: &str, parent_id: Option<&str>, cycle: u64, gen: u64) -> Self {
        Self {
            identity: GaspIdentity {
                id: id.to_string(),
                awakened: false,
                species: "gasp".into(),
            },
            skills: HashMap::new(),
            memory: GaspMemory {
                kb_node_count: 0,
                experience_cycles: 0,
            },
            journal: GaspJournal::default(),
            lineage: GaspLineage {
                parent_id: parent_id.map(|s| s.to_string()),
                generation: gen,
                cycle,
            },
        }
    }

    /// 注册一个技能单元 (映射 branches/atoms 注入)。
    pub fn register_skill(&mut self, name: &str) {
        self.skills.insert(
            name.to_string(),
            GaspSkill {
                name: name.to_string(),
                promoted: true,
            },
        );
        self.journal.append("skill", &format!("registered: {}", name));
    }

    /// 唤醒 (映射 ConsciousnessTree::awaken())。
    pub fn awaken(&mut self) {
        self.identity.awakened = true;
        self.journal.append("awaken", "GASP repo incarnated");
    }

    /// GASP 五维度是否全部就位 (采纳完整性自检)。
    pub fn is_complete(&self) -> bool {
        !self.identity.id.is_empty()
            && self.identity.awakened
            && self.memory.kb_node_count > 0
            && !self.journal.entries.is_empty()
    }
}

impl SelfTest for GaspRepo {
    fn name(&self) -> &'static str {
        "GaspRepo"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if self.identity.id.is_empty() {
            errs.push("GASP identity.id empty — schema adopt failed".into());
        }
        if !self.identity.awakened {
            errs.push("GASP identity not awakened — clone did not incarnate".into());
        }
        if self.memory.kb_node_count == 0 {
            errs.push("GASP memory.kb_node_count == 0 — no ConsciousnessTree soil grounding".into());
        }
        if self.journal.entries.is_empty() {
            errs.push("GASP journal empty — ConsciousnessTree lineage/journal gap unfilled".into());
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gasp_repo_five_dimensions_complete() {
        let mut r = GaspRepo::new("gasp-1", None, 1, 0);
        r.awaken();
        r.memory.kb_node_count = 42;
        r.register_skill("self-edit");
        assert!(r.is_complete());
        assert_eq!(r.skills.len(), 1);
        assert!(r.self_test().is_ok());
    }

    #[test]
    fn test_gasp_lineage_cycle_mapping() {
        let r = GaspRepo::new("gasp-2", Some("gasp-1"), 7, 3);
        assert_eq!(r.lineage.parent_id.as_deref(), Some("gasp-1"));
        assert_eq!(r.lineage.cycle, 7);
        assert_eq!(r.lineage.generation, 3);
    }

    #[test]
    fn test_selftest_flags_unawakened_repo() {
        let r = GaspRepo::new("gasp-3", None, 1, 0);
        // not awakened, kb 0, journal empty → 三处缺口
        assert!(r.self_test().is_err());
        let errs = r.self_test().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("not awakened")));
        assert!(errs.iter().any(|e| e.contains("journal empty")));
    }
}
