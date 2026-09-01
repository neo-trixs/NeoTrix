//! Prompt Template System — 提示模板管理
//!
//! 吸收 KB 经验:
//! - 模板定义/变量替换
//! - 条件逻辑
//! - 循环展开
//! - 版本管理
//! - A/B 测试

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 提示模板管理器
pub struct PromptTemplateManager {
    templates: HashMap<String, PromptTemplate>,
    variables: HashMap<String, serde_json::Value>,
    stats: TemplateStats,
}

/// 提示模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template: String,
    pub variables: Vec<TemplateVariable>,
    pub version: u32,
    pub tags: Vec<String>,
    pub metadata: TemplateMetadata,
}

/// 模板变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub var_type: VariableType,
    pub default: Option<serde_json::Value>,
    pub required: bool,
    pub description: Option<String>,
}

/// 变量类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

/// 模板元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub author: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
    pub avg_quality_score: f64,
}

/// 渲染结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedPrompt {
    pub template_id: String,
    pub rendered: String,
    pub variables_used: HashMap<String, serde_json::Value>,
    pub token_estimate: usize,
}

/// 模板统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStats {
    pub total_templates: u64,
    pub total_renders: u64,
    pub avg_render_time_ms: f64,
    pub most_used: Vec<(String, u64)>,
}

/// 条件表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalExpression {
    pub condition: String,
    pub then_branch: String,
    pub else_branch: Option<String>,
}

impl PromptTemplateManager {
    /// 创建新的模板管理器
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            variables: HashMap::new(),
            stats: TemplateStats {
                total_templates: 0,
                total_renders: 0,
                avg_render_time_ms: 0.0,
                most_used: Vec::new(),
            },
        }
    }

    /// 注册模板
    pub fn register_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
        self.stats.total_templates += 1;
    }

    /// 设置全局变量
    pub fn set_variable(&mut self, name: &str, value: serde_json::Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// 渲染模板
    pub fn render(&mut self, template_id: &str, variables: &HashMap<String, serde_json::Value>) -> Result<RenderedPrompt, String> {
        let template = self.templates.get(template_id)
            .ok_or_else(|| format!("Template {} not found", template_id))?;

        // 合并变量 (局部 > 全局)
        let mut all_variables = self.variables.clone();
        all_variables.extend(variables.clone());

        // 检查必需变量
        for var in &template.variables {
            if var.required && !all_variables.contains_key(&var.name) {
                if var.default.is_none() {
                    return Err(format!("Missing required variable: {}", var.name));
                }
            }
        }

        // 渲染模板
        let rendered = self.render_template(&template.template, &all_variables)?;

        // 估算 token 数
        let token_estimate = rendered.len() / 4; // 粗略估计

        self.stats.total_renders += 1;

        Ok(RenderedPrompt {
            template_id: template_id.to_string(),
            rendered,
            variables_used: all_variables,
            token_estimate,
        })
    }

    /// 渲染模板字符串
    fn render_template(&self, template: &str, variables: &HashMap<String, serde_json::Value>) -> Result<String, String> {
        let mut result = template.to_string();

        // 替换变量 {{variable_name}}
        for (name, value) in variables {
            let placeholder = format!("{{{{{}}}}}", name);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Array(arr) => serde_json::to_string(arr).unwrap_or_default(),
                serde_json::Value::Object(obj) => serde_json::to_string(obj).unwrap_or_default(),
                serde_json::Value::Null => "null".to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        // 处理条件语句 {% if condition %}...{% else %}...{% endif %}
        result = self.process_conditionals(&result, variables)?;

        // 处理循环 {% for item in items %}...{% endfor %}
        result = self.process_loops(&result, variables)?;

        Ok(result)
    }

    /// 处理条件语句
    fn process_conditionals(&self, template: &str, variables: &HashMap<String, serde_json::Value>) -> Result<String, String> {
        let mut result = template.to_string();

        // 简化版: 处理简单的 {% if variable %}...{% endif %}
        while let Some(if_start) = result.find("{% if ") {
            let condition_end = result[if_start..].find("%}").ok_or("Invalid conditional syntax")?;
            let condition = result[if_start + 6..if_start + condition_end].trim();

            // 查找对应的 endif
            let endif_pattern = "{% endif %}";
            let endif_pos = result[if_start..].find(endif_pattern).ok_or("Missing endif")?;
            let block = &result[if_start + condition_end + 2..if_start + endif_pos];

            // 检查条件
            let condition_met = if let Some(value) = variables.get(condition) {
                match value {
                    serde_json::Value::Bool(b) => *b,
                    serde_json::Value::String(s) => !s.is_empty(),
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    serde_json::Value::Null => false,
                    _ => true,
                }
            } else {
                false
            };

            let replacement = if condition_met {
                block.trim().to_string()
            } else {
                String::new()
            };

            result = format!("{}{}{}", &result[..if_start], replacement, &result[if_start + endif_pos + endif_pattern.len()..]);
        }

        Ok(result)
    }

    /// 处理循环
    fn process_loops(&self, template: &str, variables: &HashMap<String, serde_json::Value>) -> Result<String, String> {
        let mut result = template.to_string();

        // 简化版: 处理简单的 {% for item in items %}...{% endfor %}
        while let Some(for_start) = result.find("{% for ") {
            let for_end = result[for_start..].find("%}").ok_or("Invalid loop syntax")?;
            let for_clause = result[for_start + 7..for_start + for_end].trim();

            // 解析: "item in items"
            let parts: Vec<&str> = for_clause.splitn(2, " in ").collect();
            if parts.len() != 2 {
                return Err("Invalid loop syntax".into());
            }
            let item_name = parts[0].trim();
            let list_name = parts[1].trim();

            // 查找对应的 endfor
            let endfor_pattern = "{% endfor %}";
            let endfor_pos = result[for_start..].find(endfor_pattern).ok_or("Missing endfor")?;
            let body = &result[for_start + for_end + 2..for_start + endfor_pos];

            // 获取列表
            if let Some(serde_json::Value::Array(arr)) = variables.get(list_name) {
                let mut expanded = String::new();
                for item in arr {
                    let mut item_vars = variables.clone();
                    item_vars.insert(item_name.to_string(), item.clone());
                    expanded.push_str(&self.render_template(body, &item_vars)?);
                    expanded.push('\n');
                }
                result = format!("{}{}{}", &result[..for_start], expanded, &result[for_start + endfor_pos + endfor_pattern.len()..]);
            } else {
                // 如果列表不存在，移除整个循环
                result = format!("{}{}", &result[..for_start], &result[for_start + endfor_pos + endfor_pattern.len()..]);
            }
        }

        Ok(result)
    }

    /// 获取统计信息
    pub fn stats(&self) -> &TemplateStats {
        &self.stats
    }
}
