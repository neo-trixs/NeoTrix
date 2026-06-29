use crate::core::nt_core_knowledge::error::KnowledgeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn now_secs() -> i64 {
    crate::core::nt_core_time::unix_now_secs() as i64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub confidence: f64,
    pub source_text: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForceGraph {
    pub nodes: Vec<KgNode>,
    pub edges: Vec<KgEdge>,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentJob {
    pub id: String,
    pub source: String,
    pub content: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub node_count: usize,
    pub edge_count: usize,
}

pub struct ResearchKnowledgeGraph {
    pub jobs: Vec<DocumentJob>,
    pub max_jobs: usize,
    pub next_job_id: u64,
    pub builtin_entity_types: Vec<String>,
}

impl ResearchKnowledgeGraph {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            max_jobs: 100,
            next_job_id: 1,
            builtin_entity_types: vec![
                "person".into(),
                "organization".into(),
                "location".into(),
                "technology".into(),
                "concept".into(),
                "event".into(),
            ],
        }
    }

    pub fn submit_document(&mut self, source: &str, content: &str) -> String {
        let id = format!("kg_{}", self.next_job_id);
        self.next_job_id += 1;
        self.jobs.push(DocumentJob {
            id: id.clone(),
            source: source.to_string(),
            content: content.to_string(),
            status: "pending".into(),
            error: None,
            created_at: now_secs(),
            completed_at: None,
            node_count: 0,
            edge_count: 0,
        });
        if self.jobs.len() > self.max_jobs {
            self.jobs.remove(0);
        }
        id
    }

    pub fn extract_and_build(
        &mut self,
        job_id: &str,
        entity_extractor: &crate::core::nt_core_knowledge::entity_extractor::EntityExtractor,
    ) -> Result<ForceGraph, KnowledgeError> {
        let job_idx = self
            .jobs
            .iter()
            .position(|j| j.id == job_id)
            .ok_or_else(|| KnowledgeError::EntryNotFound(format!("job_not_found:{}", job_id)))?;
        let content = self.jobs[job_idx].content.clone();
        self.jobs[job_idx].status = "processing".into();

        let entities = entity_extractor.extract_entities(&content);
        let triples = entity_extractor.extract_relations(&content, &entities);

        let mut node_map: HashMap<String, KgNode> = HashMap::new();
        let mut edges: Vec<KgEdge> = Vec::new();

        for mention in &entities {
            let key = mention.name.clone();
            if !node_map.contains_key(&key) {
                node_map.insert(
                    key.clone(),
                    KgNode {
                        id: key.clone(),
                        label: mention.name.clone(),
                        node_type: mention.entity_type.name().to_string(),
                        confidence: mention.confidence,
                        source_text: mention.surface_form.clone(),
                        timestamp: now_secs(),
                    },
                );
            }
        }

        for triple in &triples {
            let subj_key = triple.subject.clone();
            let obj_key = triple.object.clone();
            if !node_map.contains_key(&subj_key) {
                node_map.insert(
                    subj_key.clone(),
                    KgNode {
                        id: subj_key.clone(),
                        label: subj_key.clone(),
                        node_type: "entity".into(),
                        confidence: triple.confidence,
                        source_text: String::new(),
                        timestamp: now_secs(),
                    },
                );
            }
            if !node_map.contains_key(&obj_key) {
                node_map.insert(
                    obj_key.clone(),
                    KgNode {
                        id: obj_key.clone(),
                        label: obj_key.clone(),
                        node_type: "entity".into(),
                        confidence: triple.confidence,
                        source_text: String::new(),
                        timestamp: now_secs(),
                    },
                );
            }
            edges.push(KgEdge {
                source: subj_key,
                target: obj_key,
                relation: triple.relation.name().to_string(),
                weight: triple.confidence,
                confidence: triple.confidence,
            });
        }

        let nodes: Vec<KgNode> = node_map.into_values().collect();
        let graph = ForceGraph {
            node_count: nodes.len(),
            edge_count: edges.len(),
            nodes,
            edges,
        };

        self.jobs[job_idx].status = "completed".into();
        self.jobs[job_idx].completed_at = Some(now_secs());
        self.jobs[job_idx].node_count = graph.nodes.len();
        self.jobs[job_idx].edge_count = graph.edges.len();

        Ok(graph)
    }

    pub fn export_force_graph_json(&self, graph: &ForceGraph) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(graph)
    }

    pub fn export_force_graph_html(&self, graph: &ForceGraph) -> String {
        let nodes_json = serde_json::to_string(&graph.nodes).unwrap_or_default();
        let edges_json = serde_json::to_string(&graph.edges).unwrap_or_default();
        let gray = "#666";
        let purple = "#a855f7";
        let light = "#ccc";
        let body_bg = "#0a0a1a";
        let mut html = String::with_capacity(4096);
        html.push_str("<!DOCTYPE html>\n<html><head><meta charset=\"utf-8\"><title>NeoTrix KG</title>\n<style>*{margin:0;padding:0}body{background:");
        html.push_str(body_bg);
        html.push_str("}svg{width:100vw;height:100vh}</style>\n</head><body>\n<svg id=\"kg\"></svg>\n<script src=\"https://d3js.org/d3.v7.min.js\"></script>\n<script>\nconst nodes = ");
        html.push_str(&nodes_json);
        html.push_str(";\nconst edges = ");
        html.push_str(&edges_json);
        html.push_str(";\nconst w = window.innerWidth, h = window.innerHeight;\nconst sim = d3.forceSimulation(nodes).force(\"link\", d3.forceLink(edges).id(d=>d.id).distance(100)).force(\"charge\", d3.forceManyBody().strength(-200)).force(\"center\", d3.forceCenter(w/2, h/2));\nconst svg = d3.select(\"#kg\").attr(\"width\",w).attr(\"height\",h);\nconst link = svg.append(\"g\").selectAll(\"line\").data(edges).join(\"line\").attr(\"stroke\",\"");
        html.push_str(gray);
        html.push_str("\").attr(\"stroke-width\",d=>Math.max(1,d.weight*3));\nconst node = svg.append(\"g\").selectAll(\"circle\").data(nodes).join(\"circle\").attr(\"r\",8).attr(\"fill\",\"");
        html.push_str(purple);
        html.push_str("\").call(d3.drag().on(\"start\",(e,d)=>{if(!e.active)sim.alphaTarget(0.3).restart();d.fx=d.x;d.fy=d.y;}).on(\"drag\",(e,d)=>{d.fx=e.x;d.fy=e.y;}).on(\"end\",(e,d)=>{if(!e.active)sim.alphaTarget(0);d.fx=null;d.fy=null;}));\nconst label = svg.append(\"g\").selectAll(\"text\").data(nodes).join(\"text\").text(d=>d.label).attr(\"fill\",\"");
        html.push_str(light);
        html.push_str("\").attr(\"font-size\",\"12px\").attr(\"dx\",12).attr(\"dy\",4);\nsim.on(\"tick\",()=>{link.attr(\"x1\",d=>d.source.x).attr(\"y1\",d=>d.source.y).attr(\"x2\",d=>d.target.x).attr(\"y2\",d=>d.target.y);node.attr(\"cx\",d=>d.x).attr(\"cy\",d=>d.y);label.attr(\"x\",d=>d.x).attr(\"y\",d=>d.y);});\n</script></body></html>");
        html
    }

    pub fn stats(&self) -> String {
        let total_jobs = self.jobs.len();
        let completed = self.jobs.iter().filter(|j| j.status == "completed").count();
        let total_nodes: usize = self.jobs.iter().map(|j| j.node_count).sum();
        let total_edges: usize = self.jobs.iter().map(|j| j.edge_count).sum();
        format!(
            "kg:{}_jobs|{}_completed|{}_nodes|{}_edges",
            total_jobs, completed, total_nodes, total_edges
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::entity_extractor::EntityExtractor;

    #[test]
    fn test_submit_document() {
        let mut kg = ResearchKnowledgeGraph::new();
        let id = kg.submit_document("test", "Einstein works at Princeton");
        assert!(id.starts_with("kg_"));
        assert_eq!(kg.jobs.len(), 1);
        assert_eq!(kg.jobs[0].status, "pending");
    }

    #[test]
    fn test_extract_and_build() {
        let mut kg = ResearchKnowledgeGraph::new();
        let extractor = EntityExtractor::new();
        let id = kg.submit_document(
            "test",
            "Einstein works at Princeton. Newton created calculus.",
        );
        let result = kg.extract_and_build(&id, &extractor);
        assert!(result.is_ok(), "extract failed: {:?}", result.err());
        let graph = result.unwrap();
        assert!(
            graph.nodes.len() >= 2,
            "expected >=2 nodes, got {}",
            graph.nodes.len()
        );
        assert!(
            graph.edges.len() >= 1,
            "expected >=1 edges, got {}",
            graph.edges.len()
        );
    }

    #[test]
    fn test_export_html_contains_script() {
        let graph = ForceGraph {
            nodes: vec![KgNode {
                id: "test".into(),
                label: "Test".into(),
                node_type: "concept".into(),
                confidence: 1.0,
                source_text: String::new(),
                timestamp: now_secs(),
            }],
            edges: Vec::new(),
            node_count: 1,
            edge_count: 0,
        };
        let kg = ResearchKnowledgeGraph::new();
        let html = kg.export_force_graph_html(&graph);
        assert!(html.contains("d3.v7.min.js"));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_stats() {
        let kg = ResearchKnowledgeGraph::new();
        let s = kg.stats();
        assert!(s.contains("kg:"));
        assert!(s.contains("jobs"));
    }
}
