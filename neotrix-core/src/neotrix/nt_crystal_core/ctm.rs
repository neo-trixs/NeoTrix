//! CTM通信机制 — Up-Tree竞争 + Down-Tree广播 + Links无意识通信
//!
//! 基于 CTM-AI + CRP + Trinity + LILITH + LEGIO 五篇前沿论文

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 通信块 — 模块间传递的基本单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub content: String,
    pub score: f64,
    pub source_module: String,
    pub chunk_type: ChunkType,
    pub metadata: HashMap<String, String>,
}

/// 块类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChunkType {
    /// 感知输入
    Perception,
    /// 记忆检索
    Memory,
    /// 行动建议
    Action,
    /// 情感信号
    Emotion,
    /// 元认知反思
    Meta,
    /// 安全检查
    Safety,
    /// 紧急中断
    Override,
}

/// 响应类型 (CRP启发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    /// 断言: 参与竞争
    Assertion { chunk: Chunk },
    /// 查询: 请求其他模块信息
    Query { question: String, source: String },
    /// 空响应: 无贡献
    Null,
    /// 紧急中断: 立即执行
    Override { action: String, priority: f64 },
}

/// Workspace: 短期记忆 (容量=1, CTM核心)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    current_chunk: Option<Chunk>,
    iteration: u64,
    pub threshold: f64,
    pub max_iterations: u64,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            current_chunk: None,
            iteration: 0,
            threshold: 0.7,
            max_iterations: 10,
        }
    }

    /// 接收Up-Tree竞争的获胜chunk
    pub fn admit(&mut self, chunk: Chunk) {
        self.current_chunk = Some(chunk);
        self.iteration += 1;
    }

    /// 检查是否通过意识门控
    pub fn is_conscious(&self) -> bool {
        self.current_chunk.as_ref()
            .map_or(false, |c| c.score >= self.threshold)
    }

    /// 检查是否应该继续迭代
    pub fn should_continue(&self) -> bool {
        self.iteration < self.max_iterations && !self.is_conscious()
    }

    /// 获取当前chunk
    pub fn current_chunk(&self) -> Option<&Chunk> {
        self.current_chunk.as_ref()
    }

    /// 获取当前迭代次数
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// 重置Workspace
    pub fn reset(&mut self) {
        self.current_chunk = None;
        self.iteration = 0;
    }
}

/// AffectState: 情感预处理 (CRP启发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectState {
    pub valence: f64,    // 正负 -1到1
    pub arousal: f64,    // 激活度 0到1
    pub dominance: f64,  // 主导度 0到1
    pub familiarity: f64, // 熟悉度 0到1
    pub urgency: f64,    // 紧急度 0到1
}

impl Default for AffectState {
    fn default() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            familiarity: 0.5,
            urgency: 0.0,
        }
    }
}

impl AffectState {
    /// 根据情感状态调整模块优先级
    pub fn adjust_priorities(
        &self,
        base_priorities: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let mut adjusted = base_priorities.clone();

        // 高arousal → 提高感知模块
        if self.arousal > 0.7 {
            *adjusted.entry("perception".into()).or_insert(0.5) += 0.2;
        }

        // 低valence → 提高安全模块
        if self.valence < -0.3 {
            *adjusted.entry("safety".into()).or_insert(0.5) += 0.3;
        }

        // 高dominance → 提高行动模块
        if self.dominance > 0.7 {
            *adjusted.entry("action".into()).or_insert(0.5) += 0.2;
        }

        // 高urgency → 提高所有模块
        if self.urgency > 0.8 {
            for (_, v) in adjusted.iter_mut() {
                *v += 0.1;
            }
        }

        adjusted
    }
}

/// Up-Tree竞争: 所有模块竞争进入Workspace
pub struct UpTreeCompetition;

impl UpTreeCompetition {
    /// 从多个响应中选择获胜者
    pub fn compete(
        responses: Vec<Response>,
        temperature: f64,
    ) -> Option<Response> {
        // 过滤出Assertion响应
        let assertions: Vec<(usize, f64)> = responses.iter()
            .enumerate()
            .filter_map(|(i, r)| {
                if let Response::Assertion { chunk } = r {
                    Some((i, chunk.score))
                } else {
                    None
                }
            })
            .collect();

        if assertions.is_empty() {
            return None;
        }

        // 检查是否有Override
        for r in &responses {
            if let Response::Override { .. } = r {
                return Some(r.clone());
            }
        }

        // 归一化分数为概率分布 (温度采样)
        let total: f64 = assertions.iter().map(|(_, s)| s).sum();
        if total <= 0.0 {
            return None;
        }

        let probabilities: Vec<f64> = assertions.iter()
            .map(|(_, s)| (s / total).powf(1.0 / temperature))
            .collect();

        // 按概率采样
        let winner_index = Self::sample_categorical(&probabilities);
        let (original_index, _) = assertions[winner_index];

        Some(responses[original_index].clone())
    }

    /// 按概率分布采样
    fn sample_categorical(probabilities: &[f64]) -> usize {
        let r: f64 = rand::random();
        let mut cumulative = 0.0;
        for (i, p) in probabilities.iter().enumerate() {
            cumulative += p;
            if r <= cumulative {
                return i;
            }
        }
        probabilities.len() - 1
    }
}

/// Down-Tree广播: 获胜chunk广播到所有模块
pub struct DownTreeBroadcast;

impl DownTreeBroadcast {
    /// 广播chunk到所有模块
    pub fn broadcast(
        chunk: &Chunk,
        modules: &mut [Box<dyn CTMModule>],
    ) {
        for module in modules.iter_mut() {
            module.write(chunk.clone());
        }
    }
}

/// Links: 模块间无意识通信 (CTM启发)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraph {
    /// 邻接矩阵
    links: Vec<Vec<bool>>,
    /// 链接强度
    strength: Vec<Vec<f64>>,
    /// 模块名称到索引的映射
    module_index: HashMap<String, usize>,
}

impl LinkGraph {
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            strength: Vec::new(),
            module_index: HashMap::new(),
        }
    }

    /// 注册模块
    pub fn register_module(&mut self, name: &str) {
        let index = self.module_index.len();
        self.module_index.insert(name.to_string(), index);

        // 扩展矩阵
        let size = self.module_index.len();
        // 扩展所有现有行的列数
        for row in self.links.iter_mut() {
            row.resize(size, false);
        }
        for row in self.strength.iter_mut() {
            row.resize(size, 0.0);
        }
        // 添加新行
        self.links.resize(size, vec![false; size]);
        self.strength.resize(size, vec![0.0; size]);
    }

    /// 形成新链接
    pub fn form_link(&mut self, name1: &str, name2: &str, strength: f64) {
        if let (Some(&i), Some(&j)) = (self.module_index.get(name1), self.module_index.get(name2)) {
            self.links[i][j] = true;
            self.links[j][i] = true;
            self.strength[i][j] = strength;
            self.strength[j][i] = strength;
        }
    }

    /// 检查两个模块是否有链接
    pub fn has_link(&self, name1: &str, name2: &str) -> bool {
        if let (Some(&i), Some(&j)) = (self.module_index.get(name1), self.module_index.get(name2)) {
            self.links[i][j]
        } else {
            false
        }
    }

    /// 获取链接强度
    pub fn link_strength(&self, name1: &str, name2: &str) -> f64 {
        if let (Some(&i), Some(&j)) = (self.module_index.get(name1), self.module_index.get(name2)) {
            self.strength[i][j]
        } else {
            0.0
        }
    }

    /// 获取模块的所有链接
    pub fn get_links(&self, name: &str) -> Vec<(String, f64)> {
        if let Some(&i) = self.module_index.get(name) {
            let mut result = Vec::new();
            for (j, &linked) in self.links[i].iter().enumerate() {
                if linked {
                    let other_name = self.module_index.iter()
                        .find(|(_, &idx)| idx == j)
                        .map(|(n, _)| n.clone())
                        .unwrap_or_default();
                    result.push((other_name, self.strength[i][j]));
                }
            }
            result
        } else {
            Vec::new()
        }
    }

    /// 获取所有已注册模块名称
    pub fn all_modules(&self) -> Vec<String> {
        self.module_index.keys().cloned().collect()
    }

    /// 移除两个模块间的链接
    pub fn remove_link(&mut self, name1: &str, name2: &str) {
        if let (Some(&i), Some(&j)) = (self.module_index.get(name1), self.module_index.get(name2)) {
            self.links[i][j] = false;
            self.links[j][i] = false;
            self.strength[i][j] = 0.0;
            self.strength[j][i] = 0.0;
        }
    }
}

/// CTM模块接口
pub trait CTMModule: Send + Sync {
    /// 模块名称
    fn name(&self) -> &str;

    /// 模块类型
    fn module_type(&self) -> ModuleType;

    /// 执行: 接收输入，生成chunk
    fn execute(&self, input: &Chunk) -> Chunk;

    /// 写入: 更新私有记忆 (接收广播)
    fn write(&mut self, chunk: Chunk);

    /// 最大响应时间 (毫秒)
    fn max_response_time_ms(&self) -> u64 {
        1000 // 默认1秒
    }
}

/// 模块类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleType {
    Perception,
    Memory,
    Action,
    Emotion,
    Meta,
    Safety,
}

/// 意识循环引擎 (10步CRP循环)
pub struct ConsciousnessLoop {
    pub workspace: Workspace,
    pub affect: AffectState,
    pub links: LinkGraph,
    pub base_priorities: HashMap<String, f64>,
}

impl ConsciousnessLoop {
    pub fn new() -> Self {
        let mut base_priorities = HashMap::new();
        base_priorities.insert("perception".into(), 0.5);
        base_priorities.insert("memory".into(), 0.5);
        base_priorities.insert("action".into(), 0.5);
        base_priorities.insert("emotion".into(), 0.5);
        base_priorities.insert("meta".into(), 0.5);
        base_priorities.insert("safety".into(), 0.5);

        Self {
            workspace: Workspace::new(),
            affect: AffectState::default(),
            links: LinkGraph::new(),
            base_priorities,
        }
    }

    /// 执行一步意识循环
    pub fn step(
        &mut self,
        input: &Chunk,
        modules: &mut [Box<dyn CTMModule>],
    ) -> Option<Chunk> {
        // Step 1-4: 感知+记忆+情感+优先级调整 (已在外部完成)

        // Step 5: 所有模块并行执行，生成chunks
        let priorities = self.affect.adjust_priorities(&self.base_priorities);
        let mut responses: Vec<Response> = Vec::new();

        for module in modules.iter() {
            let chunk = module.execute(input);
            let priority = priorities.get(module.name()).copied().unwrap_or(0.5);
            let adjusted_score = chunk.score * priority;

            responses.push(Response::Assertion {
                chunk: Chunk {
                    score: adjusted_score,
                    ..chunk
                },
            });
        }

        // Step 6: Up-Tree竞争
        let winner = UpTreeCompetition::compete(responses, 1.0)?;

        // Step 7: 检查Override
        if let Response::Override { action, .. } = &winner {
            // 紧急中断，直接返回
            return Some(Chunk {
                content: action.clone(),
                score: 1.0,
                source_module: "override".into(),
                chunk_type: ChunkType::Override,
                metadata: HashMap::new(),
            });
        }

        // Step 8: 接收到Workspace
        if let Response::Assertion { chunk } = winner {
            self.workspace.admit(chunk.clone());

            // Step 9: Down-Tree广播
            DownTreeBroadcast::broadcast(&chunk, modules);

            // Step 10: 记忆形成 (由各模块自行处理)

            return Some(chunk);
        }

        None
    }
}
