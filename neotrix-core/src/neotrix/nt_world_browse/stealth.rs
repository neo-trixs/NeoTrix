use rand::Rng;

use crate::core::nt_core_agent::UserAgentRotation;

const PLATFORMS: &[&str] = &["Win32", "MacIntel", "Linux x86_64"];
const LANGUAGES: &[&str] = &["en-US,en;q=0.9", "en-GB,en;q=0.9", "en-CA,en;q=0.8"];

pub struct Fingerprint {
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub viewport: (u32, u32),
    pub timezone: String,
}

impl Fingerprint {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let viewports = [
            (1440, 900),
            (1920, 1080),
            (1366, 768),
            (1680, 1050),
            (2560, 1440),
        ];
        Self {
            user_agent: UserAgentRotation::default().next().to_string(),
            platform: PLATFORMS[rng.gen_range(0..PLATFORMS.len())].to_string(),
            language: LANGUAGES[rng.gen_range(0..LANGUAGES.len())].to_string(),
            viewport: viewports[rng.gen_range(0..viewports.len())],
            timezone: "America/New_York".to_string(),
        }
    }

    pub fn chrome_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        args.push("--disable-blink-features=AutomationControlled".into());
        args.push(format!("--user-agent={}", self.user_agent));
        args.push(format!(
            "--window-size={},{}",
            self.viewport.0, self.viewport.1
        ));
        args.push("--disable-features=ChromeWhatsNewUI,ChromeLabs,ChromeMenu".into());
        args.push("--no-first-run".into());
        args.push("--no-default-nt_world_browse-check".into());
        args.push("--disable-background-networking".into());
        args.push("--disable-sync".into());
        args.push("--disable-translate".into());
        args.push("--disable-nt_io_notifys".into());
        args.push("--hide-scrollbars".into());
        args.push("--mute-audio".into());
        args.push("--disable-client-side-phishing-detection".into());
        args.push("--disable-component-update".into());
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_random_basic() {
        let fp = Fingerprint::random();
        assert!(!fp.user_agent.is_empty(), "user_agent empty");
        assert!(!fp.platform.is_empty(), "platform empty");
        assert!(!fp.language.is_empty(), "language empty");
        assert!(fp.viewport.0 > 0 && fp.viewport.1 > 0, "viewport zero");
        assert_eq!(fp.timezone, "America/New_York");
    }

    #[test]
    fn test_fingerprint_random_ua_valid() {
        for _ in 0..100 {
            let fp = Fingerprint::random();
            assert!(
                fp.user_agent.starts_with("Mozilla/"),
                "unexpected UA: {}",
                fp.user_agent
            );
            assert!(!fp.user_agent.is_empty());
        }
    }

    #[test]
    fn test_fingerprint_random_platform_from_pool() {
        let pool: Vec<String> = PLATFORMS.iter().map(|s| s.to_string()).collect();
        for _ in 0..50 {
            let fp = Fingerprint::random();
            assert!(
                pool.contains(&fp.platform),
                "unexpected platform: {}",
                fp.platform
            );
        }
    }

    #[test]
    fn test_chrome_args_has_expected_flags() {
        let fp = Fingerprint::random();
        let args = fp.chrome_args();
        assert!(args.iter().any(|a| a.starts_with("--user-agent=")));
        assert!(args.iter().any(|a| a.starts_with("--window-size=")));
        assert!(args.contains(&"--disable-blink-features=AutomationControlled".to_string()));
    }

    #[test]
    fn test_chrome_args_count() {
        let fp = Fingerprint::random();
        assert_eq!(fp.chrome_args().len(), 14);
    }

    #[test]
    fn test_fingerprints_vary() {
        let fp1 = Fingerprint::random();
        let fp2 = Fingerprint::random();
        let same = fp1.user_agent == fp2.user_agent
            && fp1.platform == fp2.platform
            && fp1.language == fp2.language;
        assert!(
            !same,
            "highly unlikely that 2 random fingerprints are identical"
        );
    }
}
