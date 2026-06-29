pub mod fable_router;
pub mod honeypot;
pub mod integrity;
pub mod obfuscate;
pub mod red_team;
pub mod sanitize;
pub use fable_router::*;

pub use honeypot::*;
pub use integrity::*;
pub use obfuscate::*;
pub use red_team::*;
pub use sanitize::{install_panic_filter, sanitize_panic, strip_source_path, SafeError};

pub struct SelfProtection {
    pub integrity: IntegrityGuard,
    pub obfuscation_key: u8,
    pub tamper_count: u64,
    pub decoy_objects: Vec<HoneypotNode>,
    pub initialized: bool,
}

impl SelfProtection {
    pub fn new() -> Self {
        Self {
            integrity: IntegrityGuard::new(),
            obfuscation_key: 0xAB,
            tamper_count: 0,
            decoy_objects: HoneypotForest::generate(),
            initialized: true,
        }
    }

    pub fn check_integrity(&mut self) -> IntegrityReport {
        let report = self.integrity.verify();
        if !report.clean {
            self.tamper_count += 1;
        }
        report
    }

    pub fn stats(&self) -> ProtectionStats {
        ProtectionStats {
            tamper_detections: self.tamper_count,
            decoy_count: self.decoy_objects.len() as u64,
            integrity_clean: self.integrity.last_clean,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProtectionStats {
    pub tamper_detections: u64,
    pub decoy_count: u64,
    pub integrity_clean: bool,
}
