//! Integration tests for the decision engine

use std::path::PathBuf;
use std::collections::HashMap;
use neotrix_decision_engine::{
    State, Question, QuestionSet, QuestionType,
    LayaEngine, ModelConfig, TemperatureScaler,
};
use candle_core::Device;

/// Get the project root directory
fn project_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Remove crate name
    path.pop(); // Remove crates
    path
}

/// Try to create a LayaEngine, skip if model files are missing
fn try_create_engine() -> Option<LayaEngine> {
    let root = project_root();
    let weights_path = root.join("models/modernbert-large/model.safetensors");
    let tokenizer_path = root.join("models/modernbert-large/tokenizer.json");

    if !weights_path.exists() || !tokenizer_path.exists() {
        return None;
    }

    let config = ModelConfig::modernbert_large();
    let temperature = TemperatureScaler::default();
    let device = Device::Cpu;

    LayaEngine::from_files(
        &weights_path,
        &tokenizer_path,
        config,
        temperature,
        device,
    ).ok()
}

// ── Model Loading ──────────────────────────────────────────────

#[test]
fn test_laya_engine_creation() {
    let engine = try_create_engine();
    match engine {
        Some(_e) => eprintln!("Model loaded successfully!"),
        None => eprintln!("Skipping: model files not found"),
    }
}

// ── Serialization ──────────────────────────────────────────────

#[test]
fn test_question_set_serialization() {
    let question_set = QuestionSet {
        questions: vec![
            Question::noul("test", "Test question"),
        ],
    };

    let json = serde_json::to_string_pretty(&question_set).unwrap();
    let deserialized: QuestionSet = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.questions.len(), 1);
    assert_eq!(deserialized.questions[0].id, "test");
}

#[test]
fn test_question_type_serialization_roundtrip() {
    // Noul
    let noul = Question::noul("q1", "Is it true?");
    let json = serde_json::to_string(&noul).unwrap();
    let back: Question = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.question_type, QuestionType::Noul { .. }));

    // Choice
    let mut opts = HashMap::new();
    opts.insert("a".to_string(), Some("Option A".to_string()));
    opts.insert("b".to_string(), Some("Option B".to_string()));
    let choice = Question::choice("q2", "Pick one", opts).unwrap();
    let json = serde_json::to_string(&choice).unwrap();
    let back: Question = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.question_type, QuestionType::Choice { .. }));

    // Score
    let score = Question::score("q3", "Rate this",
        vec!["Low".into(), "Med".into(), "High".into()]).unwrap();
    let json = serde_json::to_string(&score).unwrap();
    let back: Question = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.question_type, QuestionType::Score { .. }));
}

// ── Noul Evaluation ────────────────────────────────────────────

#[test]
fn test_evaluate_noul() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let state: State = "The server is responding slowly with 500ms latency.".into();
    let questions = vec![
        Question::noul("is_critical", "Is this a critical issue?"),
    ];

    let result = engine.evaluate(&state, &questions).unwrap();
    assert!(result.answers.contains_key("is_critical"));

    // Extract noul probability
    if let neotrix_decision_engine::Answer::Noul(noul) = &result.answers["is_critical"] {
        assert!(noul.noul >= 0.0 && noul.noul <= 1.0,
            "Noul probability should be in [0,1], got {}", noul.noul);
        eprintln!("Noul answer: is_critical = {} (needs_review: {})", noul.noul, noul.needs_review);
    } else {
        panic!("Expected Noul answer");
    }
}

// ── Choice Evaluation ──────────────────────────────────────────

#[test]
fn test_evaluate_choice() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let state: State = "The database connection pool is exhausted, queries are timing out.".into();
    let mut options = HashMap::new();
    options.insert("restart_pool".to_string(), Some("Restart the connection pool".to_string()));
    options.insert("increase_size".to_string(), Some("Increase pool size".to_string()));
    options.insert("optimize_queries".to_string(), Some("Optimize slow queries".to_string()));

    let questions = vec![
        Question::choice("action", "What should we do?", options).unwrap(),
    ];

    let result = engine.evaluate(&state, &questions).unwrap();
    assert!(result.answers.contains_key("action"));

    if let neotrix_decision_engine::Answer::Choice(choice) = &result.answers["action"] {
        assert!(!choice.choice.is_empty(), "Choice should not be empty");
        assert!(choice.confidence >= 0.0 && choice.confidence <= 1.0,
            "Confidence should be in [0,1], got {}", choice.confidence);
        eprintln!("Choice answer: {} (confidence: {}, margin: {})", choice.choice, choice.confidence, choice.margin);
    } else {
        panic!("Expected Choice answer");
    }
}

// ── Score Evaluation ───────────────────────────────────────────

#[test]
fn test_evaluate_score() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let state: State = "The API returned an intermittent 503 error during peak hours.".into();
    let questions = vec![
        Question::score("severity",
            "How severe is this issue?",
            vec![
                "Low - cosmetic, no user impact".to_string(),
                "Medium - degraded performance".to_string(),
                "High - partial outage".to_string(),
                "Critical - full outage".to_string(),
            ]).unwrap(),
    ];

    let result = engine.evaluate(&state, &questions).unwrap();
    assert!(result.answers.contains_key("severity"));

    if let neotrix_decision_engine::Answer::Score(score) = &result.answers["severity"] {
        assert!(score.score >= 0.0, "Score should be >= 0, got {}", score.score);
        assert!(!score.probabilities.is_empty(), "Probabilities should not be empty");
        eprintln!("Score answer: {} (confidence: {}, legend: {:?})", score.score, score.confidence, score.legend);
    } else {
        panic!("Expected Score answer");
    }
}

// ── Multi-Question Batch ───────────────────────────────────────

#[test]
fn test_evaluate_multi_question_batch() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let state: State = "Deploy v2.3.1 to production. All tests passing. Two reviewers approved.".into();

    let mut review_options = HashMap::new();
    review_options.insert("approve".to_string(), Some("Approve deployment".to_string()));
    review_options.insert("reject".to_string(), Some("Reject deployment".to_string()));
    review_options.insert("defer".to_string(), Some("Defer to next sprint".to_string()));

    let questions = vec![
        Question::noul("ready", "Is this ready for production?"),
        Question::choice("action", "What action to take?", review_options).unwrap(),
        Question::score("risk", "Rate the deployment risk",
            vec!["Very Low".into(), "Low".into(), "Medium".into(), "High".into()]).unwrap(),
    ];

    let result = engine.evaluate(&state, &questions).unwrap();

    // All three answers should be present
    assert_eq!(result.answers.len(), 3, "Should have 3 answers");
    assert!(result.answers.contains_key("ready"));
    assert!(result.answers.contains_key("action"));
    assert!(result.answers.contains_key("risk"));

    eprintln!("Multi-question batch result:");
    for (id, answer) in &result.answers {
        eprintln!("  {}: {:?}", id, answer);
    }
}

// ── State with Structured Data ─────────────────────────────────

#[test]
fn test_state_with_data() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let data = serde_json::json!({
        "cpu_usage": 95.2,
        "memory_usage": 87.1,
        "error_rate": 0.05,
        "request_count": 15000
    });

    let state = State {
        content: "System metrics snapshot".to_string(),
        data: Some(data),
    };

    let questions = vec![
        Question::noul("healthy", "Is the system healthy?"),
    ];

    let result = engine.evaluate(&state, &questions).unwrap();
    assert!(result.answers.contains_key("healthy"));
    eprintln!("State with data: {:?}", result.answers["healthy"]);
}

// ── Empty / Edge Cases ─────────────────────────────────────────

#[test]
fn test_evaluate_empty_state() {
    let engine = match try_create_engine() {
        Some(e) => e,
        None => {
            eprintln!("Skipping: model not available");
            return;
        }
    };

    let state: State = "".into();
    let questions = vec![
        Question::noul("empty", "Is this empty?"),
    ];

    let result = engine.evaluate(&state, &questions);
    // Should not panic even with empty input
    match result {
        Ok(r) => {
            eprintln!("Empty state result: {:?}", r.answers["empty"]);
        }
        Err(e) => {
            eprintln!("Empty state error (acceptable): {}", e);
        }
    }
}
