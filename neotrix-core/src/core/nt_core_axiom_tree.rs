//! NT-CORE 架构公理推演树 — 对标网文"先立世界观公理再推演"（乌贼方法论）
//!
//! 网文顶级作者（爱潜水的乌贼）的核心方法：先定世界观公理（"万物皆来自最初造物主"），
//! 再推演出力量体系（非凡特性定律 → 序列化 + 扮演法）。世界观先于升级。
//!
//! 本模块把 NeoTrix 的架构公理（R-P1 零 unsafe / 指针守恒 / Dark Forest / The Spice Must Flow）
//! 显式化为"推演树"：公理 → 推导定律 → 模块约束，让架构决策从"经验驱动"升级为"公理推演驱动"。
//!
//! 推演树结构（对标网文设定集）：
//!   Axiom (公理) → DerivedLaw (定律) → ModuleConstraint (模块约束)
//!   每条公理有"违反后果"（对标网文设定违反的代价：境界跌落/反噬）

/// 架构公理（对标网文"世界观底层铁律"）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axiom {
    pub id: &'static str,
    pub name: &'static str,
    pub statement: &'static str,
    /// 违反后果（对标网文"设定违反的代价"）
    pub violation_consequence: &'static str,
}

/// 从公理推导出的定律（对标"非凡特性定律"）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedLaw {
    pub id: &'static str,
    pub axiom_id: &'static str,
    pub name: &'static str,
    pub statement: &'static str,
}

/// 定律对具体模块的约束（对标"序列化+扮演法"落地）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleConstraint {
    pub id: &'static str,
    pub law_id: &'static str,
    pub module: &'static str,
    pub constraint: &'static str,
}

/// 架构公理推演树
#[derive(Debug, Clone, Default)]
pub struct AxiomTree {
    pub axioms: Vec<Axiom>,
    pub laws: Vec<DerivedLaw>,
    pub constraints: Vec<ModuleConstraint>,
}

impl AxiomTree {
    /// 构建完整推演树（公理 → 定律 → 约束）
    pub fn build() -> Self {
        let axioms = vec![
            Axiom {
                id: "AX-1",
                name: "零 unsafe 铁律",
                statement: "核心代码禁止 unsafe（R-P1）",
                violation_consequence: "内存安全边界失守，整个系统可信度崩塌（对标：设定崩坏）",
            },
            Axiom {
                id: "AX-2",
                name: "指针守恒",
                statement: "AGENTS.md 不含任何 per-cycle 增长区，经验只存 KB",
                violation_consequence: "指引文档膨胀，门禁拒绝写入（对标：设定库失控）",
            },
            Axiom {
                id: "AX-3",
                name: "Dark Forest 生存法则",
                statement: "每个模块必须编译 + 测试 + 有消费者，否则删除",
                violation_consequence: "死代码累积，系统熵增（对标：无用的设定被遗忘）",
            },
            Axiom {
                id: "AX-4",
                name: "The Spice Must Flow",
                statement: "数据管线必须输入→变换→输出无断点",
                violation_consequence: "数据断流，模块饿死（对标：因果链断裂）",
            },
            Axiom {
                id: "AX-5",
                name: "接线门（R-P79）",
                statement: "外部技术吸收必须同 session 接线到生产路径",
                violation_consequence: "延期死代码，吸收无价值（对标：考据未融入设定）",
            },
        ];

        let laws = vec![
            DerivedLaw {
                id: "LW-1.1",
                axiom_id: "AX-1",
                name: "安全边界定律",
                statement: "所有核心模块必须 #![forbid(unsafe_code)]",
            },
            DerivedLaw {
                id: "LW-1.2",
                axiom_id: "AX-1",
                name: "证据签名定律",
                statement: "所有安全决策必须带签名与证据链",
            },
            DerivedLaw {
                id: "LW-2.1",
                axiom_id: "AX-2",
                name: "KB 唯一存储定律",
                statement: "cycle 指针/摘要/全文只存 KB experience hub",
            },
            DerivedLaw {
                id: "LW-2.2",
                axiom_id: "AX-2",
                name: "门禁执行定律",
                statement: "AGENTS.md 结构由插件门禁校验，不依赖自律",
            },
            DerivedLaw {
                id: "LW-3.1",
                axiom_id: "AX-3",
                name: "消费者定律",
                statement: "每个模块必须有生产路径消费者",
            },
            DerivedLaw {
                id: "LW-3.2",
                axiom_id: "AX-3",
                name: "编译测试定律",
                statement: "每个模块必须编译 + 测试通过",
            },
            DerivedLaw {
                id: "LW-4.1",
                axiom_id: "AX-4",
                name: "数据流定律",
                statement: "每个模块必须有明确输入→变换→输出",
            },
            DerivedLaw {
                id: "LW-4.2",
                axiom_id: "AX-4",
                name: "断点检测定律",
                statement: "数据管线断点必须被检测并报告",
            },
            DerivedLaw {
                id: "LW-5.1",
                axiom_id: "AX-5",
                name: "同会话接线定律",
                statement: "吸收的技术必须同 session 接线生产路径",
            },
            DerivedLaw {
                id: "LW-5.2",
                axiom_id: "AX-5",
                name: "强化不平行定律",
                statement: "吸收强化现有节点，禁止平行适配器（R-P42）",
            },
        ];

        let constraints = vec![
            ModuleConstraint { id: "MC-1.1.1", law_id: "LW-1.1", module: "nt_core_*", constraint: "每个核心 crate 顶部 #![forbid(unsafe_code)]" },
            ModuleConstraint { id: "MC-1.2.1", law_id: "LW-1.2", module: "nt_shield", constraint: "SafetyDecision 带 kernel_version + 证据签名" },
            ModuleConstraint { id: "MC-2.1.1", law_id: "LW-2.1", module: "nt_memory", constraint: "experience 命名空间唯一存储路径" },
            ModuleConstraint { id: "MC-2.2.1", law_id: "LW-2.2", module: "AGENTS.md", constraint: "agents-guard.js 门禁校验结构" },
            ModuleConstraint { id: "MC-3.1.1", law_id: "LW-3.1", module: "nt_world", constraint: "每个爬取模块有消费方" },
            ModuleConstraint { id: "MC-3.2.1", law_id: "LW-3.2", module: "全部模块", constraint: "cargo check + cargo test 双验证" },
            ModuleConstraint { id: "MC-4.1.1", law_id: "LW-4.1", module: "SEAL pipeline", constraint: "每阶段输入→变换→输出显式" },
            ModuleConstraint { id: "MC-4.2.1", law_id: "LW-4.2", module: "converge_check", constraint: "Phase-0 检测断点" },
            ModuleConstraint { id: "MC-5.1.1", law_id: "LW-5.1", module: "nt_mind", constraint: "吸收同 session 接线" },
            ModuleConstraint { id: "MC-5.2.1", law_id: "LW-5.2", module: "nt_mind", constraint: "强化现有节点不建平行模块" },
        ];

        Self { axioms, laws, constraints }
    }

    /// 从公理推演其定律（对标乌贼"一条规则推演整个社会"）
    pub fn derive(&self, axiom_id: &str) -> Vec<&DerivedLaw> {
        self.laws.iter().filter(|l| l.axiom_id == axiom_id).collect()
    }

    /// 从定律推演模块约束
    pub fn constraints_for(&self, law_id: &str) -> Vec<&ModuleConstraint> {
        self.constraints.iter().filter(|c| c.law_id == law_id).collect()
    }

    /// 公理 → 完整推演链（定律 + 约束）
    pub fn trace(&self, axiom_id: &str) -> Vec<(&DerivedLaw, Vec<&ModuleConstraint>)> {
        self.derive(axiom_id)
            .into_iter()
            .map(|law| (law, self.constraints_for(law.id)))
            .collect()
    }

    /// 检查模块是否满足某公理推导的约束（对标"设定检查"）
    pub fn check_module(&self, module: &str) -> Vec<&ModuleConstraint> {
        self.constraints
            .iter()
            .filter(|c| c.module == module || c.module == "全部模块")
            .collect()
    }

    /// 打印完整推演树（人类可读）
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("=== 架构公理推演树 (公理 → 定律 → 模块约束) ===\n");
        for axiom in &self.axioms {
            out.push_str(&format!(
                "\n[{}] {} — {}\n  违反后果: {}\n",
                axiom.id, axiom.name, axiom.statement, axiom.violation_consequence
            ));
            for (law, cons) in self.trace(axiom.id) {
                out.push_str(&format!("  └─ {} {} — {}\n", law.id, law.name, law.statement));
                for c in cons {
                    out.push_str(&format!("       └─ {} [{}] {}\n", c.id, c.module, c.constraint));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_builds_complete() {
        let tree = AxiomTree::build();
        assert_eq!(tree.axioms.len(), 5);
        assert_eq!(tree.laws.len(), 10);
        assert_eq!(tree.constraints.len(), 10);
    }

    #[test]
    fn test_every_axiom_derives_laws() {
        let tree = AxiomTree::build();
        for axiom in &tree.axioms {
            let laws = tree.derive(axiom.id);
            assert!(!laws.is_empty(), "公理 {} 必须推导出定律", axiom.id);
            for law in laws {
                let cons = tree.constraints_for(law.id);
                assert!(!cons.is_empty(), "定律 {} 必须有模块约束", law.id);
            }
        }
    }

    #[test]
    fn test_trace_chain_integrity() {
        let tree = AxiomTree::build();
        // 对标乌贼: 从"造物主"公理推演"序列体系" — 每条公理推演链完整
        let chain = tree.trace("AX-1");
        assert_eq!(chain.len(), 2, "AX-1 应推导 2 条定律");
        assert!(chain.iter().all(|(_, cons)| !cons.is_empty()));
    }

    #[test]
    fn test_check_module_constraints() {
        let tree = AxiomTree::build();
        let shield_cons = tree.check_module("nt_shield");
        assert!(!shield_cons.is_empty(), "nt_shield 应有约束");
        // 全部模块约束对所有模块生效
        let all_cons = tree.check_module("nt_memory");
        assert!(all_cons.iter().any(|c| c.module == "全部模块"));
    }

    #[test]
    fn test_render_contains_all_axioms() {
        let tree = AxiomTree::build();
        let rendered = tree.render();
        for axiom in &tree.axioms {
            assert!(rendered.contains(axiom.id), "渲染应包含 {}", axiom.id);
        }
    }
}