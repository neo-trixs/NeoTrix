#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverType {
    Kernel,
    Filesystem,
    Network,
    Storage,
    Display,
    Input,
}

impl std::fmt::Display for DriverType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kernel => write!(f, "Kernel"),
            Self::Filesystem => write!(f, "Filesystem"),
            Self::Network => write!(f, "Network"),
            Self::Storage => write!(f, "Storage"),
            Self::Display => write!(f, "Display"),
            Self::Input => write!(f, "Input"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
    pub driver_type: DriverType,
    pub signed: bool,
    pub vulnerabilities: Vec<String>,
}

pub struct DriverAnalyzer;

impl DriverAnalyzer {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, name: &str, version: &str, signed: bool) -> DriverInfo {
        let mut vulns = Vec::new();
        if !signed {
            vulns.push("Driver not signed".into());
        }
        if version.contains("0.1") || version.contains("beta") {
            vulns.push("Early version".into());
        }
        DriverInfo {
            name: name.to_string(),
            version: version.to_string(),
            driver_type: DriverType::Kernel,
            signed,
            vulnerabilities: vulns,
        }
    }
    pub fn risk_score(&self, info: &DriverInfo) -> f64 {
        100.0 - if !info.signed { 30.0 } else { 0.0 } - info.vulnerabilities.len() as f64 * 10.0
    }
}
impl Default for DriverAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_signed() {
        let a = DriverAnalyzer::new();
        let info = a.analyze("nvidia", "535.0", true);
        assert!(info.vulnerabilities.is_empty());
        assert_eq!(a.risk_score(&info), 100.0);
    }
    #[test]
    fn test_unsigned() {
        let a = DriverAnalyzer::new();
        let info = a.analyze("custom", "0.1-beta", false);
        assert_eq!(info.vulnerabilities.len(), 2);
        assert!(a.risk_score(&info) < 70.0);
    }
}
