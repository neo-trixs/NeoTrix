//! Language and script detection for routing

use crate::types::*;
use crate::error::Result;

/// Language router for multi-backend models
pub struct Router {
    english: Box<dyn DecisionBackend>,
    multilingual: Box<dyn DecisionBackend>,
    typed: Box<dyn DecisionBackend>,
}

/// Decision backend trait
pub trait DecisionBackend {
    fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult>;
}

impl Router {
    /// Create a new router
    pub fn new(
        english: Box<dyn DecisionBackend>,
        multilingual: Box<dyn DecisionBackend>,
        typed: Box<dyn DecisionBackend>,
    ) -> Self {
        Self {
            english,
            multilingual,
            typed,
        }
    }
    
    /// Route and evaluate
    pub fn predict(&self, state: &State, questions: &[Question]) -> Result<EvaluationResult> {
        let script = detect_script(&state.content);
        
        let backend = match script {
            Script::Latin => &self.english,
            Script::Devanagari | Script::Arabic | Script::CJK => &self.multilingual,
            Script::TypedDecisions => &self.typed,
            Script::Unknown => &self.english,
        };
        
        backend.predict(state, questions)
    }
    
    /// Get route decision
    pub fn route(&self, state: &State) -> RouteDecision {
        let script = detect_script(&state.content);
        
        match script {
            Script::Latin => RouteDecision {
                model: "english".to_string(),
                reason: "Latin script detected".to_string(),
            },
            Script::Devanagari => RouteDecision {
                model: "multilingual".to_string(),
                reason: "Devanagari script detected".to_string(),
            },
            Script::Arabic => RouteDecision {
                model: "multilingual".to_string(),
                reason: "Arabic script detected".to_string(),
            },
            Script::CJK => RouteDecision {
                model: "multilingual".to_string(),
                reason: "CJK script detected".to_string(),
            },
            Script::TypedDecisions => RouteDecision {
                model: "typed".to_string(),
                reason: "Typed decisions workflow".to_string(),
            },
            Script::Unknown => RouteDecision {
                model: "english".to_string(),
                reason: "Unknown script, defaulting to English".to_string(),
            },
        }
    }
}

/// Route decision
pub struct RouteDecision {
    pub model: String,
    pub reason: String,
}

/// Script type
#[derive(Debug, Clone, Copy)]
pub enum Script {
    Latin,
    Devanagari,
    Arabic,
    CJK,
    TypedDecisions,
    Unknown,
}

/// Detect script from text
pub fn detect_script(text: &str) -> Script {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len() as f64;
    
    if total == 0.0 {
        return Script::Unknown;
    }
    
    let latin = chars.iter().filter(|c| is_latin(**c)).count() as f64 / total;
    let devanagari = chars.iter().filter(|c| is_devanagari(**c)).count() as f64 / total;
    let arabic = chars.iter().filter(|c| is_arabic(**c)).count() as f64 / total;
    let cjk = chars.iter().filter(|c| is_cjk(**c)).count() as f64 / total;
    
    if latin > 0.5 {
        Script::Latin
    } else if devanagari > 0.3 {
        Script::Devanagari
    } else if arabic > 0.3 {
        Script::Arabic
    } else if cjk > 0.3 {
        Script::CJK
    } else {
        Script::Unknown
    }
}

fn is_latin(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn is_devanagari(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c)
}

fn is_arabic(c: char) -> bool {
    ('\u{0600}'..='\u{06FF}').contains(&c)
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
        || ('\u{3400}'..='\u{4DBF}').contains(&c)
        || ('\u{F900}'..='\u{FAFF}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Script Detection ───────────────────────────────────────

    #[test]
    fn test_detect_latin() {
        assert!(matches!(detect_script("Hello world"), Script::Latin));
        assert!(matches!(detect_script("The server is down"), Script::Latin));
    }

    #[test]
    fn test_detect_cjk() {
        assert!(matches!(detect_script("服务器状态异常"), Script::CJK));
        assert!(matches!(detect_script("数据库连接失败"), Script::CJK));
    }

    #[test]
    fn test_detect_devanagari() {
        assert!(matches!(detect_script("सर्वर स्थिति"), Script::Devanagari));
    }

    #[test]
    fn test_detect_arabic() {
        assert!(matches!(detect_script("حالة الخادم"), Script::Arabic));
    }

    #[test]
    fn test_detect_unknown_empty() {
        assert!(matches!(detect_script(""), Script::Unknown));
    }

    #[test]
    fn test_detect_unknown_short() {
        // Very short text that doesn't clearly belong to any script
        assert!(matches!(detect_script("123"), Script::Unknown));
    }

    #[test]
    fn test_detect_mixed_latin_dominant() {
        // Mostly Latin with some numbers/punctuation
        assert!(matches!(detect_script("Error 500: server timeout"), Script::Latin));
    }

    #[test]
    fn test_detect_mixed_cjk_dominant() {
        // Mostly CJK with some Latin
        assert!(matches!(detect_script("服务器返回500错误"), Script::CJK));
    }

    // ── Route Decision ─────────────────────────────────────────

    #[test]
    fn test_route_latin() {
        let router = MockRouter::create();
        let state: State = "The server is down".into();
        let decision = router.route(&state);
        assert_eq!(decision.model, "english");
    }

    #[test]
    fn test_route_cjk() {
        let router = MockRouter::create();
        let state: State = "服务器状态异常".into();
        let decision = router.route(&state);
        assert_eq!(decision.model, "multilingual");
    }

    // ── Character Detection Helpers ────────────────────────────

    #[test]
    fn test_is_latin() {
        assert!(is_latin('a'));
        assert!(is_latin('Z'));
        assert!(!is_latin('5')); // digits are not in a-z / A-Z
        assert!(!is_latin('中'));
        assert!(!is_latin(' '));
    }

    #[test]
    fn test_is_cjk() {
        assert!(is_cjk('中'));
        assert!(is_cjk('国'));
        assert!(!is_latin('中'));
        assert!(!is_cjk('a'));
    }

    #[test]
    fn test_is_devanagari() {
        assert!(is_devanagari('अ'));
        assert!(is_devanagari('क'));
        assert!(!is_devanagari('a'));
    }

    #[test]
    fn test_is_arabic() {
        assert!(is_arabic('ا'));
        assert!(is_arabic('ب'));
        assert!(!is_arabic('a'));
    }

    // ── Mock Backend ───────────────────────────────────────────

    struct MockBackend {
        name: String,
    }

    impl DecisionBackend for MockBackend {
        fn predict(&self, _state: &State, _questions: &[Question]) -> Result<EvaluationResult> {
            let mut answers = std::collections::HashMap::new();
            answers.insert("mock_q".to_string(), Answer::Noul(NoulAnswer {
                value: true,
                probability: 0.5,
                status: DecisionStatus::Selected,
            }));
            Ok(EvaluationResult {
                answers,
                model: Some(self.name.clone()),
                usage: None,
            })
        }
    }

    struct MockRouter;

    impl MockRouter {
        fn create() -> Router {
            Router {
                english: Box::new(MockBackend { name: "english".to_string() }),
                multilingual: Box::new(MockBackend { name: "multilingual".to_string() }),
                typed: Box::new(MockBackend { name: "typed".to_string() }),
            }
        }
    }

    #[test]
    fn test_router_predict_routes_to_english() {
        let router = MockRouter::create();
        let state: State = "Test state".into();
        let questions = vec![Question::noul("q1", "test?")];
        let result = router.predict(&state, &questions).unwrap();
        assert_eq!(result.model.as_deref(), Some("english"));
    }

    #[test]
    fn test_router_predict_routes_to_multilingual() {
        let router = MockRouter::create();
        let state: State = "服务器状态".into();
        let questions = vec![Question::noul("q1", "test?")];
        let result = router.predict(&state, &questions).unwrap();
        assert_eq!(result.model.as_deref(), Some("multilingual"));
    }
}
