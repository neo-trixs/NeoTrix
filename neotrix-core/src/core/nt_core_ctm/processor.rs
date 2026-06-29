use super::chunk::{Chunk, ExternalScores};
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

pub trait LtmProcessor: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, query: &[u8], time_step: usize, external: Option<&ExternalScores>) -> Chunk;
    fn read_memory(&self) -> &[Vec<u8>];
    fn write_memory(&mut self, chunk: &Chunk);
}

pub struct SpatialProcessor {
    name: String,
    memory: Vec<Vec<u8>>,
}

impl SpatialProcessor {
    pub fn new() -> Self {
        Self {
            name: "spatial_scene".to_string(),
            memory: Vec::new(),
        }
    }
}

impl LtmProcessor for SpatialProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, query: &[u8], time_step: usize, external: Option<&ExternalScores>) -> Chunk {
        let gist = if !self.memory.is_empty() {
            let refs: Vec<&[u8]> = self.memory.iter().map(|v| v.as_slice()).collect();
            QuantizedVSA::bundle(&refs)
        } else {
            query.to_vec()
        };
        let mut chunk = Chunk::new(&self.name, time_step, gist);
        chunk.relevance = 0.5;
        chunk.confidence = 0.6;
        chunk.surprise = 0.1;
        if let Some(ext) = external {
            chunk.apply_external(ext, &self.name);
        }
        chunk
    }

    fn read_memory(&self) -> &[Vec<u8>] {
        &self.memory
    }

    fn write_memory(&mut self, chunk: &Chunk) {
        self.memory.push(chunk.gist.clone());
        if self.memory.len() > 16 {
            self.memory.remove(0);
        }
    }
}

pub struct PhysicsProcessor {
    name: String,
    memory: Vec<Vec<u8>>,
}

impl PhysicsProcessor {
    pub fn new() -> Self {
        Self {
            name: "physics_common_sense".to_string(),
            memory: Vec::new(),
        }
    }
}

impl LtmProcessor for PhysicsProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, query: &[u8], time_step: usize, external: Option<&ExternalScores>) -> Chunk {
        let gist = if !self.memory.is_empty() {
            let refs: Vec<&[u8]> = self.memory.iter().map(|v| v.as_slice()).collect();
            QuantizedVSA::bundle(&refs)
        } else {
            query.to_vec()
        };
        let mut chunk = Chunk::new(&self.name, time_step, gist);
        chunk.relevance = 0.6;
        chunk.confidence = 0.7;
        chunk.surprise = 0.15;
        if let Some(ext) = external {
            chunk.apply_external(ext, &self.name);
        }
        chunk
    }

    fn read_memory(&self) -> &[Vec<u8>] {
        &self.memory
    }

    fn write_memory(&mut self, chunk: &Chunk) {
        self.memory.push(chunk.gist.clone());
        if self.memory.len() > 16 {
            self.memory.remove(0);
        }
    }
}

pub struct GoalProcessor {
    name: String,
    memory: Vec<Vec<u8>>,
}

impl GoalProcessor {
    pub fn new() -> Self {
        Self {
            name: "goal_directed".to_string(),
            memory: Vec::new(),
        }
    }
}

impl LtmProcessor for GoalProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, query: &[u8], time_step: usize, external: Option<&ExternalScores>) -> Chunk {
        let gist = if !self.memory.is_empty() {
            let refs: Vec<&[u8]> = self.memory.iter().map(|v| v.as_slice()).collect();
            QuantizedVSA::bundle(&refs)
        } else {
            query.to_vec()
        };
        let mut chunk = Chunk::new(&self.name, time_step, gist);
        chunk.relevance = 0.7;
        chunk.confidence = 0.5;
        chunk.surprise = 0.2;
        chunk.additional_questions = vec!["what is the next step?".to_string()];
        if let Some(ext) = external {
            chunk.apply_external(ext, &self.name);
        }
        chunk
    }

    fn read_memory(&self) -> &[Vec<u8>] {
        &self.memory
    }

    fn write_memory(&mut self, chunk: &Chunk) {
        self.memory.push(chunk.gist.clone());
        if self.memory.len() > 16 {
            self.memory.remove(0);
        }
    }
}

pub struct EpisodicProcessor {
    name: String,
    memory: Vec<Vec<u8>>,
}

impl EpisodicProcessor {
    pub fn new() -> Self {
        Self {
            name: "episodic_memory".to_string(),
            memory: Vec::new(),
        }
    }
}

impl LtmProcessor for EpisodicProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, query: &[u8], time_step: usize, external: Option<&ExternalScores>) -> Chunk {
        let gist = if !self.memory.is_empty() {
            let refs: Vec<&[u8]> = self.memory.iter().map(|v| v.as_slice()).collect();
            QuantizedVSA::bundle(&refs)
        } else {
            query.to_vec()
        };
        let mut chunk = Chunk::new(&self.name, time_step, gist);
        chunk.relevance = 0.4;
        chunk.confidence = 0.8;
        chunk.surprise = 0.05;
        if let Some(ext) = external {
            chunk.apply_external(ext, &self.name);
        }
        chunk
    }

    fn read_memory(&self) -> &[Vec<u8>] {
        &self.memory
    }

    fn write_memory(&mut self, chunk: &Chunk) {
        self.memory.push(chunk.gist.clone());
        if self.memory.len() > 16 {
            self.memory.remove(0);
        }
    }
}

pub fn default_processors() -> Vec<Box<dyn LtmProcessor>> {
    vec![
        Box::new(SpatialProcessor::new()),
        Box::new(PhysicsProcessor::new()),
        Box::new(GoalProcessor::new()),
        Box::new(EpisodicProcessor::new()),
    ]
}
