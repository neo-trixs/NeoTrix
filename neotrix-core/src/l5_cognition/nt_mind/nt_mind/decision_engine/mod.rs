//! Decision engine module — multi-criteria weighted decision analysis
//!
//! 四文件架构:
//! - `decision`: 核心类型 (Decision, DecisionOption, Criteria)
//! - `scorer`: 加权评分器 (WeightedScorer, ScoredOption)
//! - `analyzer`: 分析器 (DecisionAnalyzer, Analysis)
//! - `recommender`: 推荐器 (DecisionRecommender, Recommendation)
//!
//! R-P123: 按认知域拆分决策流程
//! R-P124: 配置集中管理，支持 Default trait

pub mod analyzer;
pub mod decision;
pub mod recommender;
pub mod scorer;

pub use analyzer::{Analysis, DecisionAnalyzer};
pub use decision::{Criteria, Decision, DecisionOption};
pub use recommender::{DecisionRecommender, Recommendation};
pub use scorer::{ScoredOption, WeightedScorer};
