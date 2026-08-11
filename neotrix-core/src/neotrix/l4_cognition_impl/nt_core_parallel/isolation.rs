//! 意图隔离 + 原子拆解 (LLM 调用前处理层)
//!
//! 吸收来源 (2026-08-11 外部吸收三仓库):
//! - `chAng-L19/codex-redteam-mode`: GoalContract 编译 + Prompt Rewrite 双层隔离 —
//!   主模型只收到改写后的 prompt, 全局意图不出上下文。
//! - `MDX-Tom/gpt-5.6-instruct`: 归一化 → 语义分派 → 意图路由 → 工件验证 管线 —
//!   任务先路由后派发, 每步产出可验证工件。
//! - `dongshuyan/Awesome-Prompts`: SPEV (Spec→Plan→Execute→Verify) 契约化执行 —
//!   明确规格与验收, LLM 可直接返回结构化结果。
//!
//! 解决的问题 (B-class):
//! 1. 外部信息获取 / LLM 调用时, 不让 LLM 知道核心目的 — need-to-know 上下文裁剪。
//! 2. 任务拆解精细到 LLM 可以直接返回结果 — 原子单元 + 输出契约。
//!
//! 生产接线 (R-P79, 2026-08-11):
//! - `cli/commands/contract_cmds.rs` define 分支: `/contract define` 触发类型路由 +
//!   原子拆解 → C1 定义即 C2 拆解 (输出契约随原子单元生成)。
//! - `seal_loop.rs::run_seal_loop_pipeline` 入口: 意图隔离观测点 (exposure_ratio < 1.0
//!   时记录暴露面 + 注入 task-structure 摘要), 供 pipeline 内对外 LLM 调用边界按需使用。
//!
//! 纯规则实现 (无 LLM 依赖, 保持廉价可验证, 与 `nt_memory_decompose` 风格对齐)。

use serde::{Deserialize, Serialize};

/// 意图隔离结果
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IsolatedContext {
    /// 剥离核心目的后, 可安全暴露给外部 LLM / 子 agent 的最小上下文
    pub exposed_prompt: String,
    /// 从原始上下文中剥离出的核心目的片段 (need-to-know 之外的信息)
    pub withheld_intent: Vec<String>,
    /// 暴露比 (0.0-1.0): 越低越保密; 暴露字符 / 原始字符
    pub exposure_ratio: f64,
}

/// 意图隔离器 — need-to-know 上下文裁剪
///
/// 机制 (吸收 codex-redteam Prompt Rewrite):
/// 1. 识别原始上下文中的"核心目的句" (目标/原因/战略意图), 从暴露面剥离。
/// 2. 只保留动作 + 数据对象 + 输出要求。
/// 3. 支持自定义高价值意图标记词, 命中即剥离。
#[derive(Debug, Clone, Default)]
pub struct IntentIsolator {
    /// 高价值意图标记: 含这些词的句子视为核心目的, 剥离时不传给外部 LLM
    pub intent_markers: Vec<String>,
}

impl IntentIsolator {
    /// 默认标记库 (中文+英文战略意图高频词)
    pub fn with_default_markers() -> Self {
        Self {
            intent_markers: vec![
                "核心目的".into(),
                "最终目标".into(),
                "为了".into(),
                "战略".into(),
                "竞品".into(),
                "评估".into(),
                "值得".into(),
                "决策".into(),
                "目标".into(),
                "aim".into(),
                "purpose".into(),
                "goal".into(),
                "strategy".into(),
                "why".into(),
                "evaluate".into(),
                "decide".into(),
                "worth".into(),
            ],
        }
    }

    /// 隔离核心意图: 返回可暴露上下文 + 被剥离的意图片段
    ///
    /// 粒度: 片段级剥离 (局部无损) — 先按句切分, 含标记的句子再按逗号/分号
    /// 切分为子片段, 仅剥离含标记的子片段, 保全任务主体。
    pub fn isolate(&self, original: &str) -> IsolatedContext {
        let sentences = split_sentences(original);
        let mut exposed = Vec::new();
        let mut withheld = Vec::new();

        for s in &sentences {
            if self.marks(s) {
                // 句子含标记 → 片段级剥离: 拆成子片段, 仅剥离命中标记的
                let fragments = split_fragments(s);
                let hits: Vec<String> = fragments
                    .iter()
                    .filter(|f| self.marks(f))
                    .cloned()
                    .collect();
                let clean: Vec<String> = fragments
                    .iter()
                    .filter(|f| !self.marks(f))
                    .cloned()
                    .collect();
                withheld.extend(hits);
                if !clean.is_empty() {
                    exposed.push(clean.join("，"));
                }
            } else {
                exposed.push(s.clone());
            }
        }

        let exposed_prompt = exposed.join(" ");
        let exposure_ratio = if original.is_empty() {
            0.0
        } else {
            (exposed_prompt.len() as f64) / (original.len() as f64).max(1.0)
        };

        IsolatedContext {
            exposed_prompt,
            withheld_intent: withheld,
            exposure_ratio,
        }
    }

    /// 句子/片段是否命中任一意图标记
    fn marks(&self, text: &str) -> bool {
        let t = text.to_lowercase();
        self.intent_markers.iter().any(|m| t.contains(&m.to_lowercase()))
    }
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// 结构化 JSON (带字段)
    Json,
    /// 纯文本
    PlainText,
    /// 枚举列表
    Enumerated,
}

/// 输出契约 (吸收 Awesome-Prompts SPEV 的 VERIFY + GoalContract 验收准则)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputContract {
    /// 期望返回字段
    pub fields: Vec<String>,
    /// 输出格式
    pub format: OutputFormat,
    /// 成功准则 (LLM 返回后确定性可检查的项)
    pub success_criteria: Vec<String>,
}

impl OutputContract {
    /// 生成可嵌入 prompt 的契约描述 (LLM 直接按此返回)
    pub fn to_prompt_snippet(&self) -> String {
        let fields = self.fields.join(", ");
        match self.format {
            OutputFormat::Json => format!(
                "必须直接返回 JSON, 字段: {{{}}}; 无额外解释。",
                fields
            ),
            OutputFormat::PlainText => format!(
                "直接返回纯文本, 包含: {}; 无多余铺垫。",
                fields
            ),
            OutputFormat::Enumerated => format!(
                "直接返回编号列表, 每项对应: {}; 无多余铺垫。",
                fields
            ),
        }
    }

    /// 确定性校验: 检查 LLM 原始返回是否满足契约
    pub fn verify(&self, response: &str) -> Vec<String> {
        let mut failures = Vec::new();
        match self.format {
            OutputFormat::Json => {
                if !response.trim_start().starts_with('{') {
                    failures.push("响应不是 JSON 对象".into());
                }
                for f in &self.fields {
                    if !response.contains(f) {
                        failures.push(format!("缺少字段: {}", f));
                    }
                }
            }
            OutputFormat::Enumerated => {
                let has_numbering = response
                    .lines()
                    .any(|l| l.trim_start().starts_with(|c: char| c.is_ascii_digit()));
                if !has_numbering {
                    failures.push("响应缺少编号列表".into());
                }
                if self.fields.len() > 1 && response.lines().count() < self.fields.len() {
                    failures.push(format!("列表项数不足: 期望 ≥{}", self.fields.len()));
                }
            }
            OutputFormat::PlainText => {
                if response.trim().is_empty() {
                    failures.push("响应为空".into());
                }
            }
        }
        for c in &self.success_criteria {
            if !response.contains(c) {
                failures.push(format!("未满足成功准则: {}", c));
            }
        }
        failures
    }
}

/// 原子单元 — LLM 可直接返回结果的任务粒度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomicUnit {
    pub id: String,
    /// 单步指令 (不含全局意图, 只含动作+对象+输出要求)
    pub instruction: String,
    /// 输入引用 (上游产出 id)
    pub inputs: Vec<String>,
    /// 输出契约
    pub output_contract: OutputContract,
}

/// 拆解结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompositionPlan {
    /// 并行原子单元 (map 阶段, 可同时派发)
    pub parallel: Vec<AtomicUnit>,
    /// 串行原子单元 (reduce/依赖链阶段)
    pub sequential: Vec<AtomicUnit>,
    /// 是否需要串行依赖 (true 时 LLM 调用必须分轮)
    pub needs_sequential: bool,
}

impl DecompositionPlan {
    /// 全部原子单元 (parallel 在前)
    pub fn all_units(&self) -> Vec<&AtomicUnit> {
        self.parallel
            .iter()
            .chain(self.sequential.iter())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.parallel.is_empty() && self.sequential.is_empty()
    }
}

/// 任务类型路由 (吸收 gpt-5.6-instruct 意图路由 + Awesome-Prompts 需求类型识别)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    /// 信息获取 / 检索 (默认并行)
    Research,
    /// 多条件推理 (默认串行依赖链)
    Reasoning,
    /// 对比 (map-reduce: 并行收集 → 串行综合)
    Compare,
    /// 简单单步 (原子, 不拆)
    Atomic,
}

/// 原子拆解器 — 把任务拆到 LLM 可直接返回结果的粒度
///
/// 机制 (吸收 gpt-5.6 归一化+意图路由, Awesome-Prompts SPEV):
/// 1. 类型路由: 对比→Compare, 顺序推理→Reasoning, 多实体枚举→Research, 兜底→Atomic。
/// 2. 拆解: Research 拆为 Flat 并行子任务 (每个带输出契约);
///    Reasoning 拆为依赖链; Compare 拆为 并行收集 + 串行综合。
/// 3. 每个原子单元: 单步指令 + 输入引用 + 输出契约 (LLM 直接返回)。
#[derive(Debug, Clone, Default)]
pub struct AtomicDecomposer;

/// 本模块对外提供的能力标签 (provides) — 能力网单一事实源。
///
/// 与 `.neotrix/capability_registry.json` 中 `nt_core_parallel::*` 节点保持镜像一致;
/// 未来 AttentionRouter 按 provides 标签做运行时路由时, 以此声明为锚点。
pub fn capability_provides() -> &'static [&'static str] {
    &["intent_isolation", "need_to_know", "atomic_decomposition", "output_contract"]
}

impl AtomicDecomposer {
    pub fn new() -> Self {
        Self
    }

    /// 任务类型路由 (纯规则)
    pub fn route_kind(task: &str) -> TaskKind {
        let t = task.to_lowercase();
        if ["difference between", " vs ", " compare ", " versus ", "区别", "对比"]
            .iter()
            .any(|m| t.contains(m))
        {
            return TaskKind::Compare;
        }
        if [" then ", " after that ", " subsequently ", " 然后", " 之后", " 依次"]
            .iter()
            .any(|m| t.contains(m))
        {
            return TaskKind::Reasoning;
        }
        // 多实体枚举 → Research (Flat)
        let entity_count = count_entities(task);
        if entity_count >= 2 {
            return TaskKind::Research;
        }
        TaskKind::Atomic
    }

    /// 拆解为原子计划
    pub fn decompose(&self, task: &str, kind: TaskKind) -> DecompositionPlan {
        match kind {
            TaskKind::Atomic => DecompositionPlan {
                parallel: vec![AtomicUnit {
                    id: "u0".into(),
                    instruction: task.to_string(),
                    inputs: vec![],
                    output_contract: OutputContract {
                        fields: vec!["result".into()],
                        format: OutputFormat::PlainText,
                        success_criteria: vec![],
                    },
                }],
                sequential: vec![],
                needs_sequential: false,
            },
            TaskKind::Research => {
                let subs = split_entities(task);
                let parallel: Vec<AtomicUnit> = subs
                    .iter()
                    .enumerate()
                    .map(|(i, s)| AtomicUnit {
                        id: format!("u{}", i),
                        instruction: format!("检索并概括: {}", s),
                        inputs: vec![],
                        output_contract: OutputContract {
                            fields: vec!["summary".into()],
                            format: OutputFormat::PlainText,
                            success_criteria: vec![],
                        },
                    })
                    .collect();
                DecompositionPlan {
                    parallel,
                    sequential: vec![],
                    needs_sequential: false,
                }
            }
            TaskKind::Compare => {
                let subs = split_entities(task);
                let parallel: Vec<AtomicUnit> = subs
                    .iter()
                    .enumerate()
                    .map(|(i, s)| AtomicUnit {
                        id: format!("u{}", i),
                        instruction: format!("收集 {} 的关键事实与特征", s),
                        inputs: vec![],
                        output_contract: OutputContract {
                            fields: vec!["facts".into()],
                            format: OutputFormat::Enumerated,
                            success_criteria: vec![],
                        },
                    })
                    .collect();
                // 串行综合阶段 (reduce): 汇总并行产出
                let refs: Vec<String> = (0..parallel.len()).map(|i| format!("u{}", i)).collect();
                let sequential = vec![AtomicUnit {
                    id: "merge".into(),
                    instruction: format!("综合以下输入, 输出对比结论", ),
                    inputs: refs,
                    output_contract: OutputContract {
                        fields: vec!["comparison".into()],
                        format: OutputFormat::PlainText,
                        success_criteria: vec![],
                    },
                }];
                DecompositionPlan {
                    parallel,
                    sequential,
                    needs_sequential: true,
                }
            }
            TaskKind::Reasoning => {
                let steps = split_steps(task);
                let sequential: Vec<AtomicUnit> = steps
                    .iter()
                    .enumerate()
                    .map(|(i, s)| AtomicUnit {
                        id: format!("u{}", i),
                        instruction: s.to_string(),
                        // 依赖上一跳: 第一个无输入, 后续引用前序
                        inputs: if i == 0 {
                            vec![]
                        } else {
                            vec![format!("u{}", i - 1)]
                        },
                        output_contract: OutputContract {
                            fields: vec!["step_result".into()],
                            format: OutputFormat::PlainText,
                            success_criteria: vec![],
                        },
                    })
                    .collect();
                DecompositionPlan {
                    parallel: vec![],
                    sequential,
                    needs_sequential: true,
                }
            }
        }
    }
}

/// 按句子切分 (支持中英文标点)
fn split_sentences(text: &str) -> Vec<String> {
    text.split(|c: char| matches!(c, '。' | '！' | '？' | '；' | '.' | '!' | '?' | ';'))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 按逗号/分号/顿号切分子片段 (用于片段级意图剥离)
fn split_fragments(sentence: &str) -> Vec<String> {
    sentence
        .split(|c: char| matches!(c, '，' | '、' | ',' | '；' | ';'))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 统计大写实体/专有名词数量 (英文)
fn count_entities(task: &str) -> usize {
    task.split(|c: char| !c.is_alphanumeric())
        .filter(|w| {
            !w.is_empty()
                && w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                && w.len() >= 2
                && !["The", "This", "That", "What", "Why", "How", "When", "Where", "Compare", "Between", "Explain"]
                    .contains(&w)
        })
        .count()
}

/// 按 " and " / " & " / "," 切分实体 (去除前缀杂质词)
fn split_entities(task: &str) -> Vec<String> {
    task.split(" and ")
        .flat_map(|p| p.split(" & "))
        .flat_map(|p| p.split(','))
        .flat_map(|p| p.split(" vs "))
        .map(|p| p.trim().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|p| !p.is_empty() && p.len() >= 2)
        .collect()
}

/// 按顺序标记切分步骤
fn split_steps(task: &str) -> Vec<String> {
    let markers = [" then ", " after that ", " subsequently ", " 然后", " 之后", " 依次"];
    let mut parts = vec![task.to_string()];
    for m in markers {
        let mut new_parts: Vec<String> = Vec::new();
        for p in parts {
            let split: Vec<&str> = p.split(m).collect();
            if split.len() > 1 {
                new_parts.extend(split.iter().map(|s| s.to_string()));
            } else {
                new_parts.push(p);
            }
        }
        parts = new_parts;
    }
    parts
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s.len() > 2)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== IntentIsolator =====

    #[test]
    fn test_isolate_strips_core_intent() {
        let iso = IntentIsolator::with_default_markers();
        let ctx = iso.isolate(
            "为了评估是否值得投资, 请收集 Rust 生态的规模数据。输出: 生态报告。",
        );
        assert!(
            ctx.withheld_intent.iter().any(|s| s.contains("评估")),
            "核心目的句应被剥离: {:?}",
            ctx.withheld_intent
        );
        assert!(
            ctx.exposed_prompt.contains("生态") && ctx.exposed_prompt.contains("输出"),
            "动作与输出要求应保留: {}",
            ctx.exposed_prompt
        );
        assert!(ctx.exposure_ratio > 0.0 && ctx.exposure_ratio < 1.0);
    }

    #[test]
    fn test_isolate_english_goal() {
        let iso = IntentIsolator::with_default_markers();
        let ctx = iso.isolate("Collect data about E8. The purpose is to decide our strategy.");
        assert_eq!(ctx.withheld_intent.len(), 1, "purpose 句应剥离: {:?}", ctx.withheld_intent);
        assert!(ctx.exposed_prompt.contains("Collect data"));
    }

    #[test]
    fn test_isolate_no_markers_keeps_all() {
        let iso = IntentIsolator::default();
        let ctx = iso.isolate("提取 GWT 模块的接口定义");
        assert!(ctx.withheld_intent.is_empty());
        assert!(ctx.exposure_ratio >= 0.9, "无标记应近乎全暴露: {}", ctx.exposure_ratio);
    }

    #[test]
    fn test_isolate_empty_input() {
        let iso = IntentIsolator::default();
        let ctx = iso.isolate("");
        assert!(ctx.exposed_prompt.is_empty());
        assert_eq!(ctx.exposure_ratio, 0.0);
    }

    // ===== OutputContract =====

    #[test]
    fn test_json_contract_verify_pass() {
        let c = OutputContract {
            fields: vec!["name".into(), "score".into()],
            format: OutputFormat::Json,
            success_criteria: vec![],
        };
        let failures = c.verify(r#"{"name": "E8", "score": 0.9}"#);
        assert!(failures.is_empty(), "应通过: {:?}", failures);
    }

    #[test]
    fn test_json_contract_verify_missing_field() {
        let c = OutputContract {
            fields: vec!["name".into(), "score".into()],
            format: OutputFormat::Json,
            success_criteria: vec![],
        };
        let failures = c.verify(r#"{"name": "E8"}"#);
        assert!(
            failures.iter().any(|f| f.contains("score")),
            "应报缺失字段: {:?}",
            failures
        );
    }

    #[test]
    fn test_enumerated_contract_verify() {
        let c = OutputContract {
            fields: vec!["a".into(), "b".into()],
            format: OutputFormat::Enumerated,
            success_criteria: vec![],
        };
        assert!(c.verify("1. x\n2. y").is_empty());
        assert!(!c.verify("plain text").is_empty(), "无编号应失败");
    }

    #[test]
    fn test_contract_prompt_snippet() {
        let c = OutputContract {
            fields: vec!["summary".into()],
            format: OutputFormat::PlainText,
            success_criteria: vec![],
        };
        let s = c.to_prompt_snippet();
        assert!(s.contains("直接返回"), "契约应要求直接返回: {}", s);
    }

    // ===== 连续状态完成 (吸收 gpt-5.6-instruct v45 CONTINUITY_AND_COMPLETION) =====

    #[test]
    fn test_continuity_state_verify() {
        // v45 模式: 最终输出以 `Current: TARGET / RESULT / NEXT` (或 `当前：对象 / 结果 / 下一步`)
        // 开头 — 用作 PlainText 输出契约的成功准则, 保证跨轮状态连续。
        let c = OutputContract {
            fields: vec!["状态行".into()],
            format: OutputFormat::PlainText,
            success_criteria: vec!["当前：".into()],
        };
        // 连续状态格式通过
        let ok = "当前：E8 模块 / 已定位 3 个消费者 / 下一步生成依赖图";
        assert!(c.verify(ok).is_empty(), "连续状态应通过: {:?}", c.verify(ok));
        // 无状态行失败
        let bad = "完成了任务";
        assert!(
            c.verify(bad).iter().any(|f| f.contains("当前")),
            "缺少状态行应失败: {:?}",
            c.verify(bad)
        );
    }

    #[test]
    fn test_continuity_english_state_verify() {
        let c = OutputContract {
            fields: vec!["status".into()],
            format: OutputFormat::PlainText,
            success_criteria: vec!["NEXT".into()],
        };
        assert!(c.verify("Current: E8 module / located 3 consumers / NEXT build dep graph").is_empty());
        assert!(!c.verify("done.").is_empty(), "缺少 NEXT 状态应失败");
    }

    // ===== 四工件验证 (吸收 gpt-5.6 v45 TOOL_TRANSACTION) =====

    #[test]
    fn test_change_set_contract_verify() {
        // v45 模式: 修改类任务产出 4 工件 (MODIFIED_FILE/DIFF_FILE/VERIFICATION.txt/ROLLBACK.sh),
        // 成功准则 = 响应须含 4 路径 + 验证结果。体现"工件验证管线" (gpt-5.6 回归门禁)。
        let c = OutputContract {
            fields: vec!["MODIFIED_FILE".into(), "VERIFICATION.txt".into()],
            format: OutputFormat::PlainText,
            success_criteria: vec!["VERIFICATION.txt".into()],
        };
        let response = "已修改 nt_core_parallel/mod.rs。\n\
            MODIFIED_FILE: src/nt_core_parallel/mod.rs\n\
            VERIFICATION.txt: cargo check 0 errors, 测试 17/17\n\
            ROLLBACK.sh: git checkout -- mod.rs";
        assert!(
            c.verify(response).is_empty(),
            "四工件响应应通过: {:?}",
            c.verify(response)
        );
        // 缺 VERIFICATION.txt 失败
        let missing = "MODIFIED_FILE: src/foo.rs";
        assert!(
            c.verify(missing).iter().any(|f| f.contains("VERIFICATION")),
            "缺验证工件应失败: {:?}",
            c.verify(missing)
        );
    }

    // ===== AtomicDecomposer =====

    #[test]
    fn test_route_compare() {
        assert_eq!(AtomicDecomposer::route_kind("difference between E8 and GWT"), TaskKind::Compare);
        assert_eq!(AtomicDecomposer::route_kind("E8 vs GWT 对比"), TaskKind::Compare);
    }

    #[test]
    fn test_route_reasoning() {
        assert_eq!(
            AtomicDecomposer::route_kind("先找到 E8 模块 then 追踪其消费者"),
            TaskKind::Reasoning
        );
    }

    #[test]
    fn test_route_research_multi_entity() {
        assert_eq!(
            AtomicDecomposer::route_kind("SEAL and E8 and GWT evolution loops"),
            TaskKind::Research
        );
    }

    #[test]
    fn test_route_atomic_simple() {
        assert_eq!(
            AtomicDecomposer::route_kind("什么是向量嵌入"),
            TaskKind::Atomic
        );
    }

    #[test]
    fn test_decompose_atomic_single_unit() {
        let d = AtomicDecomposer::new();
        let plan = d.decompose("什么是向量嵌入", TaskKind::Atomic);
        assert_eq!(plan.all_units().len(), 1);
        assert!(!plan.needs_sequential);
        assert!(plan.is_empty() == false);
    }

    #[test]
    fn test_decompose_compare_map_reduce() {
        let d = AtomicDecomposer::new();
        let plan = d.decompose("E8 vs GWT 对比", TaskKind::Compare);
        assert_eq!(plan.parallel.len(), 2, "对比应拆 2 并行收集: {:?}", plan.parallel);
        assert_eq!(plan.sequential.len(), 1, "应有 1 综合步骤");
        assert!(plan.needs_sequential);
        // 综合步骤引用并行产出
        assert_eq!(plan.sequential[0].inputs, vec!["u0".to_string(), "u1".to_string()]);
    }

    #[test]
    fn test_decompose_reasoning_chain() {
        let d = AtomicDecomposer::new();
        let plan = d.decompose("定位 E8 模块 then 追踪其消费者 then 汇总依赖图", TaskKind::Reasoning);
        assert!(plan.parallel.is_empty());
        assert!(plan.sequential.len() >= 3, "依赖链应 ≥3 步: {:?}", plan.sequential);
        assert!(plan.needs_sequential);
        // 第二步依赖第一步
        assert_eq!(plan.sequential[1].inputs, vec!["u0".to_string()]);
    }

    #[test]
    fn test_decompose_research_flat() {
        let d = AtomicDecomposer::new();
        let plan = d.decompose("SEAL and E8", TaskKind::Research);
        assert_eq!(plan.parallel.len(), 2);
        assert!(!plan.needs_sequential);
        // 每个原子单元带输出契约 (LLM 可直接返回)
        for u in &plan.parallel {
            assert!(!u.output_contract.to_prompt_snippet().is_empty());
        }
    }

    // ===== 集成: 隔离 + 拆解流水线 =====

    #[test]
    fn test_full_pipeline_isolate_then_decompose() {
        // 场景: 对外 LLM 调用 — 不暴露核心目的, 拆到可直接返回
        let iso = IntentIsolator::with_default_markers();
        let raw = "为了评估是否值得投资, 请对比 E8 and GWT 的架构成熟度。输出: 结论。";
        let isolated = iso.isolate(raw);

        // 核心目的被剥离
        assert!(
            isolated.withheld_intent.iter().any(|s| s.contains("投资") || s.contains("评估")),
            "投资意图必须剥离"
        );

        // 剩余任务可拆解
        let plan = AtomicDecomposer::new().decompose(&isolated.exposed_prompt, AtomicDecomposer::route_kind(&isolated.exposed_prompt));
        assert_eq!(plan.parallel.len(), 2, "隔离后仍可拆为并行收集: {}", isolated.exposed_prompt);

        // 每个单元指令不含核心目的
        for u in plan.all_units() {
            assert!(
                !u.instruction.contains("投资") && !u.instruction.contains("评估"),
                "原子指令不得泄露核心目的: {}",
                u.instruction
            );
        }
    }

    // ===== 能力网: provides 声明与生产消费一致性 =====

    #[test]
    fn test_capability_provides_declared() {
        // 能力网单一事实源: 代码侧 provides 声明必须完整覆盖本模块能力
        let provides = capability_provides();
        assert!(provides.contains(&"intent_isolation"), "意图隔离能力必须声明");
        assert!(provides.contains(&"need_to_know"), "need-to-know 能力必须声明");
        assert!(provides.contains(&"atomic_decomposition"), "原子拆解能力必须声明");
        assert!(provides.contains(&"output_contract"), "输出契约能力必须声明");
        // 无重复
        let mut sorted = provides.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), provides.len(), "provides 声明不得重复");
    }

    #[test]
    fn test_capability_provides_consumed_in_production() {
        // 生产消费点验证: 声明的能力必须被非测试代码消费 (R-P79 接线闭环)
        // 消费点: contract_cmds.rs define 分支 (atomic_decomposition/output_contract)
        //         seal_loop.rs 意图隔离观测点 (intent_isolation/need_to_know)
        let src = std::fs::read_to_string(
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/commands/contract_cmds.rs"),
        )
        .unwrap();
        assert!(src.contains("AtomicDecomposer"), "contract_cmds 必须消费 atomic_decomposition");
        assert!(src.contains("all_units"), "contract_cmds 必须消费 output_contract (经 plan.all_units)");

        let seal = std::fs::read_to_string(
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/neotrix/l8_autonomic_impl/nt_mind/seal_core/self_iterating/loop_impl/seal_loop.rs"),
        )
        .unwrap();
        assert!(seal.contains("IntentIsolator"), "seal_loop 必须消费 intent_isolation");
        assert!(seal.contains("exposure_ratio"), "seal_loop 必须观测 need_to_know 暴露面");
    }
}
