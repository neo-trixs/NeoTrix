#[derive(Debug, Clone)]
pub struct LayerState {
    pub name: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub corrections: Vec<Correction>,
}

#[derive(Debug, Clone)]
pub struct Correction {
    pub from_layer: String,
    pub to_layer: String,
    pub adjustment: f64,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct TlcmResult {
    pub layers: Vec<LayerState>,
    pub overall_confidence: f64,
    pub corrections_applied: usize,
}

pub struct Tlcm {
    layers: Vec<LayerState>,
    correction_threshold: f64,
}

impl Tlcm {
    pub fn new(correction_threshold: f64) -> Self {
        Self {
            layers: Vec::new(),
            correction_threshold,
        }
    }

    pub fn add_layer(&mut self, name: &str, confidence: f64) {
        self.layers.push(LayerState {
            name: name.to_string(),
            confidence,
            evidence: Vec::new(),
            corrections: Vec::new(),
        });
    }

    pub fn add_evidence(&mut self, layer_name: &str, evidence: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|l| l.name == layer_name) {
            layer.evidence.push(evidence.to_string());
        }
    }

    pub fn apply_corrections(&mut self) -> TlcmResult {
        let mut corrections = 0;
        let layer_names: Vec<String> = self.layers.iter().map(|l| l.name.clone()).collect();
        for i in 0..self.layers.len() {
            for j in 0..self.layers.len() {
                if i == j {
                    continue;
                }
                let diff = (self.layers[i].confidence - self.layers[j].confidence).abs();
                if diff > self.correction_threshold {
                    let adjustment = (diff - self.correction_threshold) * 0.1;
                    let from = layer_names[j].clone();
                    let to = layer_names[i].clone();
                    self.layers[i].confidence = (self.layers[i].confidence + adjustment).min(1.0);
                    self.layers[i].corrections.push(Correction {
                        from_layer: from.clone(),
                        to_layer: to.clone(),
                        adjustment,
                        reason: format!("Layer {} corrected by {}", to, from),
                    });
                    corrections += 1;
                }
            }
        }
        let overall = if self.layers.is_empty() {
            0.0
        } else {
            self.layers.iter().map(|l| l.confidence).sum::<f64>() / self.layers.len() as f64
        };
        TlcmResult {
            layers: self.layers.clone(),
            overall_confidence: overall,
            corrections_applied: corrections,
        }
    }

    pub fn layers(&self) -> &[LayerState] {
        &self.layers
    }

    pub fn confidence(&self) -> f64 {
        if self.layers.is_empty() {
            0.0
        } else {
            self.layers.iter().map(|l| l.confidence).sum::<f64>() / self.layers.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_correction() {
        let mut t = Tlcm::new(0.1);
        t.add_layer("perception", 0.9);
        t.add_layer("reasoning", 0.3);
        let r = t.apply_corrections();
        assert!(r.corrections_applied > 0);
        assert!(r.overall_confidence > 0.3);
    }

    #[test]
    fn test_no_correction_needed() {
        let mut t = Tlcm::new(0.5);
        t.add_layer("a", 0.6);
        t.add_layer("b", 0.55);
        let r = t.apply_corrections();
        assert_eq!(r.corrections_applied, 0);
    }
}
