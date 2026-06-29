// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

/// Sensory modality types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensoryChannel {
    Vision,
    Audio,
    Text,
    System,
}

/// A grounded sensor reading
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub channel: SensoryChannel,
    pub data: Vec<f64>,
    pub timestamp: u64,
    pub confidence: f64,
}

/// Grounded sensory buffer — connects external input to VSA space
#[derive(Debug, Clone)]
pub struct SensorGrounding {
    pub buffer: Vec<SensorReading>,
    pub max_buffer: usize,
    pub active_channels: Vec<SensoryChannel>,
    pub tick: u64,
}

impl SensorGrounding {
    pub fn new() -> Self {
        SensorGrounding {
            buffer: Vec::with_capacity(64),
            max_buffer: 1000,
            active_channels: vec![SensoryChannel::Text, SensoryChannel::System],
            tick: 0,
        }
    }

    pub fn with_channels(channels: Vec<SensoryChannel>) -> Self {
        SensorGrounding {
            buffer: Vec::with_capacity(64),
            max_buffer: 1000,
            active_channels: channels,
            tick: 0,
        }
    }

    pub fn ingest(&mut self, channel: SensoryChannel, data: Vec<f64>, confidence: f64) {
        if !self.active_channels.contains(&channel) {
            return;
        }
        if self.buffer.len() >= self.max_buffer {
            self.buffer.remove(0);
        }
        self.buffer.push(SensorReading {
            channel,
            data,
            timestamp: self.tick,
            confidence,
        });
        self.tick += 1;
    }

    pub fn latest(&self, channel: SensoryChannel) -> Option<&SensorReading> {
        self.buffer.iter().rev().find(|r| r.channel == channel)
    }

    pub fn latest_n(&self, n: usize) -> Vec<&SensorReading> {
        let n = n.min(self.buffer.len());
        self.buffer.iter().skip(self.buffer.len() - n).collect()
    }

    pub fn channel_active(&self, channel: &SensoryChannel) -> bool {
        self.active_channels.contains(channel)
    }

    pub fn enable_channel(&mut self, channel: SensoryChannel) {
        if !self.active_channels.contains(&channel) {
            self.active_channels.push(channel);
        }
    }

    pub fn avg_confidence(&self) -> f64 {
        let n = self.buffer.len();
        if n == 0 {
            return 0.0;
        }
        self.buffer.iter().map(|r| r.confidence).sum::<f64>() / n as f64
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn summarize(&self) -> String {
        let vision = self
            .buffer
            .iter()
            .filter(|r| r.channel == SensoryChannel::Vision)
            .count();
        let audio = self
            .buffer
            .iter()
            .filter(|r| r.channel == SensoryChannel::Audio)
            .count();
        let text = self
            .buffer
            .iter()
            .filter(|r| r.channel == SensoryChannel::Text)
            .count();
        format!(
            "SensorGrounding: {} readings (v={}, a={}, t={}), avg_conf={:.2}, active={:?}",
            self.buffer.len(),
            vision,
            audio,
            text,
            self.avg_confidence(),
            self.active_channels,
        )
    }

    pub fn to_vsa(&self, dim: usize) -> Vec<u8> {
        if self.buffer.is_empty() {
            return vec![0; dim];
        }
        let mut vsa = vec![0u8; dim];
        for reading in self.buffer.iter().rev().take(10) {
            for (i, &v) in reading.data.iter().enumerate() {
                if i < dim {
                    vsa[i] = vsa[i].wrapping_add((v * 255.0) as u8);
                }
            }
        }
        vsa
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_reading(ch: SensoryChannel) -> SensorReading {
        SensorReading {
            channel: ch,
            data: vec![0.5; 16],
            timestamp: 0,
            confidence: 0.9,
        }
    }

    #[test]
    fn test_ingest_and_latest() {
        let mut sg = SensorGrounding::new();
        sg.ingest(SensoryChannel::Vision, vec![0.5; 16], 0.9);
        assert!(sg.latest(SensoryChannel::Vision).is_some());
        assert!(sg.latest(SensoryChannel::Audio).is_none());
    }

    #[test]
    fn test_channel_filtering() {
        let mut sg = SensorGrounding::new();
        sg.ingest(SensoryChannel::Vision, vec![0.5; 16], 0.9);
        assert!(sg.latest(SensoryChannel::Vision).is_none());
    }

    #[test]
    fn test_enable_channel() {
        let mut sg = SensorGrounding::new();
        sg.enable_channel(SensoryChannel::Vision);
        sg.ingest(SensoryChannel::Vision, vec![0.5; 16], 0.9);
        assert!(sg.latest(SensoryChannel::Vision).is_some());
    }

    #[test]
    fn test_summarize() {
        let mut sg = SensorGrounding::new();
        sg.enable_channel(SensoryChannel::Vision);
        sg.ingest(SensoryChannel::Vision, vec![0.5; 16], 0.9);
        sg.ingest(SensoryChannel::Text, vec![0.5; 16], 0.8);
        let s = sg.summarize();
        assert!(s.contains("SensorGrounding"));
    }

    #[test]
    fn test_to_vsa() {
        let mut sg = SensorGrounding::new();
        sg.enable_channel(SensoryChannel::Vision);
        sg.ingest(SensoryChannel::Vision, vec![0.5; 64], 0.9);
        let vsa = sg.to_vsa(64);
        assert_eq!(vsa.len(), 64);
    }

    #[test]
    fn test_avg_confidence() {
        let mut sg = SensorGrounding::new();
        sg.enable_channel(SensoryChannel::Vision);
        sg.ingest(SensoryChannel::Vision, vec![], 0.5);
        sg.ingest(SensoryChannel::Vision, vec![], 1.0);
        assert!((sg.avg_confidence() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_latest_n() {
        let mut sg = SensorGrounding::new();
        sg.enable_channel(SensoryChannel::Vision);
        for _ in 0..5 {
            sg.ingest(SensoryChannel::Vision, vec![], 0.5);
        }
        assert_eq!(sg.latest_n(3).len(), 3);
    }
}

/// Maps sensor readings to VSA vectors — deterministic projection from SensorReading → f64 vector
pub struct VsaGrounding {
    pub vsa_vectors: Vec<Vec<f64>>,
    pub channel_map: HashMap<SensoryChannel, Vec<usize>>,
    pub dim: usize,
    pub max_vectors: usize,
}

impl VsaGrounding {
    pub fn new(dim: usize) -> Self {
        VsaGrounding {
            vsa_vectors: Vec::new(),
            channel_map: HashMap::new(),
            dim,
            max_vectors: 1000,
        }
    }

    /// Generate a deterministic VSA vector from a SensorReading
    pub fn ingest_to_vsa(&mut self, reading: &SensorReading) -> Vec<f64> {
        let mut vec = Vec::with_capacity(self.dim);
        let seed = reading.timestamp.wrapping_mul(2654435761);
        for i in 0..self.dim {
            let mut v = 0.0f64;
            for (j, &d) in reading.data.iter().enumerate() {
                let h = seed.wrapping_add((i as u64) << 32 | j as u64);
                let phase = (h as f64).sin() * 0.5 + 0.5;
                v += d * phase;
            }
            vec.push(v / (reading.data.len().max(1) as f64));
        }
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
        let idx = self.vsa_vectors.len();
        if idx < self.max_vectors {
            self.vsa_vectors.push(vec.clone());
            self.channel_map
                .entry(reading.channel)
                .or_default()
                .push(idx);
        }
        vec
    }

    /// Get all vector references for a given channel
    pub fn channel_vectors(&self, channel: SensoryChannel) -> Vec<&[f64]> {
        self.channel_map
            .get(&channel)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&i| self.vsa_vectors.get(i))
                    .map(|v| v.as_slice())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Bundle all vectors from given channels via pointwise addition + normalize
    pub fn cross_channel_bind(&self, channels: &[SensoryChannel]) -> Vec<f64> {
        let mut sum = vec![0.0f64; self.dim];
        let mut count = 0usize;
        for ch in channels {
            for v in self.channel_vectors(*ch) {
                for (s, &vv) in sum.iter_mut().zip(v.iter()) {
                    *s += vv;
                }
                count += 1;
            }
        }
        if count > 0 {
            let norm: f64 = sum.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
                for s in sum.iter_mut() {
                    *s /= norm;
                }
            }
        }
        sum
    }

    /// Cosine similarity between two stored vectors by index
    pub fn similarity(&self, a: usize, b: usize) -> f64 {
        let va = match self.vsa_vectors.get(a) {
            Some(v) => v,
            None => return 0.0,
        };
        let vb = match self.vsa_vectors.get(b) {
            Some(v) => v,
            None => return 0.0,
        };
        let dot: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = va.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = vb.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na * nb < 1e-12 {
            return 0.0;
        }
        (dot / (na * nb)).clamp(-1.0, 1.0)
    }
}

/// Priority-based ingest queue — readings queued with priority, drained above threshold
pub struct PriorityIngest {
    pub priority_queues: HashMap<SensoryChannel, Vec<(f64, SensorReading)>>,
    pub max_per_channel: usize,
}

impl PriorityIngest {
    pub fn new(max_per_channel: usize) -> Self {
        PriorityIngest {
            priority_queues: HashMap::new(),
            max_per_channel,
        }
    }

    /// Enqueue a reading with a priority score
    pub fn enqueue(&mut self, reading: SensorReading, priority: f64) {
        let entry = self.priority_queues.entry(reading.channel).or_default();
        entry.push((priority, reading));
        entry.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        if entry.len() > self.max_per_channel {
            entry.truncate(self.max_per_channel);
        }
    }

    /// Drain all readings with priority >= min_priority
    pub fn drain_high_priority(&mut self, min_priority: f64) -> Vec<SensorReading> {
        let mut result = Vec::new();
        for (_ch, queue) in self.priority_queues.iter_mut() {
            let mut kept = Vec::new();
            for (pri, reading) in queue.drain(..) {
                if pri >= min_priority {
                    result.push(reading);
                } else {
                    kept.push((pri, reading));
                }
            }
            *queue = kept;
        }
        result
    }

    /// Number of queued readings for a channel
    pub fn channel_load(&self, channel: SensoryChannel) -> usize {
        self.priority_queues
            .get(&channel)
            .map(|q| q.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod grounding_tests {
    use super::*;

    fn sample_reading(ch: SensoryChannel, ts: u64) -> SensorReading {
        SensorReading {
            channel: ch,
            data: vec![0.3; 8],
            timestamp: ts,
            confidence: 0.85,
        }
    }

    #[test]
    fn test_vsa_grounding_new() {
        let vg = VsaGrounding::new(64);
        assert_eq!(vg.dim, 64);
        assert!(vg.vsa_vectors.is_empty());
        assert!(vg.channel_map.is_empty());
    }

    #[test]
    fn test_ingest_to_vsa_creates_vector() {
        let mut vg = VsaGrounding::new(16);
        let r = sample_reading(SensoryChannel::Vision, 1);
        let vec = vg.ingest_to_vsa(&r);
        assert_eq!(vec.len(), 16);
        assert_eq!(vg.vsa_vectors.len(), 1);
        // vector should be unit-norm
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_channel_vectors_filters() {
        let mut vg = VsaGrounding::new(8);
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Vision, 1));
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Audio, 2));
        assert_eq!(vg.channel_vectors(SensoryChannel::Vision).len(), 1);
        assert_eq!(vg.channel_vectors(SensoryChannel::Audio).len(), 1);
        assert!(vg.channel_vectors(SensoryChannel::Text).is_empty());
    }

    #[test]
    fn test_cross_channel_bind() {
        let mut vg = VsaGrounding::new(8);
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Vision, 1));
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Audio, 2));
        let bound = vg.cross_channel_bind(&[SensoryChannel::Vision, SensoryChannel::Audio]);
        assert_eq!(bound.len(), 8);
        let norm: f64 = bound.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_priority_enqueue_drain() {
        let mut pq = PriorityIngest::new(10);
        pq.enqueue(sample_reading(SensoryChannel::Vision, 1), 0.3);
        pq.enqueue(sample_reading(SensoryChannel::Vision, 2), 0.8);
        pq.enqueue(sample_reading(SensoryChannel::Vision, 3), 0.6);
        let drained = pq.drain_high_priority(0.7);
        assert_eq!(drained.len(), 1);
        assert_eq!(pq.channel_load(SensoryChannel::Vision), 2);
    }

    #[test]
    fn test_channel_load() {
        let mut pq = PriorityIngest::new(5);
        assert_eq!(pq.channel_load(SensoryChannel::Audio), 0);
        pq.enqueue(sample_reading(SensoryChannel::Audio, 1), 0.5);
        pq.enqueue(sample_reading(SensoryChannel::Audio, 2), 0.9);
        assert_eq!(pq.channel_load(SensoryChannel::Audio), 2);
    }

    #[test]
    fn test_vsa_similarity_identical() {
        let mut vg = VsaGrounding::new(16);
        let r = sample_reading(SensoryChannel::Vision, 1);
        vg.ingest_to_vsa(&r);
        let r2 = sample_reading(SensoryChannel::Vision, 1);
        vg.ingest_to_vsa(&r2);
        let sim = vg.similarity(0, 1);
        assert!((sim - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_channel_ingest() {
        let mut vg = VsaGrounding::new(8);
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Vision, 10));
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Audio, 20));
        vg.ingest_to_vsa(&sample_reading(SensoryChannel::Text, 30));
        assert_eq!(vg.vsa_vectors.len(), 3);
        assert_eq!(vg.channel_vectors(SensoryChannel::Vision).len(), 1);
        assert_eq!(vg.channel_vectors(SensoryChannel::Audio).len(), 1);
        assert_eq!(vg.channel_vectors(SensoryChannel::Text).len(), 1);
    }
}
