#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QualiaEncoding {
    Direct,
    Compressed { ratio: f64 },
    Sparse { active_fraction: f64 },
}

#[derive(Debug, Clone)]
pub struct QualiaChunk {
    pub encoding: QualiaEncoding,
    pub data: Vec<f64>,
    pub original_dims: usize,
    pub compression_ratio: f64,
}

impl QualiaChunk {
    pub fn compress(vsa_vector: &[f64], target_dims: usize) -> Self {
        let od = vsa_vector.len();
        let ratio = od as f64 / target_dims as f64;
        let cs = (od as f64 / target_dims as f64).ceil() as usize;
        let mut data = Vec::with_capacity(target_dims);
        for i in 0..target_dims {
            let start = i * cs;
            let end = (start + cs).min(od);
            let sum: f64 = vsa_vector[start..end].iter().sum();
            data.push(sum / (end - start) as f64);
        }
        Self {
            encoding: QualiaEncoding::Compressed { ratio },
            data,
            original_dims: od,
            compression_ratio: ratio,
        }
    }

    pub fn decompress(&self) -> Vec<f64> {
        let scale = (self.original_dims as f64 / self.data.len() as f64).ceil() as usize;
        let mut r = Vec::with_capacity(self.original_dims);
        for val in &self.data {
            for _ in 0..scale {
                r.push(*val);
            }
        }
        r.truncate(self.original_dims);
        r
    }

    pub fn fidelity(&self) -> f64 {
        1.0 - (1.0 / self.compression_ratio).min(0.5)
    }
}

#[derive(Debug, Clone)]
pub struct QualiaLayer {
    pub current: Option<QualiaChunk>,
    pub history: Vec<QualiaChunk>,
    pub max_history: usize,
}

impl QualiaLayer {
    pub fn new() -> Self {
        Self {
            current: None,
            history: Vec::new(),
            max_history: 10,
        }
    }

    pub fn encode(&mut self, vsa: &[f64], target_dims: usize, _encoding: QualiaEncoding) {
        let chunk = QualiaChunk::compress(vsa, target_dims);
        self.history.push(chunk.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        self.current = Some(chunk);
    }

    pub fn decode(&self) -> Option<Vec<f64>> {
        self.current.as_ref().map(|c| c.decompress())
    }
    pub fn latest_fidelity(&self) -> f64 {
        self.current.as_ref().map_or(0.0, |c| c.fidelity())
    }
}

impl Default for QualiaLayer {
    fn default() -> Self {
        Self::new()
    }
}
