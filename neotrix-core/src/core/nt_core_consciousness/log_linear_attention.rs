#[derive(Debug, Clone)]
pub struct LogLinearConfig {
    pub base: f64,
    pub state_dims: usize,
    pub max_seq_len: usize,
}

impl Default for LogLinearConfig {
    fn default() -> Self {
        Self {
            base: 2.0,
            state_dims: 128,
            max_seq_len: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogLinearState {
    pub levels: Vec<Vec<f64>>,
    pub level_capacities: Vec<usize>,
    pub timestep: usize,
    pub config: LogLinearConfig,
}

impl LogLinearState {
    pub fn new(config: LogLinearConfig) -> Self {
        let num_levels = (config.max_seq_len as f64).log(config.base).ceil() as usize + 1;
        let capacities: Vec<usize> = (0..num_levels)
            .map(|i| (config.base.powi(i as i32)) as usize)
            .collect();
        Self {
            levels: vec![Vec::new(); num_levels],
            level_capacities: capacities,
            timestep: 0,
            config,
        }
    }

    pub fn update(&mut self, input: &[f64]) {
        self.timestep += 1;
        let mut carry = input.to_vec();
        for level in 0..self.levels.len() {
            self.levels[level].extend_from_slice(&carry);
            let cap = self.level_capacities[level] * self.config.state_dims;
            if self.levels[level].len() >= cap {
                let chunk: Vec<f64> = self.levels[level].drain(..cap).collect();
                carry = self.average_chunks(&chunk);
            } else {
                break;
            }
        }
    }

    fn average_chunks(&self, data: &[f64]) -> Vec<f64> {
        let dim = self.config.state_dims;
        if data.len() < dim {
            return data.to_vec();
        }
        let num = data.len() / dim;
        let mut result = vec![0.0; dim];
        for i in 0..num {
            for j in 0..dim {
                result[j] += data[i * dim + j];
            }
        }
        for j in 0..dim {
            result[j] /= num as f64;
        }
        result
    }

    pub fn attention(&self) -> Vec<f64> {
        let dim = self.config.state_dims;
        let mut result = vec![0.0; dim];
        let mut tw = 0.0;
        for (i, level) in self.levels.iter().enumerate() {
            let w = 1.0 / (i as f64 + 1.0);
            for chunk in level.chunks(dim) {
                if chunk.len() == dim {
                    for j in 0..dim {
                        result[j] += chunk[j] * w;
                    }
                    tw += w;
                }
            }
        }
        if tw > 0.0 {
            for j in 0..dim {
                result[j] /= tw;
            }
        }
        result
    }

    pub fn memory_usage(&self) -> usize {
        self.levels.iter().map(|l| l.len()).sum()
    }
}

#[derive(Debug, Clone)]
pub struct LogLinearAttention {
    pub state: LogLinearState,
}

impl LogLinearAttention {
    pub fn new(config: LogLinearConfig) -> Self {
        Self {
            state: LogLinearState::new(config),
        }
    }
    pub fn step(&mut self, input: &[f64]) {
        self.state.update(input);
    }
    pub fn query(&self) -> Vec<f64> {
        self.state.attention()
    }
}
