// KBBridge Implementation
// Knowledge Base bridge — semantic search, experience store, absorption

use uniffi;
use std::sync::{Arc, RwLock};
use crate::neotrix::ffi::types::*;
use std::collections::HashMap;

struct KBBridgeInner {
    nodes: HashMap<String, KBResult>,
    embeddings: Vec<HyperVector>,
}

#[derive(Clone)]
#[derive(uniffi::Object)]
pub struct KBBridgeImpl {
    inner: Arc<RwLock<KBBridgeInner>>,
}

#[uniffi::export]
impl KBBridgeImpl {
    #[uniffi::constructor]
    pub fn init(_config: NeoTrixConfig) -> Result<Self, NeoTrixError> {
        Ok(Self {
            inner: Arc::new(RwLock::new(KBBridgeInner {
                nodes: HashMap::new(),
                embeddings: Vec::new(),
            })),
        })
    }

    pub fn query(&self, q: KBQuery) -> Vec<KBResult> {
        let inner = self.inner.read().unwrap();
        let lower = q.query.to_lowercase();
        let mut results: Vec<KBResult> = inner
            .nodes
            .values()
            .filter(|r| r.namespace == q.namespace)
            .filter(|r| r.content.to_lowercase().contains(&lower) || r.metadata.values().any(|v| v.to_lowercase().contains(&lower)))
            .cloned()
            .collect();
        for r in results.iter_mut() {
            let hits = r.content.to_lowercase().matches(&lower).count();
            r.score = (hits as f32 * 0.5).min(1.0).max(0.1);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(q.limit as usize);
        results
    }

    pub fn store_experience(&self, content: &str, namespace: &str, metadata: HashMap<String, String>) -> Result<String, NeoTrixError> {
        if content.is_empty() {
            return Err(NeoTrixError::InvalidInput);
        }
        let mut inner = self.inner.write().unwrap();
        let id = format!("{}-{}", namespace, inner.nodes.len() + 1);
        let embedding = HyperVector {
            dimensions: 1024,
            data: hash_to_bytes(&content),
            sparsity: 0.1,
        };
        inner.nodes.insert(id.clone(), KBResult {
            id: id.clone(),
            namespace: namespace.into(),
            content: content.into(),
            embedding,
            metadata,
            score: 1.0,
        });
        let emb = inner.nodes.get(&id).unwrap().embedding.clone();
        inner.embeddings.push(emb);
        Ok(id)
    }

    pub fn get_stats(&self) -> KBStats {
        let inner = self.inner.read().unwrap();
        let mut namespaces = HashMap::new();
        for node in inner.nodes.values() {
            *namespaces.entry(node.namespace.clone()).or_insert(0u64) += 1;
        }
        KBStats {
            total_nodes: inner.nodes.len() as u64,
            total_edges: 0,
            namespaces,
            storage_mb: (inner.nodes.len() as f32 * 0.5) / 1024.0,
            index_status: "indexed".into(),
        }
    }

    pub fn semantic_search(&self, query: &str, namespace: &str, limit: u32) -> Vec<KBResult> {
        let inner = self.inner.read().unwrap();
        let q_vec = hash_to_bytes(query);
        let mut scored: Vec<(f32, KBResult)> = inner
            .nodes
            .values()
            .filter(|r| r.namespace == namespace)
            .map(|r| {
                let sim = cosine_similarity(&q_vec, &r.embedding.data);
                (sim, r.clone())
            })
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit as usize);
        scored.into_iter().map(|(s, mut r)| {
            r.score = s;
            r
        }).collect()
    }

    pub fn get_related(&self, concept_id: &str, limit: u32) -> Vec<KBResult> {
        let inner = self.inner.read().unwrap();
        let base = match inner.nodes.get(concept_id) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let mut scored: Vec<(f32, KBResult)> = inner
            .nodes
            .values()
            .filter(|r| r.id != concept_id)
            .map(|r| (cosine_similarity(&base.embedding.data, &r.embedding.data), r.clone()))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit as usize);
        scored.into_iter().map(|(s, mut r)| {
            r.score = s;
            r
        }).collect()
    }

    pub fn run_absorption(&self) -> AbsorptionProgress {
        let inner = self.inner.read().unwrap();
        AbsorptionProgress {
            pending: 0,
            in_progress: 0,
            completed: inner.nodes.len() as u32,
            failed: 0,
            current_item: "absorption-complete".into(),
        }
    }
}

fn hash_to_bytes(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
        out.push((h >> 56) as u8);
    }
    while out.len() < 128 {
        h = h.wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(0xbf58476d1ce4e5b9);
        out.push((h >> 56) as u8);
    }
    out
}

fn cosine_similarity(a: &[u8], b: &[u8]) -> f32 {
    let len = a.len().min(b.len());
    let mut dot = 0u64;
    let mut na = 0u64;
    let mut nb = 0u64;
    for i in 0..len {
        dot += (a[i] as u64) * (b[i] as u64);
        na += (a[i] as u64) * (a[i] as u64);
        nb += (b[i] as u64) * (b[i] as u64);
    }
    if na == 0 || nb == 0 {
        0.0
    } else {
        (dot as f32) / ((na as f32).sqrt() * (nb as f32).sqrt())
    }
}