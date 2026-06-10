/// Graceful degradation: subsystem capability gating

/// Which subsystems are available
#[derive(Debug, Clone)]
pub struct CapabilityStatus {
    pub jepa: bool,
    pub kb: bool,
    pub vision: bool,
    pub hypercube: bool,
    pub crypto: bool,
    pub gwt: bool,
}

impl CapabilityStatus {
    pub fn all_available() -> Self {
        Self {
            jepa: true, kb: true, vision: true,
            hypercube: true, crypto: true, gwt: true,
        }
    }

    pub fn minimal() -> Self {
        Self {
            jepa: false, kb: false, vision: false,
            hypercube: true, crypto: false, gwt: false,
        }
    }

    /// Detect actual availability from SelfIteratingBrain
    pub fn detect(brain_has_jepa: bool, brain_has_kb: bool, brain_has_crypto: bool) -> Self {
        Self {
            jepa: brain_has_jepa,
            kb: brain_has_kb,
            vision: false,
            hypercube: true,
            crypto: brain_has_crypto,
            gwt: true,
        }
    }

    /// How many subsystems are available
    pub fn available_count(&self) -> usize {
        [self.jepa, self.kb, self.vision, self.hypercube, self.crypto, self.gwt]
            .iter().filter(|&&x| x).count()
    }
}

/// Degradation level based on available subsystems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradationLevel {
    Full,
    Reduced,
    Limited,
    Minimal,
}

impl DegradationLevel {
    pub fn from_capabilities(status: &CapabilityStatus) -> Self {
        match status.available_count() {
            6 => DegradationLevel::Full,
            4..=5 => DegradationLevel::Reduced,
            2..=3 => DegradationLevel::Limited,
            _ => DegradationLevel::Minimal,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DegradationLevel::Full => "all subsystems operational",
            DegradationLevel::Reduced => "some subsystems unavailable, reduced capability",
            DegradationLevel::Limited => "limited operation, critical subsystems only",
            DegradationLevel::Minimal => "minimal operation, pure HyperCube mode",
        }
    }
}

/// Apply a recoverable error wrapper around a stage's result.
pub fn degrade_on_error(stage_name: &str, result: Result<(), crate::neotrix::nt_core_error::NeoTrixError>) -> Result<(), crate::neotrix::nt_core_error::NeoTrixError> {
    match result {
        Ok(()) => Ok(()),
        Err(e) => {
            log::warn!("[degradation] stage '{}' failed, degrading: {}", stage_name, e);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_available_full() {
        let status = CapabilityStatus::all_available();
        assert_eq!(DegradationLevel::from_capabilities(&status), DegradationLevel::Full);
    }

    #[test]
    fn test_minimal_level() {
        let status = CapabilityStatus::minimal();
        assert_eq!(DegradationLevel::from_capabilities(&status), DegradationLevel::Minimal);
    }

    #[test]
    fn test_detect_empty() {
        let status = CapabilityStatus::detect(false, false, false);
        assert_eq!(status.available_count(), 2);
        assert_eq!(DegradationLevel::from_capabilities(&status), DegradationLevel::Limited);
    }

    #[test]
    fn test_detect_full() {
        let status = CapabilityStatus::detect(true, true, true);
        assert!(status.available_count() >= 3);
    }

    #[test]
    fn test_degradation_descriptions() {
        assert!(DegradationLevel::Full.description().contains("all"));
        assert!(DegradationLevel::Minimal.description().contains("minimal"));
    }
}
