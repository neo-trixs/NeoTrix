use rand::Rng;
use std::time::Duration;

pub struct Humanizer {
    pub typing_speed: (u64, u64),
    pub scroll_speed: (u64, u64),
    pub mouse_delay: (u64, u64),
    pub think_time: (u64, u64),
}

impl Humanizer {
    pub fn new() -> Self {
        Self {
            typing_speed: (40, 120),
            scroll_speed: (100, 300),
            mouse_delay: (50, 200),
            think_time: (500, 2000),
        }
    }

    pub fn delay_ms(&self, range: (u64, u64)) -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(range.0..range.1)
    }

    pub fn random_delay(&self) -> Duration {
        Duration::from_millis(self.delay_ms((300, 1500)))
    }

    pub fn think_delay(&self) -> Duration {
        Duration::from_millis(self.delay_ms(self.think_time))
    }

    pub fn typing_delay(&self) -> Duration {
        Duration::from_millis(self.delay_ms(self.typing_speed))
    }

    pub fn jitter_ms(&self, base_ms: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..(base_ms / 3).max(10));
        base_ms + jitter
    }

    pub fn simulate_page_read(&self, text_len: usize) -> Duration {
        let reading_time = (text_len as f64 / 200.0).ceil() as u64;
        let mut rng = rand::thread_rng();
        let think = rng.gen_range(300..1500);
        Duration::from_millis(reading_time * 1000 + think)
    }
}

impl Default for Humanizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanizer_new_defaults() {
        let h = Humanizer::new();
        assert_eq!(h.typing_speed, (40, 120));
        assert_eq!(h.scroll_speed, (100, 300));
        assert_eq!(h.mouse_delay, (50, 200));
        assert_eq!(h.think_time, (500, 2000));
    }

    #[test]
    fn test_humanizer_default_equals_new() {
        assert_eq!(Humanizer::default().typing_speed, Humanizer::new().typing_speed);
    }

    #[test]
    fn test_delay_ms_in_range() {
        let h = Humanizer::new();
        for _ in 0..30 {
            let d = h.delay_ms((100, 200));
            assert!(d >= 100 && d < 200, "delay {} out of [100,200)", d);
        }
    }

    #[test]
    fn test_random_delay_bounds() {
        let h = Humanizer::new();
        for _ in 0..30 {
            let d = h.random_delay();
            let ms = d.as_millis();
            assert!(ms >= 300 && ms < 1500, "random_delay {}ms out of range", ms);
        }
    }

    #[test]
    fn test_think_delay_bounds() {
        let h = Humanizer::new();
        for _ in 0..30 {
            let d = h.think_delay();
            let ms = d.as_millis();
            assert!(ms >= 500 && ms < 2000, "think_delay {}ms out of range", ms);
        }
    }

    #[test]
    fn test_typing_delay_bounds() {
        let h = Humanizer::new();
        for _ in 0..30 {
            let d = h.typing_delay();
            let ms = d.as_millis();
            assert!(ms >= 40 && ms < 120, "typing_delay {}ms out of range", ms);
        }
    }

    #[test]
    fn test_jitter_ms_increases_base() {
        let h = Humanizer::new();
        for base in [50u64, 100, 500, 1000] {
            let j = h.jitter_ms(base);
            assert!(j >= base, "jitter {} < base {}", j, base);
            let max_jitter = (base / 3).max(10);
            assert!(j <= base + max_jitter, "jitter {} too high for base {} (max_jitter {})", j, base, max_jitter);
        }
    }

    #[test]
    fn test_simulate_page_read_zero() {
        let h = Humanizer::new();
        let d = h.simulate_page_read(0);
        assert!(d.as_millis() >= 300 && d.as_millis() < 1500, "zero text read {}ms", d.as_millis());
    }

    #[test]
    fn test_simulate_page_read_long_text() {
        let h = Humanizer::new();
        let d = h.simulate_page_read(2000);
        assert!(d.as_millis() >= 10000, "long text read {}ms too short", d.as_millis());
    }
}
