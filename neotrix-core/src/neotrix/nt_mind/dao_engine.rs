//! 道引擎 — 从万物的本源规则出发的知识架构
//!
//! 原理: 万物归于道 → 道生一(统一场) → 一生二(二元) → 二生三(三元) → 三生万物
//! 造物主视角: 先有规则(道),再有物质(一),再有相互作用(二),再有现象(三),再有万物
//!
//! 逆推链: 万物←现象←规律←相互作用←物质←能量←信息←道
//! 即: 所有互联网碎片信息,可逆推回少数几条本源规则

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 道的层级 — 从最抽象到最具体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DaoLevel {
    /// 道: 本源规则 (最底层抽象)
    Dao,
    /// 一: 统一场 (信息/能量/物质三位一体)
    OneUnified,
    /// 二: 二元性 (阴阳/时空/因果/对称)
    TwoDuality,
    /// 三: 三元组 (涌现三要素)
    ThreeEmergence,
    /// 万物: 具体现象/事物
    TenThousand,
}

/// 本源规则 — 道的具体表达
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoRule {
    pub id: String,
    pub name: String,
    pub layer: DaoLevel,
    pub formulation: String,        // 数学/逻辑表达
    pub manifestation: Vec<String>, // 在万物层面的体现
    pub connects_to: Vec<String>,   // 连接的其他规则
    pub inverse_path: Vec<String>,  // 逆推路径
}

/// 道引擎 — 从规则逆推到具体现象 / 从现象追溯到规则
pub struct DaoEngine {
    pub rules: Vec<DaoRule>,
    pub domain_to_rules: HashMap<String, Vec<String>>, // 领域→对应规则
    pub phenomena_to_rules: HashMap<String, Vec<String>>, // 现象→对应规则
}

impl Default for DaoEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DaoEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::init_rules(),
            domain_to_rules: Self::init_domain_mappings(),
            phenomena_to_rules: Self::init_phenomena_mappings(),
        }
    }

    fn init_rules() -> Vec<DaoRule> {
        vec![
            // ═══════ 道: 本源规则 (Layer 0) ═══════
            DaoRule {
                id: "DAO-00".into(),
                name: "道即本源".into(),
                layer: DaoLevel::Dao,
                formulation: "道可道,非常道;名可名,非常名. 万物生于有,有生于无.".into(),
                manifestation: vec!["一切存在的前提".into(), "不可被完全描述的奠基性实在".into()],
                connects_to: vec!["DAO-01".into(), "DAO-02".into()],
                inverse_path: vec!["万物 → 规则 → 相互作用 → 物质能量 → 道".into()],
            },
            // ═══════ 一: 统一场 (Layer 1) ═══════
            DaoRule {
                id: "DAO-01".into(),
                name: "信息=能量=物质 三位一体".into(),
                layer: DaoLevel::OneUnified,
                formulation: "E=mc² (质能等价); S=k·lnW (信息熵); 信息是物理的(Landauer原理)"
                    .into(),
                manifestation: vec![
                    "质量与能量可转换".into(),
                    "信息擦除需耗能(Landauer极限)".into(),
                    "宇宙总能量守恒".into(),
                    "信息不能凭空产生或消灭".into(),
                ],
                connects_to: vec!["DAO-10".into(), "DAO-11".into(), "DAO-12".into()],
                inverse_path: vec!["计算机→晶体管→半导体→量子力学→E=mc²".into()],
            },
            // ═══════ 二: 二元性 (Layer 2) ═══════
            DaoRule {
                id: "DAO-10".into(),
                name: "时空二元: 存在与变化的舞台".into(),
                layer: DaoLevel::TwoDuality,
                formulation: "ds² = -c²dt² + dx² + dy² + dz² (闵可夫斯基时空)".into(),
                manifestation: vec![
                    "任何事件都有时空坐标".into(),
                    "光速是宇宙速度极限".into(),
                    "引力=时空弯曲".into(),
                    "因果律:原因必须先于结果".into(),
                ],
                connects_to: vec!["DAO-20".into(), "DAO-21".into()],
                inverse_path: vec!["GPS→相对论修正→光速不变→闵可夫斯基时空→时空二元".into()],
            },
            DaoRule {
                id: "DAO-11".into(),
                name: "阴阳二元: 对立统一的普遍性".into(),
                layer: DaoLevel::TwoDuality,
                formulation: "∀x, ∃¬x (任何事物都有其对立面); 对立面互相依存,互相转化".into(),
                manifestation: vec![
                    "正负电荷".into(),
                    "物质-反物质".into(),
                    "生-死".into(),
                    "秩序-混沌".into(),
                    "0-1(二进制)".into(),
                    "正-负(数学)".into(),
                ],
                connects_to: vec!["DAO-20".into(), "DAO-22".into()],
                inverse_path: vec!["计算机→二进制→阴阳→对立统一规律".into()],
            },
            DaoRule {
                id: "DAO-12".into(),
                name: "因果二元: 万物皆有因果".into(),
                layer: DaoLevel::TwoDuality,
                formulation: "P(A→B) = P(B|A)·P(A) 原因的概率决定结果的概率".into(),
                manifestation: vec![
                    "牛顿力学(因果决定)".into(),
                    "量子力学(概率因果)".into(),
                    "经济学(供需因果)".into(),
                    "历史学(事件因果链)".into(),
                ],
                connects_to: vec!["DAO-20".into(), "DAO-23".into()],
                inverse_path: vec!["机器学习→统计因果→相关性→因果性→因果律".into()],
            },
            // ═══════ 三: 涌现 (Layer 3) ═══════
            DaoRule {
                id: "DAO-20".into(),
                name: "三生万物: 涌现的普遍机制".into(),
                layer: DaoLevel::ThreeEmergence,
                formulation: "当N个简单单元按规则相互作用时,出现单个单元不具备的新性质".into(),
                manifestation: vec![
                    "水分子(H₂O)不湿,但大量水分子集体涌现出'湿性'".into(),
                    "单个神经元不思考,但860亿个神经元涌现出意识".into(),
                    "单个人类不文明,但80亿人涌现出文明".into(),
                ],
                connects_to: vec!["DAO-30".into(), "DAO-31".into(), "DAO-32".into()],
                inverse_path: vec!["意识→神经网络→神经元→突触→涌现→三生万物".into()],
            },
            // ═══════ 万物: 具体领域 (Layer 4+) ═══════
            DaoRule {
                id: "DAO-30".into(),
                name: "物理规则 → 宇宙万物".into(),
                layer: DaoLevel::TenThousand,
                formulation: "四大基本力(引力/电磁/强核/弱核) + 标准模型粒子 + 量子场论".into(),
                manifestation: vec!["从基本粒子到恒星/行星/生命/文明的一切物理现象".into()],
                connects_to: vec![],
                inverse_path: vec!["具体发明→工程→应用物理→理论物理→基本力→道".into()],
            },
            DaoRule {
                id: "DAO-31".into(),
                name: "化学规则 → 物质万物".into(),
                layer: DaoLevel::TenThousand,
                formulation: "元素周期表 + 化学键(离子/共价/金属) + 化学反应".into(),
                manifestation: vec!["从DNA到塑料到混凝土到药物的一切化学物质".into()],
                connects_to: vec![],
                inverse_path: vec!["新材料→化学合成→分子结构→原子价→元素周期表→道".into()],
            },
            DaoRule {
                id: "DAO-32".into(),
                name: "生命规则 → 生物万物".into(),
                layer: DaoLevel::TenThousand,
                formulation: "DNA(信息存储) + RNA(信息传递) + 蛋白质(功能执行) + 自然选择".into(),
                manifestation: vec!["从细菌到蓝鲸到人类到整个生态系统的所有生命形式".into()],
                connects_to: vec![],
                inverse_path: vec!["疾病→基因→DNA→RNA→蛋白质→中心法则→生命起源→道".into()],
            },
            DaoRule {
                id: "DAO-33".into(),
                name: "意识规则 → 心智万物".into(),
                layer: DaoLevel::TenThousand,
                formulation: "神经元网络 + 涌现 + 信息整合 + 具身认知".into(),
                manifestation: vec!["从感官知觉到抽象思维到自我意识到集体智慧".into()],
                connects_to: vec![],
                inverse_path: vec!["AI→神经网络→意识→大脑→生物神经元→电化学信号→道".into()],
            },
            DaoRule {
                id: "DAO-34".into(),
                name: "社会规则 → 文明万物".into(),
                layer: DaoLevel::TenThousand,
                formulation: "语言(信息传递) + 合作(群体优势) + 知识积累(跨代传递) = 文明".into(),
                manifestation: vec!["从部落到国家到全球化到互联网的整个文明演化".into()],
                connects_to: vec![],
                inverse_path: vec!["互联网→文字→语言→社会合作→工具使用→文明→道".into()],
            },
        ]
    }

    fn init_domain_mappings() -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(
            "物理".into(),
            vec!["DAO-01".into(), "DAO-10".into(), "DAO-30".into()],
        );
        m.insert("化学".into(), vec!["DAO-01".into(), "DAO-31".into()]);
        m.insert("生物".into(), vec!["DAO-32".into(), "DAO-20".into()]);
        m.insert(
            "计算机".into(),
            vec!["DAO-11".into(), "DAO-20".into(), "DAO-33".into()],
        );
        m.insert("医学".into(), vec!["DAO-32".into(), "DAO-01".into()]);
        m.insert("社会".into(), vec!["DAO-34".into(), "DAO-20".into()]);
        m.insert("工程".into(), vec!["DAO-30".into(), "DAO-31".into()]);
        m.insert(
            "哲学".into(),
            vec!["DAO-00".into(), "DAO-11".into(), "DAO-12".into()],
        );
        m
    }

    fn init_phenomena_mappings() -> HashMap<String, Vec<String>> {
        let mut m = HashMap::new();
        m.insert(
            "智能手机".into(),
            vec![
                "DAO-01".into(),
                "DAO-10".into(),
                "DAO-30".into(),
                "DAO-31".into(),
            ],
        );
        m.insert(
            "互联网".into(),
            vec!["DAO-34".into(), "DAO-11".into(), "DAO-20".into()],
        );
        m.insert(
            "AI".into(),
            vec!["DAO-33".into(), "DAO-20".into(), "DAO-12".into()],
        );
        m.insert("核能".into(), vec!["DAO-01".into(), "DAO-30".into()]);
        m.insert("基因编辑".into(), vec!["DAO-32".into(), "DAO-01".into()]);
        m.insert(
            "区块链".into(),
            vec!["DAO-12".into(), "DAO-34".into(), "DAO-11".into()],
        );
        m
    }

    /// 从具体现象逆推到本源规则
    pub fn trace_to_dao(&self, phenomenon: &str) -> Vec<String> {
        let mut chain = Vec::new();
        chain.push(format!("🌍 现象: {}", phenomenon));

        if let Some(rule_ids) = self.phenomena_to_rules.get(phenomenon) {
            for rid in rule_ids {
                if let Some(rule) = self.rules.iter().find(|r| r.id == *rid) {
                    chain.push(format!("  ↕ 归属于: {:?} - {}", rule.layer, rule.name));
                    for inv in &rule.inverse_path {
                        chain.push(format!("  ← 逆推路径: {}", inv));
                    }
                }
            }
        }

        chain.push("  ↓ 一直追溯到...".to_string());
        chain.push("☯ 道: 万物的本源".to_string());

        chain
    }

    /// 从道向下生成到具体现象
    pub fn generate_from_dao(&self, target_domain: &str) -> Vec<String> {
        let mut chain = Vec::new();
        chain.push("☯ 道 → 从本源出发...".to_string());
        chain.push("  ↓ 信息=能量=物质 三位一体".to_string());
        chain.push("  ↓ 时空二元 + 阴阳二元 + 因果二元".to_string());
        chain.push("  ↓ 涌现三生万物".to_string());
        chain.push(format!("  ↓ {}", target_domain));

        if let Some(rule_ids) = self.domain_to_rules.get(target_domain) {
            for rid in rule_ids {
                if let Some(rule) = self.rules.iter().find(|r| r.id == *rid) {
                    chain.push(format!("  → {:?}: {}", rule.layer, rule.name));
                }
            }
        }

        chain
    }

    /// 知识引擎的终极架构图
    pub fn ultimate_architecture(&self) -> String {
        let mut arch = String::new();
        arch.push_str("╔══════════════════════════════════════════════════════════╗\n");
        arch.push_str("║     KNOWLEDGE ENGINE ULTIMATE ARCHITECTURE           ║\n");
        arch.push_str("║     知识引擎终极架构 · 从道到万物的全息映射        ║\n");
        arch.push_str("╚══════════════════════════════════════════════════════════╝\n\n");

        arch.push_str("Layer 0: ☯ 道 — 本源规则\n");
        arch.push_str("  道可道,非常道. 万物生于有,有生于无.\n\n");

        arch.push_str("Layer 1: ⚛ 一 — 三位一体\n");
        arch.push_str("  ├─ E=mc² (质量=能量)\n");
        arch.push_str("  ├─ S=k·lnW (信息=熵)\n");
        arch.push_str("  └─ 一切信息都是物理的\n\n");

        arch.push_str("Layer 2: ☯ 二 — 三元二元性\n");
        arch.push_str("  ├─ 时空二元: ds²= -c²dt²+dx²+dy²+dz²\n");
        arch.push_str("  ├─ 阴阳二元: ∀x∃¬x 对立统一\n");
        arch.push_str("  └─ 因果二元: P(A→B)=P(B|A)·P(A)\n\n");

        arch.push_str("Layer 3: 三 — 涌现机制\n");
        arch.push_str("  简单单元×大量×规则交互 = 涌现新性质\n");
        arch.push_str("  ├─ H₂O→湿性  ├─ 神经元→意识  ├─ 个人→文明\n\n");

        arch.push_str("Layer 4: 万物 — 具体知识域\n");
        arch.push_str("  ├─ 🌌 物理(DAO-30): 标准模型/四大基本力/量子场论\n");
        arch.push_str("  ├─ 🧪 化学(DAO-31): 元素周期表/化学键/反应\n");
        arch.push_str("  ├─ 🧬 生命(DAO-32): DNA/RNA/蛋白质/自然选择\n");
        arch.push_str("  ├─ 🧠 意识(DAO-33): 神经网络/涌现/信息整合\n");
        arch.push_str("  └─ 🏛 文明(DAO-34): 语言/合作/知识积累\n\n");

        arch.push_str("逆推链(从散落信息→道):\n");
        arch.push_str("  Internet碎片 → 领域知识 → 特定规则 → 基本力/元素/生命法则 → 本源(道)\n\n");

        arch.push_str("正向链(从道→具体造物):\n");
        arch.push_str("  道 → 质能信息 → 时空/阴阳/因果 → 涌现 → 物理/化学/生命 → 具体造物\n");

        arch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dao_engine_new() {
        let de = DaoEngine::new();
        assert!(de.rules.len() >= 1);
    }

    #[test]
    fn test_trace_to_dao() {
        let de = DaoEngine::new();
        let chain = de.trace_to_dao("智能手机");
        assert!(!chain.is_empty());
        assert!(chain[0].contains("智能手机"));
    }

    #[test]
    fn test_generate_from_dao() {
        let de = DaoEngine::new();
        let chain = de.generate_from_dao("生物");
        assert!(!chain.is_empty());
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_architecture_diagram() {
        let de = DaoEngine::new();
        let arch = de.ultimate_architecture();
        assert!(arch.contains("Layer 0"));
        assert!(arch.contains("Layer 4"));
    }
}
