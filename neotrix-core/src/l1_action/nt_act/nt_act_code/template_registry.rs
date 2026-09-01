//! CodeTemplateRegistry — 可复用代码变换模板库
//!
//! 每个模板定义:
//!   - 源模式 (regex 匹配已有代码)
//!   - 目标模板 (带 $var 替换变量)
//!   - 适用条件
//!   - 分类

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 模板分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateCategory {
    /// 添加测试模块
    TestStub,
    /// 添加错误处理 (unwrap → match / ?)
    ErrorHandling,
    /// 添加模块声明到 mod.rs
    ModuleDeclaration,
    /// 常量定义规范化
    ConstDefinition,
    /// 函数提取 (大函数拆小)
    FunctionExtraction,
    /// 导入整理 (添加缺失 use)
    ImportOrganization,
    /// 类型注解添加
    TypeAnnotation,
    /// 文档注释添加
    DocComment,
}

/// 单个代码变换模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTemplate {
    pub name: String,
    pub category: TemplateCategory,
    /// 匹配源代码的正则 (用于定位插入点)
    pub source_pattern: String,
    /// 替换模板, 支持 $1 $2 等捕获组引用
    pub target_template: String,
    /// 适用文件扩展名 ["rs", "tsx"]
    pub applicability: Vec<String>,
    /// 需要添加的 use 导入
    pub required_imports: Vec<String>,
    /// 置信度 (0.0-1.0), 从成功应用统计得出
    pub confidence: f64,
}

/// 模板注册表
#[derive(Debug, Clone)]
pub struct CodeTemplateRegistry {
    templates: Vec<CodeTemplate>,
}

impl CodeTemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: Self::builtin_templates(),
        }
    }

    /// 注册新模板
    pub fn register(&mut self, template: CodeTemplate) {
        // 如果同名已存在, 替换
        if let Some(pos) = self.templates.iter().position(|t| t.name == template.name) {
            self.templates[pos] = template;
        } else {
            self.templates.push(template);
        }
    }

    /// 获取某一分类的所有模板
    pub fn by_category(&self, category: TemplateCategory) -> Vec<&CodeTemplate> {
        self.templates.iter().filter(|t| t.category == category).collect()
    }

    /// 查找适用于某文件的模板
    pub fn applicable_to(&self, file: &str, category: Option<TemplateCategory>) -> Vec<&CodeTemplate> {
        let ext = file.rsplit('.').next().unwrap_or("");
        self.templates
            .iter()
            .filter(|t| t.applicability.contains(&ext.to_string()))
            .filter(|t| category.as_ref().map(|c| t.category == *c).unwrap_or(true))
            .collect()
    }

    /// 获取所有模板
    pub fn all(&self) -> &[CodeTemplate] {
        &self.templates
    }

    /// 根据模板 + 参数生成代码
    pub fn instantiate(template: &CodeTemplate, vars: &HashMap<String, String>) -> String {
        let mut result = template.target_template.clone();
        for (key, val) in vars {
            result = result.replace(&format!("${}", key), val);
        }
        result
    }

    /// 内置模板
    fn builtin_templates() -> Vec<CodeTemplate> {
        vec![
            CodeTemplate {
                name: "test_stub_rust".into(),
                category: TemplateCategory::TestStub,
                source_pattern: r"\n}".to_string(),
                target_template: "\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_$name() {\n        let instance = $struct::new();\n        assert!(true);\n    }\n}\n".into(),
                applicability: vec!["rs".into()],
                required_imports: vec![],
                confidence: 0.95,
            },
            CodeTemplate {
                name: "unwrap_to_match".into(),
                category: TemplateCategory::ErrorHandling,
                source_pattern: r"(let \$\{var\} = .+)\.unwrap\(\)".to_string(),
                target_template: "$1.map_err(|e| format!(\"$msg: {}\", e))?".into(),
                applicability: vec!["rs".into()],
                required_imports: vec![],
                confidence: 0.6,
            },
            CodeTemplate {
                name: "module_declaration".into(),
                category: TemplateCategory::ModuleDeclaration,
                source_pattern: r"(pub mod \$\{last\};)".to_string(),
                target_template: "$1\npub mod $name;".into(),
                applicability: vec!["rs".into()],
                required_imports: vec![],
                confidence: 0.9,
            },
            CodeTemplate {
                name: "doc_comment_pub_fn".into(),
                category: TemplateCategory::DocComment,
                source_pattern: r"(pub fn \$\{name\}\()".to_string(),
                target_template: "/// $description\n$1".into(),
                applicability: vec!["rs".into()],
                required_imports: vec![],
                confidence: 0.7,
            },
            CodeTemplate {
                name: "add_debug_derive".into(),
                category: TemplateCategory::TypeAnnotation,
                source_pattern: r"(#\[derive\(.*)\)".to_string(),
                target_template: "$1, Debug)".into(),
                applicability: vec!["rs".into()],
                required_imports: vec!["use std::fmt::Debug;".into()],
                confidence: 0.85,
            },
        ]
    }
}

impl Default for CodeTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_has_builtins() {
        let reg = CodeTemplateRegistry::new();
        assert!(reg.all().len() >= 5);
    }

    #[test]
    fn test_by_category() {
        let reg = CodeTemplateRegistry::new();
        let tests = reg.by_category(TemplateCategory::TestStub);
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "test_stub_rust");
    }

    #[test]
    fn test_applicable_to_rs() {
        let reg = CodeTemplateRegistry::new();
        let apps = reg.applicable_to("foo.rs", None);
        assert!(!apps.is_empty());
        assert!(apps.iter().all(|t| t.applicability.contains(&"rs".to_string())));
    }

    #[test]
    fn test_register() {
        let mut reg = CodeTemplateRegistry::new();
        let t = CodeTemplate {
            name: "custom".into(),
            category: TemplateCategory::ImportOrganization,
            source_pattern: "".into(),
            target_template: "".into(),
            applicability: vec!["rs".into()],
            required_imports: vec![],
            confidence: 0.5,
        };
        reg.register(t);
        assert_eq!(reg.by_category(TemplateCategory::ImportOrganization).len(), 1);
    }

    #[test]
    fn test_register_replaces_same_name() {
        let mut reg = CodeTemplateRegistry::new();
        let t1 = CodeTemplate {
            name: "test_stub_rust".into(),
            category: TemplateCategory::ImportOrganization,
            source_pattern: "".into(),
            target_template: "".into(),
            applicability: vec!["rs".into()],
            required_imports: vec![],
            confidence: 0.5,
        };
        reg.register(t1);
        assert_eq!(reg.by_category(TemplateCategory::TestStub).len(), 0);
        assert_eq!(reg.by_category(TemplateCategory::ImportOrganization).len(), 1);
    }

    #[test]
    fn test_instantiate_simple() {
        let reg = CodeTemplateRegistry::new();
        let t = &reg.all()[0];
        let mut vars = HashMap::new();
        vars.insert("name".into(), "my_test".into());
        vars.insert("struct".into(), "MyStruct".into());
        let result = CodeTemplateRegistry::instantiate(t, &vars);
        assert!(result.contains("my_test"));
        assert!(result.contains("MyStruct"));
    }

    #[test]
    fn test_applicable_to_tsx_none() {
        let reg = CodeTemplateRegistry::new();
        let apps = reg.applicable_to("foo.tsx", None);
        assert!(apps.is_empty());
    }
}
