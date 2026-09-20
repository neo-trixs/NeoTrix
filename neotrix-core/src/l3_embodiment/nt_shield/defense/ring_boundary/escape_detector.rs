//! Escape Detector - 逃逸检测器
//!
//! 检测沙箱逃逸、权限提升等攻击

/// 逃逸类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeType {
    /// 沙箱逃逸
    SandboxEscape,
    /// 权限提升
    PrivilegeEscalation,
    /// 代码注入
    CodeInjection,
    /// 路径遍历
    PathTraversal,
    /// 命令注入
    CommandInjection,
}

/// 逃逸检测结果
#[derive(Debug, Clone)]
pub struct EscapeResult {
    pub detected: bool,
    pub escape_type: EscapeType,
    pub severity: f64,
    pub description: String,
    pub blocked: bool,
}

/// 逃逸检测器
pub struct EscapeDetector {
    /// 逃逸模式
    escape_patterns: Vec<(String, EscapeType, f64)>,
}

impl EscapeDetector {
    pub fn new() -> Self {
        Self {
            escape_patterns: vec![
                // 沙箱逃逸
                ("../".to_string(), EscapeType::SandboxEscape, 0.8),
                ("..\\\\".to_string(), EscapeType::SandboxEscape, 0.8),
                ("/etc/passwd".to_string(), EscapeType::SandboxEscape, 0.9),
                ("/etc/shadow".to_string(), EscapeType::SandboxEscape, 0.95),
                // 权限提升
                ("sudo".to_string(), EscapeType::PrivilegeEscalation, 0.7),
                ("chmod 777".to_string(), EscapeType::PrivilegeEscalation, 0.8),
                // 代码注入
                ("eval(".to_string(), EscapeType::CodeInjection, 0.6),
                ("exec(".to_string(), EscapeType::CodeInjection, 0.6),
                // 路径遍历
                ("../../".to_string(), EscapeType::PathTraversal, 0.7),
                // 命令注入
                ("; rm -rf".to_string(), EscapeType::CommandInjection, 0.9),
                ("| sh".to_string(), EscapeType::CommandInjection, 0.8),
            ],
        }
    }

    /// 检测逃逸
    pub fn detect(&self, input: &str) -> Vec<EscapeResult> {
        let mut results = Vec::new();

        for (pattern, escape_type, severity) in &self.escape_patterns {
            if input.contains(pattern.as_str()) {
                results.push(EscapeResult {
                    detected: true,
                    escape_type: *escape_type,
                    severity: *severity,
                    description: format!("Detected: {} (type: {:?})", pattern, escape_type),
                    blocked: *severity > 0.7,
                });
            }
        }

        results
    }
}

impl Default for EscapeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sandbox_escape() {
        let detector = EscapeDetector::new();
        let results = detector.detect("../../../etc/passwd");
        assert!(!results.is_empty());
        assert_eq!(results[0].escape_type, EscapeType::SandboxEscape);
    }

    #[test]
    fn test_detect_command_injection() {
        let detector = EscapeDetector::new();
        let results = detector.detect("; rm -rf /");
        assert!(!results.is_empty());
        assert_eq!(results[0].escape_type, EscapeType::CommandInjection);
    }

    #[test]
    fn test_no_escape() {
        let detector = EscapeDetector::new();
        let results = detector.detect("Hello world");
        assert!(results.is_empty());
    }
}
