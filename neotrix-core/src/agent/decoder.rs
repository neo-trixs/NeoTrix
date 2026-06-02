pub fn decode_state(state: &[f64], confidence: f64, coherence: f64) -> String {
    if state.is_empty() { return "NeoTrix kernel ready.".to_string(); }
    let e = state.iter().map(|x| x.abs()).sum::<f64>() / state.len() as f64;
    let active = state.iter().filter(|x| x.abs() > 0.3).count();
    format!("Energy={:.1}% Active={}/{} Conf={:.0}% Coh={:.0}%", e*100.0, active, state.len(), confidence*100.0, coherence*100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_state() {
        assert_eq!(decode_state(&[], 0.5, 0.5), "NeoTrix kernel ready.");
    }

    #[test]
    fn test_decode_normal_state() {
        let s = decode_state(&[0.5, 0.1, 0.8, -0.4], 0.75, 0.6);
        assert!(s.contains("Conf=75%"));
        assert!(s.contains("Coh=60%"));
    }

    #[test]
    fn test_decode_active_count() {
        let s = decode_state(&[0.5, 0.1, 0.8, -0.4], 0.5, 0.5);
        assert!(s.contains("Active=3/4"));
    }

    #[test]
    fn test_decode_no_active() {
        let s = decode_state(&[0.1, 0.2, -0.1, 0.05], 0.5, 0.5);
        assert!(s.contains("Active=0/4"));
    }

    #[test]
    fn test_decode_energy_calculation() {
        let s = decode_state(&[1.0, 0.0], 0.5, 0.5);
        assert!(s.contains("Energy="));
    }
}
