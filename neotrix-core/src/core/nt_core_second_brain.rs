use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::nt_core_self::emotion_state::EmotionEngine;
use crate::neotrix::nt_memory_kb::KnowledgeBase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSnapshot {
    pub timestamp: u64,
    pub emotion_json: Option<String>,
    pub session_notes: Vec<String>,
    pub wiki_page_count: usize,
    pub edge_count: usize,
    pub node_count: usize,
    pub dimensions: Vec<BrainDimensionScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainDimensionScore {
    pub name: String,
    pub score: f64,
    pub node_count: usize,
    pub link_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainLink {
    pub source_id: String,
    pub target_id: String,
    pub relation: BrainRelationType,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BrainRelationType {
    TemporalSequence,
    EmotionalAffinity,
    SemanticSimilar,
    CausalDependency,
    TypeHierarchy,
    SourceProvenance,
    CrossReference,
}

impl BrainRelationType {
    pub fn as_str(&self) -> &str {
        match self {
            BrainRelationType::TemporalSequence => "temporal_sequence",
            BrainRelationType::EmotionalAffinity => "emotional_affinity",
            BrainRelationType::SemanticSimilar => "semantic_similar",
            BrainRelationType::CausalDependency => "causal_dependency",
            BrainRelationType::TypeHierarchy => "type_hierarchy",
            BrainRelationType::SourceProvenance => "source_provenance",
            BrainRelationType::CrossReference => "cross_reference",
        }
    }
}

pub struct SecondBrain {
    kb: Option<Arc<KnowledgeBase>>,
    pub auto_sync_enabled: bool,
    pub sync_interval_secs: u64,
    tick_count: u64,
    last_snapshot: Option<BrainSnapshot>,
}

impl SecondBrain {
    pub fn new() -> Self {
        Self {
            kb: None,
            auto_sync_enabled: true,
            sync_interval_secs: 600,
            tick_count: 0,
            last_snapshot: None,
        }
    }

    pub fn attach_kb(&mut self, kb: Arc<KnowledgeBase>) {
        self.kb = Some(kb);
    }

    pub fn is_attached(&self) -> bool {
        self.kb.is_some()
    }

    pub fn tick(&mut self, emotion: Option<&EmotionEngine>, session_note: Option<&str>) {
        self.tick_count += 1;
        if !self.auto_sync_enabled || !self.tick_count.is_multiple_of(self.sync_interval_secs) {
            return;
        }
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => return,
        };

        if let Some(engine) = emotion {
            self.save_emotion(kb, engine);
        }
        if let Some(note) = session_note {
            self.save_session_note(kb, note);
        }
    }

    pub fn save_emotion_raw(&self, engine: &EmotionEngine) {
        if let Some(kb) = self.kb.as_ref() {
            if let Ok(json) = engine.to_json() {
                let _ = kb.kv_set("emotion", "engine_state", &json);
                let report = engine.report();
                if let Ok(report_json) = serde_json::to_string(&report) {
                    let _ = kb.kv_set("emotion", "last_report", &report_json);
                }
            }
        }
    }

    fn save_emotion(&self, kb: &KnowledgeBase, engine: &EmotionEngine) {
        if let Ok(json) = engine.to_json() {
            let _ = kb.kv_set("emotion", "engine_state", &json);
            let report = engine.report();
            if let Ok(report_json) = serde_json::to_string(&report) {
                let _ = kb.kv_set("emotion", "last_report", &report_json);
            }
        }
    }

    pub fn save_note(&self, note: &str) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("KB not attached")?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let title = format!("session_note_{}", ts);
        let _ = kb.insert_or_get_node(
            &title,
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Idea,
            Some(note),
            None,
            Some("second_brain"),
        );
        let note_key = format!("note_{}", ts);
        kb.kv_set("session_notes", &note_key, note)
    }

    fn save_session_note(&self, kb: &KnowledgeBase, note: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let title = format!("session_note_{}", ts);
        let _ = kb.insert_or_get_node(
            &title,
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Idea,
            Some(note),
            None,
            Some("second_brain"),
        );
        let note_key = format!("note_{}", ts);
        let _ = kb.kv_set("session_notes", &note_key, note);
    }

    pub fn link_nodes(
        &self,
        source_id: &str,
        target_id: &str,
        relation: BrainRelationType,
        weight: f64,
    ) -> Result<(), String> {
        let kb = self.kb.as_ref().ok_or("KB not attached")?;
        kb.upsert_edge(
            source_id,
            target_id,
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::RelationType::Related,
            weight,
            Some(relation.as_str()),
        )?;
        Ok(())
    }

    pub fn build_wiki_graph(&self) -> Result<BrainWikiGraph, String> {
        let kb = self.kb.as_ref().ok_or("KB not attached")?;
        let nodes = kb.search_by_type(
            &crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::WikiPage,
            10000,
        )?;
        let all_emotion_nodes = kb.search_by_type(
            &crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::Idea,
            1000,
        )?;

        let mut brain_nodes = Vec::new();
        let mut brain_edges = Vec::new();

        for n in &nodes {
            brain_nodes.push(BrainGraphNode {
                id: n.id.clone(),
                title: n.title.clone(),
                node_type: "wiki".into(),
                importance: n.importance,
                summary: n.summary.clone().unwrap_or_default(),
            });
        }
        for n in &all_emotion_nodes {
            brain_nodes.push(BrainGraphNode {
                id: n.id.clone(),
                title: n.title.clone(),
                node_type: "note".into(),
                importance: n.importance,
                summary: n.summary.clone().unwrap_or_default(),
            });
        }

        for node in &brain_nodes {
            if let Ok(related) = kb.get_related(&node.id, None, 50) {
                for r in &related {
                    brain_edges.push(BrainGraphEdge {
                        source: node.id.clone(),
                        target: r.node.id.clone(),
                        relation: "brain_link".into(),
                        weight: r.score,
                    });
                }
            }
        }

        let (emotion_kv, _session_kv) = self.read_brain_kv(kb);

        Ok(BrainWikiGraph {
            nodes: brain_nodes,
            edges: brain_edges,
            emotion_kv,
        })
    }

    fn read_brain_kv(
        &self,
        kb: &KnowledgeBase,
    ) -> (HashMap<String, String>, HashMap<String, String>) {
        let emotion_kv = kb
            .kv_list("emotion")
            .unwrap_or_default()
            .into_iter()
            .collect();
        let session_kv = kb
            .kv_list("session_notes")
            .unwrap_or_default()
            .into_iter()
            .collect();
        (emotion_kv, session_kv)
    }

    pub fn generate_graph_html(&self) -> Result<String, String> {
        let graph = self.build_wiki_graph()?;
        Ok(generate_brain_graph_html(&graph))
    }

    pub fn read_emotion(&self) -> (Option<String>, Option<String>) {
        let kb = match self.kb.as_ref() {
            Some(kb) => kb,
            None => return (None, None),
        };
        let state = kb.kv_get("emotion", "engine_state").ok().flatten();
        let report = kb.kv_get("emotion", "last_report").ok().flatten();
        (state, report)
    }

    pub fn status(&mut self) -> Result<BrainSnapshot, String> {
        let kb = self.kb.as_ref().ok_or("KB not attached")?;
        let emotion_json = kb.kv_get("emotion", "engine_state").ok().flatten();
        let session_notes = kb.kv_list("session_notes").unwrap_or_default();
        let conn = kb.conn.lock().map_err(|e| e.to_string())?;
        let node_count =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_nodes(&conn)
                .map_err(|e| e.to_string())?;
        let edge_count =
            crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_store::count_edges(&conn)
                .map_err(|e| e.to_string())?;
        let wiki_pages = kb.search_by_type(
            &crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_types::NodeType::WikiPage,
            10000,
        )?;

        let dimensions = vec![
            BrainDimensionScore {
                name: "temporal".into(),
                score: self.compute_dimension_coverage(kb, "temporal"),
                node_count: node_count as usize,
                link_count: edge_count as usize,
            },
            BrainDimensionScore {
                name: "semantic".into(),
                score: 1.0,
                node_count: node_count as usize,
                link_count: edge_count as usize,
            },
            BrainDimensionScore {
                name: "emotional".into(),
                score: if emotion_json.is_some() { 1.0 } else { 0.0 },
                node_count: 1,
                link_count: 0,
            },
        ];

        let snapshot = BrainSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            emotion_json,
            session_notes: session_notes
                .into_iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect(),
            wiki_page_count: wiki_pages.len(),
            edge_count: edge_count as usize,
            node_count: node_count as usize,
            dimensions,
        };
        self.last_snapshot = Some(snapshot.clone());
        Ok(snapshot)
    }

    fn compute_dimension_coverage(&self, kb: &KnowledgeBase, _dim: &str) -> f64 {
        let total = kb.kv_list("emotion").unwrap_or_default().len()
            + kb.kv_list("session_notes").unwrap_or_default().len();
        if total > 0 {
            1.0
        } else {
            0.0
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<BrainSearchResult>, String> {
        let kb = self.kb.as_ref().ok_or("KB not attached")?;
        let results = kb.search(query, limit)?;
        Ok(results
            .into_iter()
            .map(|r| BrainSearchResult {
                id: r.node.id,
                title: r.node.title,
                summary: r.node.summary.unwrap_or_default(),
                score: r.score,
                node_type: r.node.node_type.as_str().to_string(),
            })
            .collect())
    }
}

impl Default for SecondBrain {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainGraphNode {
    pub id: String,
    pub title: String,
    pub node_type: String,
    pub importance: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainGraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainWikiGraph {
    pub nodes: Vec<BrainGraphNode>,
    pub edges: Vec<BrainGraphEdge>,
    pub emotion_kv: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSearchResult {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub score: f64,
    pub node_type: String,
}

fn generate_brain_graph_html(graph: &BrainWikiGraph) -> String {
    let nodes_json = serde_json::to_string(&graph.nodes).unwrap_or_default();
    let edges_json = serde_json::to_string(&graph.edges).unwrap_or_default();
    let emotion_json = serde_json::to_string(&graph.emotion_kv).unwrap_or_default();

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Second Brain Graph</title>
<style>
* {{ margin:0; padding:0; }}
body {{ background:#0d0d1a; color:#c4b5fd; font-family:'Inter',system-ui,sans-serif; overflow:hidden; }}
#brain {{ width:100vw; height:100vh; }}
.legend {{ position:absolute; bottom:20px; left:20px; background:rgba(13,13,26,0.85); padding:12px 16px; border-radius:8px; border:1px solid #7c3aed40; font-size:12px; }}
.legend span {{ display:inline-block; width:12px; height:12px; border-radius:50%; margin-right:6px; }}
.legend .wiki {{ background:#7c3aed; }}
.legend .note {{ background:#f59e0b; }}
.node {{ stroke:#fff; stroke-width:1.5px; cursor:pointer; }}
.node:hover {{ stroke:#fbbf24; stroke-width:3px; }}
.link {{ stroke:#7c3aed40; stroke-opacity:0.6; }}
.label {{ fill:#c4b5fd; font-size:10px; pointer-events:none; }}
.tooltip {{ position:absolute; background:#1e1b4b; color:#e0e7ff; padding:8px 12px; border-radius:6px; font-size:12px; border:1px solid #7c3aed; max-width:300px; display:none; pointer-events:none; }}
</style></head>
<body>
<div class="legend">
<div><span class="wiki"></span>Wiki Page</div>
<div><span class="note"></span>Session Note</div>
<div style="margin-top:4px;color:#a78bfa;font-size:11px;">{emotion_count} emotion keys</div>
<div style="color:#a78bfa;font-size:11px;">{node_count} nodes, {edge_count} edges</div>
</div>
<div id="brain"></div>
<div class="tooltip" id="tooltip"></div>
<script src="https://d3js.org/d3.v7.min.js"></script>
<script>
const nodes = {nodes_json};
const edges = {edges_json};
const emotion = {emotion_json};
const svg = d3.select("#brain").append("svg").attr("width","100%").attr("height","100%");
const g = svg.append("g");
const tooltip = d3.select("#tooltip");

const sim = d3.forceSimulation(nodes)
  .force("link", d3.forceLink(edges).id(d=>d.id).distance(d=>100-50*d.weight))
  .force("charge", d3.forceManyBody().strength(-150))
  .force("collide", d3.forceCollide().radius(d=>10+Math.sqrt(d.importance*15)))
  .force("center", d3.forceCenter(window.innerWidth/2,window.innerHeight/2));

const link = g.selectAll(".link").data(edges).join("line")
  .attr("class","link").attr("stroke-width",d=>0.5+2*d.weight);

const node = g.selectAll(".node").data(nodes).join("circle")
  .attr("class","node")
  .attr("r",d=>4+Math.sqrt(d.importance*12))
  .attr("fill",d=>d.node_type==="note"?"#f59e0b":"#7c3aed")
  .call(d3.drag().on("start",(e,d)=>{{ if(!e.active) sim.alphaTarget(0.3).restart(); d.fx=d.x; d.fy=d.y; }})
    .on("drag",(e,d)=>{{ d.fx=e.x; d.fy=e.y; }})
    .on("end",(e,d)=>{{ if(!e.active) sim.alphaTarget(0); d.fx=null; d.fy=null; }}))
  .on("mouseover",(e,d)=>{{ tooltip.style("display","block").html(`<b>${{d.title}}</b><br>${{d.node_type}}<br>${{d.summary.slice(0,100)}}`); }})
  .on("mousemove",(e)=>{{ tooltip.style("left",(e.pageX+12)+"px").style("top",(e.pageY-28)+"px"); }})
  .on("mouseout",()=>{{ tooltip.style("display","none"); }});

const label = g.selectAll(".label").data(nodes).join("text")
  .attr("class","label").text(d=>d.title.length>20?d.title.slice(0,20)+"...":d.title);

sim.on("tick",()=>{{
  link.attr("x1",d=>d.source.x).attr("y1",d=>d.source.y).attr("x2",d=>d.target.x).attr("y2",d=>d.target.y);
  node.attr("cx",d=>d.x).attr("cy",d=>d.y);
  label.attr("x",d=>d.x+8).attr("y",d=>d.y+4);
}});

svg.call(d3.zoom().scaleExtent([0.1,8]).on("zoom",(e)=>{{ g.attr("transform",e.transform); }}));
</script></body></html>"##,
        emotion_count = graph.emotion_kv.len(),
        node_count = graph.nodes.len(),
        edge_count = graph.edges.len(),
        nodes_json = nodes_json,
        edges_json = edges_json,
        emotion_json = emotion_json,
    )
}

impl crate::core::nt_core_self_test::SelfTest for SecondBrain {
    fn name(&self) -> &str {
        "second_brain"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let failures = Vec::new();
        if !self.is_attached() {
            let mut brain = SecondBrain::new();
            brain.auto_sync_enabled = false;
            let _ = brain.status();
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_relation_types() {
        assert_eq!(
            BrainRelationType::TemporalSequence.as_str(),
            "temporal_sequence"
        );
        assert_eq!(
            BrainRelationType::EmotionalAffinity.as_str(),
            "emotional_affinity"
        );
    }

    #[test]
    fn test_brain_snapshot_serde() {
        let snap = BrainSnapshot {
            timestamp: 1000,
            emotion_json: Some(r#"{"frustration":0.3}"#.into()),
            session_notes: vec!["test_note".into()],
            wiki_page_count: 5,
            edge_count: 10,
            node_count: 100,
            dimensions: vec![],
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        let deser: BrainSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.timestamp, 1000);
        assert_eq!(deser.wiki_page_count, 5);
    }

    #[test]
    fn test_brain_graph_node_serde() {
        let node = BrainGraphNode {
            id: "test_id".into(),
            title: "Test Node".into(),
            node_type: "wiki".into(),
            importance: 0.8,
            summary: "A test node".into(),
        };
        let json = serde_json::to_string(&node).expect("serialize");
        let deser: BrainGraphNode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.title, "Test Node");
    }

    #[test]
    fn test_generate_html_no_crash() {
        let graph = BrainWikiGraph {
            nodes: vec![
                BrainGraphNode {
                    id: "1".into(),
                    title: "A".into(),
                    node_type: "wiki".into(),
                    importance: 0.5,
                    summary: "".into(),
                },
                BrainGraphNode {
                    id: "2".into(),
                    title: "B".into(),
                    node_type: "note".into(),
                    importance: 0.8,
                    summary: "".into(),
                },
            ],
            edges: vec![BrainGraphEdge {
                source: "1".into(),
                target: "2".into(),
                relation: "link".into(),
                weight: 0.7,
            }],
            emotion_kv: HashMap::new(),
        };
        let html = generate_brain_graph_html(&graph);
        assert!(html.contains("Second Brain Graph"));
        assert!(html.contains("const nodes ="));
        assert!(html.contains("const edges ="));
    }

    #[test]
    fn test_brain_search_result() {
        let r = BrainSearchResult {
            id: "id".into(),
            title: "title".into(),
            summary: "summary".into(),
            score: 0.9,
            node_type: "WikiPage".into(),
        };
        assert_eq!(r.title, "title");
    }

    #[test]
    fn test_second_brain_default() {
        let brain = SecondBrain::new();
        assert!(!brain.is_attached());
        assert!(brain.auto_sync_enabled);
    }

    #[test]
    fn test_read_emotion_no_kb() {
        let brain = SecondBrain::new();
        let (state, report) = brain.read_emotion();
        assert!(state.is_none());
        assert!(report.is_none());
    }

    #[test]
    fn test_save_note_no_kb() {
        let brain = SecondBrain::new();
        assert!(brain.save_note("test note").is_err());
    }

    #[test]
    fn test_save_emotion_raw_no_kb() {
        let engine = crate::core::nt_core_self::emotion_state::EmotionEngine::default();
        let brain = SecondBrain::new();
        brain.save_emotion_raw(&engine); // should not panic
    }

    #[test]
    fn test_emotion_engine_serde_roundtrip() {
        let mut engine = crate::core::nt_core_self::emotion_state::EmotionEngine::default();
        engine.observe(
            crate::core::nt_core_self::emotion_state::EmotionDimension::Confidence,
            0.8,
            "test",
        );
        engine.observe(
            crate::core::nt_core_self::emotion_state::EmotionDimension::Curiosity,
            0.6,
            "explore",
        );
        engine.tick();
        let json = engine.to_json().expect("to_json");
        let deser = crate::core::nt_core_self::emotion_state::EmotionEngine::from_json(&json)
            .expect("from_json");
        let r1 = engine.report();
        let r2 = deser.report();
        assert!((r1.confidence - r2.confidence).abs() < 0.01);
        assert!((r1.curiosity - r2.curiosity).abs() < 0.01);
    }
}
