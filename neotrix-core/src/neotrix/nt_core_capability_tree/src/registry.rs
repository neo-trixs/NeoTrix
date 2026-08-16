//! 能力注册表

use crate::evolution::{EvolutionAction, EvolutionPlan};
use crate::node::{Domain, NodeLayer};
pub use crate::node::CapabilityNode;
use indexmap::IndexMap;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

/// 最短路径结果 (意识能力网最优解路由)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortestPath {
    /// 从起点到终点的节点 id 序列 (含两端)
    pub path: Vec<String>,
    /// 跳数 (边数)
    pub hops: usize,
    /// 加权成本 (成熟度折扣后; 越低越优)
    pub cost: f64,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("节点已存在: {0}")]
    AlreadyExists(String),
    #[error("节点不存在: {0}")]
    NotFound(String),
    #[error("依赖循环: {0} -> {1}")]
    CircularDependency(String, String),
    #[error("非法层级跨度: {0} (L{1}) 依赖 {2} (L{3})")]
    InvalidLayerSpan(String, u8, String, u8),
    #[error("序列化错误: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 注册表核心
#[derive(Debug, Clone)]
pub struct CapabilityRegistry {
    pub nodes: IndexMap<String, CapabilityNode>,  // id -> node
    domain_index: HashMap<Domain, Vec<String>>,
    layer_index: HashMap<NodeLayer, Vec<String>>,
    constellation_index: HashMap<u8, Vec<String>>,  // C0-C6 -> node ids
    provides_index: HashMap<String, Vec<String>>,   // capability tag -> node ids
    dag: petgraph::Graph<String, ()>,               // 依赖图
    node_indices: HashMap<String, petgraph::graph::NodeIndex>,
    /// 经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)
    /// 值结构: {domain, capability, action, rationale, signal, promoted_at}
    pub experience_targets: Vec<serde_json::Value>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            domain_index: HashMap::new(),
            layer_index: HashMap::new(),
            constellation_index: HashMap::new(),
            provides_index: HashMap::new(),
            dag: petgraph::Graph::new(),
            node_indices: HashMap::new(),
            experience_targets: Vec::new(),
        }
    }

    /// 注册新节点
    pub fn register(&mut self, node: CapabilityNode) -> Result<(), RegistryError> {
        if self.nodes.contains_key(&node.id) {
            return Err(RegistryError::AlreadyExists(node.id));
        }

        // 验证依赖存在: requires 可引用节点 ID 或 provides tag (双命名空间)。
        // 修复: 仅查 provides_index 会误报所有节点 ID 引用 (requires 语义是 ID)。
        for req in &node.requires {
            let resolved = self.nodes.contains_key(req) || self.provides_index.contains_key(req);
            if !resolved {
                // 允许前向声明，但发出警告
                eprintln!("[capability_tree] WARNING: dependency '{}' not yet registered for node '{}'", req, node.id);
            }
        }

        // 添加到 DAG
        let idx = self.dag.add_node(node.id.clone());
        self.node_indices.insert(node.id.clone(), idx);

        // 建立依赖边 (幂等: requires 中重复或已存在的边跳过, 防平行边)
        for req in &node.requires {
            if let Some(&req_idx) = self.node_indices.get(req) {
                if self.dag.find_edge(idx, req_idx).is_none() {
                    self.dag.add_edge(idx, req_idx, ());
                }
            }
        }

        // 检查循环依赖
        if petgraph::algo::is_cyclic_directed(&self.dag) {
            // 回滚
            self.dag.remove_node(idx);
            self.node_indices.remove(&node.id);
            return Err(RegistryError::CircularDependency(node.id, "circular".into()));
        }

        // 更新索引
        let id = node.id.clone();
        let domain = node.domain;
        let layer = node.layer;
        let constellation = node.constellation as u8;
        let provides = node.provides.clone();

        self.domain_index.entry(domain).or_default().push(id.clone());
        self.layer_index.entry(layer).or_default().push(id.clone());
        self.constellation_index.entry(constellation).or_default().push(id.clone());
        for tag in provides {
            self.provides_index.entry(tag).or_default().push(id.clone());
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// 获取节点
    pub fn get(&self, id: &str) -> Option<&CapabilityNode> {
        self.nodes.get(id)
    }

    /// 可变获取
    pub fn get_mut(&mut self, id: &str) -> Option<&mut CapabilityNode> {
        self.nodes.get_mut(id)
    }

    /// 删除节点 (仅允许无依赖者时)
    pub fn remove(&mut self, id: &str) -> Result<CapabilityNode, RegistryError> {
        let node = self.nodes.shift_remove(id)
            .ok_or(RegistryError::NotFound(id.to_string()))?;

        if !node.dependents.is_empty() {
            return Err(RegistryError::CircularDependency(
                id.to_string(),
                format!("has dependents: {:?}", node.dependents)
            ));
        }

        // 从 DAG 移除
        if let Some(&idx) = self.node_indices.get(id) {
            self.dag.remove_node(idx);
            self.node_indices.remove(id);
        }

        // 更新索引
        if let Some(v) = self.domain_index.get_mut(&node.domain) {
            v.retain(|x| x != id);
        }
        if let Some(v) = self.layer_index.get_mut(&node.layer) {
            v.retain(|x| x != id);
        }
        if let Some(v) = self.constellation_index.get_mut(&(node.constellation as u8)) {
            v.retain(|x| x != id);
        }
        for tag in &node.provides {
            if let Some(v) = self.provides_index.get_mut(tag) {
                v.retain(|x| x != id);
            }
        }

        // 从依赖者的 dependents 移除
        for req in &node.requires {
            if let Some(dep_node) = self.nodes.get_mut(req) {
                dep_node.remove_dependent(id);
            }
        }

        Ok(node)
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, from: &str, to: &str) -> Result<(), RegistryError> {
        let from_idx = *self.node_indices.get(from).ok_or_else(|| RegistryError::NotFound(from.into()))?;
        let to_idx = *self.node_indices.get(to).ok_or_else(|| RegistryError::NotFound(to.into()))?;

        // 检查层级跨度 (L0 只能被 L1+ 依赖，不能依赖 L3+ 等)
        let from_node = &self.nodes[from];
        let to_node = &self.nodes[to];
        let from_layer = from_node.layer as u8;
        let to_layer = to_node.layer as u8;
        
        // 允许向下依赖 (高层依赖低层)，禁止向上依赖超过 1 层
        if to_layer > from_layer + 1 {
            return Err(RegistryError::InvalidLayerSpan(
                from.into(), from_layer, to.into(), to_layer
            ));
        }

        // 幂等守卫: 边已存在则跳过 (petgraph Graph 是 multigraph, 允许平行边;
        // load→save 循环会累加重复边, 污染能力网统计与最优解路由)。
        if self.dag.find_edge(from_idx, to_idx).is_some() {
            self.sync_dependency_fields(from, to);
            return Ok(());
        }

        self.dag.add_edge(from_idx, to_idx, ());
        
        if petgraph::algo::is_cyclic_directed(&self.dag) {
            // 回滚刚加的边；边必然存在（add_edge 刚成功），但防御性处理避免 panic。
            if let Some(edge) = self.dag.find_edge(from_idx, to_idx) {
                self.dag.remove_edge(edge);
            }
            return Err(RegistryError::CircularDependency(from.into(), to.into()));
        }

        self.sync_dependency_fields(from, to);
        Ok(())
    }

    /// 同步双向依赖字段 (dependents + requires)。
    ///
    /// 语义: 保证 DAG 边与节点字段一致 — dependents 记录"谁依赖我",
    /// requires 记录"我依赖谁" (与 DAG node_indices 一致)。add_dependency
    /// 新建边与幂等命中 (边已存在) 两条路径都调用, 确保字段同步。
    fn sync_dependency_fields(&mut self, from: &str, to: &str) {
        if let Some(node) = self.nodes.get_mut(to) {
            node.add_dependent(from.into());
        }
        if let Some(node) = self.nodes.get_mut(from) {
            if !node.requires.iter().any(|r| r == to) {
                node.requires.push(to.to_string());
            }
        }
    }

    /// 查询: 按领域
    pub fn by_domain(&self, domain: Domain) -> Vec<&CapabilityNode> {
        self.domain_index.get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按层级
    pub fn by_layer(&self, layer: NodeLayer) -> Vec<&CapabilityNode> {
        self.layer_index.get(&layer)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按星座等级
    pub fn by_constellation(&self, level: u8) -> Vec<&CapabilityNode> {
        self.constellation_index.get(&level)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 按提供能力标签
    pub fn by_provides(&self, tag: &str) -> Vec<&CapabilityNode> {
        self.provides_index.get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 查询: 所有 L0 Primitive
    pub fn all_primitives(&self) -> Vec<&CapabilityNode> {
        self.by_layer(NodeLayer::L0Primitive)
    }

    /// 查询: 所有待晋升节点 (Cn 全绿 + 无阻塞)
    pub fn promotable_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes.values()
            .filter(|n| !n.deprecated && n.constellation.next().is_some())
            .collect()
    }

    /// 查询: 孤儿节点 (无 dependents 且非入口点)
    pub fn orphan_nodes(&self) -> Vec<&CapabilityNode> {
        self.nodes.values()
            .filter(|n| n.dependents.is_empty() && !n.is_constellation())
            .collect()
    }

    /// 查询: 过期节点 (C0/C1 超 3 周期无更新)
    pub fn stale_nodes(&self, cycles_threshold: u32) -> Vec<&CapabilityNode> {
        // 简化: 基于 evolution_log 的最后 cycle 判断
        // 缺陷修复 (R-P42 吸收): 新注册节点 (evolution_log 为空或仅 1 条吸收记录)
        // 不应被误判为 stale — stale 只针对"已存在多 cycle 但无新演化"的节点。
        // 判定: 有 >=2 条演化记录 (非吸收期) 且记录数 < 阈值 → stale。
        self.nodes.values()
            .filter(|n| {
                n.constellation as u8 <= 1 &&
                n.evolution_log.len() >= 2 &&
                n.evolution_log.len() < cycles_threshold as usize
            })
            .collect()
    }

    /// 获取依赖拓扑序 (用于构建顺序)
    pub fn topological_order(&self) -> Result<Vec<String>, RegistryError> {
        use petgraph::algo::toposort;
        let order = toposort(&self.dag, None)
            .map_err(|_| RegistryError::CircularDependency("graph".into(), "has cycles".into()))?;
        Ok(order.into_iter().map(|idx| self.dag[idx].clone()).collect())
    }

    /// 获取反向依赖 (谁依赖我)
    pub fn reverse_dependencies(&self, id: &str) -> Vec<String> {
        self.nodes.get(id).map(|n| n.dependents.clone()).unwrap_or_default()
    }

    /// 获取直接依赖 (我依赖谁)
    pub fn direct_dependencies(&self, id: &str) -> Vec<String> {
        self.nodes.get(id).map(|n| n.requires.clone()).unwrap_or_default()
    }

    /// 统计信息
    pub fn stats(&self) -> RegistryStats {
        let mut by_domain = HashMap::new();
        let mut by_layer = HashMap::new();
        let mut by_constellation = HashMap::new();

        for node in self.nodes.values() {
            *by_domain.entry(node.domain.as_str().to_string()).or_insert(0) += 1;
            *by_layer.entry(node.layer.as_str().to_string()).or_insert(0) += 1;
            *by_constellation.entry(node.constellation.as_str().to_string()).or_insert(0) += 1;
        }

        RegistryStats {
            total_nodes: self.nodes.len(),
            by_domain,
            by_layer,
            by_constellation,
            deprecated_count: self.nodes.values().filter(|n| n.deprecated).count(),
            primitive_count: self.all_primitives().len(),
        }
    }

    /// 检查是否存在循环依赖
    pub fn has_cycles(&self) -> bool {
        petgraph::algo::is_cyclic_directed(&self.dag)
    }

    /// 导出为可序列化结构 (边去重: petgraph multigraph 可能含平行边,
    /// 历史遗留 load→save 循环产生的重复边不再写出)。
    pub fn export(&self) -> RegistryExport {
        let mut edges = Vec::new();
        let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
        for edge in self.dag.edge_references() {
            let a_idx = edge.source();
            let b_idx = edge.target();
            let a = &self.dag[a_idx];
            let b = &self.dag[b_idx];
            if seen.insert((a.clone(), b.clone())) {
                edges.push((a.clone(), b.clone()));
            }
        }
        RegistryExport {
            nodes: self.nodes.values().cloned().collect(),
            edges,
            experience_targets: self.experience_targets.clone(),
        }
    }

    /// 经验目标 → 演化计划 (断链 #2 修复: 后台自动消费 experience_targets)。
    ///
    /// 消费 distill 蒸馏写入的 experience_targets (capability_registry.json),
    /// 每个 target 映射为 Strengthen (已有节点提供该标签) 或 Budding (新 exp:: 节点)。
    /// 与 CLI `scan --apply` 的 experience_target_plans 逻辑一致, 抽为注册表公共 API
    /// 供后台自动进化 (handle_capability_auto_evolve) 复用 — 消除"仅 CLI 手动消费"断链。
    ///
    /// 注意: 本方法只生成计划, 不修改 experience_targets; 消费方执行后自行 clear()。
    pub fn plan_experience_targets(&self, cycle: &str) -> Vec<EvolutionPlan> {
        let mut plans = Vec::new();
        let targets = &self.experience_targets;
        if targets.is_empty() {
            return plans;
        }
        // 去重: 同域同标签只生成一个计划 (防重复 id 注册失败)
        let mut already_planned: std::collections::HashSet<String> = std::collections::HashSet::new();
        for t in targets {
            let Some(domain_s) = t.get("domain").and_then(|d| d.as_str()) else { continue };
            let Some(signal) = t.get("signal").and_then(|s| s.as_f64()) else { continue };
            let Some(rationale) = t.get("rationale").and_then(|r| r.as_str()) else { continue };
            let Some(domain) = Domain::parse(domain_s) else { continue };
            let capability_tag = t.get("capability").and_then(|c| c.as_str()).unwrap_or("").to_string();
            if capability_tag.is_empty() {
                continue;
            }
            // 意识体觉醒目标: 不映射能力节点, 仅记录 (消费在 NT-META 层)
            if capability_tag.starts_with("consciousness::") {
                continue;
            }
            if !already_planned.insert(format!("{}::{}", domain_s, capability_tag)) {
                continue;
            }
            // 找域内提供该标签的现有节点 → Strengthen; 缺失 → Bud
            let candidates: Vec<&CapabilityNode> = self
                .by_domain(domain)
                .into_iter()
                .filter(|n| n.provides.iter().any(|p| p == &capability_tag) && !n.deprecated)
                .collect();
            if let Some(target) = candidates.first() {
                plans.push(EvolutionPlan {
                    cycle: cycle.to_string(),
                    actions: vec![EvolutionAction::Strengthen {
                        node_id: target.id.clone(),
                        note: format!("{} | signal={:.2}", rationale, signal),
                    }],
                    rationale: format!("经验驱动: 强化 {} | {}", capability_tag, rationale),
                });
            } else {
                plans.push(EvolutionPlan {
                    cycle: cycle.to_string(),
                    actions: vec![EvolutionAction::Budding {
                        new_node_id: format!("exp::{}::{}", domain.as_str().to_lowercase(), capability_tag),
                        domain,
                        provides: vec![capability_tag.clone()],
                        layer: NodeLayer::L0Primitive,
                        note: format!("经验驱动新节点: {}", rationale),
                    }],
                    rationale: format!("经验驱动: 新建 {} | {}", capability_tag, rationale),
                });
            }
        }
        plans
    }

    /// exp:: 虚拟节点老化回收 (断链 #3 修复: exp:: 节点只增不灭)。
    ///
    /// 经验蒸馏 Bud 的 exp:: 虚拟节点是临时载体; 超过 `days` 天无任何演化活动
    /// (evolution_log 最新时间戳距今超过阈值) 说明该经验已不再被强化,
    /// 返回其 id 列表供 auto_scan 标记 deprecated / prune。
    /// 真实模块节点 (非 exp:: 前缀) 永不回收 — 回收只针对经验虚拟节点。
    pub fn aged_exp_nodes(&self, days: u64) -> Vec<String> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        self.nodes
            .values()
            .filter(|n| n.id.starts_with("exp::"))
            .filter(|n| {
                // 无演化日志 → 视为从未活动, 直接老化
                let last = n.evolution_log.iter().map(|e| e.timestamp).max();
                match last {
                    Some(ts) => ts < cutoff,
                    None => true,
                }
            })
            .map(|n| n.id.clone())
            .collect()
    }

    // ─────────────────────────────────────────────────────────────
    // 最短路径智能算法 (意识能力网最优解路由)
    // ─────────────────────────────────────────────────────────────

    /// 节点成熟度权重: C0=1.0, C1=0.85, C2=0.7, C3=0.55, C4=0.4, C5=0.3, C6=0.2
    /// 越成熟 (高 C) 成本越低 → 算法倾向走已验证路径。
    fn maturity_weight(constellation: u8) -> f64 {
        match constellation {
            0 => 1.0,
            1 => 0.85,
            2 => 0.70,
            3 => 0.55,
            4 => 0.40,
            5 => 0.30,
            _ => 0.20,
        }
    }

    /// 多维最优解成本 (LoopX 吸收: maturity + evidence + gates)。
    ///
    /// 成本 = 成熟度权重 + evidence 修正 + deprecated 门禁惩罚。
    /// - maturity: 越成熟成本越低 (已验证路径)
    /// - evidence: evolution_log 条数 ≥3 → 有充分证据, −0.1; =0 → 无证据, +0.1
    /// - gates: deprecated 节点 → +5.0 (几乎不可选, 但允许兜底而非硬排除)
    fn node_cost(&self, node: &CapabilityNode) -> f64 {
        let mut cost = Self::maturity_weight(node.constellation as u8);
        let evidence = node.evolution_log.len();
        if evidence >= 3 {
            cost -= 0.1;
        } else if evidence == 0 {
            cost += 0.1;
        }
        if node.deprecated {
            cost += 5.0;
        }
        cost.max(0.05)
    }

    /// 路径总成本: 路径上所有节点 cost 之和。
    fn path_cost(&self, path: &[String]) -> f64 {
        path.iter()
            .map(|id| self.nodes.get(id).map(|n| self.node_cost(n)).unwrap_or(1.0))
            .sum()
    }

    /// BFS 最短依赖链: 从目标节点沿依赖边走到最近的 primitive (L0)。
    /// 返回跳数最少的路径 (无权最短路径)。
    pub fn shortest_path_to_primitive(&self, target: &str) -> Option<ShortestPath> {
        let target_idx = *self.node_indices.get(target)?;
        let mut prev: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> = HashMap::new();
        let mut dist: HashMap<petgraph::graph::NodeIndex, usize> = HashMap::new();
        let mut queue: VecDeque<petgraph::graph::NodeIndex> = VecDeque::new();
        dist.insert(target_idx, 0);
        queue.push_back(target_idx);

        let mut found: Option<petgraph::graph::NodeIndex> = None;
        while let Some(cur) = queue.pop_front() {
            let cur_dist = dist[&cur];
            // 到达 primitive (无 requires) → 最短路径终点
            if self.nodes.get(&self.dag[cur]).map(|n| n.requires.is_empty()).unwrap_or(false) {
                found = Some(cur);
                break;
            }
            // 沿依赖边 (cur -> requires) 扩展
            for edge in self.dag.edges_directed(cur, Direction::Outgoing) {
                let next = edge.target();
                dist.entry(next).or_insert_with(|| {
                    prev.insert(next, cur);
                    queue.push_back(next);
                    cur_dist + 1
                });
            }
        }

        let end = found?;
        // 回溯路径
        let mut path = vec![self.dag[end].clone()];
        let mut cur = end;
        while let Some(&p) = prev.get(&cur) {
            path.push(self.dag[p].clone());
            cur = p;
        }
        path.reverse();
        let hops = path.len().saturating_sub(1);
        let cost = self.path_cost(&path);
        Some(ShortestPath { path, hops, cost })
    }

    /// 加权最优路径: 从 `from` 到 `to` 的 Dijkstra 最短路径。
    /// cost 语义: 路径上所有节点成本之和 (含两端, 与 BFS path_cost 一致)。
    /// 返回 None 表示不可达。
    pub fn optimal_path_between(&self, from: &str, to: &str) -> Option<ShortestPath> {
        let from_idx = *self.node_indices.get(from)?;
        let to_idx = *self.node_indices.get(to)?;
        if from_idx == to_idx {
            let cost = self.nodes.get(from).map(|n| self.node_cost(n)).unwrap_or(0.0);
            return Some(ShortestPath { path: vec![from.to_string()], hops: 0, cost });
        }

        // Dijkstra (手动维护距离表); dist[from] 含起点成本 (统一 cost 语义)
        let start_cost = self.nodes.get(from).map(|n| self.node_cost(n)).unwrap_or(0.0);
        let mut dist: HashMap<petgraph::graph::NodeIndex, f64> = HashMap::new();
        let mut prev: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> = HashMap::new();
        let mut visited: std::collections::HashSet<petgraph::graph::NodeIndex> = std::collections::HashSet::new();
        dist.insert(from_idx, start_cost);

        loop {
            // 选未访问最小距离节点
            let cur = dist.iter()
                .filter(|(k, _)| !visited.contains(k))
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| *k);
            let Some(cur) = cur else { break };
            if cur == to_idx { break; }
            visited.insert(cur);
            let cur_dist = dist[&cur];

            // 沿依赖边扩展 (cur -> requires)
            for edge in self.dag.edges_directed(cur, Direction::Outgoing) {
                let next = edge.target();
                if visited.contains(&next) { continue; }
                let w = self.nodes.get(&self.dag[next])
                    .map(|n| self.node_cost(n))
                    .unwrap_or(1.0);
                let nd = cur_dist + w;
                if nd < *dist.get(&next).unwrap_or(&f64::INFINITY) {
                    dist.insert(next, nd);
                    prev.insert(next, cur);
                }
            }
        }

        if !prev.contains_key(&to_idx) {
            return None;
        }
        // 回溯
        let mut path = vec![self.dag[to_idx].clone()];
        let mut cur = to_idx;
        while let Some(&p) = prev.get(&cur) {
            path.push(self.dag[p].clone());
            cur = p;
        }
        path.reverse();
        let hops = path.len().saturating_sub(1);
        let cost = dist.get(&to_idx).copied().unwrap_or(0.0);
        Some(ShortestPath { path, hops, cost })
    }

    /// 最优解路由: 给定目标能力 tag, 返回所有提供该能力的节点中
    /// 加权成本最低者 (LoopX 吸收: 多维最优解, 非单纯最短跳数)。
    pub fn optimal_provider(&self, capability_tag: &str) -> Option<ShortestPath> {
        let providers = self.by_provides(capability_tag);
        providers.iter()
            // 对每个 provider 计算"最优依赖链" (Dijkstra 加权),
            // 而非 BFS 最短跳数 — 成熟度 + evidence + gates 全部参与选优。
            .filter_map(|n| self.optimal_path_to_primitive(&n.id))
            .min_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 最优依赖链 (Dijkstra 加权): 从目标节点到最近 primitive 的
    /// 加权成本最低路径。与 BFS `shortest_path_to_primitive` 的区别:
    /// 使用多维 cost (成熟度 + evidence + deprecated 门禁) 而非跳数。
    pub fn optimal_path_to_primitive(&self, target: &str) -> Option<ShortestPath> {
        let target_idx = *self.node_indices.get(target)?;
        let mut dist: HashMap<petgraph::graph::NodeIndex, f64> = HashMap::new();
        let mut prev: HashMap<petgraph::graph::NodeIndex, petgraph::graph::NodeIndex> = HashMap::new();
        let mut visited: std::collections::HashSet<petgraph::graph::NodeIndex> = std::collections::HashSet::new();
        dist.insert(target_idx, 0.0);

        let mut found: Option<petgraph::graph::NodeIndex> = None;
        loop {
            let cur = dist.iter()
                .filter(|(k, _)| !visited.contains(k))
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(k, _)| *k);
            let Some(cur) = cur else { break };
            visited.insert(cur);
            let cur_dist = dist[&cur];
            // 到达 primitive (无 requires) → 最优路径终点 (贪心: 首个弹出的必为最优)
            if self.nodes.get(&self.dag[cur]).map(|n| n.requires.is_empty()).unwrap_or(false) {
                found = Some(cur);
                break;
            }
            for edge in self.dag.edges_directed(cur, Direction::Outgoing) {
                let next = edge.target();
                if visited.contains(&next) { continue; }
                let w = self.nodes.get(&self.dag[next])
                    .map(|n| self.node_cost(n))
                    .unwrap_or(1.0);
                let nd = cur_dist + w;
                if nd < *dist.get(&next).unwrap_or(&f64::INFINITY) {
                    dist.insert(next, nd);
                    prev.insert(next, cur);
                }
            }
        }

        let end = found?;
        // 回溯路径
        let mut path = vec![self.dag[end].clone()];
        let mut cur = end;
        while let Some(&p) = prev.get(&cur) {
            path.push(self.dag[p].clone());
            cur = p;
        }
        path.reverse();
        let hops = path.len().saturating_sub(1);
        let cost = dist.get(&end).copied().unwrap_or(0.0);
        Some(ShortestPath { path, hops, cost })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryExport {
    pub nodes: Vec<CapabilityNode>,
    pub edges: Vec<(String, String)>,
    /// 经验驱动迭代目标 (distill 蒸馏写入, scan --apply 消费)。
    /// 默认空: 旧文件无此字段时兼容反序列化。
    #[serde(default)]
    pub experience_targets: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RegistryStats {
    pub total_nodes: usize,
    pub by_domain: HashMap<String, usize>,
    pub by_layer: HashMap<String, usize>,
    pub by_constellation: HashMap<String, usize>,
    pub deprecated_count: usize,
    pub primitive_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::CapabilityNode;

    fn build_test_registry() -> CapabilityRegistry {
        let mut reg = CapabilityRegistry::new();
        // L0 primitives
        reg.register(CapabilityNode::new_primitive("p1".into(), Domain::Core, vec!["read".into()])).unwrap();
        reg.register(CapabilityNode::new_primitive("p2".into(), Domain::Core, vec!["bash".into()])).unwrap();
        // L1 composite: c1 requires p1
        let mut c1 = CapabilityNode::new_primitive("c1".into(), Domain::Core, vec!["grep".into()]);
        c1.requires = vec!["p1".into()];
        reg.register(c1).unwrap();
        // L2 composite: c2 requires c1 + p2
        let mut c2 = CapabilityNode::new_primitive("c2".into(), Domain::Core, vec!["websearch".into()]);
        c2.requires = vec!["c1".into(), "p2".into()];
        reg.register(c2).unwrap();
        // L3 composite: c3 requires c2 (longer path)
        let mut c3 = CapabilityNode::new_primitive("c3".into(), Domain::Core, vec!["websearch".into()]);
        c3.requires = vec!["c2".into()];
        reg.register(c3).unwrap();
        reg
    }

    #[test]
    fn test_shortest_path_to_primitive() {
        let reg = build_test_registry();
        let sp = reg.shortest_path_to_primitive("c2").expect("c2 reachable");
        // c2 -> c1 -> p1 (2 hops) 或 c2 -> p2 (1 hop); BFS 应选最短: c2 -> p2
        assert_eq!(sp.hops, 1, "BFS must pick shortest: {:?}", sp.path);
        assert_eq!(sp.path, vec!["c2".to_string(), "p2".to_string()]);
    }

    #[test]
    fn test_optimal_path_between() {
        let reg = build_test_registry();
        let sp = reg.optimal_path_between("c2", "p1").expect("reachable");
        assert_eq!(sp.path, vec!["c2".to_string(), "c1".to_string(), "p1".to_string()]);
        assert_eq!(sp.hops, 2);
    }

    #[test]
    fn test_optimal_provider_picks_cheapest() {
        let reg = build_test_registry();
        // websearch 由 c2 (2 依赖) 和 c3 (3 依赖) 提供 → 应选 c2
        let sp = reg.optimal_provider("websearch").expect("provider exists");
        assert_eq!(sp.path[0], "c2", "must pick cheapest provider: {:?}", sp.path);
    }

    #[test]
    fn test_unreachable_returns_none() {
        let reg = build_test_registry();
        assert!(reg.shortest_path_to_primitive("ghost").is_none());
        assert!(reg.optimal_path_between("c2", "ghost").is_none());
    }

    /// requires 语义: 引用节点 ID (非 provides tag)。注册校验需同时接受
    /// 节点 ID 与 tag 双命名空间 — 仅查 provides_index 会误报节点 ID 引用。
    #[test]
    fn test_requires_node_id_resolves_without_false_warning() {
        let mut reg = CapabilityRegistry::new();
        // 依赖先注册 (全路径 ID, 提供 tag 'mode_routing')
        let dep = CapabilityNode::new_primitive(
            "nt_core_gwt::mode_router".into(), Domain::Core, vec!["mode_routing".into()],
        );
        reg.register(dep).unwrap();

        // 消费者 requires 引用节点 ID (非 tag) → 应无警告、DAG 边成立
        let mut consumer = CapabilityNode::new_primitive(
            "nt_core_parallel::atomic_decomposition".into(), Domain::Core, vec!["atomic".into()],
        );
        consumer.requires = vec!["nt_core_gwt::mode_router".into()];
        reg.register(consumer).unwrap();

        // requires 字段被保留 (字段级, 与 DAG 边互补)
        let c = reg.get("nt_core_parallel::atomic_decomposition").unwrap();
        assert_eq!(c.requires, vec!["nt_core_gwt::mode_router".to_string()]);
        // DAG 边建立: 消费者 → 依赖
        let from = reg.node_indices.get("nt_core_parallel::atomic_decomposition").unwrap();
        let to = reg.node_indices.get("nt_core_gwt::mode_router").unwrap();
        assert!(reg.dag.find_edge(*from, *to).is_some(), "ID 引用应建 DAG 边");
        // 依赖节点可被 ID 解析 (not yet registered 误报根源)
        assert!(reg.get("nt_core_gwt::mode_router").is_some());
    }

    /// 多维最优解: deprecated 节点 (gates) 应被避开 — Dijkstra 加权路由。
    #[test]
    fn test_optimal_avoids_deprecated() {
        let mut reg = build_test_registry();
        // 把 p2 标记 deprecated → optimal_path_to_primitive("c2") 应走 c2->c1->p1
        let p2 = reg.get_mut("p2").unwrap();
        p2.deprecated = true;
        p2.deprecated_reason = Some("test".into());
        let sp = reg.optimal_path_to_primitive("c2").expect("c2 reachable");
        assert_eq!(sp.path[1], "c1", "must avoid deprecated p2: {:?}", sp.path);
        assert_eq!(sp.hops, 2);
    }

    /// 多维最优解: evidence (evolution_log ≥3) 降低成本。
    #[test]
    fn test_evidence_reduces_cost() {
        let reg = build_test_registry();
        // p1 无证据 (+0.1), p2 无证据 (+0.1) → 两 primitive 同成本
        let sp_p1 = reg.shortest_path_to_primitive("c1").unwrap();
        // c1 -> p1: cost(c1) + cost(p1)
        let cost_before = sp_p1.cost;

        // 给 p1 加 3 条 evolution_log → 证据充分 −0.1
        let mut reg2 = build_test_registry();
        let node = CapabilityNode::new_primitive("extra".into(), Domain::Core, vec!["x".into()]);
        let entry = crate::node::EvolutionLogEntry {
            cycle: "c1".into(),
            op: crate::node::EvolutionOp::Strengthen,
            from_nodes: vec![],
            to_node: None,
            note: "evidence".into(),
            timestamp: chrono::Utc::now(),
        };
        // 直接构造带证据的 p1
        reg2.get_mut("p1").unwrap().evolution_log = vec![entry.clone(), entry.clone(), entry];
        let sp_p1_ev = reg2.shortest_path_to_primitive("c1").unwrap();
        assert!(sp_p1_ev.cost < cost_before, "evidence must reduce cost: {} < {}", sp_p1_ev.cost, cost_before);
    }

    /// 幂等守卫: add_dependency 对已存在的边必须跳过 (petgraph multigraph
    /// 允许平行边, 无条件 add_edge 会让 load→save 循环累加重复边)。
    #[test]
    fn test_add_dependency_is_idempotent() {
        let mut reg = build_test_registry();
        // build_test_registry 有 4 条唯一边: c1->p1, c2->c1, c2->p2, c3->c2
        let before = reg.export().edges.len();
        reg.add_dependency("c1", "p1").unwrap();
        reg.add_dependency("c1", "p1").unwrap();
        reg.add_dependency("c1", "p1").unwrap();
        let export = reg.export();
        let dup = export.edges.iter().filter(|(a, b)| a == "c1" && b == "p1").count();
        assert_eq!(dup, 1, "重复边必须被去重, 得到 {}", dup);
        assert_eq!(export.edges.len(), before, "幂等命中不得新增边");
    }

    /// export 去重: 即使 DAG 内已混入平行边, 导出也必须去重。
    #[test]
    fn test_export_dedupes_parallel_edges() {
        let mut reg = build_test_registry();
        // 模拟历史遗留: 直接向 DAG 塞入平行边 (绕过 add_dependency 幂等)
        let from_idx = *reg.node_indices.get("c2").unwrap();
        let to_idx = *reg.node_indices.get("p2").unwrap();
        reg.dag.add_edge(from_idx, to_idx, ());
        reg.dag.add_edge(from_idx, to_idx, ());
        let export = reg.export();
        let dup = export.edges.iter().filter(|(a, b)| a == "c2" && b == "p2").count();
        assert_eq!(dup, 1, "export 必须去重平行边, 得到 {}", dup);
    }
}