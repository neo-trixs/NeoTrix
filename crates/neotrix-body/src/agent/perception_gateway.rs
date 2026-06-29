use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PerceptionGateway {
    channels: Vec<PerceptionChannel>,
    attention_buffer: Vec<PerceptionEvent>,
    broadcast_history: VecDeque<PerceptionEvent>,
    vsa_context: [u64; 4],
    max_buffer: usize,
}

#[derive(Debug, Clone)]
pub struct PerceptionChannel {
    pub name: String,
    pub priority: u8,
    pub vsa_signature: [u64; 4],
    pub last_event_ms: u64,
    pub event_count: u64,
}

#[derive(Debug, Clone)]
pub struct PerceptionEvent {
    pub id: String,
    pub channel: String,
    pub content: serde_json::Value,
    pub salience: f64,
    pub vsa_encoding: [u64; 4],
    pub timestamp_ms: u64,
    pub requires_attention: bool,
}

#[derive(Debug, Clone)]
pub struct BroadcastContent {
    pub winner_event: PerceptionEvent,
    pub context_vsa: [u64; 4],
    pub competing_events: Vec<PerceptionEvent>,
}

#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub event_count: u64,
    pub last_event_ms: u64,
    pub avg_salience: f64,
    pub total_broadcasts: u64,
}

impl PerceptionGateway {
    pub fn new(max_buffer: usize) -> Self {
        Self {
            channels: Vec::new(),
            attention_buffer: Vec::with_capacity(max_buffer),
            broadcast_history: VecDeque::with_capacity(100),
            vsa_context: [0; 4],
            max_buffer,
        }
    }

    pub fn register_channel(&mut self, name: &str, priority: u8) {
        if !self.channels.iter().any(|c| c.name == name) {
            let sig = Self::compute_channel_vsa(name, priority);
            self.channels.push(PerceptionChannel {
                name: name.into(),
                priority,
                vsa_signature: sig,
                last_event_ms: 0,
                event_count: 0,
            });
        }
    }

    pub fn unregister_channel(&mut self, name: &str) {
        self.channels.retain(|c| c.name != name);
    }

    pub fn push_event(&mut self, channel: &str, content: serde_json::Value, salience: f64) {
        if let Some(ch) = self.channels.iter_mut().find(|c| c.name == channel) {
            ch.last_event_ms = Self::now_ms();
            ch.event_count += 1;
        }
        let event = PerceptionEvent {
            id: format!("pe_{}_{}", channel, Self::now_ms()),
            channel: channel.into(),
            content,
            salience,
            vsa_encoding: Self::vsa_encode_event(channel, salience),
            timestamp_ms: Self::now_ms(),
            requires_attention: salience > 0.5,
        };
        self.attention_buffer.push(event);
        if self.attention_buffer.len() > self.max_buffer {
            self.attention_buffer.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));
            self.attention_buffer.truncate(self.max_buffer);
        }
    }

    pub fn compute_salience(&self, event: &PerceptionEvent) -> f64 {
        let priority_weight = self.channels.iter()
            .find(|c| c.name == event.channel)
            .map(|c| c.priority as f64 / 255.0)
            .unwrap_or(0.5);
        let novelty: f64 = if event.salience < 0.01 { 1.0 } else { 0.5 };
        let recency = if Self::now_ms() - event.timestamp_ms < 5000 { 1.0 } else { 0.3 };
        priority_weight * 0.4 + novelty * 0.3 + recency * 0.3
    }

    pub fn attend(&mut self) -> Option<BroadcastContent> {
        if self.attention_buffer.is_empty() {
            return None;
        }
        let saliences: Vec<f64> = self
            .attention_buffer
            .iter()
            .map(|event| self.compute_salience(event))
            .collect();
        for (event, sal) in self.attention_buffer.iter_mut().zip(saliences) {
            event.salience = sal;
        }
        self.attention_buffer.sort_by(|a, b| b.salience.partial_cmp(&a.salience).unwrap_or(std::cmp::Ordering::Equal));
        let winner = self.attention_buffer.remove(0);
        let competing: Vec<PerceptionEvent> = self.attention_buffer.drain(..3.min(self.attention_buffer.len())).collect();
        let context_vsa = Self::xor_vsa(&winner.vsa_encoding, &self.vsa_context);
        Some(BroadcastContent {
            winner_event: winner,
            context_vsa,
            competing_events: competing,
        })
    }

    pub fn broadcast(&mut self, content: BroadcastContent) {
        self.vsa_context = content.context_vsa;
        self.broadcast_history.push_back(content.winner_event.clone());
        if self.broadcast_history.len() > 100 {
            self.broadcast_history.pop_front();
        }
    }

    pub fn recent_broadcasts(&self, count: usize) -> Vec<&PerceptionEvent> {
        self.broadcast_history.iter().rev().take(count).collect()
    }

    pub fn channel_stats(&self) -> HashMap<String, ChannelStats> {
        let mut stats = HashMap::new();
        for ch in &self.channels {
            let events: Vec<&PerceptionEvent> = self.broadcast_history.iter().filter(|e| e.channel == ch.name).collect();
            let avg_sal = if events.is_empty() { 0.0 } else { events.iter().map(|e| e.salience).sum::<f64>() / events.len() as f64 };
            stats.insert(ch.name.clone(), ChannelStats {
                event_count: ch.event_count,
                last_event_ms: ch.last_event_ms,
                avg_salience: avg_sal,
                total_broadcasts: events.len() as u64,
            });
        }
        stats
    }

    pub fn vsa_encode_content(content: &serde_json::Value) -> [u64; 4] {
        let s = serde_json::to_string(content).unwrap_or_default();
        Self::hash_bytes(s.as_bytes())
    }

    fn vsa_encode_event(channel: &str, salience: f64) -> [u64; 4] {
        let sal_bytes = salience.to_le_bytes();
        let combined: Vec<u8> = channel.bytes().chain(sal_bytes.iter().copied()).collect();
        Self::hash_bytes(&combined)
    }

    fn compute_channel_vsa(name: &str, priority: u8) -> [u64; 4] {
        let h1: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        let h2: u64 = name.bytes().rev().fold(0u64, |acc, b| acc.wrapping_mul(37).wrapping_add(b as u64));
        [h1, h2, h1.wrapping_add(priority as u64), h2.wrapping_mul(priority as u64)]
    }

    fn hash_bytes(bytes: &[u8]) -> [u64; 4] {
        let h1 = bytes.iter().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(31).wrapping_add(*b as u64 ^ (i as u64 * 7)));
        let h2 = bytes.iter().rev().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(37).wrapping_add(*b as u64 ^ (i as u64 * 13)));
        let h3 = bytes.iter().step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(41).wrapping_add(*b as u64));
        let h4 = bytes.iter().skip(1).step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(43).wrapping_add(*b as u64));
        [h1 ^ h3, h2 ^ h4, h1.wrapping_add(h2), h3.wrapping_add(h4)]
    }

    fn xor_vsa(a: &[u64; 4], b: &[u64; 4]) -> [u64; 4] {
        [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]]
    }

    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(channel: &str, salience: f64) -> PerceptionEvent {
        PerceptionEvent {
            id: format!("test_{}", channel),
            channel: channel.into(),
            content: serde_json::json!({"key": "value"}),
            salience,
            vsa_encoding: [0; 4],
            timestamp_ms: PerceptionGateway::now_ms(),
            requires_attention: salience > 0.5,
        }
    }

    #[test]
    fn test_register_channel() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("tls_fingerprint", 200);
        assert_eq!(gw.channels.len(), 1);
    }

    #[test]
    fn test_no_duplicate_channels() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("test", 100);
        gw.register_channel("test", 200);
        assert_eq!(gw.channels.len(), 1);
    }

    #[test]
    fn test_unregister_channel() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("test", 100);
        gw.unregister_channel("test");
        assert_eq!(gw.channels.len(), 0);
    }

    #[test]
    fn test_push_event_updates_channel() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("browser", 150);
        gw.push_event("browser", serde_json::json!({"url": "https://x.com"}), 0.8);
        let stats = gw.channel_stats();
        assert_eq!(stats.get("browser").unwrap().event_count, 1);
    }

    #[test]
    fn test_attend_returns_winner() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("high", 200);
        gw.register_channel("low", 50);
        gw.push_event("low", serde_json::json!({"x": 1}), 0.1);
        gw.push_event("high", serde_json::json!({"y": 2}), 0.9);
        let result = gw.attend();
        assert!(result.is_some());
        assert_eq!(result.unwrap().winner_event.channel, "high");
    }

    #[test]
    fn test_attend_empty_buffer() {
        let mut gw = PerceptionGateway::new(100);
        assert!(gw.attend().is_none());
    }

    #[test]
    fn test_broadcast_updates_context() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("ch", 100);
        gw.push_event("ch", serde_json::json!({"a": 1}), 0.7);
        let content = gw.attend().unwrap();
        let ctx_before = gw.vsa_context;
        gw.broadcast(content);
        assert_ne!(gw.vsa_context, ctx_before);
    }

    #[test]
    fn test_recent_broadcasts() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("ch", 100);
        for i in 0..5 {
            gw.push_event("ch", serde_json::json!({"i": i}), 0.5);
            if let Some(content) = gw.attend() {
                gw.broadcast(content);
            }
        }
        assert_eq!(gw.recent_broadcasts(10).len(), 5);
    }

    #[test]
    fn test_channel_stats_report() {
        let mut gw = PerceptionGateway::new(100);
        gw.register_channel("a", 100);
        gw.register_channel("b", 200);
        gw.push_event("a", serde_json::json!({"x": 1}), 0.5);
        if let Some(c) = gw.attend() { gw.broadcast(c); }
        let stats = gw.channel_stats();
        assert!(stats.contains_key("a"));
        assert!(stats.contains_key("b"));
    }

    #[test]
    fn test_vsa_encoding_deterministic() {
        let json = serde_json::json!({"hello": "world", "num": 42});
        let a = PerceptionGateway::vsa_encode_content(&json);
        let b = PerceptionGateway::vsa_encode_content(&json);
        assert_eq!(a, b);
    }

    #[test]
    fn test_requires_attention_flag() {
        let gw = PerceptionGateway::new(100);
        let event = make_event("test", 0.8);
        assert!(event.requires_attention);
        let event2 = make_event("test", 0.3);
        assert!(!event2.requires_attention);
    }

    #[test]
    fn test_attention_buffer_truncation() {
        let mut gw = PerceptionGateway::new(5);
        gw.register_channel("ch", 100);
        for i in 0..20 {
            gw.push_event("ch", serde_json::json!({"i": i}), (i as f64) / 20.0);
        }
        assert!(gw.attention_buffer.len() <= 5);
    }
}
