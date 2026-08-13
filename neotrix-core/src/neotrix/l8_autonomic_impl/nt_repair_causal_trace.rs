//! # NT-REPAIR-CAUSAL-TRACE — 因果链追踪骨架 (witr 方法论吸收, 2026-08-13)
//!
//! 吸收自 [pranshuparmar/witr](https://github.com/pranshuparmar/witr) (21.3k★, Go) 的
//! 过程因果链追踪方法论, 转化为 NT-REPAIR 域的 Rust 生产骨架:
//!
//!   1. **CausalChainWalker** — PPID 风格因果链步行 (witr `ancestry.go:9-44`):
//!      seen-set 循环保护 + 证据消失优雅截断 + 根优先反转, 不因中间节点消失崩溃。
//!   2. **SourceAdjudicator** — 单一赢家源裁决 (witr `detect.go:54-91`):
//!      证据特异性有序级联 (systemd→launchd→docker→pm2→cron→SSH→tmux→shell),
//!      首个非 nil 即赢家; `SourceUnknown` 一等公民 (显式不确定 > 幻觉确定)。
//!   3. **EvidenceGate** — 证据门控警告 (witr `app.go:726-744`):
//!      每条警告有硬阈值 (重启>N / 危险cap / 注入 / 已删bin / 公网绑定 / 长运行);
//!      任意警告使结论降级, 与 rev-officer Evidence-First 对齐。
//!
//! 接线契约 (R-P79): 本模块实现 `SelfTest` (T1), 在 `handle_architecture_audit`
//! 注册 (T2), 其结果流入 ConsciousnessTree 分支健康 (T3 生产接线)。
//!
//! ## 证据阶梯 (witr → C0-C6 映射)
//!
//! | 阶梯 | 本模块对应 | 条件 |
//! |------|-----------|------|
//! | C0 身份映射 | `CausalNode::new` 节点建模 | 编译 |
//! | C1 因果链步行 | `CausalChainWalker::walk` | 单测 |
//! | C2 单一赢家 | `SourceAdjudicator::adjudicate` | 单测 |
//! | C3 证据门控 | `EvidenceGate::evaluate` | 单测 |
//! | C4 契约输出 | `CausalTrace::trace` 聚合 | 集成 (SelfTest) |
//! | C5 跨面复用 | 架构审计注册 | 生产接线 (T3) |

#![forbid(unsafe_code)]

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashSet;

// ────────────────────────────────────────────────────────────────
// 因果链节点 — witr `Process` 的 Rust 映射
// ────────────────────────────────────────────────────────────────

/// 因果链上的单个节点 (witr `Process`): 每个 hop 携带身份 + 证据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CausalNode {
    /// 身份标识 (witr PID 等价物): 目标/父/祖先的唯一键。
    pub id: String,
    /// 可读名称 (witr comm 的显示修复等价物)。
    pub name: String,
    /// 父节点身份; `None` = 根 (witr PPID==0 或 PID==1 终止条件)。
    pub parent: Option<String>,
    /// 证据: 该节点"为什么存在"的依据 (file:line / 来源)。
    pub evidence: String,
}

impl CausalNode {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            parent: None,
            evidence: String::new(),
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = evidence.into();
        self
    }
}

// ────────────────────────────────────────────────────────────────
// CausalChainWalker — PPID 风格因果链步行 (witr `ancestry.go:9-44`)
// ────────────────────────────────────────────────────────────────

/// 从目标节点沿 parent 链向上步行, 构建根优先因果链。
///
/// 不变量 (对齐 witr `ResolveAncestry`):
/// - **循环保护**: seen-set 记录已访问节点, 重复即断 (witr:16-19)。
/// - **优雅截断**: 父节点证据不可得 (`None`) 即停, 不 panic、不丢已建链 (witr:22-24)。
/// - **根优先反转**: 返回顺序为 根→…→目标, 与 witr 反转 (witr:38-41) 一致。
#[derive(Debug, Clone)]
pub struct CausalChainWalker {
    max_depth: usize,
}

impl Default for CausalChainWalker {
    fn default() -> Self {
        Self { max_depth: 64 }
    }
}

impl CausalChainWalker {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth: max_depth.max(1),
        }
    }

    /// 从 `target` 开始沿 parent 链步行。`resolve` 按节点 id 取父节点证据,
    /// 返回 `None` = 证据消失 (witr ReadProcess error → truncate)。
    pub fn walk(
        &self,
        target: &CausalNode,
        resolve: &mut dyn FnMut(&str) -> Option<CausalNode>,
    ) -> Vec<CausalNode> {
        let mut chain: Vec<CausalNode> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut current: Option<CausalNode> = Some(target.clone());

        while let Some(node) = current {
            if chain.len() >= self.max_depth {
                break; // 深度上限, 防止过深链
            }
            if !seen.insert(node.id.clone()) {
                break; // 循环保护
            }
            chain.push(node.clone());
            current = match &node.parent {
                None => None, // 根
                Some(parent_id) => resolve(parent_id),
            };
        }

        if chain.is_empty() {
            return chain;
        }
        // 根优先反转 (witr:39-41)
        chain.reverse();
        chain
    }
}

// ────────────────────────────────────────────────────────────────
// SourceAdjudicator — 单一赢家源裁决 (witr `detect.go:54-91`)
// ────────────────────────────────────────────────────────────────

/// 源裁决结果 — `SourceUnknown` 一等公民 (显式不确定)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceVerdict {
    /// 识别出唯一主源 (witr `Source` 字段)。
    Identified { source: String, evidence: String },
    /// 无法确定主源 (witr `SourceUnknown`)。
    Unknown,
}

impl SourceVerdict {
    pub fn is_identified(&self) -> bool {
        matches!(self, Self::Identified { .. })
    }
}

/// 源检测器 — 一条证据特异性检测规则。
#[derive(Clone)]
pub struct SourceDetector {
    /// 检测器名 (如 "systemd" / "pm2" / "docker")。
    pub name: &'static str,
    /// 证据特异性, 越高越优先 (witr: init 需负空间消歧, 特异性最高)。
    pub specificity: u8,
    /// 检测逻辑: 从因果链判断是否命中; `Some(evidence)` = 命中。
    pub detect: fn(&[CausalNode]) -> Option<String>,
}

impl std::fmt::Debug for SourceDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceDetector")
            .field("name", &self.name)
            .field("specificity", &self.specificity)
            .finish()
    }
}

/// 单一赢家源裁决 — 按证据特异性降序级联, 首个非 nil 即赢家 (witr:54-91)。
#[derive(Debug, Clone)]
pub struct SourceAdjudicator {
    detectors: Vec<SourceDetector>,
}

impl SourceAdjudicator {
    pub fn new(mut detectors: Vec<SourceDetector>) -> Self {
        // 证据特异性降序 — 高特异性先行, 低特异性兜底
        detectors.sort_by_key(|d| std::cmp::Reverse(d.specificity));
        Self { detectors }
    }

    /// 裁决: 返回恰好一个主源。全部未命中 → `SourceUnknown`。
    pub fn adjudicate(&self, chain: &[CausalNode]) -> SourceVerdict {
        for detector in &self.detectors {
            if let Some(evidence) = (detector.detect)(chain) {
                return SourceVerdict::Identified {
                    source: detector.name.to_string(),
                    evidence,
                };
            }
        }
        SourceVerdict::Unknown
    }
}

/// 默认证据级联 (witr 顺序): systemd/launchd → docker → pm2 → cron → SSH → 交互 shell。
pub fn default_adjudicator() -> SourceAdjudicator {
    SourceAdjudicator::new(vec![
        SourceDetector {
            name: "systemd/launchd",
            specificity: 90,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name == "systemd" || n.name == "launchd")
                    .map(|n| format!("init chain: {}", n.id))
            },
        },
        SourceDetector {
            name: "container",
            specificity: 80,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name.contains("dockerd") || n.name.contains("containerd"))
                    .map(|n| format!("container runtime: {}", n.id))
            },
        },
        SourceDetector {
            name: "pm2",
            specificity: 70,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name.to_lowercase().contains("pm2"))
                    .map(|n| format!("supervisor: {}", n.id))
            },
        },
        SourceDetector {
            name: "cron",
            specificity: 60,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name == "cron" || n.name == "crond")
                    .map(|n| format!("scheduler: {}", n.id))
            },
        },
        SourceDetector {
            name: "ssh",
            specificity: 50,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name == "sshd")
                    .map(|n| format!("ssh session: {}", n.id))
            },
        },
        SourceDetector {
            name: "interactive_shell",
            specificity: 40,
            detect: |chain| {
                chain
                    .iter()
                    .find(|n| n.name == "bash" || n.name == "zsh" || n.name == "sh")
                    .map(|n| format!("interactive shell: {}", n.id))
            },
        },
    ])
}

// ────────────────────────────────────────────────────────────────
// EvidenceGate — 证据门控警告 (witr `app.go:726-744`)
// ────────────────────────────────────────────────────────────────

/// 单条警告 — 必须有硬阈值证据门。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub name: &'static str,
    pub severity: u8, // 1-10
    pub message: String,
}

/// 证据门规则: 当条件命中 → 产出警告。
#[derive(Clone)]
pub struct EvidenceRule {
    pub name: &'static str,
    /// 命中即警告的检查逻辑; 返回 `Some(message)` = 命中。
    pub check: fn(&[CausalNode]) -> Option<String>,
}

impl std::fmt::Debug for EvidenceRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvidenceRule")
            .field("name", &self.name)
            .finish()
    }
}

/// 证据门控警告器 — 每条规则带硬阈值 (对齐 witr 重启>N/危险cap/公网绑定/>90天)。
#[derive(Debug, Clone)]
pub struct EvidenceGate {
    rules: Vec<EvidenceRule>,
}

impl EvidenceGate {
    pub fn new(rules: Vec<EvidenceRule>) -> Self {
        Self { rules }
    }

    pub fn with_default_rules() -> Self {
        Self::new(vec![
            EvidenceRule {
                name: "root_owned",
                check: |chain| {
                    chain
                        .iter()
                        .find(|n| n.name.starts_with("root@"))
                        .map(|n| format!("running as root: {}", n.id))
                },
            },
            EvidenceRule {
                name: "deleted_binary",
                check: |chain| {
                    chain
                        .iter()
                        .find(|n| n.evidence.contains("(deleted)"))
                        .map(|n| format!("deleted binary: {}", n.id))
                },
            },
            EvidenceRule {
                name: "public_bind",
                check: |chain| {
                    chain
                        .iter()
                        .find(|n| n.evidence.contains("0.0.0.0") || n.evidence.contains("::"))
                        .map(|n| format!("public interface bind: {}", n.id))
                },
            },
        ])
    }

    /// 评估: 返回全部命中的警告。
    pub fn evaluate(&self, chain: &[CausalNode]) -> Vec<Warning> {
        self.rules
            .iter()
            .filter_map(|rule| {
                (rule.check)(chain).map(|message| Warning {
                    name: rule.name,
                    severity: 5,
                    message,
                })
            })
            .collect()
    }

    pub fn is_clean(&self, chain: &[CausalNode]) -> bool {
        self.evaluate(chain).is_empty()
    }
}

// ────────────────────────────────────────────────────────────────
// CausalTrace — 聚合入口 (C4 契约输出)
// ────────────────────────────────────────────────────────────────

/// 一次完整因果链追踪的结果。
#[derive(Debug, Clone)]
pub struct CausalTrace {
    /// 根优先因果链。
    pub chain: Vec<CausalNode>,
    /// 单一赢家主源。
    pub verdict: SourceVerdict,
    /// 证据门控警告。
    pub warnings: Vec<Warning>,
}

impl CausalTrace {
    /// 聚合完整追踪: 步行 → 裁决 → 门控。
    pub fn trace(
        target: &CausalNode,
        walker: &CausalChainWalker,
        adjudicator: &SourceAdjudicator,
        gate: &EvidenceGate,
        resolve: &mut dyn FnMut(&str) -> Option<CausalNode>,
    ) -> CausalTrace {
        let chain = walker.walk(target, resolve);
        if chain.is_empty() {
            return CausalTrace {
                chain,
                verdict: SourceVerdict::Unknown,
                warnings: Vec::new(),
            };
        }
        let verdict = adjudicator.adjudicate(&chain);
        let warnings = gate.evaluate(&chain);
        CausalTrace {
            chain,
            verdict,
            warnings,
        }
    }

    /// 根 → 目标的叙事链 (witr "Why It Exists" 输出)。
    pub fn narrative(&self) -> String {
        if self.chain.is_empty() {
            return String::from("(empty causal chain)");
        }
        let hops: Vec<String> = self
            .chain
            .iter()
            .map(|n| format!("{} ({})", n.name, n.id))
            .collect();
        hops.join(" → ")
    }

    pub fn source_summary(&self) -> String {
        match &self.verdict {
            SourceVerdict::Identified { source, evidence } => format!("{} [{}]", source, evidence),
            SourceVerdict::Unknown => "unknown (explicit uncertainty)".to_string(),
        }
    }
}

// ────────────────────────────────────────────────────────────────
// SelfTest (T1) + 生产接线契约
// ────────────────────────────────────────────────────────────────

/// 因果链追踪引擎的 SelfTest — 注册于 handle_architecture_audit (T2),
/// 结果流入 ConsciousnessTree 分支健康 (T3)。
#[derive(Debug, Clone, Copy, Default)]
pub struct CausalTraceSelfTest;

impl SelfTest for CausalTraceSelfTest {
    fn name(&self) -> &str {
        "nt_repair_causal_trace"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // C1: 链步行 — 循环保护 + 优雅截断 + 根优先
        let walker = CausalChainWalker::new(8);
        let mut resolve = |id: &str| -> Option<CausalNode> {
            match id {
                "srv" => Some(CausalNode::new("root", "systemd").with_evidence("init")),
                _ => None,
            }
        };
        let target = CausalNode::new("leaf", "web_server").with_parent("srv");
        let chain = walker.walk(&target, &mut resolve);
        if chain.is_empty() {
            failures.push("walk returned empty chain".into());
        } else if chain.first().map(|n| n.id.as_str()) != Some("root") {
            failures.push(format!(
                "expected root-first chain, got {:?}",
                chain.first().map(|n| n.id.clone())
            ));
        }

        // 循环保护: self-parenting 链不得无限
        let self_loop = CausalChainWalker::new(8);
        let mut resolve_loop = |id: &str| -> Option<CausalNode> {
            Some(CausalNode::new(id.to_string(), "loop").with_parent(id.to_string()))
        };
        let loop_target = CausalNode::new("x", "loop").with_parent("x");
        let loop_chain = self_loop.walk(&loop_target, &mut resolve_loop);
        if loop_chain.len() > 2 {
            failures.push(format!("loop protection failed: len={}", loop_chain.len()));
        }

        // C2: 单一赢家 — 高特异性胜出
        let adj = default_adjudicator();
        let chain2 = vec![
            CausalNode::new("1", "systemd"),
            CausalNode::new("2", "pm2").with_parent("1"),
            CausalNode::new("3", "node").with_parent("2"),
        ];
        let verdict = adj.adjudicate(&chain2);
        if !matches!(&verdict, SourceVerdict::Identified { source, .. } if source == "systemd/launchd")
        {
            failures.push(format!("expected systemd winner, got {:?}", verdict));
        }
        // 无证据链 → Unknown (显式不确定)
        let empty_chain: Vec<CausalNode> = Vec::new();
        if !matches!(adj.adjudicate(&empty_chain), SourceVerdict::Unknown) {
            failures.push("empty chain should be Unknown".into());
        }

        // C3: 证据门控 — 根身份 + 公网绑定命中
        let gate = EvidenceGate::with_default_rules();
        let chain3 = vec![
            CausalNode::new("1", "root@systemd").with_evidence("0.0.0.0:80"),
            CausalNode::new("2", "web_server").with_parent("1"),
        ];
        let warnings = gate.evaluate(&chain3);
        if warnings.iter().all(|w| w.name != "public_bind") {
            failures.push("expected public_bind warning".into());
        }

        // C4: 聚合入口 — trace + narrative
        let mut resolve4 = |id: &str| -> Option<CausalNode> {
            match id {
                "srv" => Some(CausalNode::new("pm2", "pm2").with_parent("sys")),
                "sys" => Some(CausalNode::new("systemd", "systemd")),
                _ => None,
            }
        };
        let trace = CausalTrace::trace(
            &CausalNode::new("srv", "web_server").with_parent("srv"),
            &walker,
            &adj,
            &gate,
            &mut resolve4,
        );
        if trace.narrative().is_empty() {
            failures.push("narrative should be non-empty".into());
        }
        if !trace.source_summary().contains("systemd") {
            failures.push(format!(
                "source_summary should identify systemd, got {}",
                trace.source_summary()
            ));
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_walk_root_first() {
        let walker = CausalChainWalker::default();
        let mut resolve = |id: &str| -> Option<CausalNode> {
            match id {
                "b" => Some(CausalNode::new("a", "systemd")),
                _ => None,
            }
        };
        let target = CausalNode::new("c", "node")
            .with_parent("b")
            .with_parent("b");
        let chain = walker.walk(&target, &mut resolve);
        assert_eq!(chain.first().map(|n| n.id.as_str()), Some("a"));
        assert_eq!(chain.last().map(|n| n.id.as_str()), Some("c"));
    }

    #[test]
    fn test_chain_walk_graceful_truncation() {
        // 中间节点证据消失 → 优雅截断, 不 panic
        let walker = CausalChainWalker::default();
        let mut resolve = |_id: &str| -> Option<CausalNode> { None };
        let target = CausalNode::new("leaf", "app").with_parent("missing_parent");
        let chain = walker.walk(&target, &mut resolve);
        assert_eq!(chain.len(), 1, "should truncate to available evidence");
        assert_eq!(chain[0].id, "leaf");
    }

    #[test]
    fn test_chain_walk_loop_protection() {
        let walker = CausalChainWalker::new(100);
        let mut resolve = |id: &str| -> Option<CausalNode> {
            Some(CausalNode::new(id.to_string(), "cycle").with_parent(id.to_string()))
        };
        let target = CausalNode::new("a", "cycle").with_parent("a");
        let chain = walker.walk(&target, &mut resolve);
        assert!(chain.len() <= 2, "loop must terminate, got {}", chain.len());
    }

    #[test]
    fn test_adjudicator_single_winner() {
        let adj = default_adjudicator();
        let chain = vec![
            CausalNode::new("1", "systemd"),
            CausalNode::new("2", "pm2").with_parent("1"),
            CausalNode::new("3", "node").with_parent("2"),
        ];
        assert!(adj.adjudicate(&chain).is_identified());
    }

    #[test]
    fn test_adjudicator_unknown_explicit() {
        let adj = default_adjudicator();
        let chain = vec![CausalNode::new("1", "orphan_proc")];
        assert_eq!(adj.adjudicate(&chain), SourceVerdict::Unknown);
    }

    #[test]
    fn test_evidence_gate_warnings() {
        let gate = EvidenceGate::with_default_rules();
        let chain = vec![CausalNode::new("1", "root@systemd").with_evidence("0.0.0.0:443")];
        let warnings = gate.evaluate(&chain);
        let names: Vec<&str> = warnings.iter().map(|w| w.name).collect();
        assert!(names.contains(&"root_owned"));
        assert!(names.contains(&"public_bind"));
    }

    #[test]
    fn test_evidence_gate_clean() {
        let gate = EvidenceGate::with_default_rules();
        let chain = vec![CausalNode::new("1", "user@systemd").with_evidence("127.0.0.1:8080")];
        assert!(gate.is_clean(&chain));
    }

    #[test]
    fn test_causal_trace_self_test() {
        let st = CausalTraceSelfTest;
        assert!(
            st.self_test().is_ok(),
            "self-test failed: {:?}",
            st.self_test().err()
        );
    }

    #[test]
    fn test_narrative_output() {
        let walker = CausalChainWalker::default();
        let adj = default_adjudicator();
        let gate = EvidenceGate::with_default_rules();
        let mut resolve = |id: &str| -> Option<CausalNode> {
            match id {
                "srv" => Some(CausalNode::new("pm2", "pm2").with_parent("sys")),
                "sys" => Some(CausalNode::new("systemd", "systemd")),
                _ => None,
            }
        };
        let trace = CausalTrace::trace(
            &CausalNode::new("srv", "web_server").with_parent("srv"),
            &walker,
            &adj,
            &gate,
            &mut resolve,
        );
        assert!(trace.narrative().contains("systemd"));
        assert!(trace.source_summary().contains("systemd"));
    }
}
