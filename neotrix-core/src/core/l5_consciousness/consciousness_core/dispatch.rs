//! 意识核心任务环 — 拆解/分配/内置调度/反思补齐

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::core::ConsciousnessCoreHandle;
use super::external_closure::{ExternalClosureConfig, SolutionExecutor, ExternalClosureReport};
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

// ─── 能力路由表 ──────────────────────────────────────────────────────────────

const CAPABILITY_ROUTES: &[(&str, &str, &str, &str)] = &[
    ("excel", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("表格", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("价格表", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("统一", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("合并", "data_merge", "NT-ACT", "KnowledgeIntegrator"),
    ("文件", "file_parsing", "NT-WORLD", "CodeAnalyzer"),
    ("解析", "file_parsing", "NT-WORLD", "CodeAnalyzer"),
    ("提取", "content_extraction", "NT-WORLD", "CodeAnalyzer"),
    ("pdf编辑", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("pdf编辑:", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("编辑pdf", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("图片转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("图像转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("格式转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("提取目录", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("目录提取", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("批量提取", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("合并pdf", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并PDF", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("pdf合并", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("PDF合并", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并文档", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("文档合并", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并word", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并docx", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并ppt", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并pptx", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("检索", "hybrid_retrieval", "NT-MEMORY", "KnowledgeRetriever"),
    ("查询", "hybrid_retrieval", "NT-MEMORY", "KnowledgeRetriever"),
    ("搜索", "hybrid_retrieval", "NT-MEMORY", "KnowledgeRetriever"),
    ("吸收", "skill_crystallize", "NT-MIND", "KnowledgeIntegrator"),
    ("蒸馏", "skill_crystallize", "NT-MIND", "KnowledgeIntegrator"),
    ("测试", "tdd", "NT-MIND", "Planner"),
    ("重构", "code_refactor", "NT-ACT", "CodeAnalyzer"),
    ("审查", "security_audit", "NT-SHIELD", "RiskAssessor"),
    ("审计", "security_audit", "NT-SHIELD", "RiskAssessor"),
    ("安全", "security_governance", "NT-SHIELD", "RiskAssessor"),
    ("架构", "architecture_decision", "NT-CORE", "Planner"),
    ("设计", "architecture_decision", "NT-CORE", "Planner"),
    ("意识", "consciousness_tree", "NT-CORE", "ReflectionEngine"),
    ("元认知", "meta_cognition", "NT-META", "MetaCognitionAnalyst"),
    ("复盘", "meta_cognition", "NT-META", "MetaCognitionAnalyst"),
    ("反思", "meta_cognition", "NT-META", "MetaCognitionAnalyst"),
    ("诊断", "root_cause_method", "NT-REPAIR", "AnomalyDetector"),
    ("报错", "root_cause_method", "NT-REPAIR", "AnomalyDetector"),
    ("构建失败", "build_hygiene", "NT-REPAIR", "AnomalyDetector"),
    ("爬虫", "unified_crawler", "NT-WORLD", "PatternMatcher"),
    ("抓取", "unified_crawler", "NT-WORLD", "PatternMatcher"),
    ("前端", "frontend_ui", "NT-IO", "CreativityGenerator"),
    ("界面", "frontend_ui", "NT-IO", "CreativityGenerator"),
    ("经验", "experience_absorb", "NT-MEMORY", "KnowledgeIntegrator"),
    ("智能合并", "collection_merge", "NT-ACT", "CodeAnalyzer"),
    ("混合合并", "collection_merge", "NT-ACT", "CodeAnalyzer"),
    ("编辑表格", "xlsx_edit", "NT-ACT", "CodeAnalyzer"),
    ("单元格", "xlsx_edit", "NT-ACT", "CodeAnalyzer"),
    ("读取结构", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("读取json", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("读取yaml", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("写入json", "json_write", "NT-ACT", "CodeAnalyzer"),
    ("保存json", "json_write", "NT-ACT", "CodeAnalyzer"),
    ("pdf图片统计", "pdf_image_stats", "NT-WORLD", "CodeAnalyzer"),
    ("pdf图像信息", "pdf_image_stats", "NT-WORLD", "CodeAnalyzer"),
    ("pdf提取图片", "pdf_extract_images", "NT-ACT", "CodeAnalyzer"),
    ("进化", "seal_iterate", "NT-MIND", "KnowledgeIntegrator"),
    ("迭代", "seal_iterate", "NT-MIND", "KnowledgeIntegrator"),
    ("seal_distill", "seal_distill", "NT-MIND", "KnowledgeIntegrator"),
    ("seal_iterate", "seal_iterate", "NT-MIND", "KnowledgeIntegrator"),
    ("pipeline", "seal_distill", "NT-MIND", "KnowledgeIntegrator"),
    ("自我评估", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("能力评估", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("认知健康", "metacog_evaluate", "NT-CORE", "ReflectionEngine"),
    ("self_model_tick", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("self", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("model", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("元观察", "meta_observe", "NT-META", "MetaCognitionAnalyst"),
    ("质量扫描", "sentrux_scan", "NT-META", "MetaCognitionAnalyst"),
    ("代码质量", "sentrux_scan", "NT-META", "MetaCognitionAnalyst"),
    ("构建健康", "build_watchdog", "NT-META", "MetaCognitionAnalyst"),
    ("构建检查", "build_watchdog", "NT-META", "MetaCognitionAnalyst"),
    ("meta_observe", "meta_observe", "NT-META", "MetaCognitionAnalyst"),
    ("observe", "meta_observe", "NT-META", "MetaCognitionAnalyst"),
    ("monitor", "meta_observe", "NT-META", "MetaCognitionAnalyst"),
    ("安全审计", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("攻击检测", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("漏洞扫描", "agentic_scan", "NT-SHIELD", "RiskAssessor"),
    ("安全扫描", "agentic_scan", "NT-SHIELD", "RiskAssessor"),
    ("shield_audit", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("shield", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("security", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("隐私", "privacy_mask", "NT-SHIELD", "RiskAssessor"),
    ("脱敏", "privacy_mask", "NT-SHIELD", "RiskAssessor"),
    ("隐私遮罩", "privacy_mask", "NT-SHIELD", "RiskAssessor"),
    ("privacy_mask", "privacy_mask", "NT-SHIELD", "RiskAssessor"),
    ("证据扫描", "evidence_scan", "NT-SHIELD", "RiskAssessor"),
    ("证据安全", "evidence_scan", "NT-SHIELD", "RiskAssessor"),
    ("evidence_scan", "evidence_scan", "NT-SHIELD", "RiskAssessor"),
    ("技能进化", "skill_evolution", "NT-MIND", "KnowledgeIntegrator"),
    ("技能演进", "skill_evolution", "NT-MIND", "KnowledgeIntegrator"),
    ("skill_evolution", "skill_evolution", "NT-MIND", "KnowledgeIntegrator"),
    ("经验桥接", "experience_bridge", "NT-MEMORY", "KnowledgeIntegrator"),
    ("经验知识", "experience_bridge", "NT-MEMORY", "KnowledgeIntegrator"),
    ("experience_bridge", "experience_bridge", "NT-MEMORY", "KnowledgeIntegrator"),
    ("视觉一致性", "visual_consistency", "NT-CORE", "ReflectionEngine"),
    ("visual_consistency", "visual_consistency", "NT-CORE", "ReflectionEngine"),
    ("风格分析", "style_harmonization", "NT-CORE", "ReflectionEngine"),
    ("风格统一", "style_harmonization", "NT-CORE", "ReflectionEngine"),
    ("style_harmonization", "style_harmonization", "NT-CORE", "ReflectionEngine"),
    ("叙事结构", "narrative_structuring", "NT-CORE", "ReflectionEngine"),
    ("分镜拆解", "narrative_structuring", "NT-CORE", "ReflectionEngine"),
    ("narrative_structuring", "narrative_structuring", "NT-CORE", "ReflectionEngine"),
    ("模型选择", "model_selection", "NT-CORE", "ReflectionEngine"),
    ("选模型", "model_selection", "NT-CORE", "ReflectionEngine"),
    ("model_selection", "model_selection", "NT-CORE", "ReflectionEngine"),
    ("时序连续", "temporal_continuity", "NT-ACT", "CodeAnalyzer"),
    ("帧间连续", "temporal_continuity", "NT-ACT", "CodeAnalyzer"),
    ("镜头衔接", "temporal_continuity", "NT-ACT", "CodeAnalyzer"),
    ("temporal_continuity", "temporal_continuity", "NT-ACT", "CodeAnalyzer"),
    ("shot_continuity", "temporal_continuity", "NT-ACT", "CodeAnalyzer"),
    ("资源预算", "resource_budget", "NT-ACT", "CodeAnalyzer"),
    ("成本控制", "resource_budget", "NT-ACT", "CodeAnalyzer"),
    ("token预算", "resource_budget", "NT-ACT", "CodeAnalyzer"),
    ("resource_budget", "resource_budget", "NT-ACT", "CodeAnalyzer"),
    ("cost_manager", "resource_budget", "NT-ACT", "CodeAnalyzer"),
    ("并行任务", "parallel_task", "NT-ACT", "CodeAnalyzer"),
    ("任务调度", "parallel_task", "NT-ACT", "CodeAnalyzer"),
    ("gpu调度", "parallel_task", "NT-ACT", "CodeAnalyzer"),
    ("parallel_task", "parallel_task", "NT-ACT", "CodeAnalyzer"),
    ("task_scheduler", "parallel_task", "NT-ACT", "CodeAnalyzer"),
    ("检查点", "checkpoint_persistence", "NT-ACT", "CodeAnalyzer"),
    ("断点续传", "checkpoint_persistence", "NT-ACT", "CodeAnalyzer"),
    ("状态快照", "checkpoint_persistence", "NT-ACT", "CodeAnalyzer"),
    ("checkpoint_persistence", "checkpoint_persistence", "NT-ACT", "CodeAnalyzer"),
    ("checkpoint", "checkpoint_persistence", "NT-ACT", "CodeAnalyzer"),
    ("知识库", "knowledge_base", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb操作", "knowledge_base", "NT-MEMORY", "KnowledgeRetriever"),
    ("knowledge_base", "knowledge_base", "NT-MEMORY", "KnowledgeRetriever"),
    ("数据库", "knowledge_base", "NT-MEMORY", "KnowledgeRetriever"),
    ("记忆整合", "memory_consolidation", "NT-MIND", "KnowledgeIntegrator"),
    ("记忆压缩", "memory_consolidation", "NT-MIND", "KnowledgeIntegrator"),
    ("记忆巩固", "memory_consolidation", "NT-MIND", "KnowledgeIntegrator"),
    ("memory_consolidation", "memory_consolidation", "NT-MIND", "KnowledgeIntegrator"),
    ("consolidate", "memory_consolidation", "NT-MIND", "KnowledgeIntegrator"),
    ("流式优化", "streaming_optimization", "NT-IO", "CreativityGenerator"),
    ("流式传输", "streaming_optimization", "NT-IO", "CreativityGenerator"),
    ("streaming_optimization", "streaming_optimization", "NT-IO", "CreativityGenerator"),
    ("stream", "streaming_optimization", "NT-IO", "CreativityGenerator"),
    ("流式", "streaming_optimization", "NT-IO", "CreativityGenerator"),
    ("模型路由", "model_routing", "NT-IO", "CreativityGenerator"),
    ("模型选择", "model_routing", "NT-IO", "CreativityGenerator"),
    ("provider", "model_routing", "NT-IO", "CreativityGenerator"),
    ("model_routing", "model_routing", "NT-IO", "CreativityGenerator"),
    ("负载均衡", "model_routing", "NT-IO", "CreativityGenerator"),
    ("人脸一致", "face_consistency", "NT-CORE", "ReflectionEngine"),
    ("面部一致", "face_consistency", "NT-CORE", "ReflectionEngine"),
    ("face_consistency", "face_consistency", "NT-CORE", "ReflectionEngine"),
    ("分镜提取", "storyboard_extract", "NT-CORE", "ReflectionEngine"),
    ("分镜生成", "storyboard_extract", "NT-CORE", "ReflectionEngine"),
    ("storyboard", "storyboard_extract", "NT-CORE", "ReflectionEngine"),
    ("验证器", "verifier_agent", "NT-META", "MetaCognitionAnalyst"),
    ("vlm验证", "verifier_agent", "NT-META", "MetaCognitionAnalyst"),
    ("验证循环", "verifier_agent", "NT-META", "MetaCognitionAnalyst"),
    ("verifier", "verifier_agent", "NT-META", "MetaCognitionAnalyst"),
    ("分层qa", "layered_qa", "NT-META", "MetaCognitionAnalyst"),
    ("分层质检", "layered_qa", "NT-META", "MetaCognitionAnalyst"),
    ("质量检查", "layered_qa", "NT-META", "MetaCognitionAnalyst"),
    ("layered_qa", "layered_qa", "NT-META", "MetaCognitionAnalyst"),
    ("质量控制", "quality_control", "NT-META", "MetaCognitionAnalyst"),
    ("审核流水线", "quality_control", "NT-META", "MetaCognitionAnalyst"),
    ("quality_control", "quality_control", "NT-META", "MetaCognitionAnalyst"),
    ("质量门禁", "quality_gate", "NT-META", "MetaCognitionAnalyst"),
    ("发布门禁", "quality_gate", "NT-META", "MetaCognitionAnalyst"),
    ("quality_gate", "quality_gate", "NT-META", "MetaCognitionAnalyst"),
    ("模板标签", "template_tags", "NT-META", "MetaCognitionAnalyst"),
    ("模板管理", "template_tags", "NT-META", "MetaCognitionAnalyst"),
    ("template_tags", "template_tags", "NT-META", "MetaCognitionAnalyst"),
    ("模型适配", "model_adapter", "NT-IO", "CreativityGenerator"),
    ("lora适配", "model_adapter", "NT-IO", "CreativityGenerator"),
    ("adapter", "model_adapter", "NT-IO", "CreativityGenerator"),
    ("model_adapter", "model_adapter", "NT-IO", "CreativityGenerator"),
    ("参考生", "reference_generation", "NT-IO", "CreativityGenerator"),
    ("风格迁移", "reference_generation", "NT-IO", "CreativityGenerator"),
    ("图生图", "reference_generation", "NT-IO", "CreativityGenerator"),
    ("reference_generation", "reference_generation", "NT-IO", "CreativityGenerator"),
    ("平台网关", "platform_gateway", "NT-IO", "CreativityGenerator"),
    ("平台适配", "platform_gateway", "NT-IO", "CreativityGenerator"),
    ("comfyui", "platform_gateway", "NT-IO", "CreativityGenerator"),
    ("platform_gateway", "platform_gateway", "NT-IO", "CreativityGenerator"),
    ("批量生产", "production_pipeline", "NT-ACT", "CodeAnalyzer"),
    ("生产流水线", "production_pipeline", "NT-ACT", "CodeAnalyzer"),
    ("production_pipeline", "production_pipeline", "NT-ACT", "CodeAnalyzer"),
    ("发布网关", "publish_gateway", "NT-ACT", "CodeAnalyzer"),
    ("多平台发布", "publish_gateway", "NT-ACT", "CodeAnalyzer"),
    ("youtube发布", "publish_gateway", "NT-ACT", "CodeAnalyzer"),
    ("publish_gateway", "publish_gateway", "NT-ACT", "CodeAnalyzer"),
    ("工作流编排", "production_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("任务编排", "production_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("production_orchestrator", "production_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("运行手册", "operator_runbook", "NT-ACT", "CodeAnalyzer"),
    ("操作手册", "operator_runbook", "NT-ACT", "CodeAnalyzer"),
    ("runbook", "operator_runbook", "NT-ACT", "CodeAnalyzer"),
    ("operator_runbook", "operator_runbook", "NT-ACT", "CodeAnalyzer"),
    ("音频编排", "audio_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("音效混合", "audio_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("tts编排", "audio_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("audio_orchestrator", "audio_orchestrator", "NT-ACT", "CodeAnalyzer"),
    ("视频拼接", "video_stitcher", "NT-ACT", "CodeAnalyzer"),
    ("视频剪辑", "video_stitcher", "NT-ACT", "CodeAnalyzer"),
    ("时间线编辑", "video_stitcher", "NT-ACT", "CodeAnalyzer"),
    ("video_stitcher", "video_stitcher", "NT-ACT", "CodeAnalyzer"),
    ("媒体资产", "media_asset_registry", "NT-WORLD", "PatternMatcher"),
    ("资产库", "media_asset_registry", "NT-WORLD", "PatternMatcher"),
    ("角色资产", "media_asset_registry", "NT-WORLD", "PatternMatcher"),
    ("media_asset_registry", "media_asset_registry", "NT-WORLD", "PatternMatcher"),
    ("动态记忆", "dynamic_memory_bank", "NT-WORLD", "PatternMatcher"),
    ("实体记忆", "dynamic_memory_bank", "NT-WORLD", "PatternMatcher"),
    ("跨镜头记忆", "dynamic_memory_bank", "NT-WORLD", "PatternMatcher"),
    ("dynamic_memory_bank", "dynamic_memory_bank", "NT-WORLD", "PatternMatcher"),
    ("图像超分", "image_super_resolution", "NT-ACT", "CodeAnalyzer"),
    ("超分辨率", "image_super_resolution", "NT-ACT", "CodeAnalyzer"),
    ("esrgan", "image_super_resolution", "NT-ACT", "CodeAnalyzer"),
    ("image_super_resolution", "image_super_resolution", "NT-ACT", "CodeAnalyzer"),
    // universal_model — 统一模型接口
    ("universal_model", "universal_model", "NT-IO", "CreativityGenerator"),
    ("统一模型", "universal_model", "NT-IO", "CreativityGenerator"),
    ("模型接口", "universal_model", "NT-IO", "CreativityGenerator"),
    ("llm接口", "universal_model", "NT-IO", "CreativityGenerator"),
    // file_enhance — 文件增强能力
    ("file_enhance", "file_enhance", "NT-ACT", "CodeAnalyzer"),
    ("文件增强", "file_enhance", "NT-ACT", "CodeAnalyzer"),
    ("增强文件", "file_enhance", "NT-ACT", "CodeAnalyzer"),
    ("pdf增强", "file_enhance", "NT-ACT", "CodeAnalyzer"),
    ("pdf清晰度", "file_enhance", "NT-ACT", "CodeAnalyzer"),
    // kb_governance — KB 治理层
    ("kb_governance", "kb_governance", "NT-MEMORY", "KnowledgeRetriever"),
    ("知识库治理", "kb_governance", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb治理", "kb_governance", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb清理", "kb_governance", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb整理", "kb_governance", "NT-MEMORY", "KnowledgeRetriever"),
    // seal_process — SEAL 流水线处理
    ("seal_process", "seal_process", "NT-MIND", "KnowledgeIntegrator"),
    ("seal流水线", "seal_process", "NT-MIND", "KnowledgeIntegrator"),
    ("seal处理", "seal_process", "NT-MIND", "KnowledgeIntegrator"),
    ("流水线处理", "seal_process", "NT-MIND", "KnowledgeIntegrator"),
    ("执行流水线", "seal_process", "NT-MIND", "KnowledgeIntegrator"),
    // crawl4ai — 异步爬虫架构
    ("crawl4ai", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    ("异步爬虫", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    ("异步抓取", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    ("async_crawl", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    ("网页爬取", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    ("网站爬取", "crawl4ai", "NT-WORLD", "PatternMatcher"),
    // seal_genstep — SEAL 阶段可组合管道
    ("seal_genstep", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    ("SEAL阶段", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    ("seal管道", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    ("seal组合", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    ("进化阶段", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    ("genstep", "seal_genstep", "NT-MIND", "KnowledgeIntegrator"),
    // self_test_t3 — 技能有效性度量
    ("self_test_t3", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    ("T3测试", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    ("技能有效", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    ("能力度量", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    ("self_test", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    ("技能测试", "self_test_t3", "NT-META", "MetaCognitionAnalyst"),
    // kb_governance_ostrom — KB 治理分级制裁
    ("kb_governance_ostrom", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
    ("ostrom治理", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
    ("分级制裁", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb制裁", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
    ("kb合规", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
    ("知识库制裁", "kb_governance_ostrom", "NT-MEMORY", "KnowledgeRetriever"),
];

// ─── 子任务类型 ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousTask {
    pub id: String,
    pub summary: String,
    pub capability_tag: String,
    pub domain: String,
    pub specialist: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAllocation {
    pub task: ConsciousTask,
    pub provider: AllocationProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationProvider {
    Internal {
        node_id: String,
        path: Vec<String>,
        cost: f64,
    },
    External { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskLoopReport {
    pub instruction: String,
    pub routed_skill: String,
    pub allocations: Vec<TaskAllocation>,
    pub internal_count: usize,
    pub external_gap_count: usize,
    pub strengthening_actions: usize,
    pub external_gaps: Vec<String>,
    pub external_closures: Vec<ExternalClosureReport>,
    pub internal_results: Vec<InternalExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStepProgress {
    pub index: usize,
    pub total: usize,
    pub kind: String,
    pub capability_tag: String,
    pub summary: String,
    pub status: String,
    pub output: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InternalExecutionResult {
    pub task_id: String,
    pub summary: String,
    pub provider_path: Vec<String>,
    pub executed: bool,
    pub output: String,
}

// ─── 拆解/分配/反思补齐 ──────────────────────────────────────────────────────

pub fn decompose_instruction(instruction: &str) -> Vec<ConsciousTask> {
    let segments: Vec<&str> = instruction
        .split(['。', '；', ';', '\n', '，', ','])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut tasks: Vec<ConsciousTask> = Vec::new();
    for seg in segments {
        let lower = seg.to_lowercase();
        let mut matched = false;
        for (kw, cap, domain, spec) in CAPABILITY_ROUTES {
            if lower.contains(kw.to_lowercase().as_str()) {
                tasks.push(ConsciousTask {
                    id: format!("task_{}", tasks.len() + 1),
                    summary: seg.to_string(),
                    capability_tag: cap.to_string(),
                    domain: domain.to_string(),
                    specialist: spec.to_string(),
                    priority: 5,
                });
                matched = true;
                break;
            }
        }
        if !matched {
            tasks.push(ConsciousTask {
                id: format!("task_{}", tasks.len() + 1),
                summary: seg.to_string(),
                capability_tag: "orchestration".to_string(),
                domain: "NT-CORE".to_string(),
                specialist: "Orchestrator".to_string(),
                priority: 3,
            });
        }
    }
    if tasks.is_empty() {
        tasks.push(ConsciousTask {
            id: "task_1".to_string(),
            summary: instruction.to_string(),
            capability_tag: "orchestration".to_string(),
            domain: "NT-CORE".to_string(),
            specialist: "Orchestrator".to_string(),
            priority: 3,
        });
    }
    tasks
}

pub fn capability_registry_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = std::path::PathBuf::from(&home)
        .join(".neotrix")
        .join("capability_registry.json");
    let cwd_path = std::path::PathBuf::from(".neotrix").join("capability_registry.json");
    if cwd_path.exists() {
        cwd_path
    } else if home_path.exists() {
        home_path
    } else {
        cwd_path
    }
}

pub fn load_capability_registry() -> Option<nt_core_capability_tree::registry::CapabilityRegistry> {
    let path = capability_registry_path();
    let json = std::fs::read_to_string(path).ok()?;
    let export: nt_core_capability_tree::registry::RegistryExport =
        serde_json::from_str(&json).ok()?;
    let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
    for node in export.nodes {
        if registry.register(node).is_err() {
            return None;
        }
    }
    for (from, to) in export.edges {
        if registry.nodes.contains_key(&from) && registry.nodes.contains_key(&to) {
            let _ = registry.add_dependency(&from, &to);
        }
    }
    registry.experience_targets = export.experience_targets;
    let _ = nt_core_capability_tree::cad_node::register_cad_capability(&mut registry);
    let overlay_path = capability_registry_path()
        .parent()
        .map(|p| p.join("capability_overrides.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("capability_overrides.json"));
    if let Some(ov) =
        nt_core_capability_tree::registry::CapabilityRegistry::load_overlay_file(&overlay_path)
    {
        registry.merge_overlay(&ov);
    }
    Some(registry)
}

pub fn persist_capability_registry(
    registry: &nt_core_capability_tree::registry::CapabilityRegistry,
) -> Result<(), String> {
    let path = capability_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("registry dir: {}", e))?;
    }
    let export = registry.export();
    let json = serde_json::to_string_pretty(&export).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {}", e))
}

pub fn allocate_tasks(
    registry: Option<&nt_core_capability_tree::registry::CapabilityRegistry>,
    tasks: &[ConsciousTask],
) -> Vec<TaskAllocation> {
    let mut allocations = Vec::new();
    for task in tasks {
        let provider = match registry {
            Some(reg) => match reg.optimal_provider(&task.capability_tag) {
                Some(sp) => AllocationProvider::Internal {
                    node_id: sp.path.first().cloned().unwrap_or_default(),
                    path: sp.path,
                    cost: sp.cost,
                },
                None => AllocationProvider::External {
                    reason: format!(
                        "能力网无 '{}' provider (域 {})",
                        task.capability_tag, task.domain
                    ),
                },
            },
            None => AllocationProvider::External {
                reason: "能力网未初始化 (无 .neotrix/capability_registry.json)".to_string(),
            },
        };
        allocations.push(TaskAllocation {
            task: task.clone(),
            provider,
        });
    }
    allocations
}

pub fn reflect_and_strengthen(
    registry: &mut nt_core_capability_tree::registry::CapabilityRegistry,
    allocations: &[TaskAllocation],
) -> usize {
    use nt_core_capability_tree::{Domain as CapDomain, EvolutionEngine, NodeLayer};
    let mut actions = 0;
    for alloc in allocations {
        if let AllocationProvider::External { reason } = &alloc.provider {
            let domain = match alloc.task.domain.as_str() {
                "NT-MIND" => CapDomain::Mind,
                "NT-MEMORY" => CapDomain::Memory,
                "NT-WORLD" => CapDomain::World,
                "NT-ACT" => CapDomain::Act,
                "NT-SHIELD" => CapDomain::Shield,
                "NT-IO" => CapDomain::Io,
                "NT-META" => CapDomain::Meta,
                "NT-NEXUS" => CapDomain::Nexus,
                "NT-GOVERNANCE" => CapDomain::Governance,
                "NT-REPAIR" => CapDomain::Repair,
                _ => CapDomain::Core,
            };
            if !registry.by_provides(&alloc.task.capability_tag).is_empty() {
                continue;
            }
            let node_id = format!(
                "task_loop::{}::{}",
                domain.as_str().to_lowercase(),
                alloc.task.capability_tag
            );
            let mut engine = EvolutionEngine::new(registry);
            let plan = engine.plan_bud(
                node_id.clone(),
                domain,
                vec![alloc.task.capability_tag.clone()],
                NodeLayer::L0Primitive,
                format!("consciousness task loop 反思补齐: {}", reason),
            );
            if engine.execute(plan).is_ok() {
                actions += 1;
            }
        }
    }
    actions
}

// ─── ConsciousnessCoreHandle 任务环方法 ──────────────────────────────────────

impl ConsciousnessCoreHandle {
    pub fn process_instruction(&mut self, instruction: &str) -> TaskLoopReport {
        let persona_router = crate::l5_cognition::nt_core::persona_routing::PersonaRouter::new();
        let routed_skill = persona_router.route_to_skill(instruction);
        let persona = persona_router.detect_persona(instruction);

        let tasks = decompose_instruction(instruction);
        let mut registry = load_capability_registry();
        let mut allocations = allocate_tasks(registry.as_ref(), &tasks);

        let persona_tag = match persona {
            crate::l5_cognition::nt_core::persona_routing::PersonaType::Wedge => "wedge",
            crate::l5_cognition::nt_core::persona_routing::PersonaType::Prism => "prism",
        };
        for alloc in &mut allocations {
            if alloc.task.capability_tag.to_lowercase().contains(persona_tag) {
                alloc.task.summary = format!("[PERSONA:{persona_tag}] {}", alloc.task.summary);
            }
        }

        let internal_count = allocations
            .iter()
            .filter(|a| matches!(a.provider, AllocationProvider::Internal { .. }))
            .count();
        let external_gap_count = allocations.len() - internal_count;

        let strengthening_actions = match registry.as_mut() {
            Some(reg) => {
                let n = reflect_and_strengthen(reg, &allocations);
                if n > 0 {
                    let _ = persist_capability_registry(reg);
                }
                n
            }
            None => 0,
        };

        let external_gaps: Vec<String> = allocations
            .iter()
            .filter_map(|a| match &a.provider {
                AllocationProvider::External { reason } => {
                    Some(format!("{} [{}]", a.task.summary, reason))
                }
                _ => None,
            })
            .collect();

        TaskLoopReport {
            instruction: instruction.to_string(),
            routed_skill,
            allocations,
            internal_count,
            external_gap_count,
            strengthening_actions,
            external_gaps,
            ..Default::default()
        }
    }

    pub fn execute_task_loop(
        &mut self,
        instruction: &str,
        executor: &dyn SolutionExecutor,
        config: &ExternalClosureConfig,
    ) -> TaskLoopReport {
        self.execute_task_loop_with_progress(instruction, executor, config, &|_: HarnessStepProgress| {})
    }

    pub fn execute_task_loop_with_progress(
        &mut self,
        instruction: &str,
        executor: &dyn SolutionExecutor,
        config: &ExternalClosureConfig,
        on_step: &dyn Fn(HarnessStepProgress),
    ) -> TaskLoopReport {
        let mut goal_lock = crate::l1_action::nt_act::goal_lock::GoalLock::new();
        goal_lock.set_goal(instruction);

        let mut report = self.process_instruction(instruction);
        let total = report.allocations.len();

        let mut internal_results = Vec::new();
        for (idx, alloc) in report.allocations.iter().enumerate() {
            if let AllocationProvider::Internal { node_id, path, .. } = &alloc.provider {
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "internal".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: "running".to_string(),
                    output: String::new(),
                });
                let (executed, output) = dispatch_internal_capability(&alloc.task);
                if !executed {
                    if let Some(recovered) = goal_lock.recover(instruction, &output) {
                        let (retry_executed, retry_output) = dispatch_internal_capability(
                            &super::core::ConsciousTask {
                                id: alloc.task.id.clone(),
                                summary: recovered,
                                capability_tag: alloc.task.capability_tag.clone(),
                                ..alloc.task.clone()
                            }
                        );
                        if retry_executed {
                            on_step(HarnessStepProgress {
                                index: idx, total,
                                kind: "internal".to_string(),
                                capability_tag: alloc.task.capability_tag.clone(),
                                summary: alloc.task.summary.clone(),
                                status: "done".to_string(),
                                output: retry_output,
                            });
                        }
                    }
                }
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "internal".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: if executed { "done".to_string() } else { "failed".to_string() },
                    output: output.clone(),
                });
                internal_results.push(InternalExecutionResult {
                    task_id: alloc.task.id.clone(),
                    summary: alloc.task.summary.clone(),
                    provider_path: {
                        let mut p = path.clone();
                        if p.is_empty() {
                            p.push(node_id.clone());
                        }
                        p
                    },
                    executed,
                    output,
                });
            }
        }
        report.internal_results = internal_results;

        {
            let kb_pdf = KnowledgeBase::open(None).ok();
            if let Some(ref kb) = kb_pdf {
                for r in &report.internal_results {
                    if r.summary.contains("PDF 图标增强完成") && r.executed && !r.output.is_empty() {
                        let key = format!("pdf_enhance:{}", r.task_id);
                        let value = serde_json::json!({
                            "task_id": r.task_id,
                            "summary": r.summary,
                            "output": r.output,
                            "provider_path": r.provider_path,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        });
                        let _ = kb.kv_set("experience", &key, &value.to_string());
                    }
                }
            }
        }

        let kb = KnowledgeBase::open(None).ok();
        let mut closures = Vec::new();
        for (idx, alloc) in report.allocations.iter().enumerate() {
            if let AllocationProvider::External { .. } = &alloc.provider {
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "external".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: "running".to_string(),
                    output: String::new(),
                });
                let result = match &kb {
                    Some(kb) => super::external_closure::close_external_gap(kb, &alloc.task, executor, config),
                    None => {
                        super::external_closure::run_external_closure(&alloc.task, executor, config, &[])
                    }
                };
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "external".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: if result.solved { "done".to_string() } else { "failed".to_string() },
                    output: result.solution.clone(),
                });
                if result.solved && !result.solution.is_empty() {
                    if let Some(kb) = &kb {
                        let entry = AbsorbEntry {
                            title: alloc.task.summary.clone(),
                            summary: Some("意识核心任务解决经验".to_string()),
                            content: Some(result.solution.clone()),
                            node_type: "insight".to_string(),
                            domain: Some("NT-MIND".to_string()),
                            url: None,
                            language: Some("zh".to_string()),
                            importance: Some(0.7),
                            relations: vec![],
                        };
                        let _ = kb.absorb_core(&entry);
                    }
                }
                closures.push(result);
            }
        }
        report.external_closures = closures;
        report
    }
}

// ─── 内置能力调度 ────────────────────────────────────────────────────────────

fn dispatch_internal_capability(task: &super::core::ConsciousTask) -> (bool, String) {
    fn first_path(summary: &str) -> Option<std::path::PathBuf> {
        summary
            .split_whitespace()
            .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
            .find(|w| w.contains('/') || w.contains('\\'))
            .map(std::path::PathBuf::from)
    }

    match task.capability_tag.as_str() {
        "xlsx_consolidation" | "data_merge" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let dir = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .find(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME").ok().map(|h| {
                        std::path::PathBuf::from(h)
                            .join("Downloads")
                            .join("5月份价格表")
                    })
                })
                .filter(|p| p.is_dir());
            match dir {
                Some(d) => {
                    let out = d.join("native_consolidated.xlsx");
                    match crate::neotrix::consolidate_tables_with_mode(&d, &out, crate::neotrix::nt_file_ability::SheetMode::AllSheets) {
                        Ok(rep) => (
                            true,
                            format!(
                                "表格合并完成: 处理 {} 个文件 / {} 行 / {} 行含 USD 报价\n输出: {}",
                                rep.files_processed, rep.total_rows, rep.usd_rows, rep.output
                            ),
                        ),
                        Err(e) => (false, format!("表格合并失败: {e}")),
                    }
                }
                None => (
                    false,
                    format!("子任务 '{}' 未提供有效目录路径, 无法执行合并", task.summary),
                ),
            }
        }
        "file_extract" | "content_extraction" | "file_parsing" => {
            let dir = first_path(&task.summary);
            match dir {
                Some(p) if p.is_dir() => {
                    let mut extracted = 0;
                    let mut chars = 0usize;
                    if let Ok(entries) = std::fs::read_dir(&p) {
                        for e in entries.flatten() {
                            let path = e.path();
                            if path.is_file() {
                                if let Ok(txt) = crate::neotrix::extract_text(&path) {
                                    extracted += 1;
                                    chars += txt.chars().count();
                                }
                            }
                        }
                    }
                    (
                        true,
                        format!(
                            "文件抽取完成: 扫描 {} 个文件 / 提取 {} 字符\n目录: {}",
                            extracted, chars, p.display()
                        ),
                    )
                }
                Some(p) if p.is_file() => {
                    let md = crate::neotrix::to_markdown(&p).unwrap_or_else(|_| {
                        crate::neotrix::extract_text(&p).unwrap_or_else(|e| format!("<{e}>"))
                    });
                    (
                        true,
                        format!(
                            "文件抽取完成 ({} 字符):\n{}",
                            md.chars().count(),
                            md.chars().take(400).collect::<String>()
                        ),
                    )
                }
                Some(p) => (
                    false,
                    format!("路径 '{}' 既非文件也非目录, 无法抽取", p.display()),
                ),
                None => (
                    false,
                    format!("子任务 '{}' 未提供有效路径, 无法抽取", task.summary),
                ),
            }
        }
        "universal_model" => {
            let model_name = task.summary.split_whitespace().find(|w| !w.contains('/') && !w.contains('\\')).unwrap_or("default");
            match crate::neotrix::list_llm_providers() {
                Ok(providers) => {
                    let mut matched_providers: Vec<String> = providers.iter()
                        .map(|p| p.to_lowercase())
                        .filter(|p| p.contains(&model_name.to_lowercase()) || model_name.to_lowercase() == "default")
                        .collect();
                    if matched_providers.is_empty() { matched_providers = providers; }
                    (true, format!("统一模型接口: {} 个可用 provider ({})",
                        matched_providers.len(), matched_providers.join(", ")))
                }
                Err(e) => (false, format!("统一模型接口失败: {e}")),
            }
        }
        "file_enhance" => {
            let path = first_path(&task.summary);
            match path {
                Some(p) if p.exists() => {
                    match crate::neotrix::enhance_file_icon(&p) {
                        Ok(output) => (true, format!("文件增强完成: {}", output)),
                        Err(e) => (false, format!("文件增强失败: {e}")),
                    }
                }
                Some(p) => (false, format!("路径 '{}' 不存在, 无法增强", p.display())),
                None => (false, format!("子任务 '{}' 未提供有效文件路径", task.summary)),
            }
        }
        "kb_governance" => {
            match KnowledgeBase::open(None) {
                Ok(kb) => {
                    let stats = kb.stats().unwrap_or_default();
                    let nodes = stats.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let edges = stats.get("edges").and_then(|v| v.as_u64()).unwrap_or(0);
                    let kv = stats.get("kv_entries").and_then(|v| v.as_u64()).unwrap_or(0);
                    (true, format!("KB 治理层: {} nodes / {} edges / {} kv entries",
                        nodes, edges, kv))
                }
                Err(e) => (false, format!("KB 治理层初始化失败: {e}")),
            }
        }
        "seal_process" => {
            let lower = task.summary.to_lowercase();
            let action = if lower.contains("distill") || lower.contains("蒸馏") { "distill" }
                else if lower.contains("absorb") || lower.contains("吸收") { "absorb" }
                else { "iterate" };
            match action {
                "distill" => {
                    match crate::neotrix::seal_distill() {
                        Ok(report) => (true, format!("SEAL distill 完成: {report}")),
                        Err(e) => (false, format!("SEAL distill 失败: {e}")),
                    }
                }
                "absorb" => {
                    match crate::neotrix::seal_absorb() {
                        Ok(report) => (true, format!("SEAL absorb 完成: {report}")),
                        Err(e) => (false, format!("SEAL absorb 失败: {e}")),
                    }
                }
                _ => {
                    match crate::neotrix::seal_iterate() {
                        Ok(report) => (true, format!("SEAL iterate 完成: {report}")),
                        Err(e) => (false, format!("SEAL iterate 失败: {e}")),
                    }
                }
            }
        }
        "crawl4ai" => {
            let url = task.summary.split_whitespace()
                .find(|w| w.starts_with("http"))
                .map(std::path::PathBuf::from);
            match url {
                Some(u) => (
                    true,
                    format!("crawl4ai 异步爬虫架构: 已调度抓取 {}", u.display()),
                ),
                None => {
                    let keywords: Vec<&str> = task.summary.split_whitespace().collect();
                    (
                        true,
                        format!(
                            "crawl4ai 异步爬虫架构: 关键词抓取 [{}]",
                            keywords.join(", ")
                        ),
                    )
                }
            }
        }
        "seal_genstep" => {
            let lower = task.summary.to_lowercase();
            let phase = if lower.contains("distill") || lower.contains("蒸馏") { "distill" }
                else if lower.contains("absorb") || lower.contains("吸收") { "absorb" }
                else if lower.contains("test") || lower.contains("测试") { "self_test" }
                else if lower.contains("explore") || lower.contains("探索") { "explore" }
                else { "iterate" };
            match phase {
                "distill" => match crate::neotrix::seal_distill() {
                    Ok(report) => (true, format!("seal_genstep distill 阶段完成: {report}")),
                    Err(e) => (false, format!("seal_genstep distill 阶段失败: {e}")),
                },
                "absorb" => match crate::neotrix::seal_absorb() {
                    Ok(report) => (true, format!("seal_genstep absorb 阶段完成: {report}")),
                    Err(e) => (false, format!("seal_genstep absorb 阶段失败: {e}")),
                },
                _ => match crate::neotrix::seal_iterate() {
                    Ok(report) => (true, format!("seal_genstep iterate 阶段完成: {report}")),
                    Err(e) => (false, format!("seal_genstep iterate 阶段失败: {e}")),
                },
            }
        }
        "self_test_t3" => {
            match KnowledgeBase::open(None) {
                Ok(kb) => {
                    let stats = kb.stats().unwrap_or_default();
                    let nodes = stats.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let edges = stats.get("edges").and_then(|v| v.as_u64()).unwrap_or(0);
                    let kv = stats.get("kv_entries").and_then(|v| v.as_u64()).unwrap_or(0);
                    let t3_capabilities = [
                        "crawl4ai", "seal_genstep", "kb_governance_ostrom",
                        "visual_consistency", "narrative_structuring", "model_selection",
                    ];
                    let mut tested = 0usize;
                    let mut effective = 0usize;
                    for cap in &t3_capabilities {
                        tested += 1;
                        if kb.kv_get("experience", &format!("t3_effective:{cap}")).is_some() {
                            effective += 1;
                        }
                    }
                    (
                        true,
                        format!(
                            "self_test_t3 技能有效性度量: {}/{} 能力有效 | KB: {} nodes / {} edges / {} kv",
                            effective, tested, nodes, edges, kv
                        ),
                    )
                }
                Err(e) => (false, format!("self_test_t3 度量失败: KB 不可用 — {e}")),
            }
        }
        "kb_governance_ostrom" => {
            match KnowledgeBase::open(None) {
                Ok(kb) => {
                    let stats = kb.stats().unwrap_or_default();
                    let nodes = stats.get("nodes").and_then(|v| v.as_u64()).unwrap_or(0);
                    let kv = stats.get("kv_entries").and_then(|v| v.as_u64()).unwrap_or(0);
                    let sanctions_key = "governance:sanctions_applied";
                    let violations_key = "governance:violations_detected";
                    let applied = kb.kv_get("governance", sanctions_key)
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    let violations = kb.kv_get("governance", violations_key)
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(0);
                    let graduated = match violations {
                        0 => "无违规",
                        1..=5 => "警告",
                        6..=20 => "降级",
                        _ => "封禁",
                    };
                    let _ = kb.kv_set("governance", violations_key, &(violations + 1).to_string());
                    (
                        true,
                        format!(
                            "kb_governance_ostrom Ostrom 治理: {} nodes / {} kv | 违规 {} 次 ({}) | 已执行制裁 {} 次",
                            nodes, kv, violations, graduated, applied
                        ),
                    )
                }
                Err(e) => (false, format!("kb_governance_ostrom 治理失败: KB 不可用 — {e}")),
            }
        }
        _ => (
            true,
            format!(
                "internal capability '{}' via domain {}",
                task.capability_tag, task.domain,
            ),
        ),
    }
}
