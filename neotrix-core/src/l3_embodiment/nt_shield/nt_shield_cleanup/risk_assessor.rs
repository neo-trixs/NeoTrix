//! Risk Assessor - 风险评估器
//!
//! 评估清理操作的风险等级
//! 域: NT-SHIELD (影卫)
//! 层: L3 Embodiment

use crate::l3_embodiment::l1_facade::*;

pub struct RiskAssessor {
    confirmation_threshold: u8,
    whitelist: Vec<String>,
}

impl RiskAssessor {
    pub fn new() -> Self {
        Self { confirmation_threshold: 60, whitelist: Vec::new() }
    }

    pub fn add_whitelist(&mut self, path: String) { self.whitelist.push(path); }
    pub fn _set_confirmation_threshold(&mut self, threshold: u8) { self.confirmation_threshold = threshold; }

    fn is_whitelisted(&self, path: &str) -> bool {
        self.whitelist.iter().any(|w| path.starts_with(w))
    }

    pub fn assess(&self, category: &ScanCategory, size_bytes: u64, age_days: u32, is_system_path: bool, path: &str) -> RiskAssessment {
        if self.is_whitelisted(path) {
            return RiskAssessment { level: RiskLevel::Safe, score: 0, reasons: vec!["白名单路径".into()], requires_confirmation: false, recommendation: "白名单路径，可以安全删除".into() };
        }
        let mut score: u8 = 0;
        let mut reasons = Vec::new();
        match category {
            ScanCategory::SystemCache | ScanCategory::TempFile => { score += 10; reasons.push("系统缓存/临时文件".into()); }
            ScanCategory::SystemLog | ScanCategory::UserLog => { score += 20; reasons.push("日志文件".into()); }
            ScanCategory::BrowserCache => { score += 15; reasons.push("浏览器缓存".into()); }
            ScanCategory::DeveloperCache => { score += 25; reasons.push("开发者缓存".into()); }
            ScanCategory::ApplicationSupport => { score += 60; reasons.push("应用支持数据".into()); }
            ScanCategory::LargeFile | ScanCategory::OldFile => { score += 50; reasons.push("大文件/旧文件".into()); }
            _ => { score += 10; }
        }
        if size_bytes > 1024 * 1024 * 1024 { score += 30; reasons.push("文件过大".into()); }
        else if size_bytes > 100 * 1024 * 1024 { score += 20; reasons.push("文件较大".into()); }
        if age_days > 365 { score += 15; reasons.push("超过1年".into()); }
        else if age_days > 90 { score += 10; reasons.push("超过90天".into()); }
        if is_system_path { score += 30; reasons.push("系统路径".into()); }
        score = score.min(100);
        let level = if score >= 80 { RiskLevel::Protected }
        else if score >= 60 { RiskLevel::Risky }
        else if score >= 30 { RiskLevel::Moderate }
        else { RiskLevel::Safe };
        let requires_confirmation = score >= self.confirmation_threshold;
        let recommendation = match level {
            RiskLevel::Safe => "可以安全删除".into(),
            RiskLevel::Moderate => "建议确认后删除".into(),
            RiskLevel::Risky => "高风险操作，需要仔细确认".into(),
            RiskLevel::Protected => "系统保护，不可删除".into(),
        };
        RiskAssessment { level, score, reasons, requires_confirmation, recommendation }
    }
}

impl Default for RiskAssessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assess_safe() {
        let a = RiskAssessor::new();
        let r = a.assess(&ScanCategory::SystemCache, 1024, 30, false, "/tmp/test");
        assert_eq!(r.level, RiskLevel::Safe);
    }

    #[test]
    fn test_assess_risky() {
        let a = RiskAssessor::new();
        let r = a.assess(&ScanCategory::ApplicationSupport, 1024 * 1024 * 100, 365, false, "/Users/test/AppSupport");
        assert!(r.score >= 60);
    }

    #[test]
    fn test_whitelist() {
        let mut a = RiskAssessor::new();
        a.add_whitelist("/safe/".into());
        let r = a.assess(&ScanCategory::LargeFile, 1024 * 1024, 30, false, "/safe/file.zip");
        assert_eq!(r.score, 0);
    }
}
