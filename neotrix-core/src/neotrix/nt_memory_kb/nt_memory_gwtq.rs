use crate::core::nt_core_gwt::module_def::SpecialistType;
use crate::core::nt_core_hex::ReasoningHexagram;

use super::nt_memory_search as search;
use super::nt_memory_store as store;
use super::nt_memory_types::{NodeType, SearchResult};


/// 将 E8 推理状态映射到 KnowledgeBase 查询关键词
fn e8_state_to_search_query(state: ReasoningHexagram) -> Vec<&'static str> {
    let mut tags = Vec::new();
    if state.abstraction() == 0 { tags.push("concrete reasoning"); } else { tags.push("abstract reasoning"); }
    if state.scope() == 0 { tags.push("focused scope"); } else { tags.push("broad scope"); }
    if state.method() == 0 { tags.push("analytical method"); } else { tags.push("generative method"); }
    if state.depth() == 0 { tags.push("deep analysis"); } else { tags.push("fast reasoning"); }
    if state.reasoning_mode() == 0 { tags.push("solo reasoning"); } else { tags.push("collaborative reasoning"); }
    if state.stance() == 0 { tags.push("certain stance"); } else { tags.push("exploratory stance"); }
    tags
}

/// 将 SpecialistType 映射到 KnowledgeBase 查询关键词
fn specialist_to_search_query(st: &SpecialistType) -> Vec<&'static str> {
    match st {
        SpecialistType::PatternMatcher => vec!["pattern recognition", "pattern matching", "signal processing"],
        SpecialistType::AnomalyDetector => vec!["anomaly detection", "outlier detection", "novelty detection"],
        SpecialistType::KnowledgeRetriever => vec!["knowledge retrieval", "information retrieval", "semantic search"],
        SpecialistType::CodeAnalyzer => vec!["code analysis", "static analysis", "program analysis"],
        SpecialistType::Planner => vec!["planning", "task planning", "automated planning"],
        SpecialistType::KnowledgeIntegrator => vec!["knowledge integration", "information fusion", "knowledge graph"],
        SpecialistType::GoalPrioritizer => vec!["goal prioritization", "goal management", "multi-objective optimization"],
        SpecialistType::RiskAssessor => vec!["risk assessment", "risk analysis", "safety analysis"],
        SpecialistType::CreativityGenerator => vec!["creativity", "creative generation", "divergent thinking"],
        SpecialistType::ReflectionEngine => vec!["self reflection", "introspection", "meta cognition"],
        SpecialistType::MetaCognitionAnalyst => vec!["meta cognition", "self awareness", "cognitive monitoring"],
        SpecialistType::AISecurity => vec!["ai nt_shield", "prompt injection", "vulnerability assessment", "llm jailbreak"],
        SpecialistType::ImageGenerator => vec!["image generation", "text to image", "diffusion model", "visual creativity"],
    }
}

/// KnowledgeBase-backed consciousness query interface
impl super::KnowledgeBase {
    /// 根据 E8 推理状态查询相关知识
    pub fn query_by_e8_state(&self, state: ReasoningHexagram, limit: usize) -> Result<Vec<SearchResult>, String> {
        let queries = e8_state_to_search_query(state);
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut results = Vec::new();
        for q in queries {
            let r = search::hybrid_search(&conn, q, limit)
                .map_err(|e| format!("Search error: {}", e))?;
            results.extend(r);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// 根据 GWT 专家模块类型查询相关知识
    pub fn query_by_specialist(&self, st: &SpecialistType, limit: usize) -> Result<Vec<SearchResult>, String> {
        let queries = specialist_to_search_query(st);
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut results = Vec::new();
        for q in queries {
            let r = search::hybrid_search(&conn, q, limit)
                .map_err(|e| format!("Search error: {}", e))?;
            results.extend(r);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }

    /// 记录意识快照到 KnowledgeBase
    pub fn record_consciousness_snapshot(&self, phi: f64, coherence: f64, is_conscious: bool, level: &str, details: &str) -> Result<String, String> {
        let title = format!("Consciousness Snapshot φ={:.3} coh={:.3}", phi, coherence);
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let summary = format!("phi={}, coherence={}, is_conscious={}, level={}, details={}", phi, coherence, is_conscious, level, details);
        let node_id = store::insert_or_get_node(&conn, &title, NodeType::Insight, Some(&summary), None, Some("consciousness"))
            .map_err(|e| format!("Insert error: {}", e))?;
        Ok(node_id)
    }

    /// 根据 E8 模式名查询知识推荐（混合搜索：FTS5 + embedding rerank）
    pub fn recommend_for_e8_mode(&self, mode_name: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        self.hybrid_rerank_search(mode_name, limit)
    }

    /// 查询 GWT 广播内容在知识图谱中的关联
    pub fn query_broadcast_context(&self, content: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        let results = search::hybrid_search(&conn, content, limit)
            .map_err(|e| format!("Search error: {}", e))?;
        Ok(results)
    }
}
