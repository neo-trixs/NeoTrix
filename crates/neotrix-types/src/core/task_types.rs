use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Default)]
pub enum OutputSchema {
    #[default]
    Text,
    Json { schema: Option<String> },
    Typed { type_name: String, schema_json: String },
}


#[derive(Clone, Debug)]
pub struct TaskOutput {
    pub raw: String,
    pub schema: OutputSchema,
    pub validated: bool,
    pub validation_error: Option<String>,
}

impl TaskOutput {
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            schema: OutputSchema::Text,
            validated: false,
            validation_error: None,
        }
    }

    pub fn with_schema(raw: String, schema: OutputSchema) -> Self {
        Self {
            raw,
            schema,
            validated: false,
            validation_error: None,
        }
    }

    pub fn validate(&mut self) -> bool {
        let result = OutputValidator::validate(&self.schema, &self.raw);
        match result {
            Ok(()) => {
                self.validated = true;
                self.validation_error = None;
                true
            }
            Err(e) => {
                self.validated = false;
                self.validation_error = Some(e);
                false
            }
        }
    }
}

pub struct OutputValidator;

impl OutputValidator {
    pub fn validate(schema: &OutputSchema, raw: &str) -> Result<(), String> {
        match schema {
            OutputSchema::Text => Ok(()),
            OutputSchema::Json { schema: _schema_str } => {
                serde_json::from_str::<serde_json::Value>(raw)
                    .map_err(|e| format!("invalid JSON: {}", e))?;
                Ok(())
            }
            OutputSchema::Typed { type_name: _, schema_json: _ } => {
                serde_json::from_str::<serde_json::Value>(raw)
                    .map_err(|e| format!("invalid JSON for typed output: {}", e))?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema_is_text() {
        let schema = OutputSchema::default();
        assert!(matches!(schema, OutputSchema::Text));
    }

    #[test]
    fn test_text_schema_always_valid() {
        let mut output = TaskOutput::new("any free-form text".into());
        assert!(output.validate());
        assert!(output.validated);
        assert!(output.validation_error.is_none());
    }

    #[test]
    fn test_json_schema_valid_json() {
        let schema = OutputSchema::Json { schema: None };
        let mut output = TaskOutput::with_schema(r#"{"name":"test"}"#.into(), schema);
        assert!(output.validate());
        assert!(output.validated);
    }

    #[test]
    fn test_json_schema_invalid_json() {
        let schema = OutputSchema::Json { schema: None };
        let mut output = TaskOutput::with_schema("not valid json".into(), schema);
        assert!(!output.validate());
        assert!(!output.validated);
        assert!(output.validation_error.is_some());
    }

    #[test]
    fn test_typed_schema_valid_json() {
        let schema = OutputSchema::Typed {
            type_name: "User".into(),
            schema_json: r#"{"type":"object"}"#.into(),
        };
        let mut output = TaskOutput::with_schema(r#"{"id":1,"name":"alice"}"#.into(), schema);
        assert!(output.validate());
    }

    #[test]
    fn test_typed_schema_invalid_json() {
        let schema = OutputSchema::Typed {
            type_name: "User".into(),
            schema_json: r#"{"type":"object"}"#.into(),
        };
        let mut output = TaskOutput::with_schema("broken".into(), schema);
        assert!(!output.validate());
        assert!(output.validation_error.is_some());
    }

    #[test]
    fn test_with_schema_preserves_raw() {
        let raw = "test output".to_string();
        let output = TaskOutput::with_schema(raw.clone(), OutputSchema::Text);
        assert_eq!(output.raw, raw);
        assert!(!output.validated);
    }

    #[test]
    fn test_output_validator_text() {
        assert!(OutputValidator::validate(&OutputSchema::Text, "anything").is_ok());
    }

    #[test]
    fn test_output_validator_json_pass() {
        assert!(OutputValidator::validate(
            &OutputSchema::Json { schema: None },
            r#"{"a":1}"#,
        ).is_ok());
    }

    #[test]
    fn test_output_validator_json_fail() {
        assert!(OutputValidator::validate(
            &OutputSchema::Json { schema: None },
            "not json",
        ).is_err());
    }
}
