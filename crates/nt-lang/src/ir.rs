/// Internal representation of an nt-test suite.
use std::path::PathBuf;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub source_file: PathBuf,
    pub tests: Vec<TestCase>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestCase {
    pub name: String,
    pub description: String,
    pub imports: Vec<String>,
    pub setup: Option<String>,
    pub code: String,
}
