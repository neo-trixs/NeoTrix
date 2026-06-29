#[derive(Debug, Clone)]
pub struct SpikeEvent {
    pub neuron_id: usize,
    pub timestamp: u64,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct SpikeProcessor {
    pub events: Vec<SpikeEvent>,
    pub threshold: f64,
}

impl SpikeProcessor {
    pub fn new(threshold: f64) -> Self {
        Self {
            events: vec![],
            threshold,
        }
    }
    pub fn fire(&mut self, neuron_id: usize, weight: f64) {
        if weight >= self.threshold {
            self.events.push(SpikeEvent {
                neuron_id,
                timestamp: self.events.len() as u64,
                weight,
            });
        }
    }
    pub fn encode(&self, dims: usize) -> Vec<f64> {
        let mut v = vec![0.0; dims];
        for e in &self.events {
            let idx = e.neuron_id % dims;
            v[idx] += e.weight;
        }
        v
    }
}
