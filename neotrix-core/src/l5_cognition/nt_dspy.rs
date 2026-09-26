//! nt_dspy — EVO-03 DSPy 声明式自优化层 (Signature / Metric / Bootstrap / Render).
//!
//! 抄 stanfordnlp/dspy 思想：声明输入输出签名，以 metric 驱动 few-shot demos 自优化，
//! 以纯函数拼装最终提示词。本文件只做同步纯逻辑，不做 IO / LLM 调用 / 全局状态。
//!
//! - `Signature`：输入 / 输出字段描述 ＋ 约束声明。
//! - `DspyMetric`：exact / fuzzy 二 metric ＋ `score` 纯函数。
//! - `BootstrapCompiler`：few-shot demos 收集器（`add_demo` / `compile`，上限 8，
//!   超限按权重替换最低）。
//! - `CompiledPrompt`：`render` 纯函数（指令 ＋ demos 拼装）。
//!
//! 纪律：R-P6 float 箝位到 [0,1]；同步纯逻辑，无 IO / 全局状态。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// demos 上限（Bootstrap few-shot 槽位数）。
pub const MAX_DEMOS: usize = 8;

// ============================================================================
// 小工具（纯函数）
// ============================================================================

/// 将 f64 箝位到 [0.0, 1.0]；NaN 归一为 0.0。
fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.max(0.0).min(1.0)
    }
}

/// 按空白切词并小写归一，供 fuzzy metric 使用。
fn normalized_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| token.to_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

// ============================================================================
// Signature — 输入 / 输出字段描述 ＋ 约束
// ============================================================================

/// 单个字段描述（输入或输出的一列）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSpec {
    /// 字段名（如 `question` / `answer`），非空。
    pub name: String,
    /// 字段语义描述（给模型看的列说明），非空。
    pub description: String,
    /// 是否必填。
    pub required: bool,
}

impl FieldSpec {
    /// 新建字段描述。
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required,
        }
    }

    /// 字段是否合法：名与描述去空白后均非空。
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.description.trim().is_empty()
    }
}

/// 声明式签名：输入字段 ＋ 输出字段 ＋ 自然语言约束。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// 签名名（如 `qa_short`），非空。
    pub name: String,
    /// 任务指令（compile 后成为 `CompiledPrompt.instruction`）。
    pub instruction: String,
    /// 输入列。
    pub inputs: Vec<FieldSpec>,
    /// 输出列。
    pub outputs: Vec<FieldSpec>,
    /// 附加约束（如 `答案不超过 50 字`）；允许为空，但其中每条去空白后非空。
    pub constraints: Vec<String>,
}

impl Signature {
    /// 新建签名。
    pub fn new(
        name: impl Into<String>,
        instruction: impl Into<String>,
        inputs: Vec<FieldSpec>,
        outputs: Vec<FieldSpec>,
        constraints: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            instruction: instruction.into(),
            inputs,
            outputs,
            constraints,
        }
    }

    /// 签名是否合法：名/指令非空，至少一入一出，字段与约束逐条合法。
    pub fn is_valid(&self) -> bool {
        if self.name.trim().is_empty() {
            return false;
        }
        if self.instruction.trim().is_empty() {
            return false;
        }
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }
        for field in self.inputs.iter().chain(self.outputs.iter()) {
            if !field.is_valid() {
                return false;
            }
        }
        for constraint in self.constraints.iter() {
            if constraint.trim().is_empty() {
                return false;
            }
        }
        true
    }

    /// 输入列数。
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// 输出列数。
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}

// ============================================================================
// DspyMetric — exact / fuzzy 二 metric ＋ score 纯函数
// ============================================================================

/// DSPy 风格打分器：精确匹配 vs 模糊匹配。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DspyMetric {
    /// 去首尾空白后字符串全等：1.0，否则 0.0。
    Exact,
    /// 词集合 Jaccard（小写归一）：[0.0, 1.0] 连续分。
    Fuzzy,
}

impl DspyMetric {
    /// 纯函数打分：`predicted` 为模型输出，`reference` 为参照答案。
    pub fn score(&self, predicted: &str, reference: &str) -> f64 {
        match self {
            DspyMetric::Exact => exact_score(predicted, reference),
            DspyMetric::Fuzzy => fuzzy_score(predicted, reference),
        }
    }
}

/// 精确分：trim 后全等得 1.0，否则 0.0；双空视作相等得 1.0。
pub fn exact_score(predicted: &str, reference: &str) -> f64 {
    if predicted.trim() == reference.trim() {
        1.0
    } else {
        0.0
    }
}

/// 模糊分：词集合 Jaccard；双空 1.0，单空 0.0，其余按交并比。
pub fn fuzzy_score(predicted: &str, reference: &str) -> f64 {
    let pred_tokens = normalized_tokens(predicted);
    let ref_tokens = normalized_tokens(reference);
    if pred_tokens.is_empty() && ref_tokens.is_empty() {
        return 1.0;
    }
    if pred_tokens.is_empty() || ref_tokens.is_empty() {
        return 0.0;
    }
    let set_pred: HashSet<String> = pred_tokens.into_iter().collect();
    let set_ref: HashSet<String> = ref_tokens.into_iter().collect();
    if set_pred.is_empty() && set_ref.is_empty() {
        return 1.0;
    }
    let mut inter: usize = 0;
    for token in set_pred.iter() {
        if set_ref.contains(token) {
            inter += 1;
        }
    }
    let union_len = set_pred.len() + set_ref.len() - inter.min(set_pred.len().min(set_ref.len()));
    if union_len == 0 {
        return 1.0;
    }
    clamp01(inter as f64 / union_len as f64)
}

// ============================================================================
// Demo ＋ BootstrapCompiler — few-shot demos 收集器
// ============================================================================

/// 单条 few-shot 演示：输入 / 输出 ＋ 权重（权重越高越值得保留）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Demo {
    /// 演示输入（按 `Signature.inputs` 拼好的文本）。
    pub input: String,
    /// 演示输出（按 `Signature.outputs` 拼好的文本）。
    pub output: String,
    /// 保留权重，已箝位到 [0.0, 1.0]。
    pub weight: f64,
}

impl Demo {
    /// 新建演示，权重自动箝位到 [0.0, 1.0]。
    pub fn new(input: impl Into<String>, output: impl Into<String>, weight: f64) -> Self {
        Self {
            input: input.into(),
            output: output.into(),
            weight: clamp01(weight),
        }
    }

    /// 演示是否合法：输入输出去空白后均非空。
    pub fn is_valid(&self) -> bool {
        !self.input.trim().is_empty() && !self.output.trim().is_empty()
    }
}

/// Bootstrap 编译器：收集 demos，满 8 后按权重替换最低。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BootstrapCompiler {
    demos: Vec<Demo>,
}

impl BootstrapCompiler {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self { demos: Vec::new() }
    }

    /// 当前 demos 数量。
    pub fn len(&self) -> usize {
        self.demos.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.demos.is_empty()
    }

    /// 只读查看 demos 切片。
    pub fn demos(&self) -> &[Demo] {
        &self.demos
    }

    /// 加入一条演示：
    /// - 非法演示直接拒绝，返回 false；
    /// - 未满 [`MAX_DEMOS`] 直接追加，返回 true；
    /// - 已满时若新权重高于当前最低则替换最低，返回 true，否则拒绝返回 false。
    pub fn add_demo(&mut self, demo: Demo) -> bool {
        if !demo.is_valid() {
            return false;
        }
        if self.demos.len() < MAX_DEMOS {
            self.demos.push(demo);
            return true;
        }
        let mut min_idx: Option<usize> = None;
        let mut min_weight = f64::INFINITY;
        for (idx, item) in self.demos.iter().enumerate() {
            if item.weight < min_weight {
                min_weight = item.weight;
                min_idx = Some(idx);
            }
        }
        match min_idx {
            Some(idx) => {
                if demo.weight > min_weight {
                    if let Some(slot) = self.demos.get_mut(idx) {
                        *slot = demo;
                        return true;
                    }
                    false
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// 编译：将签名指令与当前 demos 打包为待渲染提示词。
    pub fn compile(&self, signature: &Signature) -> CompiledPrompt {
        CompiledPrompt {
            instruction: signature.instruction.clone(),
            demos: self.demos.clone(),
        }
    }
}

// ============================================================================
// CompiledPrompt — render 纯函数：指令 ＋ demos 拼装
// ============================================================================

/// 已编译提示词：指令 ＋ 有序 demos，`render` 为纯函数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledPrompt {
    /// 任务指令（来自 `Signature.instruction`）。
    pub instruction: String,
    /// 有序 few-shot 演示。
    pub demos: Vec<Demo>,
}

impl CompiledPrompt {
    /// 新建已编译提示词。
    pub fn new(instruction: impl Into<String>, demos: Vec<Demo>) -> Self {
        Self {
            instruction: instruction.into(),
            demos,
        }
    }

    /// demos 是否为空。
    pub fn is_empty_demos(&self) -> bool {
        self.demos.is_empty()
    }

    /// demos 数量。
    pub fn demo_count(&self) -> usize {
        self.demos.len()
    }

    /// 纯函数渲染：空 demos 时只返回指令；否则按
    /// `指令 ＋ [Demo i]/Input/Output/Weight` 顺序拼装。
    pub fn render(&self) -> String {
        if self.demos.is_empty() {
            return self.instruction.clone();
        }
        let mut out = String::new();
        out.push_str(&self.instruction);
        for (idx, demo) in self.demos.iter().enumerate() {
            out.push_str("\n\n[Demo ");
            out.push_str(&(idx + 1).to_string());
            out.push_str("]\nInput: ");
            out.push_str(&demo.input);
            out.push_str("\nOutput: ");
            out.push_str(&demo.output);
            out.push_str("\nWeight: ");
            out.push_str(&format!("{:.2}", clamp01(demo.weight)));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signature() -> Signature {
        Signature::new(
            "qa_short",
            "Answer briefly.",
            vec![FieldSpec::new("question", "user question", true)],
            vec![FieldSpec::new("answer", "short answer", true)],
            vec!["keep under 50 chars".to_string()],
        )
    }

    #[test]
    fn exact_score_boundary() {
        assert_eq!(exact_score("", ""), 1.0);
        assert_eq!(exact_score("  ", ""), 1.0);
        assert_eq!(exact_score("", "nonempty"), 0.0);
        assert_eq!(exact_score("hello", "hello"), 1.0);
        assert_eq!(exact_score("  hello  ", "hello"), 1.0);
        assert_eq!(exact_score("Hello", "hello"), 0.0);
        assert_eq!(DspyMetric::Exact.score("a", "a"), 1.0);
        assert_eq!(DspyMetric::Exact.score("a", "b"), 0.0);
    }

    #[test]
    fn fuzzy_score_boundary() {
        assert_eq!(fuzzy_score("", ""), 1.0);
        assert_eq!(fuzzy_score("", "hello"), 0.0);
        assert_eq!(fuzzy_score("hello", ""), 0.0);
        assert_eq!(fuzzy_score("hello world", "hello world"), 1.0);
        assert_eq!(fuzzy_score("Hello World", "hello world"), 1.0);
        assert_eq!(fuzzy_score("a b c", "x y z"), 0.0);
        let partial = fuzzy_score("hello world", "hello rust");
        assert!(partial > 0.0 && partial < 1.0);
        let via_enum = DspyMetric::Fuzzy.score("hello world", "hello world");
        assert_eq!(via_enum, 1.0);
    }

    #[test]
    fn bootstrap_cap_replaces_lowest_weight() {
        let sig = test_signature();
        let mut compiler = BootstrapCompiler::new();
        for i in 0..MAX_DEMOS {
            let weight = 0.10 + i as f64 * 0.05;
            let demo = Demo::new(
                format!("q{i}"),
                format!("a{i}"),
                weight,
            );
            assert!(compiler.add_demo(demo));
        }
        assert_eq!(compiler.len(), MAX_DEMOS);
        let before_min = compiler
            .demos()
            .iter()
            .map(|d| d.weight)
            .fold(f64::INFINITY, |a, b| a.min(b));
        assert_eq!(before_min, clamp01(0.10));
        assert!(compiler.add_demo(Demo::new("q_new", "a_new", 0.95)));
        assert_eq!(compiler.len(), MAX_DEMOS);
        let has_new = compiler.demos().iter().any(|d| d.input == "q_new");
        assert!(has_new);
        let after_min = compiler
            .demos()
            .iter()
            .map(|d| d.weight)
            .fold(f64::INFINITY, |a, b| a.min(b));
        assert!(after_min > before_min);
        let compiled = compiler.compile(&sig);
        assert_eq!(compiled.demo_count(), MAX_DEMOS);
    }

    #[test]
    fn bootstrap_rejects_low_weight_when_full_and_rejects_invalid() {
        let mut compiler = BootstrapCompiler::new();
        for i in 0..MAX_DEMOS {
            assert!(compiler.add_demo(Demo::new(
                format!("q{i}"),
                format!("a{i}"),
                0.80
            )));
        }
        assert!(!compiler.add_demo(Demo::new("q_low", "a_low", 0.10)));
        assert_eq!(compiler.len(), MAX_DEMOS);
        assert!(!compiler.add_demo(Demo::new("", "a_ok", 0.99)));
        assert!(!compiler.add_demo(Demo::new("q_ok", "   ", 0.99)));
        assert_eq!(compiler.len(), MAX_DEMOS);
    }

    #[test]
    fn render_assembles_instruction_and_demos() {
        let sig = test_signature();
        let mut compiler = BootstrapCompiler::new();
        assert!(compiler.add_demo(Demo::new("q1", "a1", 0.90)));
        assert!(compiler.add_demo(Demo::new("q2", "a2", 0.80)));
        let compiled = compiler.compile(&sig);
        let text = compiled.render();
        assert!(text.contains("Answer briefly."));
        assert!(text.contains("[Demo 1]"));
        assert!(text.contains("[Demo 2]"));
        assert!(text.contains("Input: q1"));
        assert!(text.contains("Output: a2"));
        let pos1 = match text.find("[Demo 1]") {
            Some(v) => v,
            None => usize::MAX,
        };
        let pos2 = match text.find("[Demo 2]") {
            Some(v) => v,
            None => 0,
        };
        assert!(pos1 < pos2);
    }

    #[test]
    fn empty_demos_render_returns_instruction_only() {
        let sig = test_signature();
        let compiler = BootstrapCompiler::new();
        assert!(compiler.is_empty());
        let compiled = compiler.compile(&sig);
        assert!(compiled.is_empty_demos());
        assert_eq!(compiled.demo_count(), 0);
        assert_eq!(compiled.render(), "Answer briefly.".to_string());
        let direct = CompiledPrompt::new("Do X.", vec![]);
        assert_eq!(direct.render(), "Do X.".to_string());
    }
}
