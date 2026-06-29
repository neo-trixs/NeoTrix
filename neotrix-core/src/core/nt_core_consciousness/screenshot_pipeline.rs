#[derive(Debug, Clone)]
pub struct ScreenshotCaptureConfig {
    pub enabled: bool,
}

impl Default for ScreenshotCaptureConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenshotPipeline {
    config: ScreenshotCaptureConfig,
}

impl ScreenshotPipeline {
    pub fn new(config: ScreenshotCaptureConfig) -> Self {
        Self { config }
    }

    pub fn capture(&mut self, _url: &str) -> Result<Vec<u8>, String> {
        if !self.config.enabled {
            return Ok(vec![]);
        }
        let tmp_dir = std::env::temp_dir().join("neotrix_screencap");
        std::fs::create_dir_all(&tmp_dir)
            .map_err(|e| format!("failed to create tmp dir: {}", e))?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let out_path = tmp_dir.join(format!("screen_{}.png", ts));
        let status = std::process::Command::new("screencapture")
            .arg("-x")
            .arg("-C")
            .arg(&out_path)
            .status()
            .map_err(|e| format!("screencapture exec failed: {}", e))?;
        if !status.success() {
            return Err(format!("screencapture exited with: {:?}", status.code()));
        }
        let raw =
            std::fs::read(&out_path).map_err(|e| format!("failed to read screenshot: {}", e))?;
        let _ = std::fs::remove_file(&out_path);
        Ok(raw)
    }
}
