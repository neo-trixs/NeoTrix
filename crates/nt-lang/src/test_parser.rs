use std::path::Path;

use serde::Deserialize;

use crate::ir::{TestCase, TestSuite};

#[derive(Debug, Deserialize)]
struct RawSuite {
    name: String,
    description: String,
    tests: Vec<RawTest>,
}

#[derive(Debug, Deserialize)]
struct RawTest {
    name: String,
    description: String,
    imports: Option<Vec<String>>,
    setup: Option<String>,
    code: String,
}

pub fn parse_file(path: &Path) -> Result<TestSuite, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
    let raw: RawSuite = serde_yaml::from_str(&content)
        .map_err(|e| format!("YAML parse error in {}: {}", path.display(), e))?;

    let tests = raw
        .tests
        .into_iter()
        .map(|t| TestCase {
            name: t.name,
            description: t.description,
            imports: t.imports.unwrap_or_default(),
            setup: t.setup,
            code: t.code,
        })
        .collect();

    Ok(TestSuite {
        name: raw.name,
        description: raw.description,
        source_file: path.to_path_buf(),
        tests,
    })
}
