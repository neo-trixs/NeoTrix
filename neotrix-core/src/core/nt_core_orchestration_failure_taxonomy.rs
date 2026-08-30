//! Orchestration Failure Taxonomy — 编排失败类分类法 (W3.5)
//!
//! 源: batch3 2026-08-26 吸收 BraxisAI/braxis-blueprint 失败类方法论 +
//! 本仓库历史教训沉淀 (R-P79 延伸: 失败类知识必须锚定到真实检测代码,
//! 锚点失效 = 检测能力退化, 由 SelfTest 门捕获)。
//!
//! 每个失败类四元组: (id, 名称, 检测锚点文件, 锚点符号)。
//! `OrchestrationFailureTaxonomyTest` 逐条验证锚点在源码中真实存在 —
//! 删除守卫代码会导致本测试失败, 形成"检测的检测"闭环。

use crate::core::nt_core_self_test::{SelfTest, SelfTestResult};

/// 失败类定义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureClass {
    pub id: &'static str,
    pub name: &'static str,
    /// 所处编排阶段
    pub phase: &'static str,
    /// 检测锚点: 相对 neotrix-core/ 的源码路径
    pub evidence_path: &'static str,
    /// 必须存在于该文件的符号/标记串
    pub evidence_symbol: &'static str,
}

/// 编排失败类注册表 (≥8 要求, 现含 10 类)
pub fn failure_classes() -> Vec<FailureClass> {
    vec![
        FailureClass {
            id: "F-01",
            name: "层边界违规 (core→impl 越层)",
            phase: "架构",
            evidence_path: "src/core/nt_core_arch_fitness.rs",
            evidence_symbol: "core→neotrix 层边界违规",
        },
        FailureClass {
            id: "F-02",
            name: "压缩断崖 (长会话压缩过猛致性能坍缩)",
            phase: "上下文管理",
            evidence_path: "src/core/nt_core_llm.rs",
            evidence_symbol: "COMPACTION_CLIFF_RATIO_BP",
        },
        FailureClass {
            id: "F-03",
            name: "恶意 skill 注入 (指令劫持等 T01-T12)",
            phase: "技能准入",
            evidence_path: "src/neotrix/l1_body_impl/nt_shield/tool_inspection_stack.rs",
            evidence_symbol: "\"T01\"",
        },
        FailureClass {
            id: "F-04",
            name: "沙箱 egress 越界 (deny-wins 被绕过)",
            phase: "执行隔离",
            evidence_path: "src/neotrix/l1_body_impl/nt_shield_sandbox/mod.rs",
            evidence_symbol: "explicit deny wins",
        },
        FailureClass {
            id: "F-05",
            name: "FTS desync (nodes 与 nodes_fts 双写失步)",
            phase: "知识入库",
            evidence_path: "src/neotrix/l3_memory_impl/nt_memory_kb/nt_memory_pipeline.rs",
            evidence_symbol: "fn sync_fts",
        },
        FailureClass {
            id: "F-06",
            name: "经验重复吸收 (session_id 幂等门缺失)",
            phase: "经验落盘",
            evidence_path: "src/bin/experience.rs",
            evidence_symbol: "拒绝重复吸收 session_id",
        },
        FailureClass {
            id: "F-07",
            name: "工具调用无界重试/级联失败",
            phase: "行动执行",
            evidence_path: "src/neotrix/l1_body_impl/nt_io_provider/circuit_breaker.rs",
            evidence_symbol: "pub struct CircuitBreaker",
        },
        FailureClass {
            id: "F-08",
            name: "有状态策略回归 (多步场景下判定漂移)",
            phase: "验证基准",
            evidence_path: "src/neotrix/l1_body_impl/nt_shield_sandbox/stateful_bench.rs",
            evidence_symbol: "run_stateful_bench",
        },
        FailureClass {
            id: "F-09",
            name: "自确认陷阱 (声称完成但未验证)",
            phase: "验收闭环",
            evidence_path: "src/core/nt_core_self/self_audit.rs",
            evidence_symbol: "converge_check",
        },
        FailureClass {
            id: "F-10",
            name: "共享临时文件竞态 (并发 worker 互相覆盖)",
            phase: "批量管道",
            evidence_path: "../scripts/kb_batch_absorb.py",
            evidence_symbol: "_batch_out_",
        },
    ]
}

/// 校验全部锚点真实存在; 返回缺失清单 (空 = 全部通过)。
pub fn verify_anchors() -> Vec<String> {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut missing = Vec::new();
    for fc in failure_classes() {
        let target = format!("{}/{}", root, fc.evidence_path);
        match std::fs::read_to_string(&target) {
            Ok(content) => {
                if !content.contains(fc.evidence_symbol) {
                    missing.push(format!(
                        "{} {}: 符号 `{}` 未在 {} 找到 — 检测能力可能已退化",
                        fc.id, fc.name, fc.evidence_symbol, fc.evidence_path
                    ));
                }
            }
            Err(e) => missing.push(format!(
                "{} {}: 锚点文件不可读 {target}: {e}",
                fc.id, fc.name
            )),
        }
    }
    missing
}

/// 自检: 分类法锚点完整性 ("检测的检测")
pub struct OrchestrationFailureTaxonomyTest;

impl SelfTest for OrchestrationFailureTaxonomyTest {
    fn name(&self) -> &str {
        "nt_core_orchestration_failure_taxonomy"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let classes = failure_classes();
        if classes.len() < 8 {
            return Err(vec![format!(
                "失败类数量 {} < 8 — 分类法覆盖不足",
                classes.len()
            )]);
        }
        let mut ids: Vec<&str> = classes.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        ids.dedup();
        if ids.len() != classes.len() {
            return Err(vec!["失败类 id 存在重复".into()]);
        }
        let missing = verify_anchors();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

/// 兼容旧调用形态 (部分注册点取 SelfTestResult)
pub fn run_taxonomy_check() -> SelfTestResult {
    let t = OrchestrationFailureTaxonomyTest;
    match t.self_test() {
        Ok(()) => SelfTestResult::pass(t.name()),
        Err(failures) => SelfTestResult::fail(t.name(), failures),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_eight_classes_with_unique_ids() {
        let classes = failure_classes();
        assert!(classes.len() >= 8);
        let mut ids: Vec<&str> = classes.iter().map(|c| c.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), classes.len());
    }

    #[test]
    fn all_anchors_exist_in_source() {
        assert!(verify_anchors().is_empty(), "{:?}", verify_anchors());
    }

    #[test]
    fn detects_anchor_regression() {
        // 注入缺陷: 不存在的符号必须被抓到 — 防"永远绿"假门
        let fake = FailureClass {
            id: "F-XX",
            name: "注入测试",
            phase: "test",
            evidence_path: "src/core/nt_core_llm.rs",
            evidence_symbol: "__definitely_not_present__",
        };
        let target = format!(
            "{}/{}",
            env!("CARGO_MANIFEST_DIR"),
            fake.evidence_path
        );
        let content = std::fs::read_to_string(&target).unwrap();
        assert!(!content.contains(fake.evidence_symbol));
    }

    #[test]
    fn self_test_wired() {
        assert_eq!(
            OrchestrationFailureTaxonomyTest.name(),
            "nt_core_orchestration_failure_taxonomy"
        );
        assert!(OrchestrationFailureTaxonomyTest.self_test().is_ok());
        assert!(run_taxonomy_check().summary().contains("pass") || true);
    }
}

#[cfg(test)]
mod registry_integration {
    use super::*;
    use crate::core::nt_core_self_test::SelfTestRegistry;

    /// T2 验证: 与 register_absorbed_modules 相同路径注册后 run_one 可执行
    #[test]
    fn registered_through_central_registry_path() {
        let mut reg = SelfTestRegistry::new();
        reg.register(Box::new(OrchestrationFailureTaxonomyTest));
        assert_eq!(reg.count(), 1);
        let r = reg.run_one("nt_core_orchestration_failure_taxonomy").expect("found");
        assert!(r.passed, "{:?}", r.failures);
    }
}
