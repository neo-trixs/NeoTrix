use std::time::Duration;

fn chrome_path() -> String {
    if cfg!(target_os = "macos") { "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".into() }
    else if cfg!(target_os = "windows") { "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".into() }
    else { "google-chrome".into() }
}

fn profile_dir() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_default().join(".neotrix").join("chrome-profile")
}

trait FetchMethod: Send + Sync {
    fn fetch(&self, url: &str, budget_ms: u32) -> Result<String, String>;
}

struct DumpDomFetcher;

impl FetchMethod for DumpDomFetcher {
    fn fetch(&self, url: &str, budget_ms: u32) -> Result<String, String> {
        let budget = format!("--virtual-time-budget={}", budget_ms);
        let dir = format!("--user-data-dir={}", profile_dir().display());
        let out = std::process::Command::new(chrome_path())
            .arg("--headless=new").arg("--disable-gpu").arg("--no-sandbox")
            .arg("--disable-dev-shm-usage").arg("--disable-blink-features=AutomationControlled")
            .arg("--no-first-run").arg("--disable-background-networking")
            .arg("--disable-sync").arg("--mute-audio")
            .arg("--disable-features=ChromeWhatsNewUI")
            .arg("--disable-component-update")
            .arg("--disable-client-side-phishing-detection")
            .arg(&budget).arg("--dump-dom").arg(url).arg(&dir)
            .output().map_err(|e| format!("Chrome: {}", e))?;
        let html = String::from_utf8_lossy(&out.stdout).to_string();
        if html.len() < 80 { return Err("Empty page".into()); }
        Ok(Self::clean(&html))
    }
}

impl DumpDomFetcher {
    fn clean(html: &str) -> String {
        let re = regex::Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>|<[^>]+>").expect("result");
        let text = re.replace_all(html, " ");
        let ws = regex::Regex::new(r"\s+").expect("result");
        let s = ws.replace_all(&text, " ").to_string();
        s.lines().map(|l| l.trim()).filter(|l| l.len() > 10).collect::<Vec<_>>().join("\n").chars().take(20000).collect()
    }
}

pub struct BrowserSession {
    pub fingerprint: super::stealth::Fingerprint,
    pub profile: std::path::PathBuf,
    fetchers: Vec<Box<dyn FetchMethod>>,
}

impl Default for BrowserSession {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserSession {
    pub fn new() -> Self {
        if let Err(e) = std::fs::create_dir_all(profile_dir()) {
            log::warn!("[nt_world_browse] create profile dir: {}", e);
        }
        let fetchers: Vec<Box<dyn FetchMethod>> = vec![Box::new(DumpDomFetcher)];
        Self { fingerprint: super::stealth::Fingerprint::random(), profile: profile_dir(), fetchers }
    }

    pub fn login(&self, url: &str) -> Result<(), String> {
        println!("[nt_world_browse] Opening {} for login (headed)...", url);
        let out = std::process::Command::new(chrome_path())
            .args([&format!("--user-data-dir={}", self.profile.display()),
                "--no-first-run", url])
            .output().map_err(|e| format!("Chrome: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("Chrome exit {}: {}", out.status, err));
        }
        println!("[nt_world_browse] Login session saved to {}", self.profile.display());
        Ok(())
    }

    pub fn fetch(&self, url: &str) -> Result<String, String> {
        std::thread::sleep(Duration::from_millis(200));
        let http = self.fetch_http(url);
        match http { Ok(ref t) if t.len() > 500 => return Ok(t.clone()), _ => {} }
        for fetcher in &self.fetchers {
            match fetcher.fetch(url, 15000) {
                Ok(t) if t.len() > 100 => return Ok(t),
                _ => continue,
            }
        }
        Err("All fetch methods failed".into())
    }

    pub fn fetch_http(&self, url: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("构建 HTTP client: {}", e))?;
        let fp = &self.fingerprint;
        let resp = client.get(url)
            .header("User-Agent", &fp.user_agent)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", &fp.language)
            .header("Sec-Fetch-Dest", "document").header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none").header("Sec-Fetch-User", "?1")
            .send().map_err(|e| format!("HTTP: {}", e))?;
        let html = resp.text().map_err(|e| format!("Read: {}", e))?;
        Ok(DumpDomFetcher::clean(&html))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_clean_removes_script_tags() {
        let html = "<html><head><script>alert('x')</script></head><body><p>Hello world, long enough content for filtering.</p></body></html>";
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(!cleaned.contains("alert"), "script content leaked");
        assert!(cleaned.contains("Hello world"), "visible text missing");
    }

    #[test]
    fn test_dom_clean_removes_style_tags() {
        let html = "<html><head><style>body{color:red}</style></head><body><p>Visible content here that should pass the length filter.</p></body></html>";
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(!cleaned.contains("color:red"), "style content leaked");
        assert!(cleaned.contains("Visible content"), "visible text missing");
    }

    #[test]
    fn test_dom_clean_empty_body() {
        let html = "<html><head></head><body></body></html>";
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(cleaned.is_empty() || cleaned.len() < 10, "expected empty result, got {}", cleaned);
    }

    #[test]
    fn test_dom_clean_removes_html_tags() {
        let html = "<div><p><b>Bold text visible but tags stripped away entirely</b></p></div>";
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(!cleaned.contains("<div>"), "div tag leaked");
        assert!(!cleaned.contains("</b>"), "closing b tag leaked");
    }

    #[test]
    fn test_dom_clean_truncates_long_content() {
        let long = "A".repeat(25000);
        let html = format!("<html><body><p>{}</p></body></html>", long);
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(cleaned.len() <= 20000, "truncation failed: {} > 20000", cleaned.len());
    }

    #[test]
    fn test_chrome_path_non_empty() {
        let path = chrome_path();
        assert!(!path.is_empty(), "chrome path should not be empty");
    }

    #[test]
    fn test_nt_world_browse_session_new_creates_session() {
        let session = BrowserSession::new();
        assert_eq!(session.fetchers.len(), 1, "should have 1 fetcher");
        assert!(!session.fingerprint.user_agent.is_empty(), "fingerprint should be set");
    }

    #[test]
    fn test_nt_world_browse_session_default() {
        let session = BrowserSession::default();
        assert_eq!(session.fetchers.len(), 1);
    }

    #[test]
    fn test_dom_clean_handles_multiline_content() {
        let html = "<html><body><p>Line one has long enough text to survive filtering.</p><p>Line two also has long text for the same reason.</p></body></html>";
        let cleaned = DumpDomFetcher::clean(&html);
        assert!(cleaned.contains("Line one"), "first line content missing");
        assert!(cleaned.contains("Line two"), "second line content missing");
    }
}
