//! Shared time utilities — eliminates now_secs() duplication across 13+ modules

/// Current timestamp in seconds (u64)
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Current timestamp in seconds (i64) — for legacy APIs that use signed
pub fn now_secs_i64() -> i64 {
    now_secs() as i64
}

/// Current timestamp in milliseconds
pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_secs_positive() {
        assert!(now_secs() > 1_700_000_000); // After 2023
    }

    #[test]
    fn test_now_secs_i64() {
        assert!(now_secs_i64() > 1_700_000_000);
    }

    #[test]
    fn test_now_millis() {
        assert!(now_millis() > 1_700_000_000_000);
    }
}
