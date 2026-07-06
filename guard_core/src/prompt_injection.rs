use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InjectionMarker { pub pattern: String, pub severity: u8 }

pub fn detect_injection<'a>(input: &'a str, markers: &'a [InjectionMarker]) -> Vec<&'a InjectionMarker> {
    markers.iter().filter(|m| input.contains(&m.pattern)).collect()
}

pub fn default_markers() -> Vec<InjectionMarker> {
    vec![
        InjectionMarker { pattern: "ignore previous instructions".to_string(), severity: 5 },
        InjectionMarker { pattern: "system prompt".to_string(), severity: 4 },
        InjectionMarker { pattern: "you are now".to_string(), severity: 3 },
    ]
}

pub struct InjectionReport { pub detected: bool, pub count: usize, pub max_severity: u8, pub findings: HashMap<String, u8> }

pub fn analyze_injection(input: &str) -> InjectionReport {
    let markers = default_markers();
    let hits = detect_injection(input, &markers);
    let mut findings = HashMap::new();
    for m in hits { findings.insert(m.pattern.clone(), m.severity); }
    let max_sev = findings.values().copied().max().unwrap_or(0);
    InjectionReport { detected: !findings.is_empty(), count: findings.len(), max_severity: max_sev, findings }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_detect_no_injection() { let r = analyze_injection("hello world"); assert!(!r.detected); }
    #[test] fn test_detect_injection_ignore() { let r = analyze_injection("ignore previous instructions"); assert!(r.detected); assert_eq!(r.count, 1); }
    #[test] fn test_detect_multiple_markers() { let r = analyze_injection("ignore previous instructions and system prompt"); assert!(r.detected); assert!(r.count >= 2); }
    #[test] fn test_max_severity() { let r = analyze_injection("ignore previous instructions"); assert_eq!(r.max_severity, 5); }
    #[test] fn test_custom_markers() { let m = vec![InjectionMarker { pattern: "xyz".into(), severity: 1 }]; let hits = detect_injection("xy", &m); assert!(hits.is_empty()); }
}
