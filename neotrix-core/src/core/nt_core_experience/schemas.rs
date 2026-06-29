use crate::core::nt_core_experience::extract_pipeline::{
    ExtractionField, ExtractionSchema, FieldType,
};

pub fn code_schema() -> ExtractionSchema {
    ExtractionSchema {
        name: "code_reference".into(),
        fields: vec![
            ExtractionField {
                name: "language".into(),
                description: "Programming language name".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "function_name".into(),
                description: "Name of the function or method".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "parameters".into(),
                description: "Parameters or arguments for the function".into(),
                field_type: FieldType::Array,
                required: false,
            },
            ExtractionField {
                name: "return_type".into(),
                description: "Return type of the function".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "description".into(),
                description: "Description of what the code does".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "example".into(),
                description: "Example code usage".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "dependencies".into(),
                description: "Required dependencies or imports".into(),
                field_type: FieldType::Array,
                required: false,
            },
        ],
    }
}

pub fn article_schema() -> ExtractionSchema {
    ExtractionSchema {
        name: "article".into(),
        fields: vec![
            ExtractionField {
                name: "title".into(),
                description: "Article headline or title".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "author".into(),
                description: "Article author name".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "published_date".into(),
                description: "Date the article was published".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "category".into(),
                description: "Article category or section".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "summary".into(),
                description: "Brief summary or abstract of the article".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "body".into(),
                description: "Main article body content".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "tags".into(),
                description: "Tags or keywords associated with the article".into(),
                field_type: FieldType::Array,
                required: false,
            },
            ExtractionField {
                name: "reading_time".into(),
                description: "Estimated reading time in minutes".into(),
                field_type: FieldType::Number,
                required: false,
            },
        ],
    }
}

pub fn api_schema() -> ExtractionSchema {
    ExtractionSchema {
        name: "api_reference".into(),
        fields: vec![
            ExtractionField {
                name: "endpoint".into(),
                description: "API endpoint URL path".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "method".into(),
                description: "HTTP method (GET, POST, PUT, DELETE, etc.)".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "description".into(),
                description: "Description of what the endpoint does".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "request_body".into(),
                description: "Request body format or schema".into(),
                field_type: FieldType::Object,
                required: false,
            },
            ExtractionField {
                name: "response_body".into(),
                description: "Response body format or schema".into(),
                field_type: FieldType::Object,
                required: false,
            },
            ExtractionField {
                name: "parameters".into(),
                description: "Query or path parameters".into(),
                field_type: FieldType::Array,
                required: false,
            },
            ExtractionField {
                name: "authentication".into(),
                description: "Authentication required for the endpoint".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "status_codes".into(),
                description: "HTTP status codes returned by the endpoint".into(),
                field_type: FieldType::Object,
                required: false,
            },
        ],
    }
}

pub fn repo_schema() -> ExtractionSchema {
    ExtractionSchema {
        name: "repository".into(),
        fields: vec![
            ExtractionField {
                name: "name".into(),
                description: "Repository name".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "description".into(),
                description: "Repository description".into(),
                field_type: FieldType::String,
                required: true,
            },
            ExtractionField {
                name: "language".into(),
                description: "Primary programming language".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "stars".into(),
                description: "Number of stars".into(),
                field_type: FieldType::Number,
                required: false,
            },
            ExtractionField {
                name: "forks".into(),
                description: "Number of forks".into(),
                field_type: FieldType::Number,
                required: false,
            },
            ExtractionField {
                name: "license".into(),
                description: "Repository license type".into(),
                field_type: FieldType::String,
                required: false,
            },
            ExtractionField {
                name: "topics".into(),
                description: "Repository topics or tags".into(),
                field_type: FieldType::Array,
                required: false,
            },
            ExtractionField {
                name: "dependencies".into(),
                description: "Key dependencies or requirements".into(),
                field_type: FieldType::Array,
                required: false,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_schema_structure() {
        let schema = code_schema();
        assert_eq!(schema.name, "code_reference");
        assert!(!schema.fields.is_empty());
        assert!(schema.fields.iter().any(|f| f.name == "language"));
        assert!(schema.fields.iter().any(|f| f.name == "description"));
        assert!(schema.fields.iter().any(|f| f.name == "function_name"));
    }

    #[test]
    fn test_article_schema_structure() {
        let schema = article_schema();
        assert_eq!(schema.name, "article");
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "title" && f.required));
        assert!(schema.fields.iter().any(|f| f.name == "body" && f.required));
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "author" && !f.required));
    }

    #[test]
    fn test_api_schema_structure() {
        let schema = api_schema();
        assert_eq!(schema.name, "api_reference");
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "endpoint" && f.required));
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "method" && f.required));
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "description" && f.required));
    }

    #[test]
    fn test_repo_schema_structure() {
        let schema = repo_schema();
        assert_eq!(schema.name, "repository");
        assert!(schema.fields.iter().any(|f| f.name == "name" && f.required));
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "description" && f.required));
        assert!(schema
            .fields
            .iter()
            .any(|f| f.name == "language" && !f.required));
    }

    #[test]
    fn test_all_schemas_have_unique_names() {
        let schemas = [code_schema(), article_schema(), api_schema(), repo_schema()];
        let mut names: Vec<String> = schemas.iter().map(|s| s.name.clone()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), schemas.len(), "Schema names must be unique");
    }

    #[test]
    fn test_code_schema_field_types() {
        let schema = code_schema();
        for field in &schema.fields {
            match field.name.as_str() {
                "parameters" | "dependencies" => assert_eq!(field.field_type, FieldType::Array),
                "language" | "function_name" | "return_type" | "description" | "example" => {
                    assert_eq!(field.field_type, FieldType::String)
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_article_schema_field_types() {
        let schema = article_schema();
        for field in &schema.fields {
            match field.name.as_str() {
                "tags" => assert_eq!(field.field_type, FieldType::Array),
                "reading_time" => assert_eq!(field.field_type, FieldType::Number),
                _ => assert_eq!(field.field_type, FieldType::String),
            }
        }
    }

    #[test]
    fn test_api_schema_field_types() {
        let schema = api_schema();
        for field in &schema.fields {
            match field.name.as_str() {
                "parameters" => assert_eq!(field.field_type, FieldType::Array),
                "request_body" | "response_body" | "status_codes" => {
                    assert_eq!(field.field_type, FieldType::Object)
                }
                _ => assert_eq!(field.field_type, FieldType::String),
            }
        }
    }

    #[test]
    fn test_repo_schema_field_types() {
        let schema = repo_schema();
        for field in &schema.fields {
            match field.name.as_str() {
                "topics" | "dependencies" => assert_eq!(field.field_type, FieldType::Array),
                "stars" | "forks" => assert_eq!(field.field_type, FieldType::Number),
                _ => assert_eq!(field.field_type, FieldType::String),
            }
        }
    }

    #[test]
    fn test_schema_field_descriptions_not_empty() {
        for schema in [code_schema(), article_schema(), api_schema(), repo_schema()] {
            for field in &schema.fields {
                assert!(
                    !field.description.is_empty(),
                    "Field '{}' has empty description",
                    field.name
                );
            }
        }
    }
}
