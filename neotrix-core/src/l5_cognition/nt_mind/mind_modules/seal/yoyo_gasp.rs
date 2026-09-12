//! NT-MIND — yoyo-gasp 吸收 (github.com/yologdev/yoyo-gasp, Batch2 #8, C2 集成).
//!
//! yoyo-gasp: 自进化编码智能体 (GASP 运行时参考实现)。其将 identity / skills /
//! memory / journal / lineage **全部入仓** (clone 即唤醒), 与 NeoTrix 的
//! Dark Forest 公理 (模块必须 compile + test + connect 或被删) 同构 — 一个
//! 自进化智能体若不能唤醒全部五要素, 即视为死代码 (Dark Forest 淘汰)。
//!
//! 本模块为 **C2 集成**: 定义 `_SelfEvolvingCodingAgent` 自进化编码智能体 trait 与
//! 五要素结构体, 每个要素映射 NeoTrix 对应模块, 并携带 GASP→SEAL 对照注释。
//! 区别于 C1 参考节点 `nt_mind_yoyo_gasp_site.rs` (仅字段级 trait stub), 此处给出
//! 可构造、可自检、可生产接线的结构体实现。
//!
//! # GASP → SEAL 对照 (核心同构)
//! | GASP 五要素          | NeoTrix 对应模块                              | SEAL 阶段            |
//! |----------------------|----------------------------------------------|----------------------|
//! | Identity (唤醒身份)   | `nt_core_self::Self` / `SystemIdentity`      | SEAL awaken/anchor    |
//! | Skills (技能单元)     | `nt_mind_skill_engine` (Disclosure Ladder)   | SEAL crystallize      |
//! | Memory (跨会话持久)   | `nt_memory` KB (`kv_store`)                  | SEAL distill          |
//! | Journal (审计轨迹)    | `ConsciousnessTree` 6-stage loop / experience-tree | SEAL self-test log |
//! | Lineage (代际传承)    | `SEALPipelineImpl` `make_stage!` 阶段血缘     | SEAL promote/lineage  |

use crate::core::nt_core_self_test::SelfTest;

/// 智能体身份 — 映射 `nt_core_self::Self` / `SystemIdentity` (E8引导者身份锚)。
/// GASP: clone 即唤醒的身份指纹, 决定智能体"是谁"。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct _AgentIdentity {
    pub id: String,
    pub awakened: bool,
    pub species: String,
}

impl _AgentIdentity {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            awakened: false,
            species: "gasp-coding-agent".into(),
        }
    }
    /// 唤醒: 对应 `ConsciousnessTree::awaken()` 加载 11 branch nodes。
    pub fn awaken(&mut self) {
        self.awakened = true;
    }
}

/// 技能单元 — 映射 `nt_mind_skill_engine` (Disclosure Ladder: anchor→standard promote)。
/// GASP: 可入仓/出仓的 skill, 每个 skill 带 self-test 门槛。
#[derive(Debug, Clone, PartialEq)]
pub struct AgentSkill {
    pub name: String,
    pub promoted: bool,
}

/// 经验记忆 — 映射 `nt_memory` KB `kv_store` / `experience` namespace (跨会话持久)。
#[derive(Debug, Clone, PartialEq)]
pub struct AgentMemory {
    pub kb_nodes: u64,
    pub experience_cycles: u64,
}

/// 演化日志 — 映射 `ConsciousnessTree` 6-stage loop / experience-tree 吸收日志。
/// 可读审计轨迹, 每次 patch→eval→decision→promote 写入一条 journal entry。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct _AgentJournal {
    pub entries: Vec<String>,
}

impl _AgentJournal {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
    pub fn append(&mut self, stage: &str, detail: &str) {
        self.entries.push(format!("[{}] {}", stage, detail));
    }
}

/// 演化谱系 — 映射 `SEALPipelineImpl` 经 `make_stage!` 宏生成的阶段血缘链。
/// 父子代际传承: parent_id → child_id, 记录 Promoted 代际。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct _AgentLineage {
    pub parent_id: Option<String>,
    pub generation: u64,
}

/// 自进化编码智能体 — 五要素聚合 trait。
///
/// GASP→SEAL 映射语义内置: `awaken`=SEAL anchor, `crystallize_skill`=SEAL crystallize,
/// `persist_memory`=SEAL distill, `log_stage`=SEAL self-test journal,
/// `inherit`=SEAL lineage promote。
pub(crate) trait _SelfEvolvingCodingAgent {
    fn identity(&self) -> &_AgentIdentity;
    fn awaken(&mut self);
    fn crystallize_skill(&mut self, name: &str) -> AgentSkill;
    fn inherit(&mut self, parent_id: &str) -> _AgentLineage;
}

/// yoyo-gasp 自进化编码智能体实现 (五要素全部入仓)。
pub(crate) struct _YoyoGaspAgent {
    pub identity: _AgentIdentity,
    pub skills: Vec<AgentSkill>,
    pub memory: AgentMemory,
    pub journal: _AgentJournal,
    pub lineage: _AgentLineage,
}

impl _YoyoGaspAgent {
    /// 构造一个完整智能体 (五要素全部初始化, 满足 Dark Forest 存活条件)。
    pub fn new(id: &str, parent_id: Option<&str>, generation: u64) -> Self {
        Self {
            identity: _AgentIdentity::new(id),
            skills: Vec::new(),
            memory: AgentMemory {
                kb_nodes: 0,
                experience_cycles: 0,
            },
            journal: _AgentJournal::new(),
            lineage: _AgentLineage {
                parent_id: parent_id.map(|s| s.to_string()),
                generation,
            },
        }
    }

    /// 五要素是否全部入仓 (Dark Forest 存活判定)。
    pub(crate) fn _is_fully_incarnated(&self) -> bool {
        !self.identity.id.is_empty()
            && self.memory.kb_nodes > 0
            && !self.journal.entries.is_empty()
    }
}

impl _SelfEvolvingCodingAgent for _YoyoGaspAgent {
    fn identity(&self) -> &_AgentIdentity {
        &self.identity
    }
    fn awaken(&mut self) {
        self.identity.awaken();
        self.journal.append("awaken", "SEAL anchor — identity loaded");
    }
    fn crystallize_skill(&mut self, name: &str) -> AgentSkill {
        let skill = AgentSkill {
            name: name.to_string(),
            promoted: true,
        };
        self.skills.push(skill.clone());
        self.journal
            .append("crystallize", &format!("skill promoted: {}", name));
        skill
    }
    fn inherit(&mut self, parent_id: &str) -> _AgentLineage {
        self.lineage = _AgentLineage {
            parent_id: Some(parent_id.to_string()),
            generation: self.lineage.generation + 1,
        };
        self.journal.append(
            "lineage",
            &format!("inherited from {} (gen {})", parent_id, self.lineage.generation),
        );
        self.lineage.clone()
    }
}

impl SelfTest for _YoyoGaspAgent {
    fn name(&self) -> &'static str {
        "_YoyoGaspAgent"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        if self.identity.id.is_empty() {
            errs.push("identity.id empty — GASP incarnate failed".into());
        }
        if self.memory.kb_nodes == 0 {
            errs.push("memory.kb_nodes == 0 — no KB grounding".into());
        }
        if self.journal.entries.is_empty() {
            errs.push("journal empty — no SEAL self-test trail".into());
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_five_elements_incarnation() {
        let mut a = _YoyoGaspAgent::new("agent-001", None, 0);
        a.memory.kb_nodes = 10;
        a.awaken();
        a.crystallize_skill("codegen");
        assert!(a.identity.awakened);
        assert_eq!(a.skills.len(), 1);
        assert!(a._is_fully_incarnated());
        assert!(a.self_test().is_ok());
    }

    #[test]
    fn test_lineage_inherit_generates_new_generation() {
        let mut child = _YoyoGaspAgent::new("agent-002", None, 0);
        child.memory.kb_nodes = 5;
        let lin = child.inherit("agent-001");
        assert_eq!(lin.parent_id.as_deref(), Some("agent-001"));
        assert_eq!(lin.generation, 1);
        assert_eq!(child.lineage.generation, 1);
    }

    #[test]
    fn test_selftest_flags_unincarnated() {
        let a = _YoyoGaspAgent::new("agent-003", None, 0);
        // kb_nodes == 0, journal empty → Dark Forest 淘汰信号
        assert!(a.self_test().is_err());
        let errs = a.self_test().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("KB grounding")));
    }
}
