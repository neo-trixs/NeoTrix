use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use super::axis::DimensionAxis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperCoord {
    values: HashMap<DimensionAxis, f64>,
}

impl Default for HyperCoord {
    fn default() -> Self {
        Self::new()
    }
}

impl HyperCoord {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn with(axis: DimensionAxis, value: f64) -> Self {
        let mut values = HashMap::new();
        values.insert(axis, value);
        Self { values }
    }

    pub fn set(&mut self, axis: DimensionAxis, value: f64) {
        self.values.insert(axis, value);
    }

    pub fn get(&self, axis: &DimensionAxis) -> f64 {
        self.values.get(axis).copied().unwrap_or(0.0)
    }

    pub fn dims(&self) -> impl Iterator<Item = (&DimensionAxis, &f64)> {
        self.values.iter()
    }
}
