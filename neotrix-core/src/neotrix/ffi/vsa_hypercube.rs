// VSA HyperCube Implementation
// Vector Symbolic Architecture — bind/bundle/permute with high-dimensional vectors

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::collections::HashMap;
use std::time::Instant;

struct VSAHyperCubeInner {
    dimensions: u32,
    sparsity: f32,
    store: HashMap<String, HyperVector>,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct VSAHyperCubeImpl {
    inner: Arc<RwLock<VSAHyperCubeInner>>,
}

#[uniffi::export]
impl VSAHyperCubeImpl {
    #[uniffi::constructor]
    pub fn init(dimensions: u32, sparsity: f32) -> Result<Self, NeoTrixError> {
        if dimensions == 0 || dimensions % 8 != 0 {
            return Err(NeoTrixError::InvalidInput);
        }
        Ok(Self {
            inner: Arc::new(RwLock::new(VSAHyperCubeInner {
                dimensions,
                sparsity,
                store: HashMap::new(),
            })),
        })
    }

    pub fn random_vector(&self, label: &str) -> HyperVector {
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        let bytes = (inner.dimensions / 8) as usize;
        let mut data = vec![0u8; bytes];
        let mut seed: u64 = 0xcbf29ce484222325;
        for b in label.as_bytes() {
            seed ^= *b as u64;
            seed = seed.wrapping_mul(0x100000001b3);
        }
        for i in 0..bytes {
            seed = seed.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(0xbf58476d1ce4e5b9);
            data[i] = (seed >> 56) as u8;
        }
        HyperVector {
            dimensions: inner.dimensions,
            data,
            sparsity: inner.sparsity,
        }
    }

    pub fn bind(&self, v1: &HyperVector, v2: &HyperVector) -> Result<HyperVector, NeoTrixError> {
        if v1.dimensions != v2.dimensions {
            return Err(NeoTrixError::InvalidInput);
        }
        let data = v1.data.iter().zip(v2.data.iter()).map(|(a, b)| a ^ b).collect();
        Ok(HyperVector {
            dimensions: v1.dimensions,
            data,
            sparsity: self.inner.read().expect("ffi rwlock poisoned").sparsity,
        })
    }

    pub fn bundle(&self, vectors: &[HyperVector]) -> Result<HyperVector, NeoTrixError> {
        if vectors.is_empty() {
            return Err(NeoTrixError::InvalidInput);
        }
        let inner = self.inner.read().expect("ffi rwlock poisoned");
        let dim = vectors[0].dimensions;
        let bytes = (dim / 8) as usize;
        if vectors.iter().any(|v| v.dimensions != dim) {
            return Err(NeoTrixError::InvalidInput);
        }
        let mut data = vec![0u8; bytes];
        for v in vectors {
            for (i, b) in v.data.iter().enumerate() {
                data[i] ^= b;
            }
        }
        Ok(HyperVector { dimensions: dim, data, sparsity: inner.sparsity })
    }

    pub fn permute(&self, vector: &HyperVector, shifts: u32) -> HyperVector {
        let bytes = (vector.dimensions / 8) as usize;
        let shift = (shifts as usize) % bytes;
        let mut data = vec![0u8; bytes];
        for i in 0..bytes {
            data[(i + shift) % bytes] = vector.data[i];
        }
        HyperVector { dimensions: vector.dimensions, data, sparsity: self.inner.read().expect("ffi rwlock poisoned").sparsity }
    }

    pub fn similarity(&self, v1: &HyperVector, v2: &HyperVector) -> Result<f32, NeoTrixError> {
        if v1.dimensions != v2.dimensions {
            return Err(NeoTrixError::InvalidInput);
        }
        let bytes = (v1.dimensions / 8) as usize;
        let mut matched = 0u32;
        let mut total = 0u32;
        for i in 0..bytes {
            for bit in 0..8 {
                let a = (v1.data[i] >> bit) & 1;
                let b = (v2.data[i] >> bit) & 1;
                total += 1;
                if a == b {
                    matched += 1;
                }
            }
        }
        Ok(matched as f32 / total as f32)
    }

    pub fn cleanup(&self, vector: &HyperVector, known: &HashMap<String, HyperVector>) -> Result<String, NeoTrixError> {
        let mut best: Option<(String, f32)> = None;
        for (label, v) in known {
            let sim = self.similarity(vector, v)?;
            if best.as_ref().map_or(true, |(_, s)| sim > *s) {
                best = Some((label.clone(), sim));
            }
        }
        best.map(|(l, _)| l).ok_or(NeoTrixError::NotFound)
    }

    pub fn store(&self, label: &str, vector: HyperVector) -> bool {
        self.inner.write().expect("ffi rwlock poisoned").store.insert(label.to_string(), vector);
        true
    }

    pub fn retrieve(&self, label: &str) -> Result<HyperVector, NeoTrixError> {
        self.inner.read().expect("ffi rwlock poisoned").store.get(label).cloned().ok_or(NeoTrixError::NotFound)
    }

    pub fn batch_operation(&self, ops: &[VSAOperation]) -> Result<Vec<VSAResult>, NeoTrixError> {
        let mut results = Vec::with_capacity(ops.len());
        for op in ops {
            let start = Instant::now();
            let (result, scores) = match op.op_type.as_str() {
                "bind" if op.vectors.len() == 2 => {
                    let r = self.bind(&op.vectors[0], &op.vectors[1])?;
                    (r, vec![self.similarity(&op.vectors[0], &op.vectors[1])?])
                }
                "bundle" => {
                    let r = self.bundle(&op.vectors)?;
                    let scores = op.vectors.iter().map(|v| self.similarity(&r, v).unwrap_or(0.0)).collect();
                    (r, scores)
                }
                "permute" if op.vectors.len() == 1 => {
                    let shifts = op.parameters.get("shifts").and_then(|s| s.parse().ok()).unwrap_or(1);
                    (self.permute(&op.vectors[0], shifts), Vec::new())
                }
                "similarity" if op.vectors.len() == 2 => {
                    let s = self.similarity(&op.vectors[0], &op.vectors[1])?;
                    (op.vectors[0].clone(), vec![s])
                }
                _ => return Err(NeoTrixError::InvalidInput),
            };
            results.push(VSAResult {
                result_vector: result,
                similarity_scores: scores,
                operation_time_ms: start.elapsed().as_millis() as u64,
            });
        }
        Ok(results)
    }
}