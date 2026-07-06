//! L7 StarPulse Protocol — 层间通信超向量消息
//!
//! StarPulse 是 NeoTrix 9 层架构中层间通信的唯一方式。
//! 更新 (2026-07-01): 引入三种寻址语义 (Broadcast/Unicast/Anycast)
//! 参考 Cotal 协议: multicast ↔ 频道广播, unicast ↔ 点对点DM, anycast ↔ 角色分发
//!
//! 寻址格式: starpulse.<layer>.<module>.<sender>.<target>
//! - Broadcast: starpulse.<layer>.<module>.<sender>.>       (发送给该层所有模块)
//! - Unicast:   starpulse.<layer>.<module>.<sender>.<target> (发送给指定模块实例)
//! - Anycast:   starpulse.anycast.<role>.<sender>            (任意一个具有该角色的模块处理)

use std::collections::HashMap;
use crate::core::nt_core_self::emotion_state::EmotionReport;

/// Unique module/agent identifier in the layer topology
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleId {
    pub layer: u8,
    pub name: String,
    pub instance: String,
}

impl ModuleId {
    pub fn new(layer: u8, name: &str, instance: &str) -> Self {
        Self { layer, name: name.to_string(), instance: instance.to_string() }
    }

    pub fn to_subject(&self) -> String {
        format!("starpulse.L{}.{}.{}", self.layer, self.name, self.instance)
    }
}

/// Three addressing modes for StarPulse messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressingMode {
    /// Broadcast to all modules on a given layer
    /// Example: starpulse.L4.> (all L4 cognition modules)
    Broadcast {
        target_layer: u8,
        target_module: Option<String>,
    },
    /// Direct message to a specific module instance
    /// Example: starpulse.L4.e8.engine-1
    Unicast(ModuleId),
    /// Deliver to any one module that provides the capability role
    /// Example: starpulse.anycast.memory-retriever
    Anycast {
        role: String,
        required_layer: Option<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct StarPulse {
    pub id: u64,
    pub ts: f64,
    pub kind: PulseKind,
    pub addressing: AddressingMode,
    pub sender: ModuleId,
    pub payload: PulsePayload,
}

impl StarPulse {
    pub fn new_broadcast(kind: PulseKind, sender: ModuleId, target_layer: u8, payload: PulsePayload) -> Self {
        Self {
            id: next_pulse_id(),
            ts: now_ms(),
            kind,
            addressing: AddressingMode::Broadcast { target_layer, target_module: None },
            sender,
            payload,
        }
    }

    pub fn new_unicast(kind: PulseKind, sender: ModuleId, target: ModuleId, payload: PulsePayload) -> Self {
        Self {
            id: next_pulse_id(),
            ts: now_ms(),
            kind,
            addressing: AddressingMode::Unicast(target),
            sender,
            payload,
        }
    }

    pub fn new_anycast(kind: PulseKind, sender: ModuleId, role: &str, payload: PulsePayload) -> Self {
        Self {
            id: next_pulse_id(),
            ts: now_ms(),
            kind,
            addressing: AddressingMode::Anycast { role: role.to_string(), required_layer: None },
            sender,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PulseKind {
    Discovery,
    Heartbeat,
    Eviction,
    Emotion,
    Skill,
    Immunity,
    Calibration,
    /// Capability bidding request (from L4 E8 → L7)
    CapabilityRequest,
    /// Capability bidding response (from L7 → L4)
    CapabilityResponse,
    /// Agent-to-Agent communication (A2A protocol)
    AgentToAgent,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum PulsePayload {
    Empty,
    Text(String),
    Emotion(EmotionPulsePayload),
    CapabilityBid {
        capability_id: String,
        maturity_score: f64,
        confidence: f64,
    },
    Heartbeat(HeartbeatPayload),
    Json(serde_json::Value),
    /// A2A task card transfer
    A2ATaskCard(Box<crate::core::l7_capability::a2a::A2ATask>),
    /// A2A agent card (discovery)
    AgentCard(crate::core::l7_capability::a2a::AgentCard),
}

#[derive(Debug, Clone)]
pub struct EmotionPulsePayload {
    pub source_layer: u8,
    pub arousal: f64,
    pub valence: f64,
    pub confidence_score: f64,
    pub dominant_dimension: String,
    pub dominant_deviation: f64,
}

impl EmotionPulsePayload {
    pub fn from_report(report: &EmotionReport) -> Self {
        Self {
            source_layer: 6,
            arousal: report.arousal,
            valence: report.valence,
            confidence_score: report.confidence_score,
            dominant_dimension: format!("{:?}", report.dominant.0),
            dominant_deviation: report.dominant.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeartbeatPayload {
    pub layer: u8,
    pub module: String,
    pub status: ModuleStatus,
    pub load: f64,
    pub active_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    Idle,
    Busy,
    Full,
    Offline,
    Degraded,
}

/// Presence registry: tracks which modules are alive and their state
#[derive(Debug, Clone)]
pub struct PresenceRegistry {
    modules: HashMap<String, HeartbeatPayload>,
    max_entries: usize,
}

impl PresenceRegistry {
    pub fn new() -> Self {
        Self { modules: HashMap::new(), max_entries: 1000 }
    }

    pub fn register(&mut self, id: &ModuleId, status: ModuleStatus) {
        if self.modules.len() >= self.max_entries {
            // Evict oldest offline module first
            if let Some(oldest_offline) = self.modules.iter()
                .find(|(_, m)| m.status == ModuleStatus::Offline)
                .map(|(k, _)| k.clone())
            {
                self.modules.remove(&oldest_offline);
            } else {
                return; // Full capacity, no offline modules to evict
            }
        }
        let key = id.to_subject();
        self.modules.insert(key, HeartbeatPayload {
            layer: id.layer,
            module: id.name.clone(),
            status,
            load: 0.0,
            active_capabilities: vec![],
        });
    }

    pub fn heartbeat(&mut self, id: &ModuleId, payload: HeartbeatPayload) {
        self.modules.insert(id.to_subject(), payload);
    }

    pub fn unregister(&mut self, id: &ModuleId) {
        self.modules.remove(&id.to_subject());
    }

    pub fn get(&self, id: &ModuleId) -> Option<&HeartbeatPayload> {
        self.modules.get(&id.to_subject())
    }

    pub fn live_modules(&self) -> Vec<&HeartbeatPayload> {
        self.modules.values().filter(|m| m.status != ModuleStatus::Offline).collect()
    }

    /// Anycast resolution: find any module that provides the given role
    pub fn resolve_anycast(&self, role: &str, layer: Option<u8>) -> Option<HeartbeatPayload> {
        self.modules.values()
            .filter(|m| m.status != ModuleStatus::Offline)
            .filter(|m| layer.map(|l| m.layer == l).unwrap_or(true))
            .find(|m| m.active_capabilities.iter().any(|c| c.contains(role)))
            .cloned()
    }
}

impl Default for PresenceRegistry {
    fn default() -> Self { Self::new() }
}

static PULSE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
fn next_pulse_id() -> u64 {
    PULSE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64() * 1000.0
}

#[derive(Debug, Clone)]
pub struct PulseBus {
    pulse_log: Vec<(PulseKind, String)>,
    max_log: usize,
    subscribers: HashMap<String, Vec<(ModuleId, String)>>,
    presence: PresenceRegistry,
}

impl PulseBus {
    pub fn new() -> Self {
        Self {
            pulse_log: Vec::with_capacity(100),
            max_log: 100,
            subscribers: HashMap::new(),
            presence: PresenceRegistry::new(),
        }
    }

    /// Broadcast pulse to all modules on a target layer
    pub fn broadcast(&self, pulse: &StarPulse) -> Vec<&HeartbeatPayload> {
        match &pulse.addressing {
            AddressingMode::Broadcast { target_layer, target_module } => {
                self.presence.live_modules().into_iter()
                    .filter(|m| m.layer == *target_layer)
                    .filter(|m| target_module.as_ref().map(|t| m.module == *t).unwrap_or(true))
                    .collect()
            }
            _ => vec![],
        }
    }

    /// Unicast pulse to a specific module
    pub fn unicast(&self, pulse: &StarPulse) -> Option<&HeartbeatPayload> {
        match &pulse.addressing {
            AddressingMode::Unicast(target) => self.presence.get(target),
            _ => None,
        }
    }

    /// Anycast pulse to the first available module with the target role
    pub fn anycast(&self, pulse: &StarPulse) -> Option<HeartbeatPayload> {
        match &pulse.addressing {
            AddressingMode::Anycast { role, required_layer } => {
                self.presence.resolve_anycast(role, *required_layer)
            }
            _ => None,
        }
    }

    pub fn subscribe(&mut self, id: &ModuleId, pulse_kind: &str) {
        self.subscribers.entry(pulse_kind.to_string())
            .or_default()
            .push((id.clone(), id.to_subject()));
    }

    pub fn unsubscribe(&mut self, id: &ModuleId, pulse_kind: &str) {
        if let Some(subs) = self.subscribers.get_mut(pulse_kind) {
            subs.retain(|(m, _)| m.to_subject() != id.to_subject());
        }
    }

    pub fn subscribers_for(&self, pulse_kind: &str) -> Vec<&ModuleId> {
        self.subscribers.get(pulse_kind)
            .map(|v| v.iter().map(|(m, _)| m).collect())
            .unwrap_or_default()
    }

    pub fn register_module(&mut self, id: &ModuleId, status: ModuleStatus) {
        self.presence.register(id, status);
        self.log(PulseKind::Discovery, format!("module registered: {}", id.to_subject()));
    }

    pub fn heartbeat_pulse(&mut self, id: &ModuleId, status: ModuleStatus, load: f64) {
        let payload = HeartbeatPayload {
            layer: id.layer,
            module: id.name.clone(),
            status,
            load,
            active_capabilities: vec![],
        };
        self.presence.heartbeat(id, payload);
    }

    pub fn emit_emotion(&mut self, payload: &EmotionPulsePayload) {
        if self.pulse_log.len() >= self.max_log {
            self.pulse_log.remove(0);
        }
        self.pulse_log.push((
            PulseKind::Emotion,
            format!(
                "L{}: arousal={:.3} valence={:.3} conf={:.3} dominant={}/dev={:.3}",
                payload.source_layer,
                payload.arousal,
                payload.valence,
                payload.confidence_score,
                payload.dominant_dimension,
                payload.dominant_deviation,
            ),
        ));
    }

    pub fn log(&mut self, kind: PulseKind, message: impl Into<String>) {
        if self.pulse_log.len() >= self.max_log {
            self.pulse_log.remove(0);
        }
        self.pulse_log.push((kind, message.into()));
    }

    pub fn recent(&self, n: usize) -> Vec<&(PulseKind, String)> {
        self.pulse_log.iter().rev().take(n).collect()
    }

    pub fn clear(&mut self) {
        self.pulse_log.clear();
    }
}

impl Default for PulseBus {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
