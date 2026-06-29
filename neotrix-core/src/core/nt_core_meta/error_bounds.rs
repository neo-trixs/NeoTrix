#[derive(Debug, Clone)]
pub enum BoundSource {
    VSAQuantization { dims: usize, bits: u32 },
    TemporalDecay { steps: u64, decay_rate: f64 },
    ModelApproximation { model: String, error: f64 },
    MeasurementNoise { variance: f64 },
    Propagation { source_count: usize },
}

#[derive(Debug, Clone)]
pub struct ErrorBound {
    pub absolute_error: f64,
    pub relative_error: f64,
    pub confidence: f64,
    pub sources: Vec<BoundSource>,
}

impl ErrorBound {
    pub fn new(absolute: f64, relative: f64, confidence: f64) -> Self {
        Self {
            absolute_error: absolute,
            relative_error: relative.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            sources: Vec::new(),
        }
    }

    pub fn zero() -> Self {
        Self {
            absolute_error: 0.0,
            relative_error: 0.0,
            confidence: 1.0,
            sources: Vec::new(),
        }
    }

    pub fn compose(&self, other: &Self) -> Self {
        let mut sources = self.sources.clone();
        sources.push(BoundSource::Propagation {
            source_count: other.sources.len() + 1,
        });
        sources.extend(other.sources.clone());
        Self {
            absolute_error: self.absolute_error + other.absolute_error,
            relative_error: self.relative_error.max(other.relative_error),
            confidence: self.confidence.min(other.confidence),
            sources,
        }
    }

    pub fn scale(&self, factor: f64) -> Self {
        Self {
            absolute_error: self.absolute_error * factor,
            relative_error: self.relative_error,
            confidence: self.confidence,
            sources: self.sources.clone(),
        }
    }

    pub fn is_within(&self, tolerance: &Self) -> bool {
        self.absolute_error <= tolerance.absolute_error
            && self.relative_error <= tolerance.relative_error
            && self.confidence >= tolerance.confidence
    }
}

#[derive(Debug, Clone)]
pub struct VsaErrorModel {
    pub dimension: usize,
    pub quantization_bits: u32,
}

impl VsaErrorModel {
    pub fn similarity_variance(&self) -> f64 {
        1.0 / self.dimension as f64
    }

    pub fn bound_for_similarity(&self, _similarity: f64) -> ErrorBound {
        let var = self.similarity_variance();
        let std_dev = var.sqrt();
        let absolute = std_dev * 3.0;
        let bits = self.quantization_bits;
        let quant_error = 1.0 / ((1u64 << bits) as f64);
        ErrorBound {
            absolute_error: absolute + quant_error,
            relative_error: (absolute + quant_error).min(1.0),
            confidence: 0.997,
            sources: vec![BoundSource::VSAQuantization {
                dims: self.dimension,
                bits: self.quantization_bits,
            }],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PredictionErrorTracker {
    pub step_errors: Vec<ErrorBound>,
    pub horizon: usize,
}

impl PredictionErrorTracker {
    pub fn new(horizon: usize) -> Self {
        Self {
            step_errors: Vec::with_capacity(horizon),
            horizon,
        }
    }

    pub fn record_step(&mut self, bound: ErrorBound) {
        if self.step_errors.len() < self.horizon {
            self.step_errors.push(bound);
        }
    }

    pub fn accumulated_error(&self) -> ErrorBound {
        let mut acc = ErrorBound::zero();
        for e in &self.step_errors {
            acc = acc.compose(e);
        }
        acc
    }

    pub fn max_steps_within(&self, tolerance: &ErrorBound) -> usize {
        let mut acc = ErrorBound::zero();
        for (i, e) in self.step_errors.iter().enumerate() {
            acc = acc.compose(e);
            if !acc.is_within(tolerance) {
                return i;
            }
        }
        self.step_errors.len()
    }
}
