//! PlantUML Emitter — markdown-viewer/skills PlantUML 语法吸收
//! 
//! mxgraph stencil 语法生成，连接类型语义，自动颜色，分组

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// PlantUML 方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    LeftToRight,
    TopToBottom,
    RightToLeft,
    BottomToTop,
}

/// 分组类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GroupType {
    Rectangle,
    Package,
    Frame,
    Cloud,
    Database,
}

/// PlantUML 节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantUmlNode {
    pub alias: String,
    pub stencil_category: String,
    pub stencil_name: String,
    pub label: String,
    pub group: Option<String>,
}

/// PlantUML 连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantUmlConnection {
    pub from: String,
    pub to: String,
    pub connection_type: ConnectionType,
    pub label: Option<String>,
}

/// 连接类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionType {
    Solid,      // -->
    Dashed,     // ..>
    Bidirectional, // --
    Labeled,    // --> : "label"
    Dotted,     // -.-
    Bold,       // ==>
}

/// PlantUML 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantUmlDocument {
    pub direction: Direction,
    pub nodes: Vec<PlantUmlNode>,
    pub connections: Vec<PlantUmlConnection>,
    pub groups: HashMap<String, GroupInfo>,
    pub title: Option<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_type: GroupType,
    pub label: String,
    pub nodes: Vec<String>,
}

/// PlantUML 生成器
#[derive(Debug)]
pub struct PlantUmlEmitter {
    stencil_registry: Option<std::sync::Arc<crate::neotrix::l8_autonomic_impl::nt_des_data_viz::stencil_registry::StencilRegistry>>,
}

impl PlantUmlEmitter {
    pub fn new() -> Self {
        Self { stencil_registry: None }
    }

    pub fn with_stencil_registry(mut self, registry: std::sync::Arc<crate::neotrix::l8_autonomic_impl::nt_des_data_viz::stencil_registry::StencilRegistry>) -> Self {
        self.stencil_registry = Some(registry);
        self
    }

    /// 从架构模板生成
    pub fn from_template(
        &self,
        template: &crate::neotrix::l8_autonomic_impl::nt_des_data_viz::architecture_templates::ArchitectureTemplate,
    ) -> PlantUmlDocument {
        let mut doc = PlantUmlDocument {
            direction: Direction::LeftToRight,
            nodes: Vec::new(),
            connections: Vec::new(),
            groups: HashMap::new(),
            title: Some(template.name.clone()),
            footer: Some(format!("Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"))),
        };

        for node in &template.nodes {
            doc.nodes.push(PlantUmlNode {
                alias: node.alias.clone(),
                stencil_category: node.stencil_category.clone(),
                stencil_name: node.stencil_name.clone(),
                label: node.label.clone(),
                group: None,
            });
        }

        for conn in &template.connections {
            doc.connections.push(PlantUmlConnection {
                from: conn.from.clone(),
                to: conn.to.clone(),
                connection_type: conn.connection_type,
                label: conn.label.clone(),
            });
        }

        doc
    }

    /// 生成 PlantUML 字符串
    pub fn emit(&self, doc: &PlantUmlDocument) -> String {
        let mut output = String::new();
        
        output.push_str("@startuml\n");
        
        // Direction
        match doc.direction {
            Direction::LeftToRight => output.push_str("left to right direction\n"),
            Direction::TopToBottom => output.push_str("top to bottom direction\n"),
            Direction::RightToLeft => output.push_str("right to left direction\n"),
            Direction::BottomToTop => output.push_str("bottom to top direction\n"),
        }

        // Title
        if let Some(title) = &doc.title {
            output.push_str(&format!("title {}\n", title));
        }

        // Skinparam for auto colors
        output.push_str("skinparam defaultFontName Arial\n");
        output.push_str("skinparam shadowing false\n");
        output.push_str("' Colors applied automatically by mxgraph stencils\n");

        // Groups
        for (group_name, group_info) in &doc.groups {
            let group_keyword = match group_info.group_type {
                GroupType::Rectangle => "rectangle",
                GroupType::Package => "package",
                GroupType::Frame => "frame",
                GroupType::Cloud => "cloud",
                GroupType::Database => "database",
            };
            output.push_str(&format!("{} \"{}\" {{\n", group_keyword, group_info.label));
            for node_alias in &group_info.nodes {
                output.push_str(&format!("  {}\n", node_alias));
            }
            output.push_str("}\n");
        }

        // Nodes (mxgraph stencil syntax)
        for node in &doc.nodes {
            let stencil_ref = if let Some(ref registry) = self.stencil_registry {
                registry.plantuml_ref(&node.stencil_category, &node.stencil_name, &node.label, &node.alias)
            } else {
                format!("mxgraph.{}.{} \"{}\" as {}", node.stencil_category, node.stencil_name, node.label, node.alias)
            };
            output.push_str(&format!("{}\n", stencil_ref));
        }

        // Connections
        for conn in &doc.connections {
            let arrow = match conn.connection_type {
                ConnectionType::Solid => "-->",
                ConnectionType::Dashed => "..>",
                ConnectionType::Bidirectional => "--",
                ConnectionType::Labeled => "-->",
                ConnectionType::Dotted => "-.-",
                ConnectionType::Bold => "==>",
            };
            
            let label = conn.label.as_ref().map(|l| format!(" : \"{}\"", l)).unwrap_or_default();
            output.push_str(&format!("{} {} {}{}\n", conn.from, arrow, conn.to, label));
        }

        // Footer
        if let Some(footer) = &doc.footer {
            output.push_str(&format!("footer {}\n", footer));
        }

        output.push_str("@enduml\n");
        
        output
    }

    /// 从任意节点/连接生成
    pub fn emit_custom(
        &self,
        direction: Direction,
        nodes: Vec<PlantUmlNode>,
        connections: Vec<PlantUmlConnection>,
        groups: HashMap<String, GroupInfo>,
        title: Option<String>,
    ) -> String {
        let doc = PlantUmlDocument {
            direction,
            nodes,
            connections,
            groups,
            title,
            footer: Some(format!("Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC"))),
        };
        self.emit(&doc)
    }

    /// 验证 PlantUML 语法 (基础)
    pub fn validate(&self, plantuml: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if !plantuml.contains("@startuml") {
            result.add_error("@startuml missing");
        }
        if !plantuml.contains("@enduml") {
            result.add_error("@enduml missing");
        }
        if !plantuml.contains("mxgraph.") {
            result.add_warning("No mxgraph stencil references found");
        }
        
        result
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let emitter = PlantUmlEmitter::new();
        
        // Test 1: Basic emission
        let doc = PlantUmlDocument {
            direction: Direction::LeftToRight,
            nodes: vec![
                PlantUmlNode { alias: "s3".to_string(), stencil_category: "aws4".to_string(), stencil_name: "s3".to_string(), label: "S3".to_string(), group: None },
                PlantUmlNode { alias: "glue".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue".to_string(), label: "Glue".to_string(), group: None },
            ],
            connections: vec![
                PlantUmlConnection { from: "s3".to_string(), to: "glue".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            groups: HashMap::new(),
            title: Some("Test".to_string()),
            footer: None,
        };
        
        let output = emitter.emit(&doc);
        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("left to right direction"));
        assert!(output.contains("mxgraph.aws4.s3"));
        assert!(output.contains("mxgraph.aws4.glue"));
        assert!(output.contains("s3 --> glue"));
        
        // Test 2: Connection types
        let conn = PlantUmlConnection { from: "a".to_string(), to: "b".to_string(), connection_type: ConnectionType::Dashed, label: Some("async".to_string()) };
        let doc2 = PlantUmlDocument { direction: Direction::LeftToRight, nodes: vec![], connections: vec![conn], groups: HashMap::new(), title: None, footer: None };
        let output2 = emitter.emit(&doc2);
        assert!(output2.contains("a ..> b : \"async\""));
        
        // Test 3: Groups
        let mut groups = HashMap::new();
        groups.insert("zone1".to_string(), GroupInfo { group_type: GroupType::Rectangle, label: "Zone 1".to_string(), nodes: vec!["a".to_string(), "b".to_string()] });
        let doc3 = PlantUmlDocument { direction: Direction::LeftToRight, nodes: vec![], connections: vec![], groups, title: None, footer: None };
        let output3 = emitter.emit(&doc3);
        assert!(output3.contains("rectangle \"Zone 1\""));
        
        // Test 4: Validation
        let result = emitter.validate("@startuml\na --> b\n@enduml");
        assert!(!result.has_errors());
        
        let result = emitter.validate("a --> b");
        assert!(result.has_errors());
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ValidationResult {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self { Self::default() }
    pub fn add_error(&mut self, msg: &str) { self.errors.push(msg.to_string()); }
    pub fn add_warning(&mut self, msg: &str) { self.warnings.push(msg.to_string()); }
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
    pub fn has_warnings(&self) -> bool { !self.warnings.is_empty() }
}

impl Default for PlantUmlEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_creation() {
        let emitter = PlantUmlEmitter::new();
        assert!(emitter.stencil_registry.is_none());
    }

    #[test]
    fn test_basic_emission() {
        let emitter = PlantUmlEmitter::new();
        let doc = PlantUmlDocument {
            direction: Direction::LeftToRight,
            nodes: vec![PlantUmlNode { alias: "a".to_string(), stencil_category: "aws4".to_string(), stencil_name: "s3".to_string(), label: "S3".to_string(), group: None }],
            connections: vec![],
            groups: HashMap::new(),
            title: Some("Test".to_string()),
            footer: None,
        };
        let output = emitter.emit(&doc);
        assert!(output.contains("@startuml"));
        assert!(output.contains("@enduml"));
        assert!(output.contains("mxgraph.aws4.s3"));
    }

    #[test]
    fn test_connection_types() {
        let emitter = PlantUmlEmitter::new();
        let conn = PlantUmlConnection { from: "a".to_string(), to: "b".to_string(), connection_type: ConnectionType::Dashed, label: Some("test".to_string()) };
        let doc = PlantUmlDocument { direction: Direction::LeftToRight, nodes: vec![], connections: vec![conn], groups: HashMap::new(), title: None, footer: None };
        let output = emitter.emit(&doc);
        assert!(output.contains("a ..> b : \"test\""));
    }

    #[test]
    fn test_self_test_passes() {
        assert!(PlantUmlEmitter::self_test().is_ok());
    }
}