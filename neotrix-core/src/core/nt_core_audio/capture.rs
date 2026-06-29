use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_SAMPLE_RATE: u32 = 16000;
const DEFAULT_CAPTURE_SECS: f64 = 3.0;

pub struct AudioCapture {
    pub tmp_dir: PathBuf,
    pub sample_rate: u32,
    pub duration_secs: f64,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct CapturedAudio {
    pub raw_wav: Vec<u8>,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub duration: Duration,
    pub timestamp: SystemTime,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            tmp_dir: std::env::temp_dir().join("neotrix_audio"),
            sample_rate: DEFAULT_SAMPLE_RATE,
            duration_secs: DEFAULT_CAPTURE_SECS,
            active: false,
        }
    }

    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn capture(&self) -> Option<CapturedAudio> {
        if !self.active {
            return None;
        }
        std::fs::create_dir_all(&self.tmp_dir).ok()?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let out_path = self.tmp_dir.join(format!("audio_{}.wav", ts));

        let _captured = self.run_capture(&out_path)?;
        let raw_wav = std::fs::read(&out_path).ok()?;
        let samples = wav_to_f32(&raw_wav);
        let _ = std::fs::remove_file(&out_path);

        Some(CapturedAudio {
            raw_wav,
            samples,
            sample_rate: self.sample_rate,
            duration: Duration::from_secs_f64(self.duration_secs),
            timestamp: SystemTime::now(),
        })
    }

    fn run_capture(&self, out_path: &PathBuf) -> Option<()> {
        let dur_ms = (self.duration_secs * 1000.0) as u32;

        // Try rec (SoX) first
        let ok = Command::new("rec")
            .arg("-q")
            .arg("--rate")
            .arg(self.sample_rate.to_string())
            .arg("--bits")
            .arg("16")
            .arg("--channels")
            .arg("1")
            .arg(out_path)
            .arg("trim")
            .arg("0")
            .arg(dur_ms.to_string())
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false);

        if ok {
            return Some(());
        }

        // Fallback: ffmpeg on macOS
        if cfg!(target_os = "macos") {
            let ok = Command::new("ffmpeg")
                .arg("-f")
                .arg("avfoundation")
                .arg("-i")
                .arg(":0")
                .arg("-ar")
                .arg(self.sample_rate.to_string())
                .arg("-ac")
                .arg("1")
                .arg("-t")
                .arg(self.duration_secs.to_string())
                .arg("-y")
                .arg(out_path)
                .status()
                .ok()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return Some(());
            }
        }

        // Fallback: arecord on Linux
        if cfg!(target_os = "linux") {
            let ok = Command::new("arecord")
                .arg("-r")
                .arg(self.sample_rate.to_string())
                .arg("-c")
                .arg("1")
                .arg("-f")
                .arg("S16_LE")
                .arg("-d")
                .arg((self.duration_secs as i64).to_string())
                .arg(out_path)
                .status()
                .ok()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                return Some(());
            }
        }

        None
    }
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

fn wav_to_f32(wav_bytes: &[u8]) -> Vec<f32> {
    if wav_bytes.len() < 44 {
        return vec![];
    }
    let data = &wav_bytes[44..];
    let sample_count = data.len() / 2;
    let mut samples = Vec::with_capacity(sample_count);
    for chunk in data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample as f32 / 32768.0);
    }
    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_capture_defaults() {
        let cap = AudioCapture::new();
        assert!(!cap.active);
        assert_eq!(cap.sample_rate, 16000);
        assert!((cap.duration_secs - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_activate_deactivate() {
        let mut cap = AudioCapture::new();
        assert!(!cap.is_active());
        cap.activate();
        assert!(cap.is_active());
        cap.deactivate();
        assert!(!cap.is_active());
    }

    #[test]
    fn test_capture_returns_none_when_inactive() {
        let cap = AudioCapture::new();
        assert!(cap.capture().is_none());
    }

    #[test]
    fn test_with_duration() {
        let cap = AudioCapture::new().with_duration(5.0);
        assert!((cap.duration_secs - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_with_sample_rate() {
        let cap = AudioCapture::new().with_sample_rate(44100);
        assert_eq!(cap.sample_rate, 44100);
    }

    #[test]
    fn test_wav_to_f32_empty() {
        let samples = wav_to_f32(&[]);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_wav_to_f32_short_header() {
        let samples = wav_to_f32(&[0u8; 20]);
        assert!(samples.is_empty());
    }

    #[test]
    fn test_wav_to_f32_basic() {
        // Minimal WAV-like header + two 16-bit samples
        let mut wav = vec![0u8; 48];
        // Write two samples at offset 44: 0x7FFF (max positive), 0x8001 (near min negative)
        wav[44] = 0xFF;
        wav[45] = 0x7F;
        wav[46] = 0x01;
        wav[47] = 0x80;
        let samples = wav_to_f32(&wav);
        assert_eq!(samples.len(), 2);
        assert!((samples[0] - 0.999969).abs() < 0.001);
        assert!((samples[1] - (-0.999969)).abs() < 0.001);
    }

    #[test]
    fn test_wav_to_f32_odd_length() {
        let mut wav = vec![0u8; 45];
        wav[44] = 0x00;
        // 45 bytes = 44 header + 1 data byte (not enough for a full sample)
        let samples = wav_to_f32(&wav);
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn test_default_impl() {
        let cap = AudioCapture::default();
        assert!(!cap.active);
    }
}
