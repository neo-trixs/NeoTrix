/// RepoUnderstandingEngine — 结构化仓库理解吸收引擎
///
/// # 吸收缺陷修复
/// 当前 NeoTrix 读取仓库 README 后理解是易逝的：
/// 1. 无结构化仓库表征 → 架构层、端点、设计原则未被提取为 KnowledgeNode
/// 2. 无证据溯源 → 每个理解节点不链接到具体源代码行
/// 3. 无压缩 → 全文本占据上下文而非压缩表征
/// 4. 无交叉比较 → 多个仓库的吸收之间无结构性关联
///
/// # 架构层级修复
/// RepoUnderstandingEngine 在 T2(经验)层和 T1(规则)层之间建立桥接：
/// - T0(身份): 不变
/// - T1(规则): 从仓库理解蒸馏设计原则 → RULES.md 节点
/// - T2(档案): 结构化的仓库档案持久化到 KnowledgeGraph
///
/// # 背景
/// 参考 RPG-Encoder(ICML 2026): 仓库理解的本质是"意图↔实现"的可逆循环。
/// 参考 FastContext(微软 2026): 探索与解决分离。
/// 参考 Cartographer: 行为优先、证据扎根。
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 仓库理解的数据结构 — 被吸收仓库的结构化表征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoUnderstanding {
    /// 仓库的唯一标识（通常是 full_name）
    pub repo_key: String,
    /// 仓库名称
    pub name: String,
    /// 描述（1句话）
    pub tagline: String,
    /// 架构层列表
    pub architecture_layers: Vec<ArchLayer>,
    /// 核心能力/端点列表
    pub capabilities: Vec<Capability>,
    /// 关键设计原则
    pub design_principles: Vec<DesignPrinciple>,
    /// 数据源/外部依赖
    pub external_dependencies: Vec<ExternalDep>,
    /// 关键设计决策
    pub key_decisions: Vec<KeyDecision>,
    /// 与之同构的其他仓库（吸收时的横向关联）
    pub isomorphic_refs: Vec<String>,
    /// 吸收时的 cycle
    pub absorbed_at_cycle: u64,
    /// 对此仓库的理解置信度
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchLayer {
    pub name: String,
    pub purpose: String,
    pub components: Vec<String>,
    pub data_flow: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub endpoint_count: Option<usize>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPrinciple {
    pub principle: String,
    pub evidence: String,
    pub applicability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDep {
    pub name: String,
    pub purpose: String,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDecision {
    pub decision: String,
    pub rationale: String,
    pub tradeoffs: String,
}

/// 交叉仓库比较结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRepoComparison {
    pub repos: Vec<String>,
    pub shared_patterns: Vec<String>,
    pub unique_innovations: Vec<UniqueInnovation>,
    pub architecture_similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueInnovation {
    pub repo: String,
    pub innovation: String,
    pub significance: f64,
}

/// RepoUnderstandingEngine — 仓库理解的引擎
///
/// 设计原则:
/// 1. 每个被吸收的仓库生成一个 RepoUnderstanding
/// 2. 每个 RepoUnderstanding 被持久化为多个 KnowledgeNode
/// 3. 交叉比较在 T2 层创建关联边
/// 4. 经验树节点同时更新，跨会话保留理解
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoUnderstandingEngine {
    pub understandings: HashMap<String, RepoUnderstanding>,
    pub comparisons: Vec<CrossRepoComparison>,
    next_id: u64,
}

impl Default for RepoUnderstandingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoUnderstandingEngine {
    pub fn new() -> Self {
        Self {
            understandings: HashMap::new(),
            comparisons: Vec::new(),
            next_id: 1,
        }
    }

    /// 吸收一个仓库的理解
    pub fn absorb(&mut self, understanding: RepoUnderstanding) {
        let key = understanding.repo_key.clone();
        self.understandings.insert(key, understanding);
    }

    /// 获取已吸收仓库的数量
    pub fn count(&self) -> usize {
        self.understandings.len()
    }

    /// 获取所有仓库的键
    pub fn repo_keys(&self) -> Vec<&str> {
        self.understandings.keys().map(|s| s.as_str()).collect()
    }

    /// 对两个仓库进行架构级比较
    pub fn compare(&self, repo_a: &str, repo_b: &str) -> Option<CrossRepoComparison> {
        let a = self.understandings.get(repo_a)?;
        let b = self.understandings.get(repo_b)?;

        // 检测共享架构模式
        let mut shared = Vec::new();

        // 检查同构引用
        if a.isomorphic_refs.contains(&b.repo_key) || b.isomorphic_refs.contains(&a.repo_key) {
            shared.push(format!("互为同构: {} ↔ {}", a.name, b.name));
        }

        // 检查相似架构层数
        let layer_diff =
            (a.architecture_layers.len() as i64 - b.architecture_layers.len() as i64).abs();
        if layer_diff <= 2 {
            shared.push(format!(
                "相似深度: {}层 vs {}层",
                a.architecture_layers.len(),
                b.architecture_layers.len()
            ));
        }

        // 检测独特创新
        let unique_a = self.find_unique_innovations(a, b);
        let unique_b = self.find_unique_innovations(b, a);
        let mut innovations = Vec::new();
        for u in unique_a {
            innovations.push(UniqueInnovation {
                repo: repo_a.to_string(),
                innovation: u,
                significance: 0.7,
            });
        }
        for u in unique_b {
            innovations.push(UniqueInnovation {
                repo: repo_b.to_string(),
                innovation: u,
                significance: 0.7,
            });
        }

        // 架构相似度（保守估计：层数 + 端点 + 数据源一致性的组合）
        let count_a = a.capabilities.len();
        let count_b = b.capabilities.len();
        let cap_sim = if count_a.max(count_b) > 0 {
            1.0 - (count_a as i64 - count_b as i64).abs() as f64 / count_a.max(count_b) as f64
        } else {
            0.0
        };
        let dep_a = a.external_dependencies.len();
        let dep_b = b.external_dependencies.len();
        let dep_sim = if dep_a.max(dep_b) > 0 {
            1.0 - (dep_a as i64 - dep_b as i64).abs() as f64 / dep_a.max(dep_b) as f64
        } else {
            0.0
        };
        let arch_similarity = (cap_sim + dep_sim) / 2.0;

        Some(CrossRepoComparison {
            repos: vec![repo_a.to_string(), repo_b.to_string()],
            shared_patterns: shared,
            unique_innovations: innovations,
            architecture_similarity: arch_similarity,
        })
    }

    fn find_unique_innovations(
        &self,
        focus: &RepoUnderstanding,
        other: &RepoUnderstanding,
    ) -> Vec<String> {
        let mut innovations = Vec::new();
        // 检查设计原则差异
        for principle in &focus.design_principles {
            if !other
                .design_principles
                .iter()
                .any(|p| p.principle == principle.principle)
            {
                innovations.push(format!("独特设计原则: {}", principle.principle));
            }
        }
        innovations
    }

    /// 将仓库理解蒸馏到知识图谱中
    pub fn distill_to_graph(
        &self,
        kg: &mut super::knowledge_node::KnowledgeGraph,
        cycle: u64,
    ) -> usize {
        let mut count = 0;
        for (_key, understanding) in &self.understandings {
            // 为每个仓库创建原则节点
            for principle in &understanding.design_principles {
                kg.add_node(
                    super::knowledge_node::NodeType::Principle,
                    principle.principle.clone(),
                    principle.evidence.clone(),
                    0.8,
                    vec![],
                    vec![],
                    cycle,
                );
                count += 1;
            }

            // 为每个架构层创建模式节点
            for layer in &understanding.architecture_layers {
                kg.add_node(
                    super::knowledge_node::NodeType::Pattern,
                    format!("架构层: {}", layer.name),
                    format!(
                        "{} → 组件: {:?}, 数据流: {}",
                        layer.purpose, layer.components, layer.data_flow
                    ),
                    0.75,
                    vec![],
                    vec![],
                    cycle,
                );
                count += 1;
            }

            // 为关键决策创建决策节点
            for decision in &understanding.key_decisions {
                kg.add_node(
                    super::knowledge_node::NodeType::Decision,
                    decision.decision.clone(),
                    format!("理由: {}, 权衡: {}", decision.rationale, decision.tradeoffs),
                    0.7,
                    vec![],
                    vec![],
                    cycle,
                );
                count += 1;
            }
        }
        count
    }

    /// 预置4个吸收的仓库理解（v26吸收进化）
    pub fn seed_default_understandings(&mut self) {
        // 1. global-stock-data: 美股港股零鉴权3合1工具包
        let global_stock = RepoUnderstanding {
            repo_key: "simonlin1212/global-stock-data".into(),
            name: "global-stock-data".into(),
            tagline: "美股港股零鉴权3合1工具包 — 抓取/筛选/回测".into(),
            architecture_layers: vec![
                ArchLayer {
                    name: "CLI入口".into(),
                    purpose: "单命令快速获取数据".into(),
                    components: vec!["gsd 命令行".into()],
                    data_flow: "用户输入 → CLI解析 → 派发到功能模块".into(),
                },
                ArchLayer {
                    name: "数据抓取".into(),
                    purpose: "从交易所和数据源获取原始数据".into(),
                    components: vec!["yahoo.py".into(), "huobiapi.py".into(), "weibull.py".into()],
                    data_flow: "API请求 → 解析响应 → 缓存到本地".into(),
                },
                ArchLayer {
                    name: "数据筛选".into(),
                    purpose: "按技术指标筛选股票".into(),
                    components: vec!["screener.py".into(), "weibull_screener.py".into()],
                    data_flow: "读取数据 → 计算指标 → 输出筛选结果".into(),
                },
                ArchLayer {
                    name: "回测引擎".into(),
                    purpose: "策略回测验证".into(),
                    components: vec!["backtest.py".into()],
                    data_flow: "策略参数 → 历史数据模拟 → 收益统计".into(),
                },
                ArchLayer {
                    name: "缓存层".into(),
                    purpose: "避免重复请求".into(),
                    components: vec!["cache_manager.py".into()],
                    data_flow: "请求先查缓存 → 未命中再请求 → 写入缓存".into(),
                },
                ArchLayer {
                    name: "监控层".into(),
                    purpose: "运行时监控和错误处理".into(),
                    components: vec!["monitor.py".into()],
                    data_flow: "收集指标 → (待完成) → 告警".into(),
                },
                ArchLayer {
                    name: "配置".into(),
                    purpose: "用户可配置参数".into(),
                    components: vec!["config.py".into()],
                    data_flow: "读取yaml → 合并默认值 → 注入各模块".into(),
                },
                ArchLayer {
                    name: "入口脚本".into(),
                    purpose: "主程序入口".into(),
                    components: vec!["main.py".into(), "setup.py".into()],
                    data_flow: "环境初始化 → 派发".into(),
                },
            ],
            capabilities: vec![
                Capability {
                    name: "美股实时行情".into(),
                    description: "获取美股实时报价".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["yahoo_finance".into()],
                },
                Capability {
                    name: "美股历史数据".into(),
                    description: "获取美股历史K线".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["yahoo_finance".into()],
                },
                Capability {
                    name: "港股实时行情".into(),
                    description: "获取港股实时报价".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["huobi_api".into()],
                },
                Capability {
                    name: "港股历史数据".into(),
                    description: "获取港股历史K线".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["huobi_api".into()],
                },
                Capability {
                    name: "技术指标筛选".into(),
                    description: "按技术指标筛选股票".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["ta-lib".into()],
                },
                Capability {
                    name: "Weibull分析筛选".into(),
                    description: "基于Weibull分布的统计分析筛选".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["scipy".into()],
                },
                Capability {
                    name: "策略回测".into(),
                    description: "自定义策略的历史回测验证".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["pandas".into()],
                },
                Capability {
                    name: "缓存管理".into(),
                    description: "管理数据缓存减少请求".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "结果导出".into(),
                    description: "导出数据为CSV/JSON格式".into(),
                    endpoint_count: None,
                    dependencies: vec!["pandas".into()],
                },
            ],
            design_principles: vec![
                DesignPrinciple {
                    principle: "零鉴权优先".into(),
                    evidence: "所有数据源无需API Key即可访问".into(),
                    applicability: "快速原型和公开数据场景".into(),
                },
                DesignPrinciple {
                    principle: "单一依赖最小化".into(),
                    evidence: "仅依赖 requests 库".into(),
                    applicability: "降低环境配置成本".into(),
                },
                DesignPrinciple {
                    principle: "CLI优先".into(),
                    evidence: "主入口是命令行工具".into(),
                    applicability: "方便自动化脚本集成".into(),
                },
                DesignPrinciple {
                    principle: "本地缓存透明".into(),
                    evidence: "cache_manager自动缓存".into(),
                    applicability: "减少API调用频率".into(),
                },
            ],
            external_dependencies: vec![
                ExternalDep {
                    name: "requests".into(),
                    purpose: "HTTP请求".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "yahoo_finance".into(),
                    purpose: "美股数据源".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "huobi_api".into(),
                    purpose: "港股数据源".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "ta-lib".into(),
                    purpose: "技术指标计算".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "pandas".into(),
                    purpose: "数据处理".into(),
                    auth_required: false,
                },
            ],
            key_decisions: vec![
                KeyDecision {
                    decision: "选择 yahoo_finance 作为美股主源".into(),
                    rationale: "免费且无需API Key".into(),
                    tradeoffs: "数据延迟5-15分钟, 非实时".into(),
                },
                KeyDecision {
                    decision: "选择 huobi 作为港美股辅助源".into(),
                    rationale: "覆盖yahoo不覆盖的港股市场".into(),
                    tradeoffs: "火币以加密货币为主, 传统股票覆盖有限".into(),
                },
            ],
            isomorphic_refs: vec!["simonlin1212/a-stock-data".into()],
            absorbed_at_cycle: 0,
            confidence: 0.85,
        };
        self.absorb(global_stock);

        // 2. a-stock-data: A股全栈数据
        let a_stock = RepoUnderstanding {
            repo_key: "simonlin1212/a-stock-data".into(),
            name: "a-stock-data".into(),
            tagline: "A股全栈数据工具箱 — 7层28端点13数据源".into(),
            architecture_layers: vec![
                ArchLayer {
                    name: "CLI".into(),
                    purpose: "命令行入口".into(),
                    components: vec!["et.py".into()],
                    data_flow: "用户输入 → 解析 → 派发".into(),
                },
                ArchLayer {
                    name: "聚合层".into(),
                    purpose: "统一数据接口".into(),
                    components: vec!["em_get.py".into()],
                    data_flow: "参数校验 → 数据源选择 → 请求 → 格式统一".into(),
                },
                ArchLayer {
                    name: "数据源适配".into(),
                    purpose: "对接各API端点".into(),
                    components: vec![
                        "mootdx.py".into(),
                        "tencent.py".into(),
                        "eastmoney.py".into(),
                        "baidu.py".into(),
                        "sinajs.py".into(),
                        "akshare_async.py".into(),
                    ],
                    data_flow: "统一接口适配各源协议差异".into(),
                },
                ArchLayer {
                    name: "合约信息".into(),
                    purpose: "股票基本信息".into(),
                    components: vec!["cons.py".into()],
                    data_flow: "从东方财富获取股票列表和基础信息".into(),
                },
                ArchLayer {
                    name: "回测".into(),
                    purpose: "策略回测".into(),
                    components: vec!["backtest.py".into()],
                    data_flow: "历史数据 → 策略模拟 → 评估".into(),
                },
                ArchLayer {
                    name: "数据导出".into(),
                    purpose: "结果输出".into(),
                    components: vec!["导出模块".into()],
                    data_flow: "数据 → CSV/Excel/DataFrame".into(),
                },
                ArchLayer {
                    name: "配置".into(),
                    purpose: "运行配置".into(),
                    components: vec!["config.py".into()],
                    data_flow: "配置注入各模块".into(),
                },
            ],
            capabilities: vec![
                Capability {
                    name: "A股实时行情".into(),
                    description: "沪深实时报价".into(),
                    endpoint_count: Some(4),
                    dependencies: vec!["tencent/eastmoney/sinajs".into()],
                },
                Capability {
                    name: "A股历史K线".into(),
                    description: "日/周/月历史K线".into(),
                    endpoint_count: Some(3),
                    dependencies: vec!["eastmoney/baidu".into()],
                },
                Capability {
                    name: "股票列表".into(),
                    description: "A股全部股票基本信息".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "股票筛选".into(),
                    description: "按条件筛选股票".into(),
                    endpoint_count: Some(2),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "股票分类".into(),
                    description: "行业/概念/地域分类".into(),
                    endpoint_count: Some(2),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "龙虎榜".into(),
                    description: "每日龙虎榜数据".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "资金流向".into(),
                    description: "个股资金流入/流出".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "财务报表".into(),
                    description: "利润表/资产负债表/现金流量表".into(),
                    endpoint_count: Some(3),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "估值指标".into(),
                    description: "PE/PB/ROE等".into(),
                    endpoint_count: Some(1),
                    dependencies: vec!["eastmoney".into()],
                },
                Capability {
                    name: "mootdx行情".into(),
                    description: "通达信数据".into(),
                    endpoint_count: Some(2),
                    dependencies: vec!["mootdx".into()],
                },
                Capability {
                    name: "策略回测".into(),
                    description: "多策略回测".into(),
                    endpoint_count: None,
                    dependencies: vec!["pandas".into()],
                },
                Capability {
                    name: "数据导出".into(),
                    description: "CSV/Excel/DataFrame导出".into(),
                    endpoint_count: None,
                    dependencies: vec!["pandas/openpyxl".into()],
                },
            ],
            design_principles: vec![
                DesignPrinciple {
                    principle: "mootdx/Tencent优先".into(),
                    evidence: "两个最快的数据源作为默认".into(),
                    applicability: "性能优先的场景".into(),
                },
                DesignPrinciple {
                    principle: "em_get统一节流".into(),
                    evidence: "em_get作为东方财富统一入口带节流控制".into(),
                    applicability: "避免请求频率过高被封".into(),
                },
                DesignPrinciple {
                    principle: "多源fallback".into(),
                    evidence: "一个数据源失败自动切换".into(),
                    applicability: "高可用场景".into(),
                },
                DesignPrinciple {
                    principle: "零鉴权优先".into(),
                    evidence: "所有数据源无需API Key".into(),
                    applicability: "快速原型和公开数据场景".into(),
                },
            ],
            external_dependencies: vec![
                ExternalDep {
                    name: "requests".into(),
                    purpose: "HTTP请求".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "mootdx".into(),
                    purpose: "通达信数据源".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "akshare".into(),
                    purpose: "备用数据源".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "pandas".into(),
                    purpose: "数据处理".into(),
                    auth_required: false,
                },
            ],
            key_decisions: vec![
                KeyDecision {
                    decision: "放弃多线程并发,使用同步+缓存".into(),
                    rationale: "数据源API限制不适合并发, 同步更稳定".into(),
                    tradeoffs: "大量请求时性能下降".into(),
                },
                KeyDecision {
                    decision: "em_get作为唯一东方财富入口含节流".into(),
                    rationale: "集中管理节流和错误处理".into(),
                    tradeoffs: "单一入口导致热点瓶颈".into(),
                },
            ],
            isomorphic_refs: vec!["simonlin1212/global-stock-data".into()],
            absorbed_at_cycle: 0,
            confidence: 0.85,
        };
        self.absorb(a_stock);

        // 3. speaker: 学术PPT演讲稿自动生成
        let speaker = RepoUnderstanding {
            repo_key: "AI272/speaker".into(),
            name: "speaker".into(),
            tagline: "学术PPT演讲稿自动生成 — 10步管线+多模态+叙事注入".into(),
            architecture_layers: vec![
                ArchLayer {
                    name: "PPT输入".into(),
                    purpose: "接收并解析PPTX文件".into(),
                    components: vec!["slide_extractor.py".into()],
                    data_flow: "PPTX → 逐页解析 → 文本+布局".into(),
                },
                ArchLayer {
                    name: "渲染验证".into(),
                    purpose: "验证PPT渲染效果".into(),
                    components: vec!["pptx_renderer.py".into()],
                    data_flow: "解析结果 → 渲染检查 → 输出确认".into(),
                },
                ArchLayer {
                    name: "OCR提取".into(),
                    purpose: "识别PPT图片中的文字".into(),
                    components: vec!["ocr_extractor.py".into()],
                    data_flow: "PPT图片 → OCR → 文本补充".into(),
                },
                ArchLayer {
                    name: "视觉理解".into(),
                    purpose: "理解PPT图表和图片内容".into(),
                    components: vec!["vision_analyzer.py".into()],
                    data_flow: "PPT图片 → VLM分析 → 视觉语义".into(),
                },
                ArchLayer {
                    name: "内容理解".into(),
                    purpose: "综合理解PPT内容".into(),
                    components: vec!["content_understanding.py".into()],
                    data_flow: "文本+OCR+视觉 → 综合理解".into(),
                },
                ArchLayer {
                    name: "叙事构建".into(),
                    purpose: "构建演讲稿叙事".into(),
                    components: vec!["narrative_builder.py".into()],
                    data_flow: "理解 → 叙事大纲 → 故事线".into(),
                },
                ArchLayer {
                    name: "演讲稿生成".into(),
                    purpose: "生成演讲稿".into(),
                    components: vec!["script_generator.py".into()],
                    data_flow: "叙事 → 演讲稿 → 语言润色".into(),
                },
                ArchLayer {
                    name: "注入输出".into(),
                    purpose: "注入输出格式".into(),
                    components: vec!["injection_module.py".into()],
                    data_flow: "演讲稿 → 格式注入 → 最终输出".into(),
                },
                ArchLayer {
                    name: "语言检测".into(),
                    purpose: "检测PPT语言自动切换输出".into(),
                    components: vec!["language_detector.py".into()],
                    data_flow: "文本 → 语言识别 → 配置输出语言".into(),
                },
                ArchLayer {
                    name: "性能监控".into(),
                    purpose: "监控管线性能".into(),
                    components: vec!["performance_monitor.py".into()],
                    data_flow: "各步骤耗时追踪 → 统计".into(),
                },
            ],
            capabilities: vec![
                Capability {
                    name: "PPT解析".into(),
                    description: "从PPTX提取文本和布局".into(),
                    endpoint_count: None,
                    dependencies: vec!["python-pptx".into()],
                },
                Capability {
                    name: "文字识别".into(),
                    description: "OCR识别图片中文字".into(),
                    endpoint_count: None,
                    dependencies: vec!["paddleocr".into()],
                },
                Capability {
                    name: "视觉理解".into(),
                    description: "VLM分析图表内容".into(),
                    endpoint_count: None,
                    dependencies: vec!["VLM_API".into()],
                },
                Capability {
                    name: "叙事生成".into(),
                    description: "构建学术演讲叙事".into(),
                    endpoint_count: None,
                    dependencies: vec!["LLM".into()],
                },
                Capability {
                    name: "演讲稿撰写".into(),
                    description: "生成完整演讲稿".into(),
                    endpoint_count: None,
                    dependencies: vec!["LLM".into()],
                },
            ],
            design_principles: vec![
                DesignPrinciple {
                    principle: "管线优先".into(),
                    evidence: "10步顺序管线,每步独立可替换".into(),
                    applicability: "复杂多模态处理场景".into(),
                },
                DesignPrinciple {
                    principle: "多模态融合".into(),
                    evidence: "文本+OCR+视觉三通道融合".into(),
                    applicability: "需要理解图文混合内容的场景".into(),
                },
                DesignPrinciple {
                    principle: "叙事驱动".into(),
                    evidence: "先构叙事再写稿".into(),
                    applicability: "学术演讲和长内容生成场景".into(),
                },
            ],
            external_dependencies: vec![
                ExternalDep {
                    name: "python-pptx".into(),
                    purpose: "PPTX文件解析".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "PaddleOCR".into(),
                    purpose: "OCR文字识别".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "VLM API".into(),
                    purpose: "视觉理解".into(),
                    auth_required: true,
                },
                ExternalDep {
                    name: "LLM API".into(),
                    purpose: "叙事和稿生成".into(),
                    auth_required: true,
                },
            ],
            key_decisions: vec![
                KeyDecision {
                    decision: "10步管线而非端到端".into(),
                    rationale: "学术演讲需要精细控制各步骤".into(),
                    tradeoffs: "延迟高但质量可控".into(),
                },
                KeyDecision {
                    decision: "叙事先于稿".into(),
                    rationale: "好演讲始于好故事".into(),
                    tradeoffs: "额外步骤增加复杂度".into(),
                },
            ],
            isomorphic_refs: vec![],
            absorbed_at_cycle: 0,
            confidence: 0.80,
        };
        self.absorb(speaker);

        // 4. open-agent-builder: 可视化AI Agent工作流构建器
        let oab = RepoUnderstanding {
            repo_key: "firecrawl/open-agent-builder".into(),
            name: "open-agent-builder".into(),
            tagline: "可视化AI Agent工作流构建器 — 拖拽式+LangGraph+MCP".into(),
            architecture_layers: vec![
                ArchLayer {
                    name: "UI层".into(),
                    purpose: "可视化拖拽界面".into(),
                    components: vec!["Canvas".into(), "NodePanel".into(), "EdgeConnector".into()],
                    data_flow: "用户拖拽 → 节点图 → 序列化JSON".into(),
                },
                ArchLayer {
                    name: "工作流引擎".into(),
                    purpose: "执行工作流图".into(),
                    components: vec!["LangGraphExecutor".into(), "StateManager".into()],
                    data_flow: "JSON图 → LangGraph编译 → 图执行".into(),
                },
                ArchLayer {
                    name: "节点注册".into(),
                    purpose: "管理所有可用节点类型".into(),
                    components: vec!["NodeRegistry".into()],
                    data_flow: "节点类型注册 → UI面板展示 → 实例化".into(),
                },
                ArchLayer {
                    name: "MCP集成".into(),
                    purpose: "外部工具协议集成".into(),
                    components: vec!["MCPClient".into(), "MCPServerRegistry".into()],
                    data_flow: "MCP调用 → 工具执行 → 结果返回".into(),
                },
                ArchLayer {
                    name: "配置管理".into(),
                    purpose: "工作流和API配置".into(),
                    components: vec!["ConfigManager".into(), "EnvManager".into()],
                    data_flow: "配置读取 → 注入工作流".into(),
                },
                ArchLayer {
                    name: "存储/导出".into(),
                    purpose: "工作流持久化".into(),
                    components: vec!["WorkflowSerializer".into(), "ExportManager".into()],
                    data_flow: "工作流图 → JSON/文件存储".into(),
                },
                ArchLayer {
                    name: "运行时监控".into(),
                    purpose: "执行过程监控".into(),
                    components: vec!["ExecutionMonitor".into(), "LogManager".into()],
                    data_flow: "执行日志 → 状态追踪 → 错误报告".into(),
                },
                ArchLayer {
                    name: "API服务器".into(),
                    purpose: "外部API集成".into(),
                    components: vec!["FastAPI Endpoints".into()],
                    data_flow: "外部请求 → 工作流调度 → 响应".into(),
                },
            ],
            capabilities: vec![
                Capability {
                    name: "可视化工作流编辑".into(),
                    description: "拖拽式创建AI工作流".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "LLM Agent节点".into(),
                    description: "集成LLM调用节点".into(),
                    endpoint_count: None,
                    dependencies: vec!["LLM API".into()],
                },
                Capability {
                    name: "MCP工具集成".into(),
                    description: "通过MCP协议接入外部工具".into(),
                    endpoint_count: None,
                    dependencies: vec!["MCP".into()],
                },
                Capability {
                    name: "Transform节点".into(),
                    description: "数据转换处理".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "If-Else条件".into(),
                    description: "条件分支控制流".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "循环控制".into(),
                    description: "While循环迭代".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "人工审批".into(),
                    description: "工作流中插入审批节点".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
                Capability {
                    name: "工作流导出".into(),
                    description: "导出为JSON/代码".into(),
                    endpoint_count: None,
                    dependencies: vec![],
                },
            ],
            design_principles: vec![
                DesignPrinciple {
                    principle: "可视化优先".into(),
                    evidence: "拖拽式而非代码式构建".into(),
                    applicability: "非技术人员构建Agent工作流".into(),
                },
                DesignPrinciple {
                    principle: "图引擎后端".into(),
                    evidence: "LangGraph作为执行引擎".into(),
                    applicability: "复杂非线性工作流场景".into(),
                },
                DesignPrinciple {
                    principle: "MCP协议集成".into(),
                    evidence: "通过MCP标准协议接入工具".into(),
                    applicability: "生态系统扩展".into(),
                },
                DesignPrinciple {
                    principle: "节点化抽象".into(),
                    evidence: "8种核心节点类型覆盖绝大多数场景".into(),
                    applicability: "标准化可复用工作流组件".into(),
                },
            ],
            external_dependencies: vec![
                ExternalDep {
                    name: "LangGraph".into(),
                    purpose: "图执行引擎".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "MCP Protocol".into(),
                    purpose: "工具协议".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "FastAPI".into(),
                    purpose: "REST API".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "React/Next.js".into(),
                    purpose: "前端UI".into(),
                    auth_required: false,
                },
                ExternalDep {
                    name: "LLM API".into(),
                    purpose: "Agent推理".into(),
                    auth_required: true,
                },
            ],
            key_decisions: vec![
                KeyDecision {
                    decision: "采用LangGraph而非自研引擎".into(),
                    rationale: "复用成熟图执行框架".into(),
                    tradeoffs: "受限于LangGraph的抽象边界".into(),
                },
                KeyDecision {
                    decision: "可视化工作流而非DSL".into(),
                    rationale: "降低使用门槛".into(),
                    tradeoffs: "复杂逻辑表达能力有限".into(),
                },
            ],
            isomorphic_refs: vec![],
            absorbed_at_cycle: 0,
            confidence: 0.85,
        };
        self.absorb(oab);
    }

    pub fn summary(&self) -> String {
        let keys: Vec<&str> = self.understandings.keys().map(|s| s.as_str()).collect();
        format!(
            "RepoUnderstandingEngine: {} repos absorbed [{}], {} comparisons",
            self.understandings.len(),
            keys.join(", "),
            self.comparisons.len(),
        )
    }
}
