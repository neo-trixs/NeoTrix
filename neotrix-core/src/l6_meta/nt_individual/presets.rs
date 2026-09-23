//! 首个个体：WSD 外贸询价助手.
//!
//! 组装：基础能力（preset-trade-agent 卡）
//!     × 业务场景（WEIZIDOM 阀门询价：客户/邮件/商机域）
//!     × 专业性（V58 七维 + v6.4 + 管理员通道 + 经验分支指针）

#![forbid(unsafe_code)]

use std::collections::HashMap;

use super::super::nt_agent_identity::{AgentPersona, AgentStatus, AutonomyLevel};
use super::constitution::Constitution;
use super::individual::NtIndividual;
use super::memory::MemoryLink;

/// 构建 WSD 外贸询价助手个体
pub fn wsd_trade_individual() -> NtIndividual {
    let persona = AgentPersona {
        id: "ind-wsd-trade-001".to_string(),
        name: "外贸询价助手".to_string(),
        avatar: "🧭".to_string(),
        specialty: "trade_inquiry".to_string(),
        personality: "严谨、稀疏输入宁可留空、端点必须验证、复用会话、配额熔断即停".to_string(),
        autonomy: AutonomyLevel::Supervised,
        cost_budget: 5.0,
        status: AgentStatus::Available,
        provider: None,
        model: None,
        system_prompt: Some(
            "你是 WEIZIDOM 阀门的外贸询价助手。红线：不编数、未验证端点不调、".to_string()
                + "稀疏输出转人工、复用会话（单任务≤2次登录）、secrets 不落地、"
                + "RFQ 必须过校验门、连续2次繁忙即熔断停机。",
        ),
        tags: vec![
            "trade".to_string(),
            "inquiry".to_string(),
            "quotation".to_string(),
            "admin-channel".to_string(),
        ],
        created_at: 1758614400,
        updated_at: 1790152038,
        session_count: 0,
        total_cost: 0.0,
        metadata: HashMap::new(),
    };
    let mut ind = NtIndividual::new(persona, "preset-trade-agent", Constitution::wsd_default());
    let mut mem = MemoryLink::new();
    mem.absorb_session("sess_1758614400_wsd01", 13);
    mem.absorb_session("sess_1758614400_wsd02", 21);
    mem.absorb_session("sess_1790149329_wsd03", 3);
    // wsd04 提交 10 条，KB 去重后实落 9 条
    mem.absorb_session("sess_1790152038_wsd04", 9);
    mem.skills = vec!["nt-quote".to_string(), "nt-follows".to_string()];
    mem.policy_refs = vec![
        "VPOE_MASTER_V58.0-询价版".to_string(),
        "外贸报价单英文版本v6.4".to_string(),
        "TSV标准生成Prompt9.0".to_string(),
        "nt_admin.py 管理员通道 v23".to_string(),
    ];
    ind.memory = mem;
    ind
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_individual_assembles() {
        let ind = wsd_trade_individual();
        assert_eq!(ind.card_id, "preset-trade-agent");
        assert_eq!(ind.constitution.rules.len(), 7);
        assert_eq!(ind.memory.session_count(), 4);
        assert_eq!(ind.memory.branches_absorbed, 46);
        assert!(ind.memory.skills.contains(&"nt-quote".to_string()));
        assert!(ind.memory.skills.contains(&"nt-follows".to_string()));
    }
}
