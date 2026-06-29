use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn unix_now_nanos() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
}

pub fn unix_now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

pub fn unix_now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_now_returns_reasonable_value() {
        let now = unix_now();
        assert!(now > 1_700_000_000);
    }

    #[test]
    fn test_unix_now_functions_consistent() {
        let secs = unix_now_secs();
        let ms = unix_now_ms();
        let nanos = unix_now_nanos();
        assert!(ms >= secs * 1000);
        assert!(nanos >= secs * 1_000_000_000);
    }
}
