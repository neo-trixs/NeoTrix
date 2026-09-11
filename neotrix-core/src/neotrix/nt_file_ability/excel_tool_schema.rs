//! Excel 解析能力 Function Calling Schema
//!
//! 为 LLM 提供标准化的工具调用接口:
//! - parse_excel: 解析 Excel 文件
//! - detect_template: 检测模板类型
//! - parse_config: 解析配置字段
//! - extract_path_metadata: 提取路径元数据

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Excel 解析工具 Schema (供 LLM Function Calling)
pub fn parse_excel_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "parse_excel",
        "description": "解析 Excel 文件，返回结构化表格数据。支持 .xlsx 格式，自动检测模板类型。",
        "parameters": {
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Excel 文件的绝对路径"
                },
                "output_format": {
                    "type": "string",
                    "enum": ["markdown", "json", "csv"],
                    "description": "输出格式: markdown (人类可读), json (结构化), csv (紧凑)",
                    "default": "json"
                },
                "max_rows": {
                    "type": "integer",
                    "description": "最大返回行数 (默认 50)",
                    "default": 50
                },
                "detect_template": {
                    "type": "boolean",
                    "description": "是否自动检测模板类型 (默认 true)",
                    "default": true
                }
            },
            "required": ["file_path"]
        }
    })
}

/// 模板检测 Schema
pub fn detect_template_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "detect_template",
        "description": "检测 Excel 文件的模板类型 (采购申请单/采购清单/通用表格)",
        "parameters": {
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Excel 文件的绝对路径"
                }
            },
            "required": ["file_path"]
        }
    })
}

/// 配置解析 Schema
pub fn parse_config_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "parse_config",
        "description": "解析逗号分隔的配置字符串为结构化数据",
        "parameters": {
            "type": "object",
            "properties": {
                "config_string": {
                    "type": "string",
                    "description": "配置字符串，如 '执行标准:美标,压力:150LB,阀体:Q235'"
                }
            },
            "required": ["config_string"]
        }
    })
}

/// 路径元数据 Schema
pub fn extract_path_metadata_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "extract_path_metadata",
        "description": "从文件路径提取元数据 (业务员/国家/客户/订单号)",
        "parameters": {
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "文件的绝对路径"
                }
            },
            "required": ["file_path"]
        }
    })
}

/// 所有 Excel 工具 Schema 列表
pub fn all_excel_tool_schemas() -> Vec<serde_json::Value> {
    vec![
        parse_excel_schema(),
        detect_template_schema(),
        parse_config_schema(),
        extract_path_metadata_schema(),
    ]
}

/// Excel 解析请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseExcelRequest {
    pub file_path: String,
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_detect_template")]
    pub detect_template: bool,
}

fn default_output_format() -> String {
    "json".to_string()
}

fn default_max_rows() -> usize {
    50
}

fn default_detect_template() -> bool {
    true
}

/// Excel 解析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseExcelResponse {
    pub file_path: String,
    pub template_type: Option<String>,
    pub total_rows: usize,
    pub total_cols: usize,
    pub output_format: String,
    pub data: String,
}

/// 模板检测请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectTemplateRequest {
    pub file_path: String,
}

/// 模板检测响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectTemplateResponse {
    pub template_type: String,
    pub confidence: f64,
    pub detected_headers: Vec<String>,
    pub column_map: HashMap<String, usize>,
}

/// 配置解析请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseConfigRequest {
    pub config_string: String,
}

/// 配置解析响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseConfigResponse {
    pub exec_std: Option<String>,
    pub pressure: Option<String>,
    pub valve_body: Option<String>,
    pub valve_stem: Option<String>,
    pub valve_plate: Option<String>,
    pub valve_seat: Option<String>,
    pub connection: Option<String>,
    pub drive: Option<String>,
    pub color: Option<String>,
    pub material: Option<String>,
    pub temperature: Option<String>,
    pub extra: HashMap<String, String>,
}

/// 路径元数据请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractPathMetadataRequest {
    pub file_path: String,
}

/// 路径元数据响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractPathMetadataResponse {
    pub salesperson: Option<String>,
    pub order_folder: Option<String>,
    pub order_no: Option<String>,
    pub order_date: Option<String>,
    pub country: Option<String>,
    pub customer: Option<String>,
    pub file_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_excel_schema() {
        let schema = parse_excel_schema();
        assert_eq!(schema["name"], "parse_excel");
        assert!(schema["properties"]["file_path"]["description"].is_string());
    }

    #[test]
    fn test_detect_template_schema() {
        let schema = detect_template_schema();
        assert_eq!(schema["name"], "detect_template");
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("file_path")));
    }

    #[test]
    fn test_parse_config_schema() {
        let schema = parse_config_schema();
        assert_eq!(schema["name"], "parse_config");
    }

    #[test]
    fn test_extract_path_metadata_schema() {
        let schema = extract_path_metadata_schema();
        assert_eq!(schema["name"], "extract_path_metadata");
    }

    #[test]
    fn test_all_schemas() {
        let schemas = all_excel_tool_schemas();
        assert_eq!(schemas.len(), 4);
        let names: Vec<&str> = schemas
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"parse_excel"));
        assert!(names.contains(&"detect_template"));
        assert!(names.contains(&"parse_config"));
        assert!(names.contains(&"extract_path_metadata"));
    }

    #[test]
    fn test_parse_excel_request_defaults() {
        let req = ParseExcelRequest {
            file_path: "/tmp/test.xlsx".to_string(),
            output_format: default_output_format(),
            max_rows: default_max_rows(),
            detect_template: default_detect_template(),
        };
        assert_eq!(req.output_format, "json");
        assert_eq!(req.max_rows, 50);
        assert!(req.detect_template);
    }

    #[test]
    fn test_detect_template_response_serialize() {
        let resp = DetectTemplateResponse {
            template_type: "PurchaseRequest".to_string(),
            confidence: 0.95,
            detected_headers: vec!["序号".to_string(), "名称".to_string()],
            column_map: HashMap::from([("serial".to_string(), 0), ("name".to_string(), 1)]),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["template_type"], "PurchaseRequest");
        assert_eq!(json["confidence"], 0.95);
    }

    #[test]
    fn test_parse_config_response_serialize() {
        let resp = ParseConfigResponse {
            exec_std: Some("美标".to_string()),
            pressure: Some("150LB".to_string()),
            valve_body: Some("Q235".to_string()),
            valve_stem: None,
            valve_plate: None,
            valve_seat: None,
            connection: None,
            drive: None,
            color: None,
            material: None,
            temperature: None,
            extra: HashMap::new(),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["exec_std"], "美标");
        assert_eq!(json["pressure"], "150LB");
    }

    #[test]
    fn test_extract_path_metadata_response_serialize() {
        let resp = ExtractPathMetadataResponse {
            salesperson: Some("张三".to_string()),
            order_folder: Some("4.01菲律宾 WSD-I-26031303".to_string()),
            order_no: Some("WSD-I-26031303".to_string()),
            order_date: Some("4.01".to_string()),
            country: Some("菲律宾".to_string()),
            customer: Some("WSD".to_string()),
            file_name: Some("报价单.xlsx".to_string()),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["salesperson"], "张三");
        assert_eq!(json["country"], "菲律宾");
    }
}
