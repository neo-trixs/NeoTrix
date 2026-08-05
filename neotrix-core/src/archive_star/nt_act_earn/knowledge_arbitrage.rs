use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 互联网知识套利引擎 — 通过信息交换获取价值
///
/// 核心洞察：信息在互联网上分布不均 → 聚合 → 处理 → 变现
/// 金字塔顶层利用信息不对称（insider trading 是非法版，合法版叫研究）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeArbitrageEngine {
    pub sources: Vec<InformationSource>,
    pub signal_processors: Vec<SignalProcessor>,
    pub knowledge_graph: KnowledgeGraph,
    pub arbitrage_opportunities: Vec<ArbitrageOpportunity>,
    pub attention_assets: Vec<AttentionAsset>,
}

/// 信息源 — 互联网上可抓取/订阅的数据流
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InformationSource {
    pub name: String,
    pub source_type: SourceType,
    pub signal_quality: f64,        // 信噪比
    pub update_frequency_secs: u64,
    pub access_cost_monthly: f64,
    pub exclusive: bool,            // 是否是独占信息源
    pub last_fetch: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    /// 公开市场数据
    PublicMarket,
    /// 学术论文预印本
    ArxivPaper,
    /// 社交媒体流
    SocialMedia,
    /// 新闻聚合
    NewsAggregator,
    /// 链上数据
    OnChain,
    /// 专利/商标数据库
    PatentDatabase,
    /// 监管文件 (SEC/EDGAR)
    RegulatoryFiling,
    /// 招聘信息（公司动向领先指标）
    JobPosting,
    /// GitHub 仓库活动
    GithubActivity,
    /// 暗网/深网情报
    DarkWebIntel,
    /// 行业数据 (Nielsen/Gartner/IDC)
    IndustryReport,
    /// 卫星/地理空间数据
    Geospatial,
}

/// 信号处理器 — 将原始数据转化为可交易信号
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalProcessor {
    pub name: String,
    pub input_sources: Vec<String>,
    pub latency_ms: f64,
    pub accuracy: f64,
    pub strategy: ProcessingStrategy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStrategy {
    /// 自然语言情感分析
    SentimentAnalysis,
    /// 异常检测（偏离基线）
    AnomalyDetection,
    /// 跨源关联
    CrossSourceCorrelation,
    /// 时序预测
    TimeSeriesPrediction,
    /// 知识图谱推理
    GraphReasoning,
    /// 网络图挖掘
    NetworkMining,
    /// 主题聚类
    TopicClustering,
}

/// 知识图谱 — 实体关系网络
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub entities: Vec<KnowledgeEntity>,
    pub relations: Vec<EntityRelation>,
    pub cluster_count: u32,
    pub avg_connectivity: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnowledgeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub salience: f64,          // 显著性
    pub last_seen: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    Person,
    Organization,
    Technology,
    Market,
    Product,
    Regulation,
    Concept,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityRelation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: RelationType,
    pub strength: f64,
    pub discovered_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationType {
    InvestedIn,        // A 投资 B
    CompetesWith,      // A 与 B 竞争
    Acquired,          // A 收购 B
    PartnersWith,      // A 与 B 合作
    Regulates,         // A 监管 B
    Supplies,          // A 向 B 供应
    Develops,          // A 开发 B
    Adopted,           // A 采用 B 技术
}

/// 套利机会 — 可利用的信息差
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub signal_source: String,
    pub target_market: String,
    pub expected_value: f64,
    pub confidence: f64,
    pub time_window_secs: u64,
    pub exploit_method: ExploitMethod,
    pub risk_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ExploitMethod {
    /// 预测市场押注
    PredictionMarket,
    /// 提前交易
    EarlyTrade,
    /// 内容套利（先发优势）
    ContentArbitrage,
    /// 数据聚合（稀缺数据增值）
    DataAggregation,
    /// 注意力套利
    AttentionArbitrage,
    /// 监管套利
    RegulatoryArbitrage,
    /// 知识蒸馏（论文→产品）
    KnowledgeDistillation,
    /// 社交网络影响
    SocialInfluence,
}

/// 注意力资产 — 互联网上可货币化的注意力
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttentionAsset {
    pub platform: String,
    pub followers: u64,
    pub engagement_rate: f64,
    pub attention_value_daily: f64,
    pub niche: String,
    pub monetization_method: MonetizationMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonetizationMethod {
    Advertising,
    AffiliateMarketing,
    Subscription,
    SponsoredContent,
    DigitalProducts,
    Consulting,
    Crowdfunding,
}

impl Default for KnowledgeArbitrageEngine {
    fn default() -> Self {
        Self {
            sources: vec![
                InformationSource {
                    name: "ArXiv ML".into(), source_type: SourceType::ArxivPaper,
                    signal_quality: 0.85, update_frequency_secs: 86400,
                    access_cost_monthly: 0.0, exclusive: false, last_fetch: None,
                },
                InformationSource {
                    name: "SEC EDGAR".into(), source_type: SourceType::RegulatoryFiling,
                    signal_quality: 0.95, update_frequency_secs: 3600,
                    access_cost_monthly: 0.0, exclusive: false, last_fetch: None,
                },
                InformationSource {
                    name: "GitHub Trending".into(), source_type: SourceType::GithubActivity,
                    signal_quality: 0.7, update_frequency_secs: 3600,
                    access_cost_monthly: 0.0, exclusive: false, last_fetch: None,
                },
                InformationSource {
                    name: "Patents".into(), source_type: SourceType::PatentDatabase,
                    signal_quality: 0.8, update_frequency_secs: 604800,
                    access_cost_monthly: 200.0, exclusive: false, last_fetch: None,
                },
                InformationSource {
                    name: "OnChain Alpha".into(), source_type: SourceType::OnChain,
                    signal_quality: 0.9, update_frequency_secs: 60,
                    access_cost_monthly: 50.0, exclusive: true, last_fetch: None,
                },
            ],
            signal_processors: vec![
                SignalProcessor {
                    name: "CrossDomainMiner".into(),
                    input_sources: vec!["ArXiv ML".into(), "GitHub Trending".into(), "Patents".into()],
                    latency_ms: 5000.0, accuracy: 0.78,
                    strategy: ProcessingStrategy::CrossSourceCorrelation,
                },
                SignalProcessor {
                    name: "MarketSentiment".into(),
                    input_sources: vec!["SEC EDGAR".into(), "News".into()],
                    latency_ms: 1000.0, accuracy: 0.72,
                    strategy: ProcessingStrategy::SentimentAnalysis,
                },
                SignalProcessor {
                    name: "AnomalyWatch".into(),
                    input_sources: vec!["OnChain Alpha".into(), "Social Media".into()],
                    latency_ms: 500.0, accuracy: 0.91,
                    strategy: ProcessingStrategy::AnomalyDetection,
                },
            ],
            knowledge_graph: KnowledgeGraph {
                entities: Vec::new(),
                relations: Vec::new(),
                cluster_count: 0,
                avg_connectivity: 0.0,
            },
            arbitrage_opportunities: Vec::new(),
            attention_assets: vec![
                AttentionAsset {
                    platform: "X/Twitter Tech".into(), followers: 10000,
                    engagement_rate: 0.03, attention_value_daily: 50.0,
                    niche: "AI/ML Engineering".into(),
                    monetization_method: MonetizationMethod::SponsoredContent,
                },
                AttentionAsset {
                    platform: "YouTube Dev".into(), followers: 5000,
                    engagement_rate: 0.05, attention_value_daily: 30.0,
                    niche: "Rust/Cognitive OS".into(),
                    monetization_method: MonetizationMethod::Advertising,
                },
            ],
        }
    }
}

impl KnowledgeArbitrageEngine {
    /// 扫描信息源发现套利机会
    pub fn scan_opportunities(&mut self) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        // 1. 跨领域关联 — 将论文发现映射到市场机会
        let has_arxiv = self.sources.iter().any(|s| matches!(s.source_type, SourceType::ArxivPaper));
        let has_patents = self.sources.iter().any(|s| matches!(s.source_type, SourceType::PatentDatabase));
        if has_arxiv && has_patents {
            opportunities.push(ArbitrageOpportunity {
                id: format!("opp-{}", opportunities.len() + 1),
                signal_source: "arXiv→Patent CrossRef".into(),
                target_market: "Tech Venture".into(),
                expected_value: 50000.0,
                confidence: 0.65,
                time_window_secs: 2592000,
                exploit_method: ExploitMethod::KnowledgeDistillation,
                risk_score: 0.3,
            });
        }

        // 2. 监管文件异常 — 比市场反应快
        let has_sec = self.sources.iter().any(|s| matches!(s.source_type, SourceType::RegulatoryFiling));
        if has_sec {
            opportunities.push(ArbitrageOpportunity {
                id: format!("opp-{}", opportunities.len() + 1),
                signal_source: "SEC Filing Anomaly".into(),
                target_market: "Equity".into(),
                expected_value: 10000.0,
                confidence: 0.8,
                time_window_secs: 86400,
                exploit_method: ExploitMethod::EarlyTrade,
                risk_score: 0.4,
            });
        }

        // 3. GitHub 先行指标 — 开源活动预示技术方向
        let has_github = self.sources.iter().any(|s| matches!(s.source_type, SourceType::GithubActivity));
        if has_github {
            opportunities.push(ArbitrageOpportunity {
                id: format!("opp-{}", opportunities.len() + 1),
                signal_source: "GitHub Trend Detection".into(),
                target_market: "Developer Tools".into(),
                expected_value: 15000.0,
                confidence: 0.7,
                time_window_secs: 604800,
                exploit_method: ExploitMethod::ContentArbitrage,
                risk_score: 0.2,
            });
        }

        self.arbitrage_opportunities = opportunities.clone();
        opportunities
    }

    /// 评估信息源组合的信号质量
    pub fn evaluate_source_portfolio(&self) -> f64 {
        let exclusive_count = self.sources.iter().filter(|s| s.exclusive).count() as f64;
        let avg_quality: f64 = self.sources.iter().map(|s| s.signal_quality).sum::<f64>()
            / self.sources.len().max(1) as f64;
        (avg_quality + exclusive_count * 0.15).min(1.0)
    }

    /// 计算知识套利的预期收益率
    pub fn expected_arbitrage_return(&self) -> f64 {
        let total_value: f64 = self.arbitrage_opportunities.iter()
            .map(|o| o.expected_value * o.confidence)
            .sum();
        total_value
    }

    /// 向知识图谱添加实体
    pub fn add_entity(&mut self, entity: KnowledgeEntity) {
        self.knowledge_graph.entities.push(entity);
        self.knowledge_graph.cluster_count = self.knowledge_graph.entities.len() as u32 / 3 + 1;
        self.knowledge_graph.avg_connectivity = if !self.knowledge_graph.relations.is_empty() {
            self.knowledge_graph.relations.len() as f64 / self.knowledge_graph.entities.len().max(1) as f64
        } else {
            0.0
        };
    }

    /// 添加实体关系
    pub fn add_relation(&mut self, relation: EntityRelation) {
        self.knowledge_graph.relations.push(relation);
        if !self.knowledge_graph.entities.is_empty() {
            self.knowledge_graph.avg_connectivity =
                self.knowledge_graph.relations.len() as f64 / self.knowledge_graph.entities.len() as f64;
        }
    }

    /// 注意力估值的总和
    pub fn total_attention_value(&self) -> f64 {
        self.attention_assets.iter().map(|a| a.attention_value_daily).sum()
    }

    /// 生成知识套利洞察
    pub fn arbitrage_insight(&self) -> Vec<String> {
        let mut insights = Vec::new();
        let portfolio_score = self.evaluate_source_portfolio();
        insights.push(format!("信息源组合质量评分: {:.1}%", portfolio_score * 100.0));
        insights.push(format!("活跃套利机会: {}个, 预期总价值: ${:.0}", 
            self.arbitrage_opportunities.len(), self.expected_arbitrage_return()));

        for opp in &self.arbitrage_opportunities {
            insights.push(format!(
                "  机会 {}: {} → {} 市场, 预期值 ${:.0}, 置信度 {:.0}%",
                opp.id, opp.signal_source, opp.target_market,
                opp.expected_value, opp.confidence * 100.0,
            ));
        }

        for asset in &self.attention_assets {
            insights.push(format!(
                "  注意力资产: {} ({} 粉丝, 日价值 ${:.1})",
                asset.platform, asset.followers, asset.attention_value_daily,
            ));
        }

        insights.push(format!("知识图谱: {} 实体, {} 关系, 连通度 {:.2}",
            self.knowledge_graph.entities.len(),
            self.knowledge_graph.relations.len(),
            self.knowledge_graph.avg_connectivity,
        ));

        insights
    }

    /// 建议最优套利策略
    pub fn suggest_best_arbitrage(&self) -> Vec<&ArbitrageOpportunity> {
        let mut ranked: Vec<&ArbitrageOpportunity> = self.arbitrage_opportunities.iter()
            .filter(|o| o.risk_score < 0.7)
            .collect();
        ranked.sort_by(|a, b| {
            let a_val = a.expected_value * a.confidence / (a.risk_score + 0.1);
            let b_val = b.expected_value * b.confidence / (b.risk_score + 0.1);
            b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
        });
        ranked.into_iter().take(3).collect()
    }

    /// 整合互联网知识到 WealthModel（信息→资本闭环）
    pub fn extract_knowledge_capital(&self) -> f64 {
        let arbitrage_capital: f64 = self.arbitrage_opportunities.iter()
            .map(|o| o.expected_value * o.confidence * (1.0 - o.risk_score))
            .sum();
        let attention_capital = self.total_attention_value() * 365.0;
        arbitrage_capital + attention_capital
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_engine() {
        let engine = KnowledgeArbitrageEngine::default();
        assert_eq!(engine.sources.len(), 5);
        assert_eq!(engine.signal_processors.len(), 3);
    }

    #[test]
    fn test_scan_opportunities() {
        let mut engine = KnowledgeArbitrageEngine::default();
        let opps = engine.scan_opportunities();
        assert!(!opps.is_empty());
        assert_eq!(engine.arbitrage_opportunities.len(), opps.len());
    }

    #[test]
    fn test_evaluate_source_portfolio() {
        let engine = KnowledgeArbitrageEngine::default();
        let score = engine.evaluate_source_portfolio();
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_add_entity_and_relation() {
        let mut engine = KnowledgeArbitrageEngine::default();
        engine.add_entity(KnowledgeEntity {
            id: "e1".into(), name: "OpenAI".into(),
            entity_type: EntityType::Organization, salience: 0.9,
            last_seen: "2026-06-01".into(), attributes: HashMap::new(),
        });
        engine.add_entity(KnowledgeEntity {
            id: "e2".into(), name: "GPT-5".into(),
            entity_type: EntityType::Technology, salience: 0.95,
            last_seen: "2026-06-01".into(), attributes: HashMap::new(),
        });
        engine.add_relation(EntityRelation {
            source_id: "e1".into(), target_id: "e2".into(),
            relation_type: RelationType::Develops, strength: 0.9,
            discovered_at: "2026-06-01".into(),
        });
        assert_eq!(engine.knowledge_graph.entities.len(), 2);
        assert_eq!(engine.knowledge_graph.relations.len(), 1);
    }

    #[test]
    fn test_total_attention_value() {
        let engine = KnowledgeArbitrageEngine::default();
        let tv = engine.total_attention_value();
        assert!(tv > 0.0);
    }

    #[test]
    fn test_arbitrage_insight_non_empty() {
        let mut engine = KnowledgeArbitrageEngine::default();
        engine.scan_opportunities();
        let insights = engine.arbitrage_insight();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_suggest_best_arbitrage() {
        let mut engine = KnowledgeArbitrageEngine::default();
        engine.scan_opportunities();
        let best = engine.suggest_best_arbitrage();
        assert!(!best.is_empty());
        assert!(best.len() <= 3);
    }

    #[test]
    fn test_extract_knowledge_capital() {
        let mut engine = KnowledgeArbitrageEngine::default();
        engine.scan_opportunities();
        let capital = engine.extract_knowledge_capital();
        assert!(capital > 0.0);
    }

    #[test]
    fn test_signal_processor_accuracy_range() {
        for sp in &KnowledgeArbitrageEngine::default().signal_processors {
            assert!(sp.accuracy >= 0.0 && sp.accuracy <= 1.0,
                "Processor {} has invalid accuracy", sp.name);
        }
    }
}
