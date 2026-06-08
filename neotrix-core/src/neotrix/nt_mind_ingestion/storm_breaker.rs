use crate::core::nt_core_consciousness::cognitive_load::CognitiveLoadMonitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningStormStatus {
    Normal,
    StormDetected { repeat_count: usize },
    Suppressed { mode: &'static str },
}

pub fn detect_reasoning_storm(monitor: &CognitiveLoadMonitor) -> ReasoningStormStatus {
    ReasoningStormStatus::Normal
}

pub fn next_storm_mode(current_iteration: u64) -> &'static str {
    match current_iteration % 3 {
        0 => "fast",
        1 => "balanced",
        2 => "deep",
        _ => "balanced",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storm_mode_alternates() {
        let modes: Vec<&str> = (0..6).map(|i| next_storm_mode(i)).collect();
        assert_eq!(modes, vec!["fast", "balanced", "deep", "fast", "balanced", "deep"]);
    }

    #[test]
    fn test_detect_storm_default() {
        let monitor = CognitiveLoadMonitor::new();
        let status = detect_reasoning_storm(&monitor);
        assert_eq!(status, ReasoningStormStatus::Normal);
    }
}
