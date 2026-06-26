pub fn socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.neotrix/neotrix-evolution.sock", home)
}

pub fn format_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_contains_sock_name() {
        let path = socket_path();
        assert!(path.contains("neotrix-evolution.sock"));
        assert!(path.contains(".neotrix"));
    }

    #[test]
    fn test_socket_path_uses_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = socket_path();
        assert!(path.starts_with(&home));
    }

    #[test]
    fn test_socket_path_fallback_on_missing_home() {
        let original_home = std::env::var("HOME").ok();
        std::env::remove_var("HOME");

        let path = socket_path();
        assert!(path.starts_with("/tmp/"));

        if let Some(h) = original_home {
            std::env::set_var("HOME", h);
        }
    }

    #[test]
    fn test_format_timestamp_format() {
        let ts = format_timestamp();
        assert_eq!(ts.len(), 8, "timestamp must be HH:MM:SS (8 chars)");
        for (i, ch) in ts.chars().enumerate() {
            if i == 2 || i == 5 {
                assert_eq!(ch, ':', "timestamp must have ':' at positions 2 and 5");
            } else {
                assert!(ch.is_ascii_digit(), "timestamp chars must be digits or ':'");
            }
        }
    }

    #[test]
    fn test_format_timestamp_padded() {
        let ts = format_timestamp();
        let parts: Vec<&str> = ts.split(':').collect();
        assert_eq!(parts.len(), 3);
        // Hours 00-23, minutes 00-59, seconds 00-59
        let h: u8 = parts[0].parse().expect("hours must be u8");
        let m: u8 = parts[1].parse().expect("minutes must be u8");
        let s: u8 = parts[2].parse().expect("seconds must be u8");
        assert!(h < 24, "hours must be 00-23, got {}", h);
        assert!(m < 60, "minutes must be 00-59, got {}", m);
        assert!(s < 60, "seconds must be 00-59, got {}", s);
    }
}
