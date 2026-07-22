pub struct IoCapability;

impl IoCapability {
    pub fn needs_external_info(input: &str) -> bool {
        let lower = input.to_lowercase();
        lower.starts_with("visit ")
            || lower.starts_with("fetch ")
            || lower.starts_with("http")
            || lower.starts_with("search ")
            || lower.starts_with("look up ")
            || lower.starts_with("find ")
            || lower.starts_with("what is ")
            || lower.starts_with("who is ")
            || lower.starts_with("how to ")
            || lower.contains("?")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_needs_info() {
        assert!(IoCapability::needs_external_info("visit https://example.com"));
    }

    #[test]
    fn test_greeting_no_info() {
        assert!(!IoCapability::needs_external_info("hello world"));
    }

    #[test]
    fn test_question_needs_info() {
        assert!(IoCapability::needs_external_info("what is rust?"));
    }

    #[test]
    fn test_command_no_info() {
        assert!(!IoCapability::needs_external_info("run cargo build"));
    }

    #[test]
    fn test_search_needs_info() {
        assert!(IoCapability::needs_external_info("search for quantum computing papers"));
    }

    #[test]
    fn test_look_up() {
        assert!(IoCapability::needs_external_info("look up the weather in Tokyo"));
    }

    #[test]
    fn test_find() {
        assert!(IoCapability::needs_external_info("find the nearest coffee shop"));
    }

    #[test]
    fn test_how_to() {
        assert!(IoCapability::needs_external_info("how to install rust"));
    }
}
