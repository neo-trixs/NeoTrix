//! L5 Agent 预置模板库 — 5 个内置 `AgentCard` 模板，馈入 L6 画廊。
//!
//! 层级：L5→L1 合法引用；本文件仅依赖
//! `crate::l1_action::nt_infra_agent_card`，禁止引用 L6。
//! 构造仅用亲眼见到的 API：`AgentCard::new`＋`with_capability`／`with_tag` 链。

use crate::l1_action::nt_infra_agent_card::{AgentCapability, AgentCard};

/// 代码评审员：静态评审＋改进建议。
pub fn code_reviewer_card() -> AgentCard {
    AgentCard::new(
        "preset-code-reviewer",
        "Code Reviewer",
        "Reviews code diffs for correctness, style and security risks.",
    )
    .with_capability(AgentCapability {
        name: "review_diff".to_string(),
        description: "Review a unified diff and list findings by severity.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["review".to_string()],
    })
    .with_capability(AgentCapability {
        name: "suggest_fix".to_string(),
        description: "Propose minimal patches for each review finding.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["review".to_string()],
    })
    .with_tag("review")
    .with_tag("code")
}

/// 文档写手：API 文档＋变更日志。
pub fn doc_writer_card() -> AgentCard {
    AgentCard::new(
        "preset-doc-writer",
        "Doc Writer",
        "Writes API docs, guides and changelogs from code and diffs.",
    )
    .with_capability(AgentCapability {
        name: "write_api_doc".to_string(),
        description: "Draft API reference for a module or endpoint.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["docs".to_string()],
    })
    .with_capability(AgentCapability {
        name: "write_changelog".to_string(),
        description: "Summarize a diff range into a changelog entry.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["docs".to_string()],
    })
    .with_tag("docs")
    .with_tag("writing")
}

/// 数据分析师：汇总统计＋趋势解读。
pub fn data_analyst_card() -> AgentCard {
    AgentCard::new(
        "preset-data-analyst",
        "Data Analyst",
        "Summarizes tabular data into stats, trends and caveats.",
    )
    .with_capability(AgentCapability {
        name: "summarize_table".to_string(),
        description: "Compute summary stats for a dataset or table.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["data".to_string()],
    })
    .with_capability(AgentCapability {
        name: "explain_trend".to_string(),
        description: "Describe trends and note data-quality caveats.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["data".to_string()],
    })
    .with_tag("data")
    .with_tag("analysis")
}

/// 安全审计员：风险点扫描＋加固建议。
pub fn security_auditor_card() -> AgentCard {
    AgentCard::new(
        "preset-security-auditor",
        "Security Auditor",
        "Scans configs and code for common security risks with fixes.",
    )
    .with_capability(AgentCapability {
        name: "scan_risk".to_string(),
        description: "List OWASP-style risks found in the given scope.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["security".to_string()],
    })
    .with_capability(AgentCapability {
        name: "harden_guide".to_string(),
        description: "Give least-privilege hardening steps for each risk.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["security".to_string()],
    })
    .with_tag("security")
    .with_tag("audit")
}

/// 测试工程师：用例设计＋失败定位。
pub fn test_engineer_card() -> AgentCard {
    AgentCard::new(
        "preset-test-engineer",
        "Test Engineer",
        "Designs test cases and triages failures to root causes.",
    )
    .with_capability(AgentCapability {
        name: "design_cases".to_string(),
        description: "Design unit and integration cases for a feature.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["test".to_string()],
    })
    .with_capability(AgentCapability {
        name: "triage_failure".to_string(),
        description: "Map a failure log to likely root cause and next check.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["test".to_string()],
    })
    .with_tag("test")
    .with_tag("quality")
}

/// 外贸询价员：询盘识别＋RFQ 提取＋价格匹配＋报价＋回信。
/// 组装个体 `l6 nt_individual::presets::wsd_trade_individual` 引用的卡。
pub fn trade_agent_card() -> AgentCard {
    AgentCard::new(
        "preset-trade-agent",
        "Trade Inquiry Agent",
        "Handles foreign-trade inquiries: RFQ extraction, price matching, quotation and reply drafts. Refuses to hallucinate prices.",
    )
    .with_capability(AgentCapability {
        name: "scan_inquiry".to_string(),
        description: "Scan emails for RFQ signals and rank an inquiry index.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_capability(AgentCapability {
        name: "extract_rfq".to_string(),
        description: "Extract product rows (valve/fitting schema) from one inquiry email.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_capability(AgentCapability {
        name: "match_price".to_string(),
        description: "Match items against the product price book; blank below threshold.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_capability(AgentCapability {
        name: "build_quotation".to_string(),
        description: "Build v6.4 quotation data (cost/0.85, FCA+freight, TERMS).".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_capability(AgentCapability {
        name: "draft_reply".to_string(),
        description: "Draft bilingual (EN/CN) reply for an inquiry email.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_capability(AgentCapability {
        name: "fetch_source".to_string(),
        description: "Fetch a browser-verified trade data source; refuse unverified paths.".to_string(),
        input_schema: None,
        output_schema: None,
        tags: vec!["trade".to_string()],
    })
    .with_tag("trade")
    .with_tag("inquiry")
    .with_tag("quotation")
}

/// 全部 6 个预置模板（画廊馈入统一入口）。
pub fn all_presets() -> Vec<AgentCard> {
    vec![
        code_reviewer_card(),
        doc_writer_card(),
        data_analyst_card(),
        security_auditor_card(),
        test_engineer_card(),
        trade_agent_card(),
    ]
}
