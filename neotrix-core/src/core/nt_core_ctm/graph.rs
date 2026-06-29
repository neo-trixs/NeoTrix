use super::chunk::{Chunk, ExternalScores};
use super::processor::LtmProcessor;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::HashMap;

pub struct ProcessorGraph {
    adjacency: HashMap<String, Vec<String>>,
}

impl ProcessorGraph {
    pub fn new(processors: &[Box<dyn LtmProcessor>]) -> Self {
        let mut adjacency = HashMap::new();
        for p in processors {
            adjacency.insert(p.name().to_string(), Vec::new());
        }
        Self { adjacency }
    }

    pub fn add_link(&mut self, p1: &str, p2: &str) {
        self.adjacency
            .entry(p1.to_string())
            .or_default()
            .push(p2.to_string());
        self.adjacency
            .entry(p2.to_string())
            .or_default()
            .push(p1.to_string());
    }

    pub fn has_link(&self, p1: &str, p2: &str) -> bool {
        self.adjacency
            .get(p1)
            .map(|neighbors| neighbors.contains(&p2.to_string()))
            .unwrap_or(false)
    }

    pub fn get_neighbors(&self, name: &str) -> Vec<String> {
        self.adjacency.get(name).cloned().unwrap_or_default()
    }

    pub fn link_count(&self) -> usize {
        self.adjacency.values().map(|v| v.len()).sum::<usize>() / 2
    }

    pub fn link_from_broadcast(&mut self, winner: &Chunk, all_chunks: &[Chunk], threshold: f64) {
        for chunk in all_chunks {
            if chunk.processor_name == winner.processor_name {
                continue;
            }
            let sim = QuantizedVSA::similarity(&winner.gist, &chunk.gist);
            if sim >= threshold {
                self.add_link(&winner.processor_name, &chunk.processor_name);
            }
        }
    }

    pub fn ask_all_parallel(
        &self,
        processors: &[Box<dyn LtmProcessor>],
        query: &[u8],
        time_step: usize,
        fuse_only: bool,
        external: Option<&ExternalScores>,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        for proc in processors {
            let do_execute = if fuse_only {
                let neighbors = self.get_neighbors(proc.name());
                !neighbors.is_empty()
            } else {
                true
            };
            if do_execute {
                chunks.push(proc.execute(query, time_step, external));
            }
        }
        chunks
    }
}
