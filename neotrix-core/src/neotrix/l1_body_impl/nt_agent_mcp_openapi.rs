// OpenAPI to MCP 转换 (来自 higress 吸收: R-P83 / R-P42)
// 将 OpenAPI 规范自动转为 MCP Server, 解决工具发现问题
// 参考 higress openapi-to-mcp 工具: https://github.com/higress-group/openapi-to-mcpserver

use crate::neotrix::l1_body_impl::nt_agent_mcp_registry::{McpRegistry, McpServerEntry, McpToolDef, McpTransport};
use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: OpenApiInfo,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
    pub head: Option<Operation>,
    pub options: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub tags: Vec<String>,
    pub security: Option<Vec<SecurityRequirement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub in_: String, // "query" | "header" | "path" | "cookie"
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
    #[serde(rename = "in")]
    pub param_in: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,
    pub items: Option<Box<Schema>>,
    pub enum_: Option<Vec<Value>>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String, // "http" | "apiKey" | "oauth2" | "openIdConnect"
    pub scheme: Option<String>, // "bearer" | "basic" etc.
    pub name: Option<String>, // for apiKey
    pub in_: Option<String>, // "query" | "header" | "cookie"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub requirements: HashMap<String, Vec<String>>,
}

pub struct OpenApiToMcpConverter {
    spec: OpenApiSpec,
    server_name: String,
    base_url: String,
}

impl OpenApiToMcpConverter {
    pub fn new(spec: OpenApiSpec, server_name: impl Into<String>) -> Self {
        let base_url = spec.servers.first().map(|s| s.url.clone()).unwrap_or_default();
        Self { spec, server_name: server_name.into(), base_url }
    }

    /// 转换为 MCP ServerEntry 列表
    pub fn convert(&self) -> Vec<McpServerEntry> {
        let mut servers = Vec::new();

        for (path, path_item) in &self.spec.paths {
            for (method, operation) in self.iter_operations(path, path_item) {
                if let Some(tool) = self.operation_to_tool(path, method, operation) {
                    let server = McpServerEntry {
                        name: format!("{}_{}_{}", self.server_name, method.to_lowercase(), self.sanitize_path(path)),
                        transport: McpTransport::Http,
                        command: None,
                        url: Some(self.resolve_url(path)),
                        tools: vec![tool],
                        healthy: true,
                        latency_ms: 0,
                        last_health_check: None,
                        init_result: Some("openapi-to-mcp".to_string()),
                    };
                    servers.push(server);
                }
            }
        }

        servers
    }

    /// 注册到现有 registry
    pub fn register_to_registry(&self, registry: &mut McpRegistry) {
        for server in self.convert() {
            registry.register(server);
        }
    }

    fn iter_operations<'a>(&self, _path: &str, item: &'a PathItem) -> Vec<(&'a str, &'a Operation)> {
        let mut ops = Vec::new();
        if let Some(op) = &item.get { ops.push(("GET", op)); }
        if let Some(op) = &item.post { ops.push(("POST", op)); }
        if let Some(op) = &item.put { ops.push(("PUT", op)); }
        if let Some(op) = &item.delete { ops.push(("DELETE", op)); }
        if let Some(op) = &item.patch { ops.push(("PATCH", op)); }
        if let Some(op) = &item.head { ops.push(("HEAD", op)); }
        if let Some(op) = &item.options { ops.push(("OPTIONS", op)); }
        ops
    }

    fn operation_to_tool(&self, path: &str, method: &str, op: &Operation) -> Option<McpToolDef> {
        let name = op.operation_id.clone().unwrap_or_else(|| {
            format!("{}_{}", method.to_lowercase(), self.sanitize_path(path))
        });
        let description = op.summary.clone().or(op.description.clone())
            .unwrap_or_else(|| format!("{} {}", method, path));

        // 从参数构建 input_schema
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &op.parameters {
            if let Some(schema) = &param.schema {
                let prop_name = param.name.clone();
                let prop_schema = self.schema_to_json_value(schema);
                properties.insert(prop_name.clone(), prop_schema);
                if param.required.unwrap_or(false) {
                    required.push(prop_name);
                }
            }
        }

        if let Some(body) = &op.request_body {
            for (media_type, mt) in &body.content {
                if media_type == "application/json" {
                    if let Some(schema) = &mt.schema {
                        properties.insert("body".into(), self.schema_to_json_value(schema));
                        if body.required.unwrap_or(false) {
                            required.push("body".into());
                        }
                    }
                }
            }
        }

        let input_schema = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert("type".into(), Value::String("object".into()));
            if !properties.is_empty() {
                map.insert("properties".into(), Value::Object(properties));
            }
            if !required.is_empty() {
                map.insert("required".into(), Value::Array(required.into_iter().map(Value::String).collect()));
            }
            map
        });

        Some(McpToolDef {
            name,
            description,
            input_schema,
            server_name: self.server_name.clone(),
            transport: McpTransport::Http,
            schema_version: Some("2020-12".into()),
        })
    }

    fn schema_to_json_value(&self, schema: &Schema) -> Value {
        let mut obj = serde_json::Map::new();
        if let Some(t) = &schema.schema_type {
            obj.insert("type".into(), Value::String(t.clone()));
        }
        if let Some(fmt) = &schema.format {
            obj.insert("format".into(), Value::String(fmt.clone()));
        }
        if let Some(items) = &schema.items {
            obj.insert("items".into(), self.schema_to_json_value(items));
        }
        if let Some(props) = &schema.properties {
            let mut prop_map = serde_json::Map::new();
            for (k, v) in props {
                prop_map.insert(k.clone(), self.schema_to_json_value(v));
            }
            obj.insert("properties".into(), Value::Object(prop_map));
        }
        if let Some(req) = &schema.required {
            obj.insert("required".into(), Value::Array(req.iter().map(|s| Value::String(s.clone())).collect()));
        }
        if let Some(ref_) = &schema.ref_ {
            // 简化处理: 如果有 $ref, 尝试解析 components/schemas
            if let Some(components) = &self.spec.components {
                if let Some(schemas) = &components.schemas {
                    if let Some(resolved) = schemas.get(ref_.strip_prefix("#/components/schemas/").unwrap_or(ref_)) {
                        return self.schema_to_json_value(resolved);
                    }
                }
            }
            obj.insert("$ref".into(), Value::String(ref_.clone()));
        }
        Value::Object(obj)
    }

    fn sanitize_path(&self, path: &str) -> String {
        let sanitized = path.replace(['/', '{', '}'], "_")
            .trim_matches('_')
            .to_string();
        // Collapse multiple underscores
        let mut result = String::new();
        let mut prev_underscore = false;
        for c in sanitized.chars() {
            if c == '_' {
                if !prev_underscore {
                    result.push(c);
                    prev_underscore = true;
                }
            } else {
                result.push(c);
                prev_underscore = false;
            }
        }
        result
    }

    fn resolve_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }
}

/// 便捷函数: 从 JSON 字符串解析并转换
pub fn openapi_json_to_mcp(json: &str, server_name: &str) -> Result<Vec<McpServerEntry>, String> {
    let spec: OpenApiSpec = serde_json::from_str(json)
        .map_err(|e| format!("Invalid OpenAPI spec: {}", e))?;
    let converter = OpenApiToMcpConverter::new(spec, server_name);
    Ok(converter.convert())
}

/// 便捷函数: 从文件加载并转换
pub fn openapi_file_to_mcp(path: &str, server_name: &str) -> Result<Vec<McpServerEntry>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read OpenAPI file: {}", e))?;
    openapi_json_to_mcp(&content, server_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        let spec = OpenApiSpec {
            openapi: "3.0".into(),
            info: OpenApiInfo { title: "Test".into(), version: "1.0".into(), description: None },
            paths: HashMap::new(),
            components: None,
            servers: vec![],
        };
        let conv = OpenApiToMcpConverter::new(spec, "test");
        assert_eq!(conv.sanitize_path("/api/v1/users"), "api_v1_users");
        assert_eq!(conv.sanitize_path("/users/{id}"), "users_id");
    }

    #[test]
    fn test_resolve_url() {
        let spec = OpenApiSpec {
            openapi: "3.0".into(),
            info: OpenApiInfo { title: "Test".into(), version: "1.0".into(), description: None },
            paths: HashMap::new(),
            components: None,
            servers: vec![Server { url: "https://api.example.com".into(), description: None }],
        };
        let conv = OpenApiToMcpConverter::new(spec, "test");
        assert_eq!(conv.resolve_url("/users"), "https://api.example.com/users");
    }

    #[test]
    fn test_schema_to_json_basic() {
        let spec = OpenApiSpec {
            openapi: "3.0".into(),
            info: OpenApiInfo { title: "Test".into(), version: "1.0".into(), description: None },
            paths: HashMap::new(),
            components: None,
            servers: vec![],
        };
        let conv = OpenApiToMcpConverter::new(spec, "test");
        let schema = Schema { schema_type: Some("string".into()), format: Some("email".into()), properties: None, required: None, ref_: None, items: None, enum_: None };
        let val = conv.schema_to_json_value(&schema);
        assert_eq!(val["type"], "string");
        assert_eq!(val["format"], "email");
    }

    #[test]
    fn test_convert_simple_get() {
        let mut paths = HashMap::new();
        paths.insert("/users".into(), PathItem {
            get: Some(Operation {
                operation_id: Some("listUsers".into()),
                summary: Some("List users".into()),
                description: None,
                parameters: vec![],
                request_body: None,
                responses: HashMap::new(),
                tags: vec![],
                security: None,
            }),
            post: None, put: None, delete: None, patch: None, head: None, options: None,
        });

        let spec = OpenApiSpec {
            openapi: "3.0.1".into(),
            info: OpenApiInfo { title: "User API".into(), version: "1.0".into(), description: None },
            paths,
            components: None,
            servers: vec![Server { url: "https://api.example.com".into(), description: None }],
        };

        let conv = OpenApiToMcpConverter::new(spec, "userapi");
        let servers = conv.convert();
        assert_eq!(servers.len(), 1);
        let tool = &servers[0].tools[0];
        assert_eq!(tool.name, "listUsers");
        assert_eq!(tool.server_name, "userapi");
        assert_eq!(tool.transport, McpTransport::Http);
        let server_url = &servers[0].url;
        assert!(server_url.as_deref().unwrap().contains("https://api.example.com/users"));
    }
}