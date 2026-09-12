#![allow(dead_code)]
//! Learned Router Module — LLMRouter 风格的学习型路由器
//!
//! 参考 ulab-uiuc/LLMRouter: KNN, SVM, MLP, MF, Elo, Graph, Hybrid 等 16+ 策略
//! NeoTrix 实现: KNNRouter (基线), MLPRouter (主力), HybridRouter (融合)
//!
//! 设计原则:
//! - 离线训练 (xRouteBench 数据) → 在线推理 (零开销)
//! - 特征: query_embedding (Qwen3-Embedding-0.6B) + task_type + user_profile
//! - 目标: alpha * quality - beta * cost (Pareto 优化)

use crate::l1_action::nt_io::nt_io_provider::provider_catalog::ProviderCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 路由特征向量 (输入)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RouteFeatures {
    /// Query embedding (768-d, Qwen3-Embedding-0.6B)
    pub query_embedding: Vec<f32>,
    /// Task type one-hot (CapabilityIntent 9 类)
    pub task_type: Vec<f32>,
    /// User profile embedding (可选, 个性化路由用)
    pub user_embedding: Option<Vec<f32>>,
    /// Query length (归一化)
    pub query_len_norm: f32,
    /// Code ratio (0-1)
    pub code_ratio: f32,
    /// Historical preference scores (per model)
    pub hist_preference: Option<HashMap<String, f32>>,
}

/// 多轮对话状态 — MultiTurnRouter 输入
/// 参考 LLMRouter Router-R1 / knnmultiroundrouter:
/// - 对话阶段感知 (寒暄 → 探索 → 深度 → 收尾)
/// - 工具链一致性 (agentic 场景同链路保持同一模型)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConversationState {
    /// 当前轮次 (0-indexed)
    pub turn_count: usize,
    /// 话题漂移度 (0-1, 相邻轮次语义距离 EMA)
    pub topic_drift: f32,
    /// 复杂度趋势 (EMA of query complexity)
    pub complexity_trend: f32,
    /// 是否处于 agentic 工具链中 (上一轮有 tool_calls)
    pub in_tool_chain: bool,
    /// 工具链当前使用的模型 (agentic 一致性锚定)
    pub tool_chain_model: Option<String>,
    /// 会话累计 token (成本预算感知)
    pub session_tokens_used: u64,
    /// 会话成本预算上限 (None = 不限)
    pub session_budget_usd: Option<f32>,
}

impl Default for ConversationState {
    fn default() -> Self {
        Self {
            turn_count: 0,
            topic_drift: 0.0,
            complexity_trend: 0.3,
            in_tool_chain: false,
            tool_chain_model: None,
            session_tokens_used: 0,
            session_budget_usd: None,
        }
    }
}

impl ConversationState {
    /// 对话阶段判定
    pub fn stage(&self) -> ConversationStage {
        if self.in_tool_chain {
            return ConversationStage::Agentic;
        }
        match self.turn_count {
            0..=2 => ConversationStage::Opening,
            3..=10 => {
                if self.complexity_trend > 0.6 {
                    ConversationStage::DeepDive
                } else {
                    ConversationStage::Exploration
                }
            }
            _ => {
                if self.complexity_trend > 0.5 {
                    ConversationStage::DeepDive
                } else if self.topic_drift > 0.7 {
                    ConversationStage::TopicSwitch
                } else {
                    ConversationStage::WrappingUp
                }
            }
        }
    }

    /// 更新每轮状态 (AgentLoop 每 turn 调用)
    pub fn advance(&mut self, query_complexity: f32, drift: f32) {
        self.turn_count += 1;
        // EMA 平滑复杂度与漂移
        self.complexity_trend = self.complexity_trend * 0.7 + query_complexity * 0.3;
        self.topic_drift = self.topic_drift * 0.6 + drift * 0.4;
    }

    /// 进入工具链 (模型锚定)
    pub fn enter_tool_chain(&mut self, model: &str) {
        self.in_tool_chain = true;
        self.tool_chain_model = Some(model.to_string());
    }

    /// 退出工具链
    pub fn exit_tool_chain(&mut self) {
        self.in_tool_chain = false;
        self.tool_chain_model = None;
    }
}

/// 对话阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStage {
    /// 开场寒暄 (前 2 轮) — 快/便宜模型即可
    Opening,
    /// 探索讨论 (3-10 轮, 低复杂度) — 平衡型
    Exploration,
    /// 深度讨论 (高复杂度) — 强模型
    DeepDive,
    /// 话题切换 — 重置评估
    TopicSwitch,
    /// 收尾总结 — 中等模型
    WrappingUp,
    /// Agentic 工具链 — 锚定当前模型保证一致性
    Agentic,
}

/// 候选模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CandidateModel {
    pub name: String,                    // 如 "aihub/glm-5.2"
    pub provider: String,                // 如 "aihub"
    pub model_id: String,                // 如 "glm-5.2"
    pub category: ProviderCategory,
    pub is_free: bool,
    pub avg_latency_ms: f32,
    pub avg_cost_per_1k: f32,
    pub quality_score: f32,              // 0-1, 基于 LLM Challenge / Ori-Eval
    pub capability_tags: Vec<String>,    // ["coding", "reasoning", ...]
}

/// 路由决策输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub selected_model: String,
    pub confidence: f32,
    pub fallback_chain: Vec<String>,
    pub expected_quality: f32,
    pub expected_cost: f32,
    pub expected_latency_ms: f32,
    pub pareto_score: f32,               // alpha*quality - beta*cost
}

/// 抽象路由器 trait
pub(crate) trait LearnedRouter: Send + Sync {
    fn route(&self, features: &RouteFeatures, candidates: &[CandidateModel]) -> RouteDecision;
    fn update(&mut self, features: &RouteFeatures, chosen: &str, reward: f32);
    fn name(&self) -> &str;

    /// 多轮对话感知路由 (默认退化为单轮 route; MultiTurnRouter 覆写)
    fn route_multi_turn(
        &self,
        features: &RouteFeatures,
        candidates: &[CandidateModel],
        _conv: &ConversationState,
    ) -> RouteDecision {
        self.route(features, candidates)
    }
}

/// KNN Router — 基线, 无需训练, 最近邻查询相似历史路由成功的模型
pub(crate) struct KNNRouter {
    k: usize,
    alpha: f32,
    beta: f32,
    /// 历史记录: (features, chosen_model, reward)
    history: Arc<RwLock<Vec<(RouteFeatures, String, f32)>>>,
}

impl KNNRouter {
    pub fn new(k: usize, alpha: f32, beta: f32) -> Self {
        Self {
            k,
            alpha,
            beta,
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn cosine_sim(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    fn features_to_vec(&self, f: &RouteFeatures) -> Vec<f32> {
        let mut v = f.query_embedding.clone();
        v.extend(&f.task_type);
        v.push(f.query_len_norm);
        v.push(f.code_ratio);
        v
    }
}

impl LearnedRouter for KNNRouter {
    fn route(&self, features: &RouteFeatures, candidates: &[CandidateModel]) -> RouteDecision {
        let history = match self.history.read() {
            Ok(h) => h,
            Err(_) => {
                // RwLock poisoned, fall back to first candidate
                return RouteDecision {
                    selected_model: candidates.first().map(|c| c.name.clone()).unwrap_or_default(),
                    confidence: 0.0,
                    fallback_chain: candidates.iter().map(|c| c.name.clone()).collect(),
                    expected_quality: 0.0,
                    expected_cost: 0.0,
                    expected_latency_ms: 0.0,
                    pareto_score: 0.0,
                };
            }
        };
        if history.is_empty() {
            // 冷启动: 回退 capability_score 排序
            let mut scored: Vec<_> = candidates.iter()
                .map(|c| (c.name.clone(), c.quality_score * self.alpha - c.avg_cost_per_1k * self.beta))
                .collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            return RouteDecision {
                selected_model: scored.first().map(|(n, _)| n.clone()).unwrap_or_default(),
                confidence: 0.5,
                fallback_chain: scored.iter().map(|(n, _)| n.clone()).collect(),
                expected_quality: candidates.iter().find(|c| c.name == scored.first().map(|(n, _)| n.as_str()).unwrap_or("")).map(|c| c.quality_score).unwrap_or(0.5),
                expected_cost: candidates.iter().find(|c| c.name == scored.first().map(|(n, _)| n.as_str()).unwrap_or("")).map(|c| c.avg_cost_per_1k).unwrap_or(0.0),
                expected_latency_ms: candidates.iter().find(|c| c.name == scored.first().map(|(n, _)| n.as_str()).unwrap_or("")).map(|c| c.avg_latency_ms).unwrap_or(1000.0),
                pareto_score: scored.first().map(|(_, s)| *s).unwrap_or(0.0),
            };
        }

        let query_vec = self.features_to_vec(features);
        // 计算相似度
        let mut sims: Vec<_> = history.iter()
            .map(|(h_f, h_model, h_reward)| {
                let h_vec = self.features_to_vec(h_f);
                let sim = self.cosine_sim(&query_vec, &h_vec);
                (h_model.clone(), sim, *h_reward)
            })
            .collect();
        sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Top-K 投票
        let k = self.k.min(sims.len());
        let mut votes: HashMap<String, f32> = HashMap::new();
        for (model, sim, reward) in sims.iter().take(k) {
            *votes.entry(model.clone()).or_insert(0.0) += sim * reward;
        }

        // 结合候选模型 Pareto 分数
        let mut best_model = candidates[0].name.clone();
        let mut best_score = f32::NEG_INFINITY;
        for c in candidates {
            let vote_score = votes.get(&c.name).copied().unwrap_or(0.0);
            let pareto = c.quality_score * self.alpha - c.avg_cost_per_1k * self.beta;
            let combined = 0.7 * pareto + 0.3 * vote_score;
            if combined > best_score {
                best_score = combined;
                best_model = c.name.clone();
            }
        }

        let chosen = candidates.iter().find(|c| c.name == best_model).unwrap();
        RouteDecision {
            selected_model: best_model,
            confidence: 0.7,
            fallback_chain: candidates.iter().map(|c| c.name.clone()).collect(),
            expected_quality: chosen.quality_score,
            expected_cost: chosen.avg_cost_per_1k,
            expected_latency_ms: chosen.avg_latency_ms,
            pareto_score: best_score,
        }
    }

    fn update(&mut self, features: &RouteFeatures, chosen: &str, reward: f32) {
        let mut history = self.history.write().unwrap();
        history.push((features.clone(), chosen.to_string(), reward));
        // 限制历史大小
        if history.len() > 10000 {
            history.drain(0..5000);
        }
    }

    fn name(&self) -> &str { "KNNRouter" }
}

/// MLP Router — 轻量 MLP, 离线训练好权重, 在线仅前向传播
pub(crate) struct MLPRouter {
    // 简化: 只存权重矩阵, 实际可用 candle/onnx 加载
    input_dim: usize,
    hidden_dim: usize,
    output_dim: usize,  // 候选模型数
    alpha: f32,
    beta: f32,
    // 权重: [hidden, input], [output, hidden]
    w1: Vec<Vec<f32>>,
    w2: Vec<Vec<f32>>,
    b1: Vec<f32>,
    b2: Vec<f32>,
    _model_to_idx: HashMap<String, usize>,
}

impl MLPRouter {
    pub fn new(candidates: &[CandidateModel], alpha: f32, beta: f32) -> Self {
        let input_dim = 768 + 9 + 2;  // embedding + task_type + len + code_ratio
        let hidden_dim = 256;
        let output_dim = candidates.len();
        let mut model_to_idx = HashMap::new();
        for (i, c) in candidates.iter().enumerate() {
            model_to_idx.insert(c.name.clone(), i);
        }
        // Xavier 初始化 (实际应加载训练好的权重)
        let w1 = (0..hidden_dim).map(|_| (0..input_dim).map(|_| rand::random::<f32>() * 0.02 - 0.01).collect()).collect();
        let w2 = (0..output_dim).map(|_| (0..hidden_dim).map(|_| rand::random::<f32>() * 0.02 - 0.01).collect()).collect();
        let b1 = vec![0.0; hidden_dim];
        let b2 = vec![0.0; output_dim];
        Self { input_dim, hidden_dim, output_dim, alpha, beta, w1, w2, b1, b2, _model_to_idx: model_to_idx }
    }

    fn forward(&self, x: &[f32]) -> Vec<f32> {
        // hidden = relu(w1 * x + b1)
        let mut hidden = vec![0.0; self.hidden_dim];
        for i in 0..self.hidden_dim {
            let mut sum = self.b1[i];
            for j in 0..self.input_dim {
                sum += self.w1[i][j] * x[j];
            }
            hidden[i] = sum.max(0.0);
        }
        // output = w2 * hidden + b2
        let mut output = vec![0.0; self.output_dim];
        for i in 0..self.output_dim {
            let mut sum = self.b2[i];
            for j in 0..self.hidden_dim {
                sum += self.w2[i][j] * hidden[j];
            }
            output[i] = sum;
        }
        output
    }

    fn features_to_vec(&self, f: &RouteFeatures) -> Vec<f32> {
        let mut v = f.query_embedding.clone();
        v.extend(&f.task_type);
        v.push(f.query_len_norm);
        v.push(f.code_ratio);
        // pad/truncate to input_dim
        v.resize(self.input_dim, 0.0);
        v
    }
}

impl LearnedRouter for MLPRouter {
    fn route(&self, features: &RouteFeatures, candidates: &[CandidateModel]) -> RouteDecision {
        let x = self.features_to_vec(features);
        let logits = self.forward(&x);
        // Softmax + Pareto 调整
        let mut scored: Vec<_> = candidates.iter().enumerate()
            .map(|(i, c)| {
                let pareto = c.quality_score * self.alpha - c.avg_cost_per_1k * self.beta;
                let combined = 0.6 * logits[i] + 0.4 * pareto;
                (c.name.clone(), combined, c.quality_score, c.avg_cost_per_1k, c.avg_latency_ms)
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let chosen = &scored[0];
        RouteDecision {
            selected_model: chosen.0.clone(),
            confidence: 0.85,
            fallback_chain: scored.iter().map(|(n, _, _, _, _)| n.clone()).collect(),
            expected_quality: chosen.2,
            expected_cost: chosen.3,
            expected_latency_ms: chosen.4,
            pareto_score: chosen.1,
        }
    }

    fn update(&mut self, _features: &RouteFeatures, _chosen: &str, _reward: f32) {
        // 在线微调: 可选, 需要反向传播实现
    }

    fn name(&self) -> &str { "MLPRouter" }
}

/// Hybrid Router — KNN + MLP 融合, 置信度加权
pub(crate) struct HybridRouter {
    knn: KNNRouter,
    mlp: MLPRouter,
    knn_weight: f32,
    mlp_weight: f32,
}

impl HybridRouter {
    pub fn new(candidates: &[CandidateModel], alpha: f32, beta: f32) -> Self {
        Self {
            knn: KNNRouter::new(5, alpha, beta),
            mlp: MLPRouter::new(candidates, alpha, beta),
            knn_weight: 0.4,
            mlp_weight: 0.6,
        }
    }
}

impl LearnedRouter for HybridRouter {
    fn route(&self, features: &RouteFeatures, candidates: &[CandidateModel]) -> RouteDecision {
        let knn_dec = self.knn.route(features, candidates);
        let mlp_dec = self.mlp.route(features, candidates);
        // 置信度加权融合
        let combined_score = knn_dec.pareto_score * self.knn_weight + mlp_dec.pareto_score * self.mlp_weight;
        let chosen = if mlp_dec.confidence > knn_dec.confidence { &mlp_dec } else { &knn_dec };
        RouteDecision {
            selected_model: chosen.selected_model.clone(),
            confidence: (knn_dec.confidence * self.knn_weight + mlp_dec.confidence * self.mlp_weight).min(0.95),
            fallback_chain: chosen.fallback_chain.clone(),
            expected_quality: chosen.expected_quality,
            expected_cost: chosen.expected_cost,
            expected_latency_ms: chosen.expected_latency_ms,
            pareto_score: combined_score,
        }
    }

    fn update(&mut self, features: &RouteFeatures, chosen: &str, reward: f32) {
        self.knn.update(features, chosen, reward);
        self.mlp.update(features, chosen, reward);
    }

    fn name(&self) -> &str { "HybridRouter" }
}

/// 多轮对话路由器 — 对话阶段感知的动态模型选择
///
/// 参考 LLMRouter Router-R1 / knnmultiroundrouter:
/// - 开场 (前 2 轮): 快/便宜模型即可 — 寒暄不需要 frontier 模型
/// - 探索 (3-10 轮, 低复杂度): 平衡型
/// - 深度 (高复杂度趋势 > 0.6): 升级到最强可用模型
/// - 收尾: 回落中等模型省钱
/// - Agentic 工具链: **锚定当前模型** — 链中换模型会破坏上下文一致性
/// - 预算感知: 会话超预算 → 强制降级到免费模型
pub(crate) struct MultiTurnRouter {
    /// 底层基础路由器 (单轮决策用)
    base: HybridRouter,
    alpha: f32,
    _beta: f32,
}

impl MultiTurnRouter {
    pub fn new(candidates: &[CandidateModel], alpha: f32, beta: f32) -> Self {
        Self {
            base: HybridRouter::new(candidates, alpha, beta),
            alpha,
            _beta: beta,
        }
    }

    /// 多轮路由入口 — 结合对话状态做阶段化决策
    pub fn route_with_conversation(
        &self,
        features: &RouteFeatures,
        candidates: &[CandidateModel],
        conv: &ConversationState,
    ) -> RouteDecision {
        // 0. Agentic 锚定: 工具链中强制保持同一模型 (上下文一致性优先)
        if let Some(anchored) = &conv.tool_chain_model {
            if candidates.iter().any(|c| &c.name == anchored) {
                let chosen = candidates.iter().find(|c| &c.name == anchored).unwrap();
                return RouteDecision {
                    selected_model: chosen.name.clone(),
                    confidence: 1.0,
                    fallback_chain: vec![chosen.name.clone()],
                    expected_quality: chosen.quality_score,
                    expected_cost: chosen.avg_cost_per_1k,
                    expected_latency_ms: chosen.avg_latency_ms,
                    pareto_score: self.alpha * chosen.quality_score,
                };
            }
        }

        // 1. 基础决策 (Pareto 排序)
        let base_decision = self.base.route(features, candidates);

        // 2. 预算检查: 超预算 → 强制降级到免费最优
        if let Some(budget) = conv.session_budget_usd {
            let est_cost_per_1k = base_decision.expected_cost;
            // 粗估: 剩余预算按当前成本还能撑多少 token
            if conv.session_tokens_used > 0 && est_cost_per_1k > 0.0 {
                let used_cost = conv.session_tokens_used as f32 / 1000.0 * est_cost_per_1k;
                if used_cost >= budget * 0.8 {
                    // 超 80% 预算 → 免费模型中最优
                    if let Some(free_best) = candidates.iter()
                        .filter(|c| c.is_free)
                        .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
                    {
                        return RouteDecision {
                            selected_model: free_best.name.clone(),
                            confidence: 0.9,
                            fallback_chain: vec![free_best.name.clone()],
                            expected_quality: free_best.quality_score,
                            expected_cost: 0.0,
                            expected_latency_ms: free_best.avg_latency_ms,
                            pareto_score: self.alpha * free_best.quality_score,
                        };
                    }
                }
            }
        }

        // 3. 阶段化调整
        match conv.stage() {
            ConversationStage::Agentic => unreachable!("tool chain handled above"),
            ConversationStage::Opening => {
                // 开场: 快模型优先 — 选延迟最低的前 30% 中 Pareto 最高者
                if let Some(fast_best) = fast_tier_pick(candidates) {
                    return decision_from(fast_best, 0.75);
                }
                base_decision
            }
            ConversationStage::DeepDive => {
                // 深度: 最强模型优先 — 忽略成本, 选 quality 最高
                if let Some(strongest) = candidates.iter()
                    .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
                {
                    return decision_from(strongest, 0.9);
                }
                base_decision
            }
            ConversationStage::WrappingUp | ConversationStage::TopicSwitch => {
                // 收尾/切换: 回落平衡型 (免费优先)
                if let Some(free_best) = candidates.iter()
                    .filter(|c| c.is_free)
                    .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
                {
                    return decision_from(free_best, 0.7);
                }
                base_decision
            }
            ConversationStage::Exploration => base_decision, // 探索期沿用基础 Pareto
        }
    }
}

impl LearnedRouter for MultiTurnRouter {
    fn route(&self, features: &RouteFeatures, candidates: &[CandidateModel]) -> RouteDecision {
        // 单轮调用退化为基础路由 (无对话状态)
        self.base.route(features, candidates)
    }

    fn route_multi_turn(&self, features: &RouteFeatures, candidates: &[CandidateModel], conv: &ConversationState) -> RouteDecision {
        self.route_with_conversation(features, candidates, conv)
    }

    fn update(&mut self, features: &RouteFeatures, chosen: &str, reward: f32) {
        self.base.update(features, chosen, reward);
    }

    fn name(&self) -> &str { "MultiTurnRouter" }
}

/// 快速层挑选 — 延迟最低的 40% 候选中选质量最高者
fn fast_tier_pick(candidates: &[CandidateModel]) -> Option<&CandidateModel> {
    if candidates.is_empty() { return None; }
    let mut sorted: Vec<&CandidateModel> = candidates.iter().collect();
    sorted.sort_by(|a, b| a.avg_latency_ms.partial_cmp(&b.avg_latency_ms).unwrap());
    let tier_size = (sorted.len() * 4 / 10).max(1);
    sorted[..tier_size].iter()
        .max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap())
        .copied()
}

fn decision_from(c: &CandidateModel, confidence: f32) -> RouteDecision {
    RouteDecision {
        selected_model: c.name.clone(),
        confidence,
        fallback_chain: vec![c.name.clone()],
        expected_quality: c.quality_score,
        expected_cost: c.avg_cost_per_1k,
        expected_latency_ms: c.avg_latency_ms,
        pareto_score: c.quality_score,
    }
}

/// 路由器工厂
pub(crate) struct RouterFactory;

impl RouterFactory {
    pub(crate) fn _create_router(router_type: &str, candidates: &[CandidateModel], alpha: f32, beta: f32) -> Box<dyn LearnedRouter> {
        match router_type {
            "knn" => Box::new(KNNRouter::new(5, alpha, beta)),
            "mlp" => Box::new(MLPRouter::new(candidates, alpha, beta)),
            "hybrid" => Box::new(HybridRouter::new(candidates, alpha, beta)),
            "multiturn" => Box::new(MultiTurnRouter::new(candidates, alpha, beta)),
            _ => Box::new(MultiTurnRouter::new(candidates, alpha, beta)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_io::nt_io_provider::provider_catalog::ProviderCategory;

    fn make_candidates() -> Vec<CandidateModel> {
        vec![
            CandidateModel { name: "aihub/glm-5.2".into(), provider: "aihub".into(), model_id: "glm-5.2".into(), category: ProviderCategory::Cloud, is_free: false, avg_latency_ms: 500.0, avg_cost_per_1k: 0.01, quality_score: 0.95, capability_tags: vec!["reasoning".into(), "analysis".into()] },
            CandidateModel { name: "llm7/codestral-latest".into(), provider: "llm7".into(), model_id: "codestral-latest".into(), category: ProviderCategory::Cloud, is_free: true, avg_latency_ms: 300.0, avg_cost_per_1k: 0.0, quality_score: 0.8, capability_tags: vec!["coding".into()] },
            CandidateModel { name: "api-airforce/grok-4.1-mini:free".into(), provider: "api-airforce".into(), model_id: "grok-4.1-mini:free".into(), category: ProviderCategory::Cloud, is_free: true, avg_latency_ms: 800.0, avg_cost_per_1k: 0.0, quality_score: 0.85, capability_tags: vec!["general".into()] },
        ]
    }

    #[test]
    fn test_knn_cold_start() {
        let candidates = make_candidates();
        let router = KNNRouter::new(5, 1.0, 0.1);
        let features = RouteFeatures {
            query_embedding: vec![0.1; 768],
            task_type: vec![0.0; 9],
            user_embedding: None,
            query_len_norm: 0.1,
            code_ratio: 0.0,
            hist_preference: None,
        };
        let dec = router.route(&features, &candidates);
        assert!(!dec.selected_model.is_empty());
        assert!(dec.confidence > 0.0);
    }

    #[test]
    fn test_hybrid_route() {
        let candidates = make_candidates();
        let router = HybridRouter::new(&candidates, 1.0, 0.1);
        let features = RouteFeatures {
            query_embedding: vec![0.1; 768],
            task_type: vec![0.0; 9],
            user_embedding: None,
            query_len_norm: 0.1,
            code_ratio: 0.5,
            hist_preference: None,
        };
        let dec = router.route(&features, &candidates);
        assert!(!dec.selected_model.is_empty());
        assert!(dec.confidence > 0.5);
    }

    #[test]
    fn test_conversation_state_stages() {
        let mut conv = ConversationState::default();
        assert_eq!(conv.stage(), ConversationStage::Opening);

        // 推进到探索期
        for _ in 0..5 {
            conv.advance(0.2, 0.1);
        }
        assert_eq!(conv.stage(), ConversationStage::Exploration);

        // 复杂度升高 → 深度讨论
        for _ in 0..3 {
            conv.advance(0.9, 0.1);
        }
        assert_eq!(conv.stage(), ConversationStage::DeepDive);
    }

    #[test]
    fn test_agentic_anchoring() {
        let candidates = make_candidates();
        let router = MultiTurnRouter::new(&candidates, 1.0, 0.1);
        let features = RouteFeatures {
            query_embedding: vec![0.1; 768],
            task_type: vec![0.0; 9],
            user_embedding: None,
            query_len_norm: 0.1,
            code_ratio: 0.0,
            hist_preference: None,
        };
        // 工具链锚定 → 强制返回锚定模型
        let mut conv = ConversationState::default();
        conv.enter_tool_chain("llm7/codestral-latest");
        let dec = router.route_with_conversation(&features, &candidates, &conv);
        assert_eq!(dec.selected_model, "llm7/codestral-latest");
        assert_eq!(dec.confidence, 1.0);

        conv.exit_tool_chain();
        let dec2 = router.route_with_conversation(&features, &candidates, &conv);
        assert!(!dec2.selected_model.is_empty());
    }

    #[test]
    fn test_budget_downgrade() {
        let candidates = make_candidates();
        let router = MultiTurnRouter::new(&candidates, 1.0, 0.1);
        let features = RouteFeatures {
            query_embedding: vec![0.1; 768],
            task_type: vec![0.0; 9],
            user_embedding: None,
            query_len_norm: 0.5,
            code_ratio: 0.8,
            hist_preference: None,
        };
        // 超预算 → 免费模型
        let conv = ConversationState {
            turn_count: 5,
            session_tokens_used: u64::MAX / 2, // 巨大消耗
            session_budget_usd: Some(0.01),
            ..Default::default()
        };
        let dec = router.route_with_conversation(&features, &candidates, &conv);
        let chosen = candidates.iter().find(|c| c.name == dec.selected_model).unwrap();
        assert!(chosen.is_free || dec.expected_cost == 0.0, "超预算应降级免费模型, got {}", dec.selected_model);
    }

    #[test]
    fn test_multiturn_deep_dive_upgrades() {
        let candidates = make_candidates();
        let router = MultiTurnRouter::new(&candidates, 1.0, 0.1);
        let features = RouteFeatures {
            query_embedding: vec![0.1; 768],
            task_type: vec![0.0; 9],
            user_embedding: None,
            query_len_norm: 0.8,
            code_ratio: 0.9,
            hist_preference: None,
        };
        // 深度讨论 → 应选质量最高的模型
        let mut conv = ConversationState::default();
        for _ in 0..6 {
            conv.advance(0.95, 0.05);
        }
        assert_eq!(conv.stage(), ConversationStage::DeepDive);
        let dec = router.route_with_conversation(&features, &candidates, &conv);
        let strongest = candidates.iter().max_by(|a, b| a.quality_score.partial_cmp(&b.quality_score).unwrap()).unwrap();
        assert_eq!(dec.selected_model, strongest.name, "深度讨论应选最强模型");
    }
}