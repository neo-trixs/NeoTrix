use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CaptchaHandler {
    config: CaptchaConfig,
    solving_history: Vec<CaptchaAttempt>,
    rate_limiter: HashMap<String, Vec<u64>>,
}

#[derive(Debug, Clone)]
pub struct CaptchaConfig {
    pub two_captcha_key: Option<String>,
    pub anti_captcha_key: Option<String>,
    pub default_timeout_ms: u64,
    pub max_retries: u32,
    pub use_ocr_fallback: bool,
}

impl Default for CaptchaConfig {
    fn default() -> Self {
        Self {
            two_captcha_key: None,
            anti_captcha_key: None,
            default_timeout_ms: 30000,
            max_retries: 3,
            use_ocr_fallback: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaptchaAttempt {
    pub captcha_type: CaptchaType,
    pub solver: SolverService,
    pub image_hash: [u64; 4],
    pub solution: Option<String>,
    pub success: bool,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptchaType {
    ImageText, ReCaptchaV2, ReCaptchaV3, HCaptcha, Geetest, CloudflareTurnstile, Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverService {
    TwoCaptcha, AntiCaptcha, OcrFallback, Manual,
}

impl CaptchaHandler {
    pub fn new(config: CaptchaConfig) -> Self {
        Self {
            config,
            solving_history: Vec::new(),
            rate_limiter: HashMap::new(),
        }
    }

    pub fn detect_type(image_b64: &str) -> CaptchaType {
        if image_b64.len() < 20 {
            return CaptchaType::Unknown;
        }
        let lower = image_b64.to_lowercase();
        if lower.contains("recaptcha") || lower.contains("g-recaptcha") {
            if lower.contains("invisible") { CaptchaType::ReCaptchaV3 } else { CaptchaType::ReCaptchaV2 }
        } else if lower.contains("hcaptcha") { CaptchaType::HCaptcha }
        else if lower.contains("geetest") || lower.contains("slide") { CaptchaType::Geetest }
        else if lower.contains("turnstile") || lower.contains("cf-turnstile") { CaptchaType::CloudflareTurnstile }
        else if lower.len() > 100 { CaptchaType::ImageText }
        else { CaptchaType::Unknown }
    }

    pub fn solve(&self, image_b64: &str, captcha_type: CaptchaType) -> Result<String, String> {
        if self.is_rate_limited("two_captcha") {
            return self.solve_with_ocr(image_b64);
        }
        match captcha_type {
            CaptchaType::ReCaptchaV2 | CaptchaType::HCaptcha => Ok("captcha_solved_mock".into()),
            CaptchaType::ImageText => self.solve_with_ocr(image_b64),
            CaptchaType::ReCaptchaV3 => Ok("0.9".into()),
            CaptchaType::Geetest => Ok("{\"challenge\":\"mock\",\"validate\":\"mock\",\"seccode\":\"mock\"}".into()),
            _ => Ok("captcha_solved_fallback".into()),
        }
    }

    pub fn solve_with_ocr(&self, image_b64: &str) -> Result<String, String> {
        if !self.config.use_ocr_fallback {
            return Err("ocr fallback disabled".into());
        }
        let hash = Self::compute_image_vsa(image_b64);
        let code = format!("ocr_{:x}{:x}", hash[0] & 0xFFFF, hash[1] & 0xFFFF);
        Ok(code)
    }

    pub fn record_attempt(&mut self, attempt: CaptchaAttempt) {
        self.solving_history.push(attempt);
        if self.solving_history.len() > 1000 {
            self.solving_history.remove(0);
        }
    }

    pub fn success_rate(&self, solver: &SolverService) -> f64 {
        let attempts: Vec<&CaptchaAttempt> = self.solving_history.iter().filter(|a| a.solver == *solver).collect();
        if attempts.is_empty() { return 0.0; }
        let successes = attempts.iter().filter(|a| a.success).count();
        successes as f64 / attempts.len() as f64
    }

    pub fn best_solver_for(&self, captcha_type: &CaptchaType) -> SolverService {
        let candidates = [SolverService::TwoCaptcha, SolverService::AntiCaptcha, SolverService::OcrFallback];
        candidates.into_iter().max_by_key(|s| {
            let rate = (self.success_rate(s) * 100.0) as u64;
            let count = self.solving_history.iter().filter(|a| a.captcha_type == *captcha_type && a.solver == *s).count() as u64;
            rate + count.min(10)
        }).unwrap_or(SolverService::OcrFallback)
    }

    pub fn compute_image_vsa(image_b64: &str) -> [u64; 4] {
        let h1: u64 = image_b64.bytes().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(31).wrapping_add(b as u64 ^ (i as u64 * 7)));
        let h2: u64 = image_b64.bytes().rev().enumerate().fold(0u64, |acc, (i, b)| acc.wrapping_mul(37).wrapping_add(b as u64 ^ (i as u64 * 13)));
        let h3: u64 = image_b64.bytes().step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(41).wrapping_add(b as u64));
        let h4: u64 = image_b64.bytes().skip(1).step_by(2).fold(0u64, |acc, b| acc.wrapping_mul(43).wrapping_add(b as u64));
        [h1 ^ h3, h2 ^ h4, h1.wrapping_add(h2), h3.wrapping_add(h4)]
    }

    fn is_rate_limited(&self, service: &str) -> bool {
        if let Some(times) = self.rate_limiter.get(service) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            let recent = times.iter().filter(|t| now - **t < 1000).count();
            recent >= 5
        } else { false }
    }

    pub fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_image_text() {
        let b64 = "iVBORw0KGgoAAAANSUhEUgAAABQAAAAUCAYAAACNiR0NAAAAKUlEQVQ4y2P4z8BQbwADBQYGBgYGBkYGJgYqAwMDAwMDAwMDAwMDAPhXBxQAAAAASUVORK5CYII=";
        assert_eq!(CaptchaHandler::detect_type(b64), CaptchaType::ImageText);
    }

    #[test]
    fn test_detect_recaptcha_v2() {
        let b64 = "data:image/g-recaptcha-response";
        assert_eq!(CaptchaHandler::detect_type(b64), CaptchaType::ReCaptchaV2);
    }

    #[test]
    fn test_detect_recaptcha_v3() {
        let b64 = "invisible g-recaptcha";
        assert_eq!(CaptchaHandler::detect_type(b64), CaptchaType::ReCaptchaV3);
    }

    #[test]
    fn test_detect_hcaptcha() {
        let b64 = "hcaptcha_widget_data_".repeat(10);
        assert_eq!(CaptchaHandler::detect_type(&b64), CaptchaType::HCaptcha);
    }

    #[test]
    fn test_detect_geetest() {
        let b64 = "geetest_slide_".repeat(10);
        assert_eq!(CaptchaHandler::detect_type(&b64), CaptchaType::Geetest);
    }

    #[test]
    fn test_detect_cloudflare() {
        let b64 = "cf_turnstile_token_".repeat(10);
        assert_eq!(CaptchaHandler::detect_type(&b64), CaptchaType::CloudflareTurnstile);
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(CaptchaHandler::detect_type("ab"), CaptchaType::Unknown);
    }

    #[test]
    fn test_solve_returns_solution() {
        let handler = CaptchaHandler::new(CaptchaConfig::default());
        assert!(handler.solve("image_data", CaptchaType::ImageText).is_ok());
    }

    #[test]
    fn test_ocr_fallback_disabled() {
        let config = CaptchaConfig { use_ocr_fallback: false, ..Default::default() };
        let handler = CaptchaHandler::new(config);
        assert!(handler.solve("image_data", CaptchaType::ImageText).is_err());
    }

    #[test]
    fn test_record_and_success_rate() {
        let mut handler = CaptchaHandler::new(CaptchaConfig::default());
        handler.record_attempt(CaptchaAttempt {
            captcha_type: CaptchaType::ImageText,
            solver: SolverService::OcrFallback,
            image_hash: [0; 4],
            solution: Some("abc".into()),
            success: true,
            latency_ms: 100,
        });
        handler.record_attempt(CaptchaAttempt {
            captcha_type: CaptchaType::ImageText,
            solver: SolverService::OcrFallback,
            image_hash: [0; 4],
            solution: None,
            success: false,
            latency_ms: 50,
        });
        assert!((handler.success_rate(&SolverService::OcrFallback) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_success_rate_empty() {
        let handler = CaptchaHandler::new(CaptchaConfig::default());
        assert_eq!(handler.success_rate(&SolverService::TwoCaptcha), 0.0);
    }

    #[test]
    fn test_best_solver_prefers_successful() {
        let mut handler = CaptchaHandler::new(CaptchaConfig::default());
        handler.record_attempt(CaptchaAttempt {
            captcha_type: CaptchaType::ReCaptchaV2, solver: SolverService::TwoCaptcha,
            image_hash: [0; 4], solution: Some("x".into()), success: true, latency_ms: 100,
        });
        handler.record_attempt(CaptchaAttempt {
            captcha_type: CaptchaType::ReCaptchaV2, solver: SolverService::AntiCaptcha,
            image_hash: [0; 4], solution: None, success: false, latency_ms: 100,
        });
        assert_eq!(handler.best_solver_for(&CaptchaType::ReCaptchaV2), SolverService::TwoCaptcha);
    }

    #[test]
    fn test_vsa_encoding_deterministic() {
        let a = CaptchaHandler::compute_image_vsa("test_image");
        let b = CaptchaHandler::compute_image_vsa("test_image");
        assert_eq!(a, b);
    }

    #[test]
    fn test_vsa_encoding_different_inputs_differ() {
        let a = CaptchaHandler::compute_image_vsa("image_a");
        let b = CaptchaHandler::compute_image_vsa("image_b");
        assert_ne!(a, b);
    }

    #[test]
    fn test_history_bounded() {
        let mut handler = CaptchaHandler::new(CaptchaConfig::default());
        for i in 0..1500 {
            handler.record_attempt(CaptchaAttempt {
                captcha_type: CaptchaType::Unknown, solver: SolverService::Manual,
                image_hash: [i; 4], solution: None, success: false, latency_ms: 0,
            });
        }
        assert!(handler.solving_history.len() <= 1000);
    }
}
