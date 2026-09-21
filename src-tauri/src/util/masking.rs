/// Mask an API key for safe display.
/// Shows first 4 and last 4 chars, replaces middle with asterisks.
/// Keys shorter than 9 chars are fully masked as "****".
/// `env:` prefixed keys and sentinel values (`no-key`, empty) are returned as-is.
pub fn mask_api_key(key: &str) -> String {
    if key.starts_with("env:") || key == "no-key" || key.is_empty() {
        key.to_string()
    } else if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "****".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_prefix_passthrough() {
        assert_eq!(mask_api_key("env:MY_SECRET"), "env:MY_SECRET");
    }

    #[test]
    fn no_key_passthrough() {
        assert_eq!(mask_api_key("no-key"), "no-key");
    }

    #[test]
    fn empty_passthrough() {
        assert_eq!(mask_api_key(""), "");
    }

    #[test]
    fn long_key_shows_first4_last4() {
        let key = "sk-1234567890abcdef";
        let masked = mask_api_key(key);
        assert!(masked.starts_with("sk-1"));
        assert!(masked.ends_with("cdef"));
        assert!(masked.contains("..."));
    }

    #[test]
    fn short_key_fully_masked() {
        assert_eq!(mask_api_key("abc"), "****");
        assert_eq!(mask_api_key("12345678"), "****");
    }

    #[test]
    fn exactly_8_chars_fully_masked() {
        assert_eq!(mask_api_key("12345678"), "****");
    }

    #[test]
    fn nine_chars_gets_masked() {
        assert_eq!(mask_api_key("123456789"), "1234...6789");
    }

    #[test]
    fn preserves_first4_last4() {
        let key = "abcdefghij";
        let masked = mask_api_key(key);
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("ghij"));
    }
}
