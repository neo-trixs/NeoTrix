//! Governance — NT-GOVERNANCE 治理层
//!
//! 吸收的能力模块 (trailofbits/skills):
//! - skill_validator: 30+ 强制规则验证器 + self-test
//! - red_team: DeepTeam 模式红队框架 (vulnerability + attack)
//!
//! ## 2026-10-07: human_oversight 占位自检已删除 (L8「绿色≠有效」)
//!
//! `human_oversight` 是 2026-08-28 从 arXiv:2608.23642 吸收的**设计级**
//! affordance (能力节点 `nt_governance::human_oversight`)。本模块自始
//! 无监督规则结构、无 enforcement hook —— 原 `HumanOversightSelfTest`
//! 无条件返回 "not wired" 并被 `.neotrix/arch-fitness-exempt.txt` 压制，
//! 是永久红占位壳而非可烧焦检验:接线真实规则不会让它转绿,只能重写它。
//! ⇒ 删除自检与注册函数; 能力节点同步 C2→C1 (去掉虚假 wiring_evidence)。
//!
//! 若未来真实落地 oversight 规则与执行钩子: 在此**新注册**一个引用真实
//! 规则列表的自检 (空列表 ⇒ fail with reason; 有规则 ⇒ 验证钩子可被触发),
//! 并同步能力节点补 file:line wiring evidence 重新升标。

pub mod skill_validator;
pub mod red_team;
