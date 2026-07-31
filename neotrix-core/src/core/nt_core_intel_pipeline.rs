#[derive(Debug, Clone, PartialEq)]
pub enum IntelRequest {
    FetchUrl(String),
    AskQuestion(String),
    ExecuteCode(String),
    RunCommand(String),
}

#[derive(Debug, Clone)]
pub struct IntelPipeline;

impl IntelPipeline {
    pub fn detect_intent(input: &str) -> IntelRequest {
        let lower = input.to_lowercase();
        if lower.starts_with("visit ") || lower.starts_with("fetch ") || lower.starts_with("http") {
            let target = input.split_once(' ').map(|x| x.1).unwrap_or(input);
            return IntelRequest::FetchUrl(target.to_string());
        }
        if lower.starts_with("run ") || lower.starts_with("exec ") {
            let cmd = input.split_once(' ').map(|x| x.1).unwrap_or(input);
            return IntelRequest::ExecuteCode(cmd.to_string());
        }
        if lower.starts_with("!") {
            return IntelRequest::RunCommand(input[1..].to_string());
        }
        IntelRequest::AskQuestion(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_url() {
        assert!(matches!(
            IntelPipeline::detect_intent("visit https://example.com"),
            IntelRequest::FetchUrl(_)
        ));
    }

    #[test]
    fn test_ask_question() {
        assert!(matches!(
            IntelPipeline::detect_intent("what is the meaning of life?"),
            IntelRequest::AskQuestion(_)
        ));
    }

    #[test]
    fn test_execute_code() {
        assert!(matches!(
            IntelPipeline::detect_intent("run cargo build"),
            IntelRequest::ExecuteCode(_)
        ));
    }

    #[test]
    fn test_run_command() {
        assert!(matches!(
            IntelPipeline::detect_intent("!ls -la"),
            IntelRequest::RunCommand(_)
        ));
    }

    #[test]
    fn test_http_prefix() {
        assert!(matches!(
            IntelPipeline::detect_intent("https://github.com"),
            IntelRequest::FetchUrl(_)
        ));
    }

    #[test]
    fn test_fetch_prefix() {
        assert!(matches!(
            IntelPipeline::detect_intent("fetch http://test.com/data"),
            IntelRequest::FetchUrl(_)
        ));
    }

    #[test]
    fn test_empty_input() {
        assert!(matches!(
            IntelPipeline::detect_intent(""),
            IntelRequest::AskQuestion(_)
        ));
    }
}
