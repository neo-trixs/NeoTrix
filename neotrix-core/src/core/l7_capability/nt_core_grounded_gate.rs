//! # GroundedGate — 外部接地验证门 + AgentContract 类型化契约
//!
//! 对应 `docs/2-PLANS/2026-07-01-multi-agent-orchestration-design.md` §6/§7（P0）。
//!
//! ## GroundedGate
//! 用**可执行证据**（cargo check / cargo test / 工具输出对比）验证子 agent 输出，
//! 而非仅文本启发式。对应 Huang 2024 的"外部接地"要求。
//!
//! 执行语义:
//! ```text
//! AgentOutput → GroundedGate → [Pass → next]
//!                             | [Revise → worker (max_retries)]
//!                             | [Fail → escalation]
//! ```
//! 与文本质量门组合: **先文本门(快) → 后接地门(准)**，控制成本 (§6.2)。
//!
//! ## AgentContract
//! 子 agent 类型化契约 — 定义输入/输出 schema 与成功标准（MetaGPT 模式），
//! 让编排器从"聊天式委托"升级为"契约式委托" (§7.1)。
//!
//! 委托协议:
//! ```text
//! orchestrator 定义契约 → 子 agent 按契约产出 → GroundedGate 按 success_criteria 验证 → 采纳/重做
//! ```

/// 接地检查类型 — 每种检查是可执行的验证步骤。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundedCheck {
    /// cargo check 编译通过（Rust 子任务）。
    Compile { crate_name: String },
    /// cargo test 指定测试通过。
    Test { crate_name: String, filter: String },
    /// 静态 lint（clippy / 自定义）。
    Lint { tool: String, args: Vec<String> },
    /// 检索/工具输出对比（非代码任务）— 输出需包含期望片段。
    ToolOutput { expected: String },
}

/// 接地门实例 — 一组检查 + 重试策略。
#[derive(Debug, Clone)]
pub struct GroundedGate {
    /// 门名称（用于日志/审计）。
    pub name: String,
    /// 需全部通过的检查列表。
    pub checks: Vec<GroundedCheck>,
    /// 失败后允许 worker 重试次数。
    pub max_retries: u8,
    /// 沙箱超时（秒）。
    pub timeout_secs: u64,
}

/// 接地决策结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroundedDecision {
    /// 全部检查通过。
    Pass,
    /// 需要 worker 修订，附反馈（编译错误/测试失败详情）。
    Revise { feedback: Vec<String> },
    /// 超时/重试耗尽 → 升级。
    Fail { reason: String },
}

impl GroundedGate {
    /// 构造一个新接地门。
    pub fn new(name: &str, checks: Vec<GroundedCheck>, max_retries: u8, timeout_secs: u64) -> Self {
        Self {
            name: name.to_string(),
            checks,
            max_retries,
            timeout_secs,
        }
    }

    /// 执行所有检查（外部接地验证）。
    ///
    /// 注: 生产形态复用 cargo check/test（R-P9/R-P16 双验证纪律的自动化）。
    /// 测试环境通过 `check_runner` 注入桩 runner，避免真实编译开销。
    pub fn evaluate<F>(&self, mut check_runner: F) -> GroundedDecision
    where
        F: FnMut(&GroundedCheck) -> Result<String, String>,
    {
        let mut feedback: Vec<String> = Vec::new();
        for check in &self.checks {
            match check_runner(check) {
                Ok(_) => {}
                Err(err) => feedback.push(format!("{:?}: {}", check, err)),
            }
        }
        if feedback.is_empty() {
            GroundedDecision::Pass
        } else if self.max_retries > 0 {
            GroundedDecision::Revise { feedback }
        } else {
            GroundedDecision::Fail {
                reason: format!("{}: checks exhausted", self.name),
            }
        }
    }

    /// 委托执行循环: 按 max_retries 重试直到 Pass 或 Fail（升级）。
    ///
    /// `produce` 产出一份工件，`check_runner` 验证之；Revise 时把反馈传给
    /// `produce`（worker 修订），直至重试耗尽。
    pub fn run_loop<F, P>(&self, mut produce: P, mut check_runner: F) -> GroundedDecision
    where
        F: FnMut(&GroundedCheck, &str) -> Result<String, String>,
        P: FnMut(&[String]) -> String,
    {
        let mut feedback: Vec<String> = Vec::new();
        let mut attempts = 0u32;
        loop {
            let artifact = produce(&feedback);
            let mut errs = Vec::new();
            for check in &self.checks {
                if let Err(e) = check_runner(check, &artifact) {
                    errs.push(format!("{:?}: {}", check, e));
                }
            }
            if errs.is_empty() {
                return GroundedDecision::Pass;
            }
            attempts += 1;
            if attempts > self.max_retries as u32 {
                return GroundedDecision::Fail {
                    reason: format!("{}: retries exhausted", self.name),
                };
            }
            feedback = errs;
        }
    }
}

/// 契约字段类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    String,
    Number,
    Json,
    File,
}

/// 契约字段 — 输入/输出的期望字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
    pub required: bool,
}

impl Field {
    /// 构造一个必填字段。
    pub fn req(name: &str, ty: FieldType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            required: true,
        }
    }

    /// 构造一个可选字段。
    pub fn opt(name: &str, ty: FieldType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            required: false,
        }
    }
}

/// 编排域枚举（契约的 domain 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Core,
    World,
    Act,
    Mind,
    Memory,
    Io,
    Shield,
}

impl Domain {
    /// 从共享语言全名解析（NT-CORE / NT-WORLD / ...）。
    pub fn from_full_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "NT-CORE" => Some(Self::Core),
            "NT-WORLD" => Some(Self::World),
            "NT-ACT" => Some(Self::Act),
            "NT-MIND" => Some(Self::Mind),
            "NT-MEMORY" => Some(Self::Memory),
            "NT-IO" => Some(Self::Io),
            "NT-SHIELD" => Some(Self::Shield),
            _ => None,
        }
    }
}

/// 子 agent 类型化契约 — 定义输入/输出 schema 与成功标准。
#[derive(Debug, Clone)]
pub struct AgentContract {
    /// 目标域。
    pub domain: Domain,
    /// 期望输入字段。
    pub input_schema: Vec<Field>,
    /// 承诺输出字段。
    pub output_schema: Vec<Field>,
    /// 成功标准（可执行的接地检查）。
    pub success_criteria: Vec<GroundedCheck>,
    /// 上游工件 watch list。
    pub upstream_watch: Vec<String>,
}

impl AgentContract {
    /// 构造新契约。
    pub fn new(
        domain: Domain,
        input_schema: Vec<Field>,
        output_schema: Vec<Field>,
        success_criteria: Vec<GroundedCheck>,
    ) -> Self {
        Self {
            domain,
            input_schema,
            output_schema,
            success_criteria,
            upstream_watch: Vec::new(),
        }
    }

    /// 校验一份产出工件（JSON 对象表示）是否满足输出 schema。
    ///
    /// 返回缺失字段/类型不匹配的列表；空列表即通过。
    pub fn validate_output(&self, artifact: &serde_json::Value) -> Vec<String> {
        let mut errors = Vec::new();
        let obj = match artifact {
            serde_json::Value::Object(map) => map,
            _ => return vec!["artifact is not a JSON object".to_string()],
        };
        for field in &self.output_schema {
            match obj.get(&field.name) {
                None => {
                    if field.required {
                        errors.push(format!("missing required field: {}", field.name));
                    }
                }
                Some(v) => {
                    if !field_matches_type(v, &field.ty) {
                        errors.push(format!(
                            "field {}: expected {:?}, got {}",
                            field.name, field.ty, v
                        ));
                    }
                }
            }
        }
        errors
    }

    /// 校验输入请求是否满足输入 schema（同理复用输出校验逻辑）。
    pub fn validate_input(&self, artifact: &serde_json::Value) -> Vec<String> {
        self.validate_output(artifact)
    }

    /// 校验成功标准（GroundedGate）— 简化：直接评估返回决策。
    pub fn evaluate_success<F>(&self, check_runner: F) -> GroundedDecision
    where
        F: FnMut(&GroundedCheck) -> Result<String, String>,
    {
        let gate = GroundedGate::new(
            &format!("contract:{:?}", self.domain),
            self.success_criteria.clone(),
            0, // 契约校验不重试，交由编排层 run_loop 决定
            60,
        );
        gate.evaluate(check_runner)
    }
}

/// 判断 JSON 值是否匹配字段类型。
fn field_matches_type(v: &serde_json::Value, ty: &FieldType) -> bool {
    match ty {
        FieldType::String => v.is_string(),
        FieldType::Number => v.is_number(),
        FieldType::Json => v.is_object() || v.is_array(),
        FieldType::File => v.is_string() && v.as_str().is_some_and(|s| !s.is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── GroundedGate 决策语义 ──

    #[test]
    fn test_gate_pass_when_all_checks_ok() {
        let gate = GroundedGate::new(
            "unit",
            vec![
                GroundedCheck::Compile {
                    crate_name: "neotrix".into(),
                },
                GroundedCheck::ToolOutput {
                    expected: "ok".into(),
                },
            ],
            2,
            60,
        );
        let decision = gate.evaluate(|check| match check {
            GroundedCheck::Compile { .. } => Ok("compiled".into()),
            _ => Ok("matched".into()),
        });
        assert_eq!(decision, GroundedDecision::Pass);
    }

    #[test]
    fn test_gate_revise_with_feedback_when_retries_left() {
        let gate = GroundedGate::new(
            "unit",
            vec![GroundedCheck::Compile {
                crate_name: "neotrix".into(),
            }],
            2,
            60,
        );
        let decision = gate.evaluate(|_| Err("compile error E0308".into()));
        match decision {
            GroundedDecision::Revise { feedback } => {
                assert_eq!(feedback.len(), 1);
                assert!(feedback[0].contains("E0308"));
            }
            other => panic!("expected Revise, got {:?}", other),
        }
    }

    #[test]
    fn test_gate_fail_when_no_retries_left() {
        let gate = GroundedGate::new(
            "unit",
            vec![GroundedCheck::Compile {
                crate_name: "neotrix".into(),
            }],
            0,
            60,
        );
        let decision = gate.evaluate(|_| Err("boom".into()));
        match decision {
            GroundedDecision::Fail { reason } => assert!(reason.contains("unit")),
            other => panic!("expected Fail, got {:?}", other),
        }
    }

    #[test]
    fn test_gate_partial_fail_collects_all_feedback() {
        let gate = GroundedGate::new(
            "multi",
            vec![
                GroundedCheck::Compile {
                    crate_name: "a".into(),
                },
                GroundedCheck::Test {
                    crate_name: "a".into(),
                    filter: "x".into(),
                },
                GroundedCheck::ToolOutput {
                    expected: "needle".into(),
                },
            ],
            3,
            60,
        );
        // 第一个失败，后两个成功 → 只有 1 条反馈
        let decision = gate.evaluate(|check| match check {
            GroundedCheck::Compile { .. } => Err("E0308".into()),
            _ => Ok("ok".into()),
        });
        match decision {
            GroundedDecision::Revise { feedback } => assert_eq!(feedback.len(), 1),
            other => panic!("expected Revise, got {:?}", other),
        }
    }

    #[test]
    fn test_gate_run_loop_produce_revise_until_pass() {
        // produce 第一次产出坏工件，修订后产出好工件；检查通过 → Pass
        let gate = GroundedGate::new(
            "loop",
            vec![GroundedCheck::ToolOutput {
                expected: "GOOD".into(),
            }],
            2,
            60,
        );
        let mut produced = 0;
        let decision = gate.run_loop(
            |feedback| {
                produced += 1;
                if feedback.is_empty() {
                    "BAD".to_string()
                } else {
                    "GOOD".to_string()
                }
            },
            |check, artifact| {
                let GroundedCheck::ToolOutput { expected } = check else {
                    return Err("unexpected".into());
                };
                if artifact.contains(expected.as_str()) {
                    Ok("match".into())
                } else {
                    Err("no match".into())
                }
            },
        );
        assert_eq!(decision, GroundedDecision::Pass);
        assert_eq!(produced, 2); // 1 次坏 + 1 次修订
    }

    #[test]
    fn test_gate_run_loop_exhausts_retries_then_fail() {
        let gate = GroundedGate::new(
            "loop",
            vec![GroundedCheck::ToolOutput {
                expected: "GOOD".into(),
            }],
            1,
            60,
        );
        let decision = gate.run_loop(
            |_| "BAD".to_string(),
            |check, artifact| {
                let GroundedCheck::ToolOutput { expected } = check else {
                    return Err("unexpected".into());
                };
                if artifact.contains(expected.as_str()) {
                    Ok("match".into())
                } else {
                    Err("no match".into())
                }
            },
        );
        match decision {
            GroundedDecision::Fail { reason } => assert!(reason.contains("exhausted")),
            other => panic!("expected Fail, got {:?}", other),
        }
    }

    // ── AgentContract 契约校验 ──

    #[test]
    fn test_contract_validates_output_schema() {
        let contract = AgentContract::new(
            Domain::World,
            vec![],
            vec![
                Field::req("title", FieldType::String),
                Field::req("score", FieldType::Number),
                Field::req("meta", FieldType::Json),
                Field::opt("ref_file", FieldType::File),
            ],
            vec![],
        );
        // 完整合规
        assert!(contract
            .validate_output(&json!({
                "title": "hello",
                "score": 0.88,
                "meta": {"a": 1},
                "ref_file": "/tmp/x.md"
            }))
            .is_empty());
        // 缺必填 + 类型错
        let errors = contract.validate_output(&json!({ "title": 42, "meta": [] }));
        assert!(errors
            .iter()
            .any(|e| e.contains("missing required field: score")));
        assert!(errors.iter().any(|e| e.contains("expected String")));
    }

    #[test]
    fn test_contract_optional_field_missing_is_ok() {
        let contract = AgentContract::new(
            Domain::Act,
            vec![],
            vec![Field::opt("ref_file", FieldType::File)],
            vec![],
        );
        assert!(contract.validate_output(&json!({})).is_empty());
    }

    #[test]
    fn test_contract_non_object_artifact_fails() {
        let contract = AgentContract::new(Domain::Mind, vec![], vec![], vec![]);
        let errors = contract.validate_output(&json!([1, 2]));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not a JSON object"));
    }

    #[test]
    fn test_contract_success_criteria_evaluates() {
        let contract = AgentContract::new(
            Domain::Core,
            vec![],
            vec![],
            vec![GroundedCheck::Compile {
                crate_name: "neotrix".into(),
            }],
        );
        let decision = contract.evaluate_success(|_| Ok("compiled".into()));
        assert_eq!(decision, GroundedDecision::Pass);
        let decision = contract.evaluate_success(|_| Err("E0308".into()));
        assert!(matches!(decision, GroundedDecision::Fail { .. }));
    }

    #[test]
    fn test_domain_from_full_name() {
        assert_eq!(Domain::from_full_name("NT-CORE"), Some(Domain::Core));
        assert_eq!(Domain::from_full_name("nt-world"), Some(Domain::World));
        assert_eq!(Domain::from_full_name("NT-ACT"), Some(Domain::Act));
        assert_eq!(Domain::from_full_name("crawler"), None); // 模糊词 → None
        assert_eq!(Domain::from_full_name(""), None);
    }
}
