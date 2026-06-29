use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PlatformRateLimit {
    pub max_calls: u32,
    pub window_secs: u64,
    pub min_interval_ms: u64,
}

pub struct RateLimiter {
    limits: HashMap<String, PlatformRateLimit>,
    windows: Mutex<HashMap<String, Vec<Instant>>>,
    last_call: Mutex<HashMap<String, Instant>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        let limits = HashMap::from([
            (
                "youtube".into(),
                PlatformRateLimit {
                    max_calls: 100,
                    window_secs: 60,
                    min_interval_ms: 200,
                },
            ),
            (
                "tiktok".into(),
                PlatformRateLimit {
                    max_calls: 50,
                    window_secs: 60,
                    min_interval_ms: 500,
                },
            ),
            (
                "instagram".into(),
                PlatformRateLimit {
                    max_calls: 30,
                    window_secs: 60,
                    min_interval_ms: 1000,
                },
            ),
            (
                "twitter".into(),
                PlatformRateLimit {
                    max_calls: 45,
                    window_secs: 900,
                    min_interval_ms: 500,
                },
            ),
            (
                "reddit".into(),
                PlatformRateLimit {
                    max_calls: 60,
                    window_secs: 60,
                    min_interval_ms: 200,
                },
            ),
            (
                "bilibili".into(),
                PlatformRateLimit {
                    max_calls: 30,
                    window_secs: 60,
                    min_interval_ms: 500,
                },
            ),
        ]);

        RateLimiter {
            limits,
            windows: Mutex::new(HashMap::new()),
            last_call: Mutex::new(HashMap::new()),
        }
    }

    pub fn wait_if_needed(&self, platform: &str) -> Duration {
        let limit = match self.limits.get(platform) {
            Some(l) => l,
            None => return Duration::ZERO,
        };

        let last_call = self.last_call.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(last) = last_call.get(platform) {
            let elapsed = last.elapsed();
            let min_interval = Duration::from_millis(limit.min_interval_ms);
            if elapsed < min_interval {
                return min_interval - elapsed;
            }
        }

        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        let calls = windows.entry(platform.to_string()).or_default();
        let now = Instant::now();
        let window = Duration::from_secs(limit.window_secs);

        calls.retain(|t| now.duration_since(*t) < window);

        if calls.len() >= limit.max_calls as usize {
            if let Some(oldest) = calls.first() {
                let wait = window - now.duration_since(*oldest);
                return wait + Duration::from_millis(100);
            }
        }

        Duration::ZERO
    }

    const MAX_PLATFORMS: usize = 1000;

    pub fn record_call(&self, platform: &str) {
        let mut last_call = self.last_call.lock().unwrap_or_else(|e| e.into_inner());
        last_call.insert(platform.to_string(), Instant::now());
        if last_call.len() > Self::MAX_PLATFORMS {
            if let Some(oldest) = last_call.iter().min_by_key(|(_, v)| *v).map(|(k, _)| k.clone()) {
                last_call.remove(&oldest);
            }
        }

        let mut windows = self.windows.lock().unwrap_or_else(|e| e.into_inner());
        let calls = windows.entry(platform.to_string()).or_default();
        let now = Instant::now();
        if let Some(limit) = self.limits.get(platform) {
            let window = Duration::from_secs(limit.window_secs);
            calls.retain(|t| now.duration_since(*t) < window);
        }
        calls.push(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_new_has_default_limits() {
        let rl = RateLimiter::new();
        assert!(rl.limits.contains_key("youtube"));
        assert!(rl.limits.contains_key("tiktok"));
        assert_eq!(rl.limits.len(), 6);
    }

    #[test]
    fn test_rate_limiter_wait_if_needed_unknown_platform_zero() {
        let rl = RateLimiter::new();
        let wait = rl.wait_if_needed("unknown_platform");
        assert_eq!(wait, Duration::ZERO);
    }

    #[test]
    fn test_rate_limiter_wait_if_needed_first_call_zero() {
        let rl = RateLimiter::new();
        let wait = rl.wait_if_needed("youtube");
        assert_eq!(wait, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_rate_limiter_record_call_then_wait() {
        let rl = RateLimiter::new();
        rl.record_call("youtube");
        tokio::time::sleep(Duration::from_millis(250)).await;
        let wait = rl.wait_if_needed("youtube");
        assert_eq!(wait, Duration::ZERO);
    }

    #[test]
    fn test_rate_limiter_youtube_limit() {
        let rl = RateLimiter::new();
        let limit = rl.limits.get("youtube").unwrap();
        assert_eq!(limit.max_calls, 100);
        assert_eq!(limit.window_secs, 60);
    }

    #[test]
    fn test_rate_limiter_tiktok_min_interval() {
        let rl = RateLimiter::new();
        let limit = rl.limits.get("tiktok").unwrap();
        assert_eq!(limit.min_interval_ms, 500);
    }

    #[test]
    fn test_rate_limiter_default_impl() {
        let rl: RateLimiter = Default::default();
        assert!(rl.limits.contains_key("twitter"));
    }
}
