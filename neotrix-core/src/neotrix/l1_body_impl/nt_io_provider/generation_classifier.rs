//! F6: Generation Classifier — per-generation taxonomy tagging + analytics
//!
//! 目标: 每次 LLM 生成完成后, 给该次生成打上分类标签 (任务类型 / 复杂度 / 领域),
//! 供 activity analytics 聚合, 驱动后续模型选择与预算分配的观察证据。
//!
//! 设计约束:
//! - R-P42: 强化既有 GatewayV2 节点 — 本模块被 gateway 消费, 不新建平行适配器。
//! - 纯启发式 (关键词/长度), 零外部依赖, 确定性输出 (可测试)。
//! - `#![forbid(unsafe_code)]` 由 crate 顶层保证。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 任务类型 — 生成物所属的任务类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// 代码生成 / 修复 / 重构
    Code,
    /// 信息抽取 / 结构化解析
    Extraction,
    /// 知识问答 / 事实查询
    Knowledge,
    /// 推理 / 分析 / 比较
    Reasoning,
    /// 总结 / 摘要
    Summarization,
    /// 创意写作
    Creative,
    /// 工具调用 / agent 动作
    ToolUse,
    /// 通用兜底
    General,
}

impl TaskType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Extraction => "extraction",
            Self::Knowledge => "knowledge",
            Self::Reasoning => "reasoning",
            Self::Summarization => "summarization",
            Self::Creative => "creative",
            Self::ToolUse => "tool_use",
            Self::General => "general",
        }
    }
}

/// 复杂度 — 任务规模 / 推理深度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Complexity {
    Trivial,
    Low,
    Medium,
    High,
}

impl Complexity {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Trivial => "trivial",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// 领域 — 知识所属行业/技术域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    /// 软件工程 / 编程
    Code,
    /// 数据 / 数据库
    Data,
    /// 科研 / 论文 / 调研
    Research,
    /// 安全 / 攻防
    Security,
    /// 创意 / 文案
    Creative,
    /// 通用兜底
    General,
}

impl Domain {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Data => "data",
            Self::Research => "research",
            Self::Security => "security",
            Self::Creative => "creative",
            Self::General => "general",
        }
    }
}

/// 一次生成的分类结果 — 三个维度 + 置信度
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Classification {
    pub task_type: TaskType,
    pub complexity: Complexity,
    pub domain: Domain,
    /// 命中信号比例 (0.0-1.0), 越低越可能是误判
    pub confidence: f64,
}

/// 业务用途归因 (Cumora llm_calls ledger 吸收, llm-ledger.ts) — 每次生成
/// 必须可归因到一个 purpose, 使「哪个业务目的烧钱最多」可回答。
/// 新增调用点 REQUIRED 补枚举 — 这是让账本完整的纪律旋钮。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LlmPurpose {
    /// 真实主任务 (agent turn / 用户指令)
    AgentTurn,
    /// 小脑过滤门 (triage / 前置判断)
    Triage,
    /// 摘要 / 压缩
    Summarization,
    /// 工具调用 / 结构化输出
    ToolUse,
    /// 嵌入 / 检索
    Embedding,
    /// 图片生成
    ImageGen,
    /// 杂项
    Utility,
}

impl LlmPurpose {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AgentTurn => "agent_turn",
            Self::Triage => "triage",
            Self::Summarization => "summarization",
            Self::ToolUse => "tool_use",
            Self::Embedding => "embedding",
            Self::ImageGen => "image_gen",
            Self::Utility => "utility",
        }
    }
}

impl Default for LlmPurpose {
    fn default() -> Self {
        Self::AgentTurn
    }
}

/// 一次已打标签的生成记录 — 供 analytics 聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRecord {
    pub model: String,
    pub classification: Classification,
    pub prompt_len: usize,
    pub response_len: usize,
    pub latency_ms: u64,
    pub tokens: u32,
    pub success: bool,
    /// 业务用途归因 (ledger 维度)
    pub purpose: LlmPurpose,
}

impl GenerationRecord {
    pub fn task_type_label(&self) -> &'static str {
        self.classification.task_type.label()
    }
    pub fn complexity_label(&self) -> &'static str {
        self.classification.complexity.label()
    }
    pub fn domain_label(&self) -> &'static str {
        self.classification.domain.label()
    }
    pub fn purpose_label(&self) -> &'static str {
        self.purpose.label()
    }
}

/// F6: 生成分类器 — 基于 prompt/response 的确定性启发式打标签
#[derive(Debug, Default, Clone)]
pub struct GenerationClassifier {
    task_keywords: HashMap<TaskType, Vec<&'static str>>,
    domain_keywords: HashMap<Domain, Vec<&'static str>>,
}

impl GenerationClassifier {
    pub fn new() -> Self {
        Self {
            task_keywords: HashMap::from([
                (
                    TaskType::Code,
                    vec![
                        "function", "implement", "refactor", "bug", "compile", "rust", "python",
                        "typescript", "cargo", "fn ", "write a program", "fix the", "src/",
                        "unit test", "tests pass",
                    ],
                ),
                (
                    TaskType::Extraction,
                    vec![
                        "extract", "parse", "json", "structured", "fields", "schema", "normalize",
                        "csv columns",
                    ],
                ),
                (
                    TaskType::Knowledge,
                    vec![
                        "what is", "who is", "explain the concept", "define", "meaning of",
                        "fact", "capital of", "history of",
                    ],
                ),
                (
                    TaskType::Reasoning,
                    vec![
                        "why", "compare", "analyze", "evaluate", "step-by-step", "reason",
                        "which is better", "trade-off", "pros and cons", "causal",
                    ],
                ),
                (
                    TaskType::Summarization,
                    vec![
                        "summarize", "summary", "condense", "tl;dr", "key points",
                        "briefly explain", "in short",
                    ],
                ),
                (
                    TaskType::Creative,
                    vec![
                        "poem", "story", "creative", "essay", "slogan", "motto", "write a letter",
                        "imagery", "metaphor",
                    ],
                ),
                (
                    TaskType::ToolUse,
                    vec![
                        "use the tool", "call the tool", "invoke", "tool call", "search the web",
                        "run the command", "query the api", "use your search", "execute the tool",
                        "use the search tool", "use the web search",
                    ],
                ),
                (TaskType::General, Vec::new()),
            ]),
            domain_keywords: HashMap::from([
                (
                    Domain::Code,
                    vec![
                        "rust", "python", "typescript", "cargo", "code", "function", "module",
                        "crate", "compiler", "unit test", "api design", "src/", "refactor",
                    ],
                ),
                (
                    Domain::Data,
                    vec![
                        "data", "database", "sql", "csv", "dataset", "query", "schema", "table",
                        "pandas", "json",
                    ],
                ),
                (
                    Domain::Research,
                    vec![
                        "paper", "research", "arxiv", "study", "survey", "citation", "benchmark",
                        "experiment", "literature",
                    ],
                ),
                (
                    Domain::Security,
                    vec![
                        "security", "vulnerability", "exploit", "attack", "xss", "injection",
                        "auth", "encrypt", "malware", "bypass",
                    ],
                ),
                (
                    Domain::Creative,
                    vec![
                        "poem", "story", "creative", "essay", "slogan", "letter", "novel",
                        "script", "dialogue",
                    ],
                ),
                (Domain::General, Vec::new()),
            ]),
        }
    }

    /// 对一次 (prompt, response) 生成打分类标签。
    /// prompt 为空时按 response 特征兜底, 两者都空返回 General/General。
    pub fn classify(&self, prompt: &str, response: &str) -> Classification {
        let prompt_lower = prompt.to_lowercase();
        let response_lower = response.to_lowercase();

        let task_type = self.detect_task(&prompt_lower, &response_lower);
        let domain = self.detect_domain(&prompt_lower, &response_lower);
        let complexity = self.detect_complexity(prompt, response);

        // 置信度 = 任务命中的信号数 (≥1 视为有信号)
        let task_hits = self.count_hits(&prompt_lower, &response_lower, &self.task_keywords[&task_type]);
        let domain_hits = self
            .count_hits(&prompt_lower, &response_lower, &self.domain_keywords[&domain]);
        let confidence = if task_hits == 0 && domain_hits == 0 {
            0.0
        } else {
            (task_hits as f64 + domain_hits as f64 * 0.5) / (task_hits as f64 + domain_hits as f64 + 1.0)
        };

        Classification {
            task_type,
            complexity,
            domain,
            confidence,
        }
    }

    /// 任务类型检测: ToolUse > Extraction > Summarization > Code > Creative > Knowledge > Reasoning > General
    /// (更具体的动作型任务优先于宽泛的 Knowledge/Reasoning)
    fn detect_task(&self, prompt: &str, response: &str) -> TaskType {
        let order = [
            TaskType::ToolUse,
            TaskType::Extraction,
            TaskType::Summarization,
            TaskType::Code,
            TaskType::Creative,
            TaskType::Knowledge,
            TaskType::Reasoning,
        ];
        for t in order {
            if self.matches(&self.task_keywords[&t], prompt, response) {
                return t;
            }
        }
        TaskType::General
    }

    /// 领域检测: 优先 Code/Data/Research/Security/Creative, 无命中 → General
    fn detect_domain(&self, prompt: &str, response: &str) -> Domain {
        let order = [
            Domain::Code,
            Domain::Data,
            Domain::Research,
            Domain::Security,
            Domain::Creative,
        ];
        for d in order {
            if self.matches(&self.domain_keywords[&d], prompt, response) {
                return d;
            }
        }
        Domain::General
    }

    /// 复杂度启发式: 长度 + 深度关键词
    /// Trivial: prompt < 20 chars; Low: < 80; Medium: < 240 或含推理词; High: 其余
    fn detect_complexity(&self, prompt: &str, response: &str) -> Complexity {
        let len = prompt.len() + response.len();
        let deep = [
            "architecture", "comprehensive", "detailed plan", "design decision",
            "multi-layer", "end-to-end", "system design",
        ]
        .iter()
        .any(|k| prompt.to_lowercase().contains(k));
        if deep {
            return Complexity::High;
        }
        if len < 20 {
            Complexity::Trivial
        } else if len < 80 {
            Complexity::Low
        } else if len < 240 {
            Complexity::Medium
        } else {
            Complexity::High
        }
    }

    fn matches(&self, keywords: &[&'static str], prompt: &str, response: &str) -> bool {
        keywords
            .iter()
            .any(|k| prompt.contains(&k.to_lowercase()) || response.contains(&k.to_lowercase()))
    }

    fn count_hits(
        &self,
        prompt: &str,
        response: &str,
        keywords: &[&'static str],
    ) -> usize {
        keywords
            .iter()
            .filter(|k| prompt.contains(&k.to_lowercase()) || response.contains(&k.to_lowercase()))
            .count()
    }
}

/// F6: 生成分析聚合器 — 按任务/复杂度/领域维度累计计数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationAnalytics {
    pub total: u64,
    pub by_task_type: HashMap<String, u64>,
    pub by_complexity: HashMap<String, u64>,
    pub by_domain: HashMap<String, u64>,
    pub by_model: HashMap<String, u64>,
    /// llm_calls ledger 归因: purpose → 计数 (供「哪个业务目的烧钱最多」)
    pub by_purpose: HashMap<String, u64>,
}

impl GenerationAnalytics {
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一次生成 (无论成功失败, success 字段保留在 record)
    pub fn record(&mut self, record: &GenerationRecord) {
        self.total += 1;
        *self.by_task_type.entry(record.task_type_label().to_string()).or_insert(0) += 1;
        *self.by_complexity.entry(record.complexity_label().to_string()).or_insert(0) += 1;
        *self.by_domain.entry(record.domain_label().to_string()).or_insert(0) += 1;
        *self.by_model.entry(record.model.clone()).or_insert(0) += 1;
        *self.by_purpose.entry(record.purpose_label().to_string()).or_insert(0) += 1;
    }

    /// 指定维度的分布 (label → count), 空样本返回空 map
    pub fn distribution(&self, dim: &str) -> HashMap<String, u64> {
        match dim {
            "task_type" => self.by_task_type.clone(),
            "complexity" => self.by_complexity.clone(),
            "domain" => self.by_domain.clone(),
            "model" => self.by_model.clone(),
            "purpose" => self.by_purpose.clone(),
            _ => HashMap::new(),
        }
    }

    /// 最热任务类型 (无记录返回 None)
    pub fn dominant_task_type(&self) -> Option<String> {
        self.by_task_type
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_generation() {
        let c = GenerationClassifier::new();
        let cls = c.classify(
            "write a function that sorts a vector in rust",
            "fn sort(v: &mut Vec<i32>) { v.sort(); }",
        );
        assert_eq!(cls.task_type, TaskType::Code);
        assert_eq!(cls.domain, Domain::Code);
        assert!(cls.confidence > 0.5);
    }

    #[test]
    fn test_classify_extraction() {
        let c = GenerationClassifier::new();
        let cls = c.classify(
            "extract the email and phone from this text",
            "alice@example.com, +86 12345",
        );
        assert_eq!(cls.task_type, TaskType::Extraction);
    }

    #[test]
    fn test_classify_summarization_dominates_general() {
        let c = GenerationClassifier::new();
        let cls = c.classify(
            "summarize the key points of the meeting",
            "three key points were discussed today",
        );
        assert_eq!(cls.task_type, TaskType::Summarization);
    }

    #[test]
    fn test_classify_complexity_tiers() {
        let c = GenerationClassifier::new();
        let trivial = c.classify("hi", "ok");
        assert_eq!(trivial.complexity, Complexity::Trivial);
        let deep = c.classify(
            "design the end-to-end system architecture for a multi-layer distributed cache",
            "long detailed answer about the architecture decision",
        );
        assert_eq!(deep.complexity, Complexity::High);
    }

    #[test]
    fn test_classify_tool_use() {
        let c = GenerationClassifier::new();
        let cls = c.classify(
            "use the search tool to look up the latest arxiv paper",
            "searching... found 3 papers",
        );
        assert_eq!(cls.task_type, TaskType::ToolUse);
    }

    #[test]
    fn test_analytics_aggregation() {
        let mut analytics = GenerationAnalytics::new();
        let c = GenerationClassifier::new();
        let rec = |prompt: &str, resp: &str| GenerationRecord {
            model: "m1".into(),
            classification: c.classify(prompt, resp),
            prompt_len: prompt.len(),
            response_len: resp.len(),
            latency_ms: 10,
            tokens: 100,
            success: true,
            purpose: LlmPurpose::AgentTurn,
        };
        analytics.record(&rec("write a rust function", "fn main() {}"));
        analytics.record(&rec("write a python function", "def main(): pass"));
        analytics.record(&rec("extract json from text", "{\"a\":1}"));
        assert_eq!(analytics.total, 3);
        assert_eq!(analytics.distribution("task_type")["code"], 2);
        assert_eq!(analytics.distribution("task_type")["extraction"], 1);
        assert_eq!(analytics.dominant_task_type().as_deref(), Some("code"));
        assert_eq!(analytics.distribution("domain")["code"], 2);
        assert_eq!(analytics.distribution("purpose")["agent_turn"], 3);
    }

    #[test]
    fn test_analytics_by_model_and_empty() {
        let mut analytics = GenerationAnalytics::new();
        assert_eq!(analytics.dominant_task_type(), None);
        assert!(analytics.distribution("unknown").is_empty());
        let c = GenerationClassifier::new();
        analytics.record(&GenerationRecord {
            model: "llm7".into(),
            classification: c.classify("what is rust", "a systems language"),
            prompt_len: 5,
            response_len: 5,
            latency_ms: 1,
            tokens: 5,
            success: true,
            purpose: LlmPurpose::Utility,
        });
        assert_eq!(analytics.by_model["llm7"], 1);
        assert_eq!(analytics.by_purpose["utility"], 1);
    }

    #[test]
    fn test_purpose_attribution() {
        let mut analytics = GenerationAnalytics::new();
        let c = GenerationClassifier::new();
        let rec = |purpose: LlmPurpose| GenerationRecord {
            model: "m2".into(),
            classification: c.classify("hi", "ok"),
            prompt_len: 2,
            response_len: 2,
            latency_ms: 1,
            tokens: 3,
            success: true,
            purpose,
        };
        analytics.record(&rec(LlmPurpose::AgentTurn));
        analytics.record(&rec(LlmPurpose::AgentTurn));
        analytics.record(&rec(LlmPurpose::Triage));
        analytics.record(&rec(LlmPurpose::Summarization));
        assert_eq!(analytics.by_purpose["agent_turn"], 2);
        assert_eq!(analytics.by_purpose["triage"], 1);
        assert_eq!(analytics.by_purpose["summarization"], 1);
        assert_eq!(LlmPurpose::Triage.label(), "triage");
    }

    #[test]
    fn test_classify_empty_returns_general() {
        let c = GenerationClassifier::new();
        let cls = c.classify("", "");
        assert_eq!(cls.task_type, TaskType::General);
        assert_eq!(cls.domain, Domain::General);
        assert_eq!(cls.complexity, Complexity::Trivial);
        assert_eq!(cls.confidence, 0.0);
    }
}
