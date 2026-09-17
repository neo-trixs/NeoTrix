//! 能力依赖管理
//!
//! 管理能力之间的依赖关系和加载顺序

use std::collections::{HashMap, HashSet, VecDeque};

/// 依赖关系
#[derive(Debug, Clone)]
pub struct Dependency {
    /// 依赖的能力ID
    pub capability_id: String,
    /// 依赖类型
    pub dependency_type: DependencyType,
    /// 是否必需
    pub required: bool,
    /// 版本约束
    pub version_constraint: Option<String>,
}

/// 依赖类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    /// 直接依赖
    Direct,
    /// 可选依赖
    Optional,
    /// 冲突依赖
    Conflicts,
    /// 提供能力
    Provides,
}

/// 依赖图
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// 节点（能力ID）
    nodes: HashSet<String>,
    /// 边（依赖关系）
    edges: HashMap<String, Vec<Dependency>>,
    /// 反向边（被依赖）
    reverse_edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, capability_id: &str) {
        self.nodes.insert(capability_id.to_string());
        self.edges
            .entry(capability_id.to_string())
            .or_insert_with(Vec::new);
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, from: &str, dependency: Dependency) {
        self.add_node(from);
        self.add_node(&dependency.capability_id);

        self.edges
            .entry(from.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.clone());

        self.reverse_edges
            .entry(dependency.capability_id.to_string())
            .or_insert_with(Vec::new)
            .push(from.to_string());
    }

    /// 检查是否有循环依赖
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &self.nodes {
            if !visited.contains(node) {
                if self.dfs_cycle_check(node, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// DFS循环检测
    fn dfs_cycle_check(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(dependencies) = self.edges.get(node) {
            for dep in dependencies {
                if !visited.contains(&dep.capability_id) {
                    if self.dfs_cycle_check(&dep.capability_id, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&dep.capability_id) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// 拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        if self.has_cycle() {
            return Err("检测到循环依赖".into());
        }

        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();

        // 计算入度
        for node in &self.nodes {
            in_degree.entry(node.clone()).or_insert(0);
        }

        for dependencies in self.edges.values() {
            for dep in dependencies {
                *in_degree.entry(dep.capability_id.clone()).or_insert(0) += 1;
            }
        }

        // 找出入度为0的节点
        for (node, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node.clone());
            }
        }

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(dependencies) = self.edges.get(&node) {
                for dep in dependencies {
                    let degree = in_degree.get_mut(&dep.capability_id).expect("key exists");
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dep.capability_id.clone());
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err("拓扑排序失败".into());
        }

        Ok(result)
    }

    /// 获取依赖
    pub fn get_dependencies(&self, capability_id: &str) -> Vec<&Dependency> {
        self.edges
            .get(capability_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// 获取被依赖
    pub fn get_dependents(&self, capability_id: &str) -> Vec<&str> {
        self.reverse_edges
            .get(capability_id)
            .map(|deps| deps.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// 检查依赖是否满足
    pub fn check_dependencies(
        &self,
        capability_id: &str,
        loaded: &HashSet<String>,
    ) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();

        if let Some(dependencies) = self.edges.get(capability_id) {
            for dep in dependencies {
                if dep.required && !loaded.contains(&dep.capability_id) {
                    missing.push(dep.capability_id.clone());
                }
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// 获取加载顺序
    pub fn get_load_order(&self) -> Result<Vec<String>, String> {
        self.topological_sort()
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> &HashSet<String> {
        &self.nodes
    }
}

/// 依赖管理器
pub struct DependencyManager {
    /// 依赖图
    graph: DependencyGraph,
    /// 已加载的能力
    loaded: HashSet<String>,
    /// 加载历史
    load_history: Vec<LoadEvent>,
}

/// 加载事件
#[derive(Debug, Clone)]
pub struct LoadEvent {
    pub capability_id: String,
    pub event_type: LoadEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// 加载事件类型
#[derive(Debug, Clone)]
pub enum LoadEventType {
    Load,
    Unload,
    Reload,
}

impl DependencyManager {
    /// 创建新的依赖管理器
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            loaded: HashSet::new(),
            load_history: Vec::new(),
        }
    }

    /// 注册能力依赖
    pub fn register_dependencies(&mut self, capability_id: &str, dependencies: Vec<Dependency>) {
        self.graph.add_node(capability_id);
        for dep in dependencies {
            self.graph.add_dependency(capability_id, dep);
        }
    }

    /// 加载能力
    pub fn load(&mut self, capability_id: &str) -> Result<(), String> {
        // 检查依赖
        if let Err(missing) = self.graph.check_dependencies(capability_id, &self.loaded) {
            let event = LoadEvent {
                capability_id: capability_id.to_string(),
                event_type: LoadEventType::Load,
                timestamp: chrono::Utc::now(),
                success: false,
                error_message: Some(format!("缺少依赖: {:?}", missing)),
            };
            self.load_history.push(event);
            return Err(format!("缺少依赖: {:?}", missing));
        }

        // 模拟加载
        self.loaded.insert(capability_id.to_string());

        let event = LoadEvent {
            capability_id: capability_id.to_string(),
            event_type: LoadEventType::Load,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        };
        self.load_history.push(event);

        Ok(())
    }

    /// 卸载能力
    pub fn unload(&mut self, capability_id: &str) -> Result<(), String> {
        // 检查是否有其他能力依赖此能力
        let dependents = self.graph.get_dependents(capability_id);
        if !dependents.is_empty() {
            return Err(format!("无法卸载: 被 {:?} 依赖", dependents));
        }

        self.loaded.remove(capability_id);

        let event = LoadEvent {
            capability_id: capability_id.to_string(),
            event_type: LoadEventType::Unload,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        };
        self.load_history.push(event);

        Ok(())
    }

    /// 按顺序加载所有能力
    pub fn load_all(&mut self) -> Result<(), String> {
        let order = self.graph.get_load_order()?;

        for capability_id in &order {
            self.load(capability_id)?;
        }

        Ok(())
    }

    /// 获取依赖图
    pub fn graph(&self) -> &DependencyGraph {
        &self.graph
    }

    /// 获取已加载的能力
    pub fn get_loaded(&self) -> &HashSet<String> {
        &self.loaded
    }

    /// 获取加载历史
    pub fn get_load_history(&self) -> &[LoadEvent] {
        &self.load_history
    }

    /// 检查能力是否已加载
    pub fn is_loaded(&self, capability_id: &str) -> bool {
        self.loaded.contains(capability_id)
    }

    /// 获取统计信息
    pub fn stats(&self) -> DependencyStats {
        DependencyStats {
            total_capabilities: self.graph.get_all_nodes().len(),
            loaded_count: self.loaded.len(),
            load_events: self.load_history.len(),
            has_cycle: self.graph.has_cycle(),
        }
    }
}

/// 依赖统计
#[derive(Debug, Clone)]
pub struct DependencyStats {
    pub total_capabilities: usize,
    pub loaded_count: usize,
    pub load_events: usize,
    pub has_cycle: bool,
}

impl Default for DependencyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_node("A");
        graph.add_node("B");
        graph.add_dependency(
            "A",
            Dependency {
                capability_id: "B".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            },
        );

        assert!(!graph.has_cycle());
    }

    #[test]
    fn cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(
            "A",
            Dependency {
                capability_id: "B".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            },
        );
        graph.add_dependency(
            "B",
            Dependency {
                capability_id: "A".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            },
        );

        assert!(graph.has_cycle());
    }

    #[test]
    fn topological_sort() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(
            "A",
            Dependency {
                capability_id: "B".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            },
        );
        graph.add_dependency(
            "B",
            Dependency {
                capability_id: "C".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            },
        );

        let order = graph.topological_sort();
        assert!(order.is_ok());
        let order = order.unwrap();
        assert!(order.iter().position(|x| x == "C") < order.iter().position(|x| x == "B"));
        assert!(order.iter().position(|x| x == "B") < order.iter().position(|x| x == "A"));
    }

    #[test]
    fn dependency_manager() {
        let mut manager = DependencyManager::new();
        manager.register_dependencies(
            "A",
            vec![Dependency {
                capability_id: "B".into(),
                dependency_type: DependencyType::Direct,
                required: true,
                version_constraint: None,
            }],
        );

        let result = manager.load("A");
        assert!(result.is_err()); // B未加载

        manager.load("B").unwrap();
        let result = manager.load("A");
        assert!(result.is_ok());
    }
}
