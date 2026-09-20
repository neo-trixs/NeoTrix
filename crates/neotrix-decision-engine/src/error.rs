//! Error types for the decision engine

use thiserror::Error;

/// Decision engine errors
#[derive(Error, Debug)]
pub enum Error {
    /// Model inference failed
    #[error("Inference error: {0}")]
    InferenceError(String),
    
    /// Failed to parse model response
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Missing answer for a question
    #[error("Missing answer for question: {0}")]
    MissingAnswer(String),
    
    /// Invalid question type
    #[error("Invalid question type: {0}")]
    InvalidQuestionType(String),
    
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Model loading failed
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Tokenizer error
    #[error("Tokenizer error: {0}")]
    TokenizerError(String),
    
    /// Weight loading error
    #[error("Weight loading error: {0}")]
    WeightLoadError(String),
    
    /// Model error
    #[error("Model error: {0}")]
    ModelError(String),
    
    /// Candle error
    #[error("Candle error: {0}")]
    CandleError(#[from] candle_core::Error),
}

/// Result type for decision engine operations
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let cases = vec![
            (Error::InferenceError("timeout".into()), "Inference error: timeout"),
            (Error::ParseError("bad json".into()), "Parse error: bad json"),
            (Error::MissingAnswer("q1".into()), "Missing answer for question: q1"),
            (Error::InvalidQuestionType("unknown".into()), "Invalid question type: unknown"),
            (Error::ModelNotFound("bert".into()), "Model not found: bert"),
            (Error::ModelLoadError("corrupt".into()), "Model loading failed: corrupt"),
            (Error::TokenizerError("bad token".into()), "Tokenizer error: bad token"),
            (Error::WeightLoadError("shape mismatch".into()), "Weight loading error: shape mismatch"),
            (Error::ModelError("forward failed".into()), "Model error: forward failed"),
        ];

        for (error, expected_msg) in cases {
            let display = format!("{}", error);
            assert_eq!(display, expected_msg, "Error display mismatch for {:?}", expected_msg);
        }
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: Error = io_err.into();
        match error {
            Error::IoError(_) => {}
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_serde_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: Error = json_err.into();
        match error {
            Error::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_candle_error_conversion() {
        let candle_err = candle_core::Error::UnexpectedShape {
            msg: "test".into(),
            expected: candle_core::Shape::from_dims(&[1]),
            got: candle_core::Shape::from_dims(&[2]),
        };
        let error: Error = candle_err.into();
        match error {
            Error::CandleError(_) => {}
            _ => panic!("Expected CandleError"),
        }
    }

    #[test]
    fn test_result_ok() {
        let r: std::result::Result<i32, ()> = Ok(42);
        let val = match r {
            Ok(v) => v,
            Err(()) => panic!("expected Ok"),
        };
        assert_eq!(val, 42);
    }

    #[test]
    fn test_result_err() {
        let r: Result<i32> = Err(Error::InferenceError("fail".into()));
        assert!(r.is_err());
    }
}
