pub mod entity_graph;
pub mod timeline;
pub mod backlinks;

#[derive(Debug, Default)]
pub struct EntityGraph {
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
}

impl EntityGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, id: String) {
        if !self.nodes.contains(&id) {
            self.nodes.push(id);
        }
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges.push((from, to));
    }
}

#[derive(Debug, Default)]
pub struct EventTimeline {
    events: Vec<(u64, String)>,
}

impl EventTimeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, ts: u64, desc: String) {
        self.events.push((ts, desc));
    }
}

#[derive(Debug, Default)]
pub struct BacklinkStore {
    links: std::collections::HashMap<String, Vec<String>>,
}

impl BacklinkStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, target: String, source: String) {
        self.links.entry(target).or_default().push(source);
    }
}
