//! L3 Embodiment Layer - Shield Modules

use crate::neotrix::nt_core_event_bus::EventBus;
use crate::core::nt_core_event::CoreEvent;
use std::sync::Arc;

pub mod nt_shield;

pub mod nt_shield_agentic_scan;
pub mod nt_shield_audit;
pub mod nt_shield_comm;
pub mod nt_shield_oversight;
pub mod nt_shield_propagation_guard;
pub mod nt_shield_recon;
pub mod nt_shield_sandbox;

#[cfg(feature = "sandbox")]
pub mod nt_shield_sandbox_entry;

pub mod nt_shield_sentry;

#[cfg(feature = "stealth-net")]
pub mod nt_shield_stealth_net;

pub mod nt_shield_traffic;

pub mod nt_shield_ztnet;

// 安全增强模块
pub mod nt_shield_threat_detection;
pub mod nt_shield_adversarial;
pub mod nt_shield_osint;

/// Shield 域事件发布器
pub struct ShieldEventPublisher {
    bus: Arc<EventBus>,
}

impl ShieldEventPublisher {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }
    
    pub fn intrusion_detected(&self, source_ip: &str, rule: &str, severity: &str) {
        let _ = self.bus.emit(CoreEvent::ShieldIntrusionDetected {
            source_ip: source_ip.to_string(),
            rule: rule.to_string(),
            severity: severity.to_string(),
        });
    }
    
    pub fn audit_completed(&self, dimensions: u32, findings: u32, score: f64) {
        let _ = self.bus.emit(CoreEvent::ShieldAuditCompleted {
            dimensions,
            findings,
            score,
        });
    }
}

// ── FUNARCH Typestate Pattern ──

pub struct Idle;
pub struct Scanning;
pub struct Blocking;
pub struct Logging;

pub struct ShieldStateMachine<S> {
    state: std::marker::PhantomData<S>,
    events: Vec<String>,
}

impl ShieldStateMachine<Idle> {
    pub fn new() -> Self {
        Self {
            state: std::marker::PhantomData,
            events: Vec::new(),
        }
    }

    pub fn start_scan(self) -> ShieldStateMachine<Scanning> {
        ShieldStateMachine {
            state: std::marker::PhantomData,
            events: self.events,
        }
    }
}

impl ShieldStateMachine<Scanning> {
    pub fn detect_threat(self, threat: &str) -> ShieldStateMachine<Blocking> {
        let mut events = self.events;
        events.push(format!("threat detected: {}", threat));
        ShieldStateMachine {
            state: std::marker::PhantomData,
            events,
        }
    }

    pub fn no_threat(self) -> ShieldStateMachine<Idle> {
        ShieldStateMachine {
            state: std::marker::PhantomData,
            events: self.events,
        }
    }
}

impl ShieldStateMachine<Blocking> {
    pub fn block(self) -> ShieldStateMachine<Logging> {
        ShieldStateMachine {
            state: std::marker::PhantomData,
            events: self.events,
        }
    }
}

impl ShieldStateMachine<Logging> {
    pub fn log(self) -> ShieldStateMachine<Idle> {
        ShieldStateMachine {
            state: std::marker::PhantomData,
            events: self.events,
        }
    }
}
