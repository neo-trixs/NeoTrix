#![forbid(unsafe_code)]
//! # NT-Harness — 统一熔炼网关 (L1 Body)
//!
//! 对标：`harness/harness` Gitness + `harness/mcp-server` 11工具→139资源
//!       `openai/codex` app-server + `HKUDS/OpenHarness` 10子系统
//!       `grok-bot-cli` gbot + `grokbot-imessage` 本地桥
//!
//! 统一口径：所有能力通过 `harness_execute` 单入口 + `CAPABILITY_ROUTES` 自然语言路由，
//! 前端零学习成本（对话即OS）。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::nt_core_consciousness_core::TaskLoopReport;
use crate::l3_embodiment::nt_shield::shield_core::receipt::AgentReceipt;

/// `AgentReceipt` 的可序列化镜像 — 与 `HarnessExecuteResponse` 共用同一 serde 实例,
/// 避免跨 crate 的 serde 实例不匹配 (test build 下 `config::_serde` 与 workspace `serde` 冲突)。
/// 字段与 `AgentReceipt` 一一对应, 回放时可由其重建 `AgentReceipt` 再 `verify()`。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunReceipt {
    pub run_id: String,
    pub input_hash: String,
    pub output_hash: String,
    pub timestamp: i64,
    pub signature: String,
}

pub mod app_server;
pub mod router;
pub mod sandbox;

pub use app_server::{ApprovalRequest, ApprovalState, HarnessThread, HarnessTurn, ThreadStore};
pub use router::{InferenceProvider, InferenceRouter, RouterConfig};
pub use sandbox::{LocalSandbox, SandboxConfig, SandboxState};

// ── 11 统一工具（对标 harness/mcp-server registry dispatch） ───────────
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HarnessTool {
    List,
    Get,
    Create,
    Update,
    Delete,
    Execute,
    Search,
    HqlRun,
    HqlValidate,
    HqlExplain,
    HqlGrammar,
}

impl HarnessTool {
    pub fn all() -> &'static [Self] {
        use HarnessTool::*;
        &[List, Get, Create, Update, Delete, Execute, Search, HqlRun, HqlValidate, HqlExplain, HqlGrammar]
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::List => "harness_list",
            Self::Get => "harness_get",
            Self::Create => "harness_create",
            Self::Update => "harness_update",
            Self::Delete => "harness_delete",
            Self::Execute => "harness_execute",
            Self::Search => "harness_search",
            Self::HqlRun => "hql:run",
            Self::HqlValidate => "hql:validate",
            Self::HqlExplain => "hql:explain",
            Self::HqlGrammar => "hql:grammar",
        }
    }
}

// ── 能力标签 ↔ API 地图（单一事实源，与 CAPABILITY_ROUTES 同源） ────────
// 前端不感知此表，全部由对话自然语言命中；此处为 Rust 侧可审计地图。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityApiEntry {
    pub capability_tag: String,
    pub domain: String,
    pub specialist: String,
    pub keywords: Vec<String>,
    pub harness_tool: HarnessTool,
    pub description: String,
}

impl CapabilityApiEntry {
    pub fn new(
        tag: &str,
        domain: &str,
        specialist: &str,
        keywords: &[&str],
        tool: HarnessTool,
        desc: &str,
    ) -> Self {
        Self {
            capability_tag: tag.to_string(),
            domain: domain.to_string(),
            specialist: specialist.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            harness_tool: tool,
            description: desc.to_string(),
        }
    }
}

/// 全量地图（与 `nt_core_consciousness_core::CAPABILITY_ROUTES` 1:1 对齐 + 3个 orchestration 兜底）
pub fn capability_api_map() -> Vec<CapabilityApiEntry> {
    vec![
        CapabilityApiEntry::new("xlsx_consolidation","NT-ACT","CodeAnalyzer",&["excel","表格","价格表","统一"],HarnessTool::Execute,"表格合并 consolidate_tables"),
        CapabilityApiEntry::new("data_merge","NT-ACT","KnowledgeIntegrator",&["合并"],HarnessTool::Execute,"数据合并"),
        CapabilityApiEntry::new("file_parsing","NT-WORLD","CodeAnalyzer",&["文件","解析"],HarnessTool::Execute,"文件解析 extract_text/to_markdown"),
        CapabilityApiEntry::new("content_extraction","NT-WORLD","CodeAnalyzer",&["提取"],HarnessTool::Execute,"内容抽取"),
        CapabilityApiEntry::new("pdf_edit","NT-ACT","CodeAnalyzer",&["pdf编辑","编辑pdf"],HarnessTool::Execute,"PDF编辑 edit_pdf"),
        CapabilityApiEntry::new("image_convert","NT-ACT","CodeAnalyzer",&["图片转换","图像转换","格式转换"],HarnessTool::Execute,"图像转换"),
        CapabilityApiEntry::new("dir_extract","NT-WORLD","CodeAnalyzer",&["提取目录","目录提取","批量提取"],HarnessTool::Execute,"目录提取"),
        CapabilityApiEntry::new("pdf_merge","NT-ACT","CodeAnalyzer",&["合并pdf","合并PDF","pdf合并"],HarnessTool::Execute,"PDF合并 merge_pdfs"),
        CapabilityApiEntry::new("doc_merge","NT-ACT","CodeAnalyzer",&["合并文档","文档合并","合并word","合并docx","合并ppt"],HarnessTool::Execute,"文档合并 merge_docx"),
        CapabilityApiEntry::new("hybrid_retrieval","NT-MEMORY","KnowledgeRetriever",&["检索","查询","搜索"],HarnessTool::Search,"混合检索 KB+VSA"),
        CapabilityApiEntry::new("skill_crystallize","NT-MIND","KnowledgeIntegrator",&["吸收","蒸馏"],HarnessTool::Create,"技能晶化 absorb_core"),
        CapabilityApiEntry::new("tdd","NT-MIND","Planner",&["测试"],HarnessTool::Execute,"TDD测试"),
        CapabilityApiEntry::new("code_refactor","NT-ACT","CodeAnalyzer",&["重构"],HarnessTool::Execute,"代码重构"),
        CapabilityApiEntry::new("security_audit","NT-SHIELD","RiskAssessor",&["审查","审计","安全"],HarnessTool::Execute,"安全审计"),
        CapabilityApiEntry::new("architecture_decision","NT-CORE","Planner",&["架构","设计"],HarnessTool::Execute,"架构决策"),
        CapabilityApiEntry::new("consciousness_tree","NT-CORE","ReflectionEngine",&["意识"],HarnessTool::Get,"意识树状态"),
        CapabilityApiEntry::new("meta_cognition","NT-META","MetaCognitionAnalyst",&["元认知","复盘","反思"],HarnessTool::Execute,"元认知"),
        CapabilityApiEntry::new("root_cause_method","NT-REPAIR","AnomalyDetector",&["诊断","报错","构建失败"],HarnessTool::Execute,"根因诊断"),
        CapabilityApiEntry::new("unified_crawler","NT-WORLD","PatternMatcher",&["爬虫","抓取"],HarnessTool::Execute,"统一爬取 nt_world_crawl"),
        CapabilityApiEntry::new("frontend_ui","NT-IO","CreativityGenerator",&["前端","界面"],HarnessTool::Execute,"前端UI"),
        CapabilityApiEntry::new("experience_absorb","NT-MEMORY","KnowledgeIntegrator",&["经验"],HarnessTool::Create,"经验吸收"),
        CapabilityApiEntry::new("orchestration","NT-CORE","Orchestrator",&["其他","通用"],HarnessTool::Execute,"通用编排兜底"),
    ]
}

/// 统一执行请求/响应（Tauri invoke 单入口）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessExecuteRequest {
    pub instruction: String,
    pub capability_tag: Option<String>,
    pub project: Option<String>,
    pub permission_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessExecuteResponse {
    pub instruction: String,
    pub capability_tag: String,
    pub domain: String,
    pub specialist: String,
    pub harness_tool: HarnessTool,
    pub allocations: Vec<String>,
    pub internal_count: usize,
    pub external_gap_count: usize,
    pub strengthening_actions: usize,
    pub external_gaps: Vec<String>,
    pub message: String,
    /// 可验证回放收据 — agent 运行完成边界 (execute_real) 成功时产出,
    /// 绑定 instruction(输入)+message(输出), 供事后审计回放校验。
    /// 镜像 `AgentReceipt` 的字段 (见 `AgentRunReceipt`), 可由其重建并 `verify()`。
    pub receipt: Option<AgentRunReceipt>,
}

/// 统一网关（L1 Body 轻封装，不建平行能力）
pub struct HarnessGateway {
    api_map: Vec<CapabilityApiEntry>,
    pub router: InferenceRouter,
    pub sandbox: LocalSandbox,
    pub threads: ThreadStore,
}

impl Default for HarnessGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl HarnessGateway {
    pub fn new() -> Self {
        Self {
            api_map: capability_api_map(),
            router: InferenceRouter::default(),
            sandbox: LocalSandbox::default(),
            threads: ThreadStore::default(),
        }
    }

    pub fn api_map(&self) -> &[CapabilityApiEntry] {
        &self.api_map
    }

    pub fn lookup_by_tag(&self, tag: &str) -> Option<&CapabilityApiEntry> {
        self.api_map.iter().find(|e| e.capability_tag == tag)
    }

    pub fn lookup_by_keyword(&self, kw: &str) -> Option<&CapabilityApiEntry> {
        let lower = kw.to_lowercase();
        self.api_map.iter().find(|e| e.keywords.iter().any(|k| lower.contains(&k.to_lowercase())))
    }

    /// 关键词→API 入口（对话即路由，复用 CAPABILITY_ROUTES 逻辑）
    pub fn resolve_instruction(&self, instruction: &str) -> Option<CapabilityApiEntry> {
        let lower = instruction.to_lowercase();
        for entry in &self.api_map {
            for kw in &entry.keywords {
                if lower.contains(&kw.to_lowercase()) {
                    return Some(entry.clone());
                }
            }
        }
        // 兜底 orchestration
        self.lookup_by_tag("orchestration").cloned()
    }

    /// 执行（轻量路由，仅计算展示用 tag；真实计数见 `execute_real`）
    pub fn execute(&self, req: HarnessExecuteRequest) -> HarnessExecuteResponse {
        let entry = req
            .capability_tag
            .as_deref()
            .and_then(|t| self.lookup_by_tag(t))
            .cloned()
            .or_else(|| self.resolve_instruction(&req.instruction))
            .unwrap_or_else(|| self.lookup_by_tag("orchestration").unwrap().clone());
        HarnessExecuteResponse {
            instruction: req.instruction.clone(),
            capability_tag: entry.capability_tag.clone(),
            domain: entry.domain.clone(),
            specialist: entry.specialist.clone(),
            harness_tool: entry.harness_tool.clone(),
            allocations: vec![format!("{} → {} ({})", entry.capability_tag, entry.domain, entry.specialist)],
            internal_count: 1,
            external_gap_count: 0,
            strengthening_actions: 0,
            external_gaps: vec![],
            message: format!("已路由: {} [{}] via {}", entry.capability_tag, entry.domain, entry.harness_tool.as_str()),
            receipt: None,
        }
    }

    /// 真实执行链路 — 复用意识核心单例 `process_instruction`（拆解→分配→反思补齐→能力网持久化），
    /// 不触网；重路径（真正试错求解）走 `ConsciousnessCoreHandle::execute_task_loop`。
    /// 展示 tag 仍取 api_map 命中，计数/缺口/补齐动作取真实报告，全过程透明。
    pub fn execute_real(&self, req: HarnessExecuteRequest, report: &TaskLoopReport) -> HarnessExecuteResponse {
        let entry = req
            .capability_tag
            .as_deref()
            .and_then(|t| self.lookup_by_tag(t))
            .cloned()
            .or_else(|| self.resolve_instruction(&req.instruction))
            .unwrap_or_else(|| self.lookup_by_tag("orchestration").unwrap().clone());
        let allocations: Vec<String> = report
            .allocations
            .iter()
            .map(|a| format!("{} → {} ({})", a.task.capability_tag, a.task.domain, a.task.specialist))
            .collect();
        let total = report.allocations.len();
        let message = format!(
            "已路由: {} [{}] via {}; 拆解 {} 子任务, 内置 {}, 外部缺口 {}, 反思补齐 {}",
            entry.capability_tag,
            entry.domain,
            entry.harness_tool.as_str(),
            total,
            report.internal_count,
            report.external_gap_count,
            report.strengthening_actions
        );
        // 可验证回放收据: agent 运行完成边界, 绑定 instruction(输入)+message(输出)。
        // run_id 用 instruction 的哈希, 保证同一指令可审计回放。
        let run_id = crate::l3_embodiment::nt_shield::shield_core::receipt::hash_content(&req.instruction);
        let receipt = AgentReceipt::emit(&run_id, &req.instruction, &message);
        // 镜像为可序列化结构 (与 HarnessExecuteResponse 同 serde 实例), 字段 1:1 对应。
        let receipt = AgentRunReceipt {
            run_id: receipt.run_id,
            input_hash: receipt.input_hash,
            output_hash: receipt.output_hash,
            timestamp: receipt.timestamp,
            signature: receipt.signature,
        };
        HarnessExecuteResponse {
            instruction: req.instruction.clone(),
            capability_tag: entry.capability_tag.clone(),
            domain: entry.domain.clone(),
            specialist: entry.specialist.clone(),
            harness_tool: entry.harness_tool.clone(),
            allocations,
            internal_count: report.internal_count,
            external_gap_count: report.external_gap_count,
            strengthening_actions: report.strengthening_actions,
            external_gaps: report.external_gaps.clone(),
            message,
            receipt: Some(receipt),
        }
    }

    pub fn tool_catalog(&self) -> Vec<HashMap<String, String>> {
        HarnessTool::all()
            .iter()
            .map(|t| {
                let mut m = HashMap::new();
                m.insert("name".to_string(), t.as_str().to_string());
                m.insert("description".to_string(), format!("Harness tool {}", t.as_str()));
                m
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_map_covers_all_tags() {
        let m = capability_api_map();
        assert!(m.len() >= 20);
        assert!(m.iter().any(|e| e.capability_tag == "xlsx_consolidation"));
        assert!(m.iter().any(|e| e.capability_tag == "orchestration"));
    }

    #[test]
    fn gateway_resolve() {
        let g = HarnessGateway::new();
        let e = g.resolve_instruction("帮我合并价格表").unwrap();
        assert_eq!(e.capability_tag, "xlsx_consolidation");
        let e2 = g.resolve_instruction("检索一下知识库").unwrap();
        assert_eq!(e2.capability_tag, "hybrid_retrieval");
        let e3 = g.resolve_instruction("完全无关的闲聊").unwrap();
        assert_eq!(e3.capability_tag, "orchestration");
    }

    #[test]
    fn gateway_execute() {
        let g = HarnessGateway::new();
        let r = g.execute(HarnessExecuteRequest { instruction: "审查这段代码".into(), capability_tag: None, project: None, permission_mode: None });
        assert_eq!(r.domain, "NT-SHIELD");
        assert_eq!(r.harness_tool, HarnessTool::Execute);
    }

    #[test]
    fn tool_catalog_complete() {
        let g = HarnessGateway::new();
        assert_eq!(g.tool_catalog().len(), 11);
    }

    #[test]
    fn harness_tool_all_and_str_roundtrip() {
        let all = HarnessTool::all();
        assert_eq!(all.len(), 11);
        // 每个 tool 的 as_str 唯一且非空
        let mut seen = std::collections::HashSet::new();
        for t in all {
            let s = t.as_str();
            assert!(!s.is_empty());
            assert!(seen.insert(s));
        }
        // 已知映射
        assert_eq!(HarnessTool::Execute.as_str(), "harness_execute");
        assert_eq!(HarnessTool::HqlRun.as_str(), "hql:run");
        assert_eq!(HarnessTool::List.as_str(), "harness_list");
    }

    #[test]
    fn capability_api_entry_new_basic() {
        let e = CapabilityApiEntry::new(
            "demo_tag",
            "NT-ACT",
            "CodeAnalyzer",
            &["关键词a", "kw_b"],
            HarnessTool::Execute,
            "演示能力",
        );
        assert_eq!(e.capability_tag, "demo_tag");
        assert_eq!(e.domain, "NT-ACT");
        assert_eq!(e.specialist, "CodeAnalyzer");
        assert_eq!(e.harness_tool, HarnessTool::Execute);
        assert_eq!(e.keywords, vec!["关键词a".to_string(), "kw_b".to_string()]);
        // 全量地图里的 entry 都带非空描述与合法 tool
        for entry in capability_api_map() {
            assert!(!entry.description.is_empty());
            assert!(HarnessTool::all().contains(&entry.harness_tool));
        }
    }

    #[test]
    fn lookup_by_tag_exact_and_keyword_case_insensitive() {
        let g = HarnessGateway::new();
        // 精确 tag
        let e = g.lookup_by_tag("hybrid_retrieval").expect("tag exists");
        assert_eq!(e.domain, "NT-MEMORY");
        assert!(g.lookup_by_tag("no_such_tag").is_none());
        // 关键词命中（大小写不敏感）
        let hit = g.lookup_by_keyword("EXCEL").expect("keyword matches");
        assert_eq!(hit.capability_tag, "xlsx_consolidation");
        // 多关键词之一命中
        let hit2 = g.lookup_by_keyword("重构一下").expect("keyword matches");
        assert_eq!(hit2.capability_tag, "code_refactor");
    }

    #[test]
    fn resolve_instruction_is_case_insensitive() {
        let g = HarnessGateway::new();
        let upper = g.resolve_instruction("合并 EXCEL 价格表").unwrap();
        let lower = g.resolve_instruction("合并 excel 价格表").unwrap();
        assert_eq!(upper.capability_tag, lower.capability_tag);
        assert_eq!(upper.capability_tag, "xlsx_consolidation");
    }

    #[test]
    fn execute_real_maps_report_fields() {
        use crate::core::nt_core_consciousness_core::{
            AllocationProvider, ConsciousTask, TaskAllocation, TaskLoopReport,
        };
        let g = HarnessGateway::new();
        let task = ConsciousTask {
            id: "task_1".into(),
            summary: "合并表格".into(),
            capability_tag: "xlsx_consolidation".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let report = TaskLoopReport {
            instruction: "合并价格表".into(),
            allocations: vec![TaskAllocation {
                task,
                provider: AllocationProvider::Internal { node_id: "n1".into(), path: vec![], cost: 0.0 },
            }],
            internal_count: 3,
            external_gap_count: 2,
            strengthening_actions: 1,
            external_gaps: vec!["gap_a".into()],
            external_closures: vec![],
            internal_results: vec![],
        };
        let req = HarnessExecuteRequest {
            instruction: "合并价格表".into(),
            capability_tag: None,
            project: None,
            permission_mode: None,
        };
        let resp = g.execute_real(req.clone(), &report);
        assert_eq!(resp.capability_tag, "xlsx_consolidation");
        assert_eq!(resp.allocations.len(), 1);
        assert_eq!(resp.internal_count, 3);
        assert_eq!(resp.external_gap_count, 2);
        assert_eq!(resp.strengthening_actions, 1);
        assert_eq!(resp.external_gaps, vec!["gap_a".to_string()]);
        assert!(resp.message.contains("拆解 1 子任务"));
        // 无 allocation 时 allocations 为空但字段仍正确映射
        let empty = TaskLoopReport::default();
        let resp2 = g.execute_real(req.clone(), &empty);
        assert!(resp2.allocations.is_empty());
        assert_eq!(resp2.internal_count, 0);
        assert_eq!(resp2.external_gap_count, 0);
    }

    #[test]
    fn execute_real_emits_verifiable_receipt() {
        // agent 运行完成边界: execute_real 成功时应产出可验证回放收据。
        let g = HarnessGateway::new();
        let req = HarnessExecuteRequest {
            instruction: "审查这段代码".into(),
            capability_tag: None,
            project: None,
            permission_mode: None,
        };
        let report = TaskLoopReport::default();
        let resp = g.execute_real(req.clone(), &report);
        let r = resp.receipt.expect("execute_real 应产出收据");
        // 镜像字段重建为 AgentReceipt 后可 verify()
        let receipt = AgentReceipt {
            run_id: r.run_id,
            input_hash: r.input_hash,
            output_hash: r.output_hash,
            timestamp: r.timestamp,
            signature: r.signature,
        };
        assert!(receipt.verify(), "agent 运行收据应能通过签名完整性校验");
        // 输入应绑定 instruction, 输出应绑定 message
        assert!(receipt.input_hash.len() == 64, "input 为 instruction 的 SHA-256");
        assert_eq!(receipt.output_hash, crate::l3_embodiment::nt_shield::shield_core::receipt::hash_content(&resp.message));
    }
}
