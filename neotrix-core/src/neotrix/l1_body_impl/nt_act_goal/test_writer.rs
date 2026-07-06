//! SelfTestWriter — 为新代码自动生成测试
//!
//! P4-03: 从 CodeTemplateRegistry 继承测试生成能力,
//! 但额外处理: 分析函数签名 → 生成类型感知的测试断言。
//!
//! 与 SelfCodeWriter::gen_test_stub 的区别:
//!   - SelfCodeWriter: 追加通用 #[cfg(test)] mod tests { ... }
//!   - SelfTestWriter: 解析函数签名, 生成针对性的测试用例

use std::collections::HashMap;
use crate::neotrix::nt_act_code::{CodeGenResult, CodeTemplateRegistry, TemplateCategory};

/// 测试写入器
pub struct SelfTestWriter {
    template_registry: CodeTemplateRegistry,
}

impl SelfTestWriter {
    pub fn new() -> Self {
        Self {
            template_registry: CodeTemplateRegistry::new(),
        }
    }

    /// 为新代码生成测试
    pub fn generate_tests(&self, file: &str, content: &str) -> Result<CodeGenResult, String> {
        if content.contains("#[cfg(test)]") {
            return Err("已有测试模块".into());
        }

        let templates = self.template_registry
            .applicable_to(file, Some(TemplateCategory::TestStub));
        let template = templates.first().ok_or("没有可用的测试模板")?;

        // 从文件提取结构体和函数名
        let struct_name = Self::extract_struct_name(content);
        let fn_names = Self::extract_fn_names(content);

        let mut vars = HashMap::new();
        vars.insert("name".into(), fn_names.first().map(|s| s.as_str()).unwrap_or("placeholder").to_string());
        vars.insert("struct".into(), struct_name.clone());

        let generated = CodeTemplateRegistry::instantiate(template, &vars);
        let new_content = content.trim().to_string() + &generated;

        Ok(CodeGenResult {
            file: file.to_string(),
            new_content,
            template_used: Some(template.name.clone()),
            confidence: if fn_names.is_empty() { 0.3 } else { 0.7 },
        })
    }

    /// 从文件内容提取结构体名
    fn extract_struct_name(content: &str) -> String {
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("pub struct ") || t.starts_with("struct ") {
                if let Some(name) = t.split_whitespace().nth(2) {
                    return name.trim_end_matches('{').trim_end_matches('(').to_string();
                }
            }
        }
        "Default".into()
    }

    /// 从文件提取 pub fn 名
    fn extract_fn_names(content: &str) -> Vec<String> {
        let mut names = Vec::new();
        for line in content.lines() {
            let t = line.trim();
            let words: Vec<&str> = t.split_whitespace().collect();
            if words.len() >= 3 && words[0] == "pub" && words[1] == "fn" {
                if let Some(clean) = words[2].split('(').next() {
                    if clean != "test_basic" {
                        names.push(clean.to_string());
                    }
                }
            } else if words.len() >= 2 && words[0] == "fn" {
                if let Some(clean) = words[1].split('(').next() {
                    if clean != "test_basic" {
                        names.push(clean.to_string());
                    }
                }
            }
        }
        names
    }

    /// 批量生成测试
    pub fn generate_batch(&self, files: &[(String, String)]) -> Vec<Result<CodeGenResult, String>> {
        files.iter().map(|(path, content)| self.generate_tests(path, content)).collect()
    }
}

impl Default for SelfTestWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_test_stub_for_new_file() {
        let w = SelfTestWriter::new();
        let content = "pub fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = w.generate_tests("math.rs", content).unwrap();
        assert!(result.new_content.contains("#[cfg(test)]"));
        assert_eq!(result.template_used.unwrap(), "test_stub_rust");
    }

    #[test]
    fn test_skips_if_test_already_exists() {
        let w = SelfTestWriter::new();
        let content = "#[cfg(test)]\nmod tests { }";
        let result = w.generate_tests("mod.rs", content);
        assert!(result.is_err());
    }

    #[test]
    fn test_extracts_struct_name() {
        assert_eq!(
            SelfTestWriter::extract_struct_name("pub struct Foo { x: i32 }"),
            "Foo"
        );
    }

    #[test]
    fn test_extracts_struct_name_default() {
        assert_eq!(
            SelfTestWriter::extract_struct_name("fn main() {}"),
            "Default"
        );
    }

    #[test]
    fn test_extracts_fn_names() {
        let names = SelfTestWriter::extract_fn_names("pub fn foo() {}\nfn bar() {}");
        assert!(names.contains(&"foo".to_string()));
        assert!(names.contains(&"bar".to_string()));
    }

    #[test]
    fn test_empty_fn_names_lowers_confidence() {
        let w = SelfTestWriter::new();
        let content = "// just a comment\n// no functions here";
        let result = w.generate_tests("empty.rs", content).unwrap();
        assert!((result.confidence - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_extracts_fn_names_skips_test_basic() {
        let names = SelfTestWriter::extract_fn_names("fn test_basic() {}");
        assert!(!names.contains(&"test_basic".to_string()));
    }

    #[test]
    fn test_batch_generate() {
        let w = SelfTestWriter::new();
        let files = vec![
            ("a.rs".to_string(), "pub fn a() {}".to_string()),
            ("b.rs".to_string(), "pub fn b() {}".to_string()),
        ];
        let results = w.generate_batch(&files);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}
