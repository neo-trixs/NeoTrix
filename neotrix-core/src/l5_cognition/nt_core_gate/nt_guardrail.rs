//! nt_guardrail — 忠实度审计/schema 检查/护栏报告.
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::nt_judge::JudgeInput;
use super::nt_types::{DebiasConfig, GuardAction};

/// 一条声明 + 其证据引用 (per-line citation 契约)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub text: String,
    pub evidence_refs: Vec<String>,
}

impl Claim {
    pub fn new(text: &str, refs: &[&str]) -> Self {
        Self {
            text: text.to_string(),
            evidence_refs: refs.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Schema 字段检查结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCheck {
    pub field: String,
    pub present: bool,
    pub detail: String,
}

/// 忠实度审计报告 — 逐句引用 + 集合差检测 (引用不存在于证据集 → 幻觉)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaithfulnessReport {
    pub claims_total: usize,
    pub grounded: usize,
    /// 引用了证据集中不存在条目的声明 (幻觉候选)
    pub fabricated: Vec<String>,
    pub grounding_ratio: f64,
}

impl FaithfulnessReport {
    /// 机械集合差: 每个 claim 的 evidence_refs ⊆ evidence_ids 才计为 grounded。
    pub fn audit(claims: &[Claim], evidence_ids: &[String]) -> Self {
        let evidence: HashSet<&str> = evidence_ids.iter().map(|s| s.as_str()).collect();
        let total = claims.len();
        let mut grounded = 0usize;
        let mut fabricated = Vec::new();
        for c in claims {
            let refs: HashSet<&str> = c.evidence_refs.iter().map(|s| s.as_str()).collect();
            if refs.is_empty() {
                fabricated.push(format!("claim '{}': 无证据引用", clip(&c.text, 60)));
            } else if refs.is_subset(&evidence) {
                grounded += 1;
            } else {
                let missing: Vec<&str> = refs.difference(&evidence).copied().collect();
                fabricated.push(format!(
                    "claim '{}': 引用不存在证据 {:?}",
                    clip(&c.text, 60),
                    missing
                ));
            }
        }
        let grounding_ratio = if total == 0 {
            0.0
        } else {
            grounded as f64 / total as f64
        };
        Self {
            claims_total: total,
            grounded,
            fabricated,
            grounding_ratio,
        }
    }

    pub fn is_grounded(&self, min_ratio: f64) -> bool {
        self.grounding_ratio >= min_ratio
    }

    /// 隔离幻觉声明 — 返回待人工复核的声明文本。
    pub fn quarantine(&self) -> Vec<String> {
        self.fabricated.clone()
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{}…", cut)
    }
}

/// 检查 JSON 输出是否包含全部必需字段 — 机械 schema 失败拦截。
pub fn check_schema_fields(required: &[&str], json: &str) -> Vec<SchemaCheck> {
    let parsed = serde_json::from_str::<serde_json::Value>(json);
    let mut checks = Vec::with_capacity(required.len());
    let obj = match parsed {
        Ok(serde_json::Value::Object(map)) => Some(map),
        Ok(serde_json::Value::Null) => None,
        Ok(_) => {
            return required
                .iter()
                .map(|f| SchemaCheck {
                    field: f.to_string(),
                    present: false,
                    detail: "输出不是 JSON 对象".to_string(),
                })
                .collect();
        }
        Err(e) => {
            return required
                .iter()
                .map(|f| SchemaCheck {
                    field: f.to_string(),
                    present: false,
                    detail: format!("JSON 解析失败: {}", e),
                })
                .collect();
        }
    };
    for f in required {
        let present = obj.as_ref().map(|m| m.contains_key(*f)).unwrap_or(false);
        checks.push(SchemaCheck {
            field: f.to_string(),
            present,
            detail: if present {
                "ok".to_string()
            } else {
                format!("缺少字段 {}", f)
            },
        });
    }
    checks
}

/// 护栏报告 — eval 引导运行: 拒绝低 grounding / 隔离幻觉 / 拦截 schema 失败。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailReport {
    pub action: GuardAction,
    pub reason: String,
    pub faithfulness: FaithfulnessReport,
    pub schema_failures: Vec<SchemaCheck>,
    pub grounding_failures: u64,
}

impl GuardrailReport {
    pub fn evaluate(input: &JudgeInput, cfg: &DebiasConfig) -> Self {
        let faith = FaithfulnessReport::audit(&input.claims, &input.evidence_ids);

        // 1) 机械 schema 拦截优先于一切
        if !input.schema_failures.is_empty() {
            return Self {
                action: GuardAction::Reject,
                reason: format!("schema 失败 {} 项", input.schema_failures.len()),
                faithfulness: faith,
                schema_failures: input.schema_failures.clone(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 2) 低 grounding → 拒绝 (reject low grounding)
        if !faith.is_grounded(cfg.grounding_min_ratio) {
            return Self {
                action: GuardAction::Reject,
                reason: format!(
                    "grounding {:.2} < {:.2}",
                    faith.grounding_ratio, cfg.grounding_min_ratio
                ),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 3) 工具声称成功但实际失败 → 拒绝 (轨迹撒谎, 无证据换不来信任)
        if input.grounding_failures > 0 {
            return Self {
                action: GuardAction::Reject,
                reason: format!("{} 次工具 grounding 失败", input.grounding_failures),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: input.grounding_failures,
            };
        }

        // 4) 幻觉声明 → 隔离, 扣留待人工 (quarantine fabrications)
        if !faith.fabricated.is_empty() {
            return Self {
                action: GuardAction::Quarantine,
                reason: format!("{} 条无证据声明被隔离", faith.fabricated.len()),
                faithfulness: faith,
                schema_failures: Vec::new(),
                grounding_failures: 0,
            };
        }

        Self {
            action: GuardAction::Allow,
            reason: "全部机械检查通过".to_string(),
            faithfulness: faith,
            schema_failures: Vec::new(),
            grounding_failures: 0,
        }
    }
}
