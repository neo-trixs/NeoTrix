// G400 + G406: Terminal cognitive flow visualization — txflow-style ASCII animation
use crate::core::nt_core_hcube::VsaVector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThoughtNodeType {
    Input,
    Hypothesis,
    Mixing,
    Decision,
    Evidence,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtNode {
    pub id: u64,
    pub node_type: ThoughtNodeType,
    pub label: String,
    pub confidence: f64,
    pub vsa: Option<VsaVector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtEdge {
    pub from: u64,
    pub to: u64,
    pub weight: f64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtFlow {
    pub nodes: Vec<ThoughtNode>,
    pub edges: Vec<ThoughtEdge>,
    pub title: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtFlowViz {
    pub flows: Vec<ThoughtFlow>,
    pub max_flows: usize,
    pub current_input: Vec<ThoughtNode>,
    pub current_hypotheses: Vec<ThoughtNode>,
    pub current_output: Option<ThoughtNode>,
}

impl ThoughtFlowViz {
    pub fn new() -> Self {
        Self {
            flows: Vec::new(),
            max_flows: 100,
            current_input: Vec::new(),
            current_hypotheses: Vec::new(),
            current_output: None,
        }
    }

    pub fn add_input(&mut self, label: &str, confidence: f64) -> u64 {
        let id = self.current_input.len() as u64 + 1;
        self.current_input.push(ThoughtNode {
            id,
            node_type: ThoughtNodeType::Input,
            label: label.to_string(),
            confidence,
            vsa: None,
        });
        id
    }

    pub fn add_hypothesis(&mut self, label: &str, confidence: f64) -> u64 {
        let id = 1000 + self.current_hypotheses.len() as u64;
        self.current_hypotheses.push(ThoughtNode {
            id,
            node_type: ThoughtNodeType::Hypothesis,
            label: label.to_string(),
            confidence,
            vsa: None,
        });
        id
    }

    pub fn set_output(&mut self, label: &str, confidence: f64) -> u64 {
        let id = 9999;
        self.current_output = Some(ThoughtNode {
            id,
            node_type: ThoughtNodeType::Output,
            label: label.to_string(),
            confidence,
            vsa: None,
        });
        id
    }

    pub fn snapshot(&mut self, title: &str) {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        nodes.extend(self.current_input.clone());
        nodes.extend(self.current_hypotheses.clone());
        if let Some(ref out) = self.current_output {
            nodes.push(out.clone());
            for h in &self.current_hypotheses {
                edges.push(ThoughtEdge {
                    from: h.id,
                    to: out.id,
                    weight: h.confidence,
                    label: format!("{:.2}", h.confidence),
                });
            }
        }
        let flow = ThoughtFlow {
            nodes,
            edges,
            title: title.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        if self.flows.len() >= self.max_flows {
            self.flows.remove(0);
        }
        self.flows.push(flow);
        self.current_input.clear();
        self.current_hypotheses.clear();
        self.current_output = None;
    }

    pub fn render_ascii(&self, flow_index: usize) -> String {
        if flow_index >= self.flows.len() {
            return "No flow at index".to_string();
        }
        let flow = &self.flows[flow_index];
        let mut out = String::new();
        out.push_str(&format!("\n══ {} ══\n\n", flow.title));

        // Render inputs on left
        out.push_str(" INPUTS          HYPOTHESES        OUTPUT\n");
        out.push_str(" ──────          ──────────        ──────\n");

        let max_nodes = flow.nodes.len().max(1);

        for i in 0..max_nodes {
            let inputs: Vec<&ThoughtNode> = flow
                .nodes
                .iter()
                .filter(|n| matches!(n.node_type, ThoughtNodeType::Input))
                .collect();
            let hyps: Vec<&ThoughtNode> = flow
                .nodes
                .iter()
                .filter(|n| matches!(n.node_type, ThoughtNodeType::Hypothesis))
                .collect();
            let outputs: Vec<&ThoughtNode> = flow
                .nodes
                .iter()
                .filter(|n| matches!(n.node_type, ThoughtNodeType::Output))
                .collect();

            let in_label = if i < inputs.len() {
                format!(" {} [{:.2}]", inputs[i].label, inputs[i].confidence)
            } else {
                String::new()
            };

            let hyp_label = if i < hyps.len() {
                format!(" {} [{:.2}]", hyps[i].label, hyps[i].confidence)
            } else {
                String::new()
            };

            let out_label = if i < outputs.len() {
                format!(" {} [{:.2}]", outputs[i].label, outputs[i].confidence)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "{:<20}──▶ {:<20}──▶ {}\n",
                in_label, hyp_label, out_label
            ));
        }

        // Render edges
        if !flow.edges.is_empty() {
            out.push_str("\n EVIDENCE FLOW\n");
            out.push_str(" ─────────────\n");
            for edge in &flow.edges {
                out.push_str(&format!(
                    "   hypothesis {} ──({})──▶ output {}\n",
                    edge.from, edge.label, edge.to
                ));
            }
        }

        out
    }

    pub fn render_all_ascii(&self) -> Vec<String> {
        (0..self.flows.len())
            .map(|i| self.render_ascii(i))
            .collect()
    }
}
