use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use crate::neotrix::nt_mind::code_graph::CodeGraph;
use crate::neotrix::nt_mind::graph_types::{
    EdgeKind, GraphNode, ImpactResult, ImpactHop,
    CodeGraphStats, EnrichedSearchResult,
};

impl CodeGraph {
    /// 简单的贪心社区检测（基于边密度）
    pub(crate) fn detect_communities(&mut self) {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let mut comm: HashMap<String, usize> = node_ids.iter().map(|n| (n.clone(), 0)).collect();
        let mut next_id = 1;

        for node_id in &node_ids {
            if comm[node_id] != 0 { continue; }
            comm.insert(node_id.clone(), next_id);
            let mut queue: VecDeque<String> = VecDeque::new();
            queue.push_back(node_id.clone());
            while let Some(current) = queue.pop_front() {
                let neighbors: Vec<String> = self.edges.iter()
                    .filter(|e| e.from == current || e.to == current)
                    .map(|e| if e.from == current { e.to.clone() } else { e.from.clone() })
                    .filter(|n| comm.get(n).copied().unwrap_or(0) == 0)
                    .collect();
                for n in neighbors {
                    comm.insert(n.clone(), next_id);
                    queue.push_back(n);
                }
            }
            next_id += 1;
        }
        self.communities = comm;
    }

    /// 影响分析：找到所有直接和间接依赖/被依赖的节点
    pub fn impact_analysis(&self, target: &str, max_depth: usize) -> ImpactResult {
        let target_id = self.resolve_id(target);
        let upstream = self.traverse(&target_id, true, max_depth);
        let downstream = self.traverse(&target_id, false, max_depth);
        ImpactResult { target: target_id, upstream, downstream }
    }

    fn traverse(&self, start: &str, reverse: bool, max_depth: usize) -> Vec<ImpactHop> {
        let mut visited = HashSet::new();
        let mut results = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((start.to_string(), 0, vec![start.to_string()]));

        while let Some((current, depth, path)) = queue.pop_front() {
            if depth >= max_depth { continue; }
            if !visited.insert(current.clone()) { continue; }

            let neighbors: Vec<String> = self.edges.iter()
                .filter(|e| {
                    if reverse { e.to == current }
                    else { e.from == current }
                })
                .map(|e| if reverse { e.from.clone() } else { e.to.clone() })
                .collect();

            for n in &neighbors {
                if let Some(node) = self.nodes.get(n) {
                    let (confidence, _no) = self.edge_confidence(n, &current, reverse);
                    let mut hop_path = path.clone();
                    hop_path.push(n.clone());
                    results.push(ImpactHop {
                        node_id: n.clone(),
                        node_name: node.name.clone(),
                        node_kind: node.kind.clone(),
                        depth: depth + 1,
                        confidence,
                        path: hop_path.clone(),
                    });
                    queue.push_back((n.clone(), depth + 1, hop_path));
                }
            }
        }
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn edge_confidence(&self, from: &str, to: &str, reverse: bool) -> (f64, usize) {
        let confs: Vec<f64> = self.edges.iter()
            .filter(|e| {
                if reverse { e.from == *from && e.to == *to }
                else { e.from == *to && e.to == *from }
            })
            .map(|e| e.confidence)
            .collect();
        let max_conf = confs.iter().cloned().fold(0.0_f64, f64::max);
        (max_conf, confs.len())
    }

    /// 按名称或 ID 解析节点
    pub fn resolve_id(&self, target: &str) -> String {
        if self.nodes.contains_key(target) {
            return target.to_string();
        }
        for (id, node) in &self.nodes {
            if node.name == target || id.ends_with(&format!("::{}", target)) {
                return id.clone();
            }
        }
        target.to_string()
    }

    /// 获取文件的所有直接依赖
    pub fn file_dependencies(&self, file_path: &Path) -> Vec<String> {
        let id = self.file_nodes.get(file_path).cloned().unwrap_or_default();
        self.edges.iter()
            .filter(|e| e.from == id && e.kind == EdgeKind::Imports)
            .map(|e| e.to.clone())
            .collect()
    }

    /// 获取文件的所有直接被依赖者
    pub fn file_dependents(&self, file_path: &Path) -> Vec<String> {
        let id = self.file_nodes.get(file_path).cloned().unwrap_or_default();
        self.edges.iter()
            .filter(|e| e.to == id && e.kind == EdgeKind::Imports)
            .map(|e| e.from.clone())
            .collect()
    }

    pub fn stats(&self) -> CodeGraphStats {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for node in self.nodes.values() {
            *type_counts.entry(node.kind.as_str().to_string()).or_insert(0) += 1;
        }
        let mut kind_counts: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *kind_counts.entry(edge.kind.as_str().to_string()).or_insert(0) += 1;
        }
        let unique_communities: HashSet<&usize> = self.communities.values().collect();
        CodeGraphStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            type_counts,
            kind_counts,
            community_count: unique_communities.len(),
        }
    }

    pub fn search_enriched(&self, query: &str, max_results: usize) -> Vec<EnrichedSearchResult> {
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(i32, &GraphNode)> = self.nodes.values()
            .filter(|n| n.name.to_lowercase().contains(&query_lower))
            .map(|n| {
                let mut score: i32 = if n.name.eq_ignore_ascii_case(&query_lower) { 10 }
                    else if n.name.to_lowercase().starts_with(&query_lower) { 7 }
                    else { 4 };
                if n.kind.as_str() == query_lower { score += 3; }
                if let Some(ref fp) = n.file_path {
                    if fp.to_string_lossy().to_lowercase().contains(&query_lower) { score += 1; }
                }
                (score, n)
            })
            .collect();
        scored.sort_by_key(|b| std::cmp::Reverse(b.0));
        scored.truncate(max_results);

        scored.into_iter().map(|(_, node)| {
            let outgoing: Vec<String> = self.edges.iter()
                .filter(|e| e.from == node.id && e.to != node.id)
                .map(|e| e.to.clone())
                .collect();
            let incoming: Vec<String> = self.edges.iter()
                .filter(|e| e.to == node.id && e.from != node.id)
                .map(|e| e.from.clone())
                .collect();
            EnrichedSearchResult {
                node: node.clone(),
                outgoing_edges: outgoing,
                incoming_edges: incoming,
                community_id: self.communities.get(&node.id).copied(),
            }
        }).collect()
    }

    pub fn get_enriched_context(&self, node_id: &str) -> Option<EnrichedSearchResult> {
        self.nodes.get(node_id).map(|node| {
            let outgoing: Vec<String> = self.edges.iter()
                .filter(|e| e.from == node.id && e.to != node.id)
                .map(|e| e.to.clone())
                .collect();
            let incoming: Vec<String> = self.edges.iter()
                .filter(|e| e.to == node.id && e.from != node.id)
                .map(|e| e.from.clone())
                .collect();
            EnrichedSearchResult {
                node: node.clone(),
                outgoing_edges: outgoing,
                incoming_edges: incoming,
                community_id: self.communities.get(&node.id).copied(),
            }
        })
    }
}
