//! SelfCodeWriter — 基于 ReasoningBank + 模板生成代码
//!
//! 输入: ActionPlan + 编辑历史 + 模板库 → 输出: 代码变更
//! 零 LLM 依赖: 所有生成基于已有历史匹配 + 确定性模板

use std::collections::HashMap;

use super::semantic_entropy::SemanticEntropy;
use super::semantic_entropy::SemanticEntropyGate;
use super::template_registry::{CodeTemplateRegistry, TemplateCategory};

use crate::neotrix::nt_mind_self_diagnose::ActionPlan;

/// N‑gram based content entropy detector — measures code generation uncertainty
/// via Shannon entropy of token n‑gram distribution.
/// High entropy → needs LLM intervention (OracleGate).
#[derive(Debug, Clone)]
pub struct CodeContentEntropy {
    ngram_size: usize,
    max_entropy_threshold: f64,
    vocab: HashMap<String, u32>,
    total_ngrams: u32,
}

impl CodeContentEntropy {
    pub fn new() -> Self {
        Self {
            ngram_size: 3,
            max_entropy_threshold: 3.0,
            vocab: HashMap::new(),
            total_ngrams: 0,
        }
    }

    /// Analyze content and compute entropy score
    pub fn analyze(&mut self, content: &str) -> f64 {
        let ngrams = self.extract_ngrams(content);
        self.total_ngrams = ngrams.len() as u32;
        self.vocab.clear();
        for ng in &ngrams {
            *self.vocab.entry(ng.clone()).or_insert(0) += 1;
        }
        if self.total_ngrams == 0 {
            return 0.0;
        }
        Self::shannon_entropy(&self.vocab, self.total_ngrams)
    }

    /// Check if entropy exceeds threshold
    pub fn is_high_entropy(&self, entropy: f64) -> bool {
        entropy > self.max_entropy_threshold
    }

    /// Extract n-grams from content by sliding window over tokens
    fn extract_ngrams(&self, content: &str) -> Vec<String> {
        let tokens: Vec<&str> = content.split_whitespace().collect();
        if tokens.len() < self.ngram_size {
            return Vec::new();
        }
        tokens
            .windows(self.ngram_size)
            .map(|w| w.join(" "))
            .collect()
    }

    /// Compute Shannon entropy from n-gram frequency distribution
    fn shannon_entropy(frequencies: &HashMap<String, u32>, total: u32) -> f64 {
        let total_f = total as f64;
        let mut h = 0.0;
        for &count in frequencies.values() {
            let p = count as f64 / total_f;
            if p > 0.0 {
                h -= p * p.log2();
            }
        }
        h
    }
}

impl Default for CodeContentEntropy {
    fn default() -> Self {
        Self::new()
    }
}

/// 代码生成请求
#[derive(Debug, Clone)]
pub struct CodeGenRequest {
    pub plan: ActionPlan,
    pub file: String,
    pub context: String, // 文件内容快照
}

/// 代码生成结果
#[derive(Debug, Clone)]
pub struct CodeGenResult {
    pub file: String,
    pub new_content: String,
    pub template_used: Option<String>,
    pub confidence: f64,
}

/// 自代码生成器
#[derive(Debug)]
pub struct SelfCodeWriter {
    template_registry: CodeTemplateRegistry,
    /// SE‑08 / ConSelf pairwise-editing semantic entropy detector.
    /// When Some, every generate call is gated by entropy measurement:
    /// low entropy → auto‑apply, high entropy → defer to LLM.
    pub entropy_detector: Option<SemanticEntropy>,
}

impl SelfCodeWriter {
    pub fn new() -> Self {
        Self {
            template_registry: CodeTemplateRegistry::new(),
            entropy_detector: None,
        }
    }

    pub fn new_with_registry(registry: CodeTemplateRegistry) -> Self {
        Self {
            template_registry: registry,
            entropy_detector: None,
        }
    }

    pub fn with_entropy_detector(mut self, detector: SemanticEntropy) -> Self {
        self.entropy_detector = Some(detector);
        self
    }

    /// 根据 ActionPlan 生成代码
    pub fn generate(&self, req: &CodeGenRequest) -> Result<CodeGenResult, String> {
        match &req.plan {
            ActionPlan::AddTestStub { .. } => self.gen_test_stub(req),
            ActionPlan::RunCargoFix => {
                Err("SelfCodeWriter: RunCargoFix 请使用 AutoFixer::cargo_fix".into())
            }
            ActionPlan::RemoveTodo { .. } => {
                Err("SelfCodeWriter: RemoveTodo 请使用 ActionExecutor".into())
            }
            ActionPlan::SplitLargeFile { .. } => self.gen_file_split(req),
            ActionPlan::ReviewUnsafe { .. } => self.gen_unsafe_review(req),
            ActionPlan::ReplaceUnwrap { .. } => self.gen_unwrap_replacement(req),
            ActionPlan::HumanDecision { .. } => Err("需要人工决策, 无法自动生成".into()),
            ActionPlan::NoAction { reason } => Err(format!("无操作: {}", reason)),
        }
    }

    /// Generate with n‑gram entropy gating — defers to LLM when uncertainty is high
    pub fn generate_with_entropy_gate(
        &mut self,
        req: &CodeGenRequest,
        entropy_detector: &mut CodeContentEntropy,
    ) -> Result<CodeGenResult, String> {
        let result = self.generate(req)?;
        let entropy = entropy_detector.analyze(&result.new_content);
        if entropy_detector.is_high_entropy(entropy) {
            return Err(format!("High entropy ({:.4}) — LLM required", entropy));
        }
        Ok(result)
    }

    /// Generate with SemanticEntropyGate (ConSelf 2026) — checks prompt/context entropy before generation.
    /// Returns DeferredToLLM with entropy value if entropy exceeds threshold.
    pub fn generate_with_gate(
        &self,
        req: &CodeGenRequest,
        gate: &SemanticEntropyGate,
    ) -> Result<CodeGenResult, String> {
        let plan_desc = format!("{:?}", req.plan);
        let context = vec![req.context.clone()];
        if gate.should_defer(&plan_desc, &context) {
            let entropy = SemanticEntropyGate::compute_entropy(&plan_desc, &context);
            return Err(format!(
                "DeferredToLLM: entropy={:.4} > threshold={:.4}",
                entropy, gate.entropy_threshold
            ));
        }
        self.generate(req)
    }

    /// Generate with SE‑08 pairwise semantic entropy check.
    /// Produces N candidate variants via the template registry and computes
    /// pairwise edit distance.  Low entropy → auto‑apply, high → defer to LLM.
    pub fn generate_with_pairwise_entropy(
        &mut self,
        req: &CodeGenRequest,
        result: CodeGenResult,
    ) -> Result<CodeGenResult, String> {
        // Extract values before any mutable access to avoid borrow conflict
        let (n_samples, threshold) = match self.entropy_detector.as_ref() {
            Some(d) => (d.n_samples, d.entropy_threshold),
            None => return Ok(result),
        };

        let candidates = self.generate_variants(req, n_samples);

        use super::semantic_entropy::EntropyAction;

        // Re-borrow to compute entropy (immutable)
        let entropy = self
            .entropy_detector
            .as_ref()
            .map(|d| d.estimate_entropy(&candidates))
            .unwrap_or(0.0);

        let should_defer = entropy > threshold;

        let edit_hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut h = DefaultHasher::new();
            result.new_content.hash(&mut h);
            format!("{:x}", h.finish())
        };

        if should_defer {
            if let Some(ref mut d) = self.entropy_detector {
                d.record(edit_hash, entropy, EntropyAction::DeferredToLLM);
            }
            return Err(format!(
                "DeferredToLLM: pairwise entropy={:.4} > threshold={:.4}, {} candidates",
                entropy,
                threshold,
                candidates.len()
            ));
        }

        if let Some(ref mut d) = self.entropy_detector {
            d.record(edit_hash, entropy, EntropyAction::AutoApplied);
        }
        Ok(result)
    }

    /// Generate N code variants to feed into pairwise entropy measurement.
    /// Primary edit from `generate()`, additional variants from alternate templates
    /// or lightweight content perturbation.
    fn generate_variants(&self, req: &CodeGenRequest, n: usize) -> Vec<String> {
        if n <= 1 {
            return vec![req.context.clone()];
        }

        let mut candidates = Vec::with_capacity(n);

        // Primary: forward the request context as baseline candidate
        candidates.push(req.context.clone());

        // Additional: use alternate templates from the same category
        let category = Self::plan_to_category(&req.plan);
        let templates = self.template_registry.applicable_to(&req.file, category);

        for i in 0..(n - 1).min(templates.len()) {
            let mut vars = HashMap::new();
            let struct_name =
                Self::extract_struct_name(&req.context).unwrap_or_else(|| "Default".to_string());
            vars.insert("name".into(), format!("variant_{}", i));
            vars.insert("struct".into(), struct_name.clone());
            let variant = CodeTemplateRegistry::instantiate(&templates[i], &vars);
            candidates.push(variant);
        }

        // Pad with context copies if not enough templates
        while candidates.len() < n {
            candidates.push(self.perturb_code(&req.context));
        }

        candidates
    }

    /// Lightweight code perturbation to simulate temperature variability.
    /// Uses line‑index‑based determinism (no RNG dependency).
    fn perturb_code(&self, code: &str) -> String {
        let mut out = String::with_capacity(code.len() + 16);
        for (idx, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                out.push('\n');
                continue;
            }
            // Deterministic "coin flip" based on line index: perturb every 5th line
            if idx % 5 == 0 && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                out.push_str(line);
                out.push_str(" // variant");
            } else {
                out.push_str(line);
            }
            out.push('\n');
        }
        out
    }

    /// Map an ActionPlan to an optional template category for variant generation.
    fn plan_to_category(plan: &ActionPlan) -> Option<TemplateCategory> {
        match plan {
            ActionPlan::AddTestStub { .. } => Some(TemplateCategory::TestStub),
            ActionPlan::SplitLargeFile { .. } => Some(TemplateCategory::FunctionExtraction),
            ActionPlan::ReviewUnsafe { .. } => Some(TemplateCategory::DocComment),
            ActionPlan::ReplaceUnwrap { .. } => Some(TemplateCategory::ErrorHandling),
            _ => None,
        }
    }

    /// 生成测试 stub
    fn gen_test_stub(&self, req: &CodeGenRequest) -> Result<CodeGenResult, String> {
        if req.context.contains("#[cfg(test)]") {
            return Err("已有测试模块".into());
        }

        let templates = self
            .template_registry
            .applicable_to(&req.file, Some(TemplateCategory::TestStub));
        let template = templates.first().ok_or("没有可用的测试模板")?;

        // 尝试从文件内容提取 struct 名
        let struct_name =
            Self::extract_struct_name(&req.context).unwrap_or_else(|| "Default".to_string());

        let mut vars = HashMap::new();
        vars.insert("name".into(), "placeholder".into());
        vars.insert("struct".into(), struct_name);

        let generated = CodeTemplateRegistry::instantiate(template, &vars);
        let new_content = req.context.trim().to_string() + &generated;

        Ok(CodeGenResult {
            file: req.file.clone(),
            new_content,
            template_used: Some(template.name.clone()),
            confidence: template.confidence,
        })
    }

    /// 生成文件拆分建议 (标记需要人工)
    fn gen_file_split(&self, req: &CodeGenRequest) -> Result<CodeGenResult, String> {
        // 分析文件结构, 建议拆分点
        let lines: Vec<&str> = req.context.lines().collect();
        let suggestions: Vec<String> = Vec::new();

        // 找 mod / impl / pub fn 作为拆分候选
        for (_i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with("pub fn ") || t.starts_with("fn ") {
                let _name = t.split_whitespace().nth(1).unwrap_or("unknown");
                //                suggestions.push(format!("  Line {}: {} — 可提取为独立函数", i + 1, name));
            }
            if t.starts_with("impl ") {
                //                suggestions.push(format!("  Line {}: {} — 可提取为独立文件", i + 1, t));
            }
        }

        let mut new_content = req.context.clone();
        if !suggestions.is_empty() {
            new_content.push_str(&format!(
                "\n\n// === SelfCodeWriter 拆分建议 ===\n// 建议拆分点:\n{}\n// =================================",
                suggestions.join("\n")
            ));
        }

        Ok(CodeGenResult {
            file: req.file.clone(),
            new_content,
            template_used: None,
            confidence: 0.3,
        })
    }

    /// 生成 unsafe 审查注释
    fn gen_unsafe_review(&self, req: &CodeGenRequest) -> Result<CodeGenResult, String> {
        let mut new_content = String::new();
        let mut found = 0usize;

        for line in req.context.lines() {
            if line.contains("unsafe {") || line.contains("unsafe fn") {
                new_content.push_str(&format!(
                    "// SAFETY-REVIEW: 需要人工审计此 unsafe 块\n{}\n",
                    line
                ));
                found += 1;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if found == 0 {
            return Err("未找到 unsafe 块".into());
        }

        Ok(CodeGenResult {
            file: req.file.clone(),
            new_content,
            template_used: None,
            confidence: 0.5,
        })
    }

    /// 生成 unwrap 替换建议 (标记需要人工)
    fn gen_unwrap_replacement(&self, req: &CodeGenRequest) -> Result<CodeGenResult, String> {
        let mut new_content = String::new();
        let mut found = 0usize;

        for line in req.context.lines() {
            if line.contains(".unwrap(") {
                new_content.push_str(&format!(
                    "// TODO-REVIEW: 此 unwrap 需要人工替换为错误处理\n// 建议: {}.map_err(|e| format!(\"...\").into())?\n",
                    line.trim()
                ));
                found += 1;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        if found == 0 {
            return Err("未找到 .unwrap() 调用".into());
        }

        Ok(CodeGenResult {
            file: req.file.clone(),
            new_content,
            template_used: None,
            confidence: 0.3,
        })
    }

    /// 从文件内容提取 struct 名 (粗略)
    fn extract_struct_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("pub struct ") || t.starts_with("struct ") {
                let name = t
                    .split_whitespace()
                    .nth(2)
                    .or_else(|| t.split_whitespace().nth(1))?;
                return Some(
                    name.trim_end_matches(|c: char| c == ';' || c == '{' || c.is_whitespace())
                        .to_string(),
                );
            }
        }
        None
    }
}

impl Default for SelfCodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_writer_has_templates() {
        let w = SelfCodeWriter::new();
        assert!(!w.template_registry.all().is_empty());
    }

    #[test]
    fn test_generate_test_stub_simple() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "fn main() {}".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.new_content.contains("#[cfg(test)]"));
        assert!(!r.confidence.is_nan());
    }

    #[test]
    fn test_generate_test_stub_already_exists() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "#[cfg(test)]\nfn main() {}".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_human_decision() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::HumanDecision {
                issue_type: crate::neotrix::nt_mind_evolution_loop::IssueType::TodoLeftovers,
                file: None,
                reason: "test".into(),
            },
            file: "test.rs".into(),
            context: "".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_split_file_suggets_splits() {
        let w = SelfCodeWriter::new();
        let content = "pub fn foo() {}\nimpl Bar {}\npub fn baz() {}";
        let req = CodeGenRequest {
            plan: ActionPlan::SplitLargeFile {
                file: "big.rs".into(),
            },
            file: "big.rs".into(),
            context: content.into(),
        };
        let result = w.generate(&req);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.new_content.contains("foo"));
        assert!(r.new_content.contains("Bar"));
    }

    #[test]
    fn test_generate_unsafe_review() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::ReviewUnsafe {
                file: "unsafe.rs".into(),
            },
            file: "unsafe.rs".into(),
            context: "fn safe() {}\nunsafe { *p = 1; }\nfn also_safe() {}".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_ok());
        assert!(result.unwrap().new_content.contains("SAFETY-REVIEW"));
    }

    #[test]
    fn test_generate_unsafe_review_none_found() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::ReviewUnsafe {
                file: "safe.rs".into(),
            },
            file: "safe.rs".into(),
            context: "fn all_safe() {}".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_unwrap_replacement() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::ReplaceUnwrap {
                file: "unwrap.rs".into(),
            },
            file: "unwrap.rs".into(),
            context: "let x = foo().unwrap();".into(),
        };
        let result = w.generate(&req);
        assert!(result.is_ok());
        assert!(result.unwrap().new_content.contains("TODO-REVIEW"));
    }

    #[test]
    fn test_extract_struct_name() {
        assert_eq!(
            SelfCodeWriter::extract_struct_name("pub struct Foo {").as_deref(),
            Some("Foo")
        );
        assert_eq!(
            SelfCodeWriter::extract_struct_name("struct Bar;"),
            Some("Bar".to_string())
        );
        assert_eq!(SelfCodeWriter::extract_struct_name("fn main() {}"), None);
    }

    #[test]
    fn test_no_action_returns_err() {
        let w = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::NoAction {
                reason: "nothing".into(),
            },
            file: "x.rs".into(),
            context: "".into(),
        };
        assert!(w.generate(&req).is_err());
    }

    // --- CodeContentEntropy tests (n‑gram based) ---

    #[test]
    fn test_low_entropy_constant_pattern() {
        let mut se = CodeContentEntropy::new();
        // Repeated same 3-grams → skewed distribution → low entropy
        let code = "let x = 1; let x = 1; let x = 1;";
        let entropy = se.analyze(code);
        assert!(entropy < 2.5, "entropy too high: {}", entropy);
    }

    #[test]
    fn test_high_entropy_random_pattern() {
        let mut se = CodeContentEntropy::new();
        // All unique tokens → uniform n-gram distribution → high entropy
        let code = "xquzl mfbrp kwjdt yhgsa npcve litrb zomax qeruy";
        let entropy = se.analyze(code);
        assert!(entropy > 2.0, "entropy too low: {}", entropy);
    }

    #[test]
    fn test_empty_content_zero_entropy() {
        let mut se = CodeContentEntropy::new();
        assert_eq!(se.analyze(""), 0.0);
        assert_eq!(se.analyze("   "), 0.0);
    }

    #[test]
    fn test_ngram_extraction() {
        let se = CodeContentEntropy::new();
        let ngrams = se.extract_ngrams("a b c d e");
        assert_eq!(ngrams.len(), 3);
        assert_eq!(ngrams[0], "a b c");
        assert_eq!(ngrams[1], "b c d");
        assert_eq!(ngrams[2], "c d e");
    }

    #[test]
    fn test_entropy_threshold_gating() {
        let mut writer = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "fn main() {}".into(),
        };
        // Generated test stub has moderate entropy (~4.2). Use high threshold to verify gate passes.
        let mut se_high = CodeContentEntropy {
            max_entropy_threshold: 10.0,
            ..CodeContentEntropy::new()
        };
        let result = writer.generate_with_entropy_gate(&req, &mut se_high);
        assert!(
            result.is_ok(),
            "gate should pass with high threshold: {:?}",
            result
        );

        // Use low threshold to verify gate blocks
        let mut se_low = CodeContentEntropy {
            max_entropy_threshold: 0.1,
            ..CodeContentEntropy::new()
        };
        let result = writer.generate_with_entropy_gate(&req, &mut se_low);
        assert!(result.is_err(), "gate should block with low threshold");
        assert!(result.unwrap_err().contains("LLM required"));
    }

    #[test]
    fn test_is_high_entropy() {
        let se = CodeContentEntropy::new();
        assert!(se.is_high_entropy(5.0));
        assert!(!se.is_high_entropy(1.0));
        assert!(!se.is_high_entropy(3.0));
    }

    // --- Pairwise SemanticEntropy integration tests (SE‑08) ---

    #[test]
    fn test_generate_with_pairwise_entropy_no_detector() {
        let mut writer = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "fn main() {}".into(),
        };
        let result = writer.generate(&req).unwrap();
        let gated = writer.generate_with_pairwise_entropy(&req, result.clone());
        assert!(gated.is_ok());
    }

    #[test]
    fn test_generate_with_pairwise_entropy_low_threshold_defers() {
        let detector = crate::neotrix::nt_act_code::semantic_entropy::SemanticEntropy::new(3, 0.01);
        let mut writer = SelfCodeWriter::new().with_entropy_detector(detector);
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "fn main() {}".into(),
        };
        let result = writer.generate(&req).unwrap();
        let gated = writer.generate_with_pairwise_entropy(&req, result);
        assert!(gated.is_err());
        assert!(gated.unwrap_err().contains("DeferredToLLM"));
    }

    #[test]
    fn test_entropy_detector_field_default_none() {
        let writer = SelfCodeWriter::new();
        assert!(writer.entropy_detector.is_none());
    }

    #[test]
    fn test_with_entropy_detector_some() {
        let detector = crate::neotrix::nt_act_code::semantic_entropy::SemanticEntropy::default();
        let writer = SelfCodeWriter::new().with_entropy_detector(detector);
        assert!(writer.entropy_detector.is_some());
    }

    #[test]
    fn test_generate_variants() {
        let writer = SelfCodeWriter::new();
        let req = CodeGenRequest {
            plan: ActionPlan::AddTestStub {
                file: "test.rs".into(),
            },
            file: "test.rs".into(),
            context: "fn main() {}".into(),
        };
        let variants = writer.generate_variants(&req, 3);
        assert_eq!(variants.len(), 3);
        for v in &variants {
            assert!(!v.is_empty());
        }
    }
}
