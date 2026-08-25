//! Architecture Templates — markdown-viewer/skills 8 架构类型吸收
//! 
//! 预定义架构模板: Data Lake, Streaming, Warehouse, ETL, Log Analytics, ML Feature Store, CDC, Multi-source BI

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 架构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureType {
    DataLake,
    RealTimeStreaming,
    DataWarehouse,
    EtlPipeline,
    LogAnalytics,
    MlFeatureStore,
    CdcPipeline,
    MultiSourceBi,
}

/// 连接类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionType {
    Solid,      // -->
    Dashed,     // ..>
    Bidirectional, // --
    Labeled,    // --> : "label"
}

/// 模板节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateNode {
    pub id: String,
    pub stencil_category: String,
    pub stencil_name: String,
    pub label: String,
    pub alias: String,
}

/// 模板连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConnection {
    pub from: String,
    pub to: String,
    pub connection_type: ConnectionType,
    pub label: Option<String>,
}

/// 架构模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureTemplate {
    pub arch_type: ArchitectureType,
    pub name: String,
    pub description: String,
    pub nodes: Vec<TemplateNode>,
    pub connections: Vec<TemplateConnection>,
    pub example_file: String,
}

/// 模板库
#[derive(Debug)]
pub struct TemplateLibrary {
    templates: HashMap<ArchitectureType, ArchitectureTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        let mut lib = Self {
            templates: HashMap::new(),
        };
        lib.register_defaults();
        lib
    }

    fn register_defaults(&mut self) {
        // Data Lake
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::DataLake,
            name: "Data Lake".to_string(),
            description: "Centralized raw data store with governance".to_string(),
            nodes: vec![
                TemplateNode { id: "s3".to_string(), stencil_category: "aws4".to_string(), stencil_name: "s3".to_string(), label: "Data Lake\n(S3)".to_string(), alias: "s3".to_string() },
                TemplateNode { id: "lake_formation".to_string(), stencil_category: "aws4".to_string(), stencil_name: "lake_formation".to_string(), label: "Lake Formation".to_string(), alias: "lf".to_string() },
                TemplateNode { id: "glue".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue".to_string(), label: "Glue\nETL".to_string(), alias: "glue".to_string() },
                TemplateNode { id: "athena".to_string(), stencil_category: "aws4".to_string(), stencil_name: "athena".to_string(), label: "Athena".to_string(), alias: "athena".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "s3".to_string(), to: "glue".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "glue".to_string(), to: "athena".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "s3".to_string(), to: "lake_formation".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/data-lake.md".to_string(),
        });

        // Real-time Streaming
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::RealTimeStreaming,
            name: "Real-time Streaming".to_string(),
            description: "Event stream processing with Kinesis/MSK".to_string(),
            nodes: vec![
                TemplateNode { id: "kinesis".to_string(), stencil_category: "aws4".to_string(), stencil_name: "kinesis".to_string(), label: "Kinesis\nData Streams".to_string(), alias: "kinesis".to_string() },
                TemplateNode { id: "msk".to_string(), stencil_category: "aws4".to_string(), stencil_name: "msk".to_string(), label: "MSK\n(Kafka)".to_string(), alias: "msk".to_string() },
                TemplateNode { id: "lambda".to_string(), stencil_category: "aws4".to_string(), stencil_name: "lambda_function".to_string(), label: "Lambda".to_string(), alias: "lambda".to_string() },
                TemplateNode { id: "opensearch".to_string(), stencil_category: "aws4".to_string(), stencil_name: "opensearch_service_data_node".to_string(), label: "OpenSearch".to_string(), alias: "os".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "kinesis".to_string(), to: "lambda".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "msk".to_string(), to: "lambda".to_string(), connection_type: ConnectionType::Dashed, label: None },
                TemplateConnection { from: "lambda".to_string(), to: "opensearch".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/real-time-streaming.md".to_string(),
        });

        // Data Warehouse
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::DataWarehouse,
            name: "Data Warehouse".to_string(),
            description: "Star-schema analytics with Redshift".to_string(),
            nodes: vec![
                TemplateNode { id: "redshift".to_string(), stencil_category: "aws4".to_string(), stencil_name: "redshift".to_string(), label: "Redshift".to_string(), alias: "rs".to_string() },
                TemplateNode { id: "glue".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue".to_string(), label: "Glue".to_string(), alias: "glue".to_string() },
                TemplateNode { id: "quicksight".to_string(), stencil_category: "aws4".to_string(), stencil_name: "quicksight".to_string(), label: "QuickSight".to_string(), alias: "qs".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "glue".to_string(), to: "redshift".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "redshift".to_string(), to: "quicksight".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/data-warehouse.md".to_string(),
        });

        // ETL Pipeline
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::EtlPipeline,
            name: "ETL Pipeline".to_string(),
            description: "Extract-Transform-Load with Glue".to_string(),
            nodes: vec![
                TemplateNode { id: "s3".to_string(), stencil_category: "aws4".to_string(), stencil_name: "s3".to_string(), label: "S3\n(Source)".to_string(), alias: "s3_src".to_string() },
                TemplateNode { id: "glue".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue".to_string(), label: "Glue\nETL".to_string(), alias: "glue".to_string() },
                TemplateNode { id: "crawlers".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue_crawlers".to_string(), label: "Glue Crawlers".to_string(), alias: "crawlers".to_string() },
                TemplateNode { id: "catalog".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue_data_catalog".to_string(), label: "Data Catalog".to_string(), alias: "catalog".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "s3".to_string(), to: "crawlers".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "crawlers".to_string(), to: "catalog".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "catalog".to_string(), to: "glue".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/etl-pipeline.md".to_string(),
        });

        // Log Analytics
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::LogAnalytics,
            name: "Log Analytics".to_string(),
            description: "Centralized logging with OpenSearch".to_string(),
            nodes: vec![
                TemplateNode { id: "firehose".to_string(), stencil_category: "aws4".to_string(), stencil_name: "kinesis_data_firehose".to_string(), label: "Firehose".to_string(), alias: "fh".to_string() },
                TemplateNode { id: "opensearch".to_string(), stencil_category: "aws4".to_string(), stencil_name: "opensearch_service".to_string(), label: "OpenSearch".to_string(), alias: "os".to_string() },
                TemplateNode { id: "lambda".to_string(), stencil_category: "aws4".to_string(), stencil_name: "lambda_function".to_string(), label: "Lambda".to_string(), alias: "lambda".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "firehose".to_string(), to: "opensearch".to_string(), connection_type: ConnectionType::Dashed, label: None },
                TemplateConnection { from: "lambda".to_string(), to: "opensearch".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/log-analytics.md".to_string(),
        });

        // ML Feature Store
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::MlFeatureStore,
            name: "ML Feature Store".to_string(),
            description: "Feature engineering pipeline".to_string(),
            nodes: vec![
                TemplateNode { id: "glue".to_string(), stencil_category: "aws4".to_string(), stencil_name: "glue".to_string(), label: "Glue".to_string(), alias: "glue".to_string() },
                TemplateNode { id: "s3".to_string(), stencil_category: "aws4".to_string(), stencil_name: "s3".to_string(), label: "S3\n(Features)".to_string(), alias: "s3_feat".to_string() },
                TemplateNode { id: "athena".to_string(), stencil_category: "aws4".to_string(), stencil_name: "athena".to_string(), label: "Athena".to_string(), alias: "athena".to_string() },
                TemplateNode { id: "emr".to_string(), stencil_category: "aws4".to_string(), stencil_name: "emr".to_string(), label: "EMR\n(Spark)".to_string(), alias: "emr".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "glue".to_string(), to: "s3".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "s3".to_string(), to: "athena".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "s3".to_string(), to: "emr".to_string(), connection_type: ConnectionType::Dashed, label: None },
            ],
            example_file: "examples/ml-feature-pipeline.md".to_string(),
        });

        // CDC Pipeline
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::CdcPipeline,
            name: "CDC Pipeline".to_string(),
            description: "Database change capture to analytics".to_string(),
            nodes: vec![
                TemplateNode { id: "dynamodb_streams".to_string(), stencil_category: "aws4".to_string(), stencil_name: "dynamodb_streams".to_string(), label: "DynamoDB\nStreams".to_string(), alias: "ddb_str".to_string() },
                TemplateNode { id: "kinesis".to_string(), stencil_category: "aws4".to_string(), stencil_name: "kinesis".to_string(), label: "Kinesis".to_string(), alias: "kinesis".to_string() },
                TemplateNode { id: "lambda".to_string(), stencil_category: "aws4".to_string(), stencil_name: "lambda_function".to_string(), label: "Lambda".to_string(), alias: "lambda".to_string() },
                TemplateNode { id: "redshift".to_string(), stencil_category: "aws4".to_string(), stencil_name: "redshift".to_string(), label: "Redshift".to_string(), alias: "rs".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "dynamodb_streams".to_string(), to: "kinesis".to_string(), connection_type: ConnectionType::Dashed, label: None },
                TemplateConnection { from: "kinesis".to_string(), to: "lambda".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "lambda".to_string(), to: "redshift".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/cdc-pipeline.md".to_string(),
        });

        // Multi-source BI
        self.register(ArchitectureTemplate {
            arch_type: ArchitectureType::MultiSourceBi,
            name: "Multi-source BI".to_string(),
            description: "Cross-database reporting".to_string(),
            nodes: vec![
                TemplateNode { id: "aurora".to_string(), stencil_category: "aws4".to_string(), stencil_name: "aurora".to_string(), label: "Aurora\n(PostgreSQL)".to_string(), alias: "aurora".to_string() },
                TemplateNode { id: "dynamodb".to_string(), stencil_category: "aws4".to_string(), stencil_name: "dynamodb".to_string(), label: "DynamoDB".to_string(), alias: "ddb".to_string() },
                TemplateNode { id: "redshift".to_string(), stencil_category: "aws4".to_string(), stencil_name: "redshift".to_string(), label: "Redshift".to_string(), alias: "rs".to_string() },
                TemplateNode { id: "quicksight".to_string(), stencil_category: "aws4".to_string(), stencil_name: "quicksight".to_string(), label: "QuickSight".to_string(), alias: "qs".to_string() },
            ],
            connections: vec![
                TemplateConnection { from: "aurora".to_string(), to: "redshift".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "dynamodb".to_string(), to: "redshift".to_string(), connection_type: ConnectionType::Solid, label: None },
                TemplateConnection { from: "redshift".to_string(), to: "quicksight".to_string(), connection_type: ConnectionType::Solid, label: None },
            ],
            example_file: "examples/multi-source-bi.md".to_string(),
        });
    }

    fn register(&mut self, template: ArchitectureTemplate) {
        self.templates.insert(template.arch_type, template);
    }

    /// 获取模板
    pub fn get(&self, arch_type: ArchitectureType) -> Option<&ArchitectureTemplate> {
        self.templates.get(&arch_type)
    }

    /// 生成 PlantUML
    pub fn generate_plantuml(&self, arch_type: ArchitectureType, stencil_registry: &crate::neotrix::l8_autonomic_impl::nt_des_data_viz::stencil_registry::StencilRegistry) -> Option<String> {
        let template = self.templates.get(&arch_type)?;
        let mut plantuml = String::new();
        
        plantuml.push_str("@startuml\n");
        plantuml.push_str("left to right direction\n");
        
        // Nodes
        for node in &template.nodes {
            let stencil_ref = stencil_registry.plantuml_ref(&node.stencil_category, &node.stencil_name, &node.label, &node.alias);
            plantuml.push_str(&format!("{}\n", stencil_ref));
        }
        
        // Connections
        for conn in &template.connections {
            let arrow = match conn.connection_type {
                ConnectionType::Solid => "-->",
                ConnectionType::Dashed => "..>",
                ConnectionType::Bidirectional => "--",
                ConnectionType::Labeled => "-->",
            };
            let label = conn.label.as_ref().map(|l| format!(" : \"{}\"", l)).unwrap_or_default();
            plantuml.push_str(&format!("{} {} {}{}\n", conn.from, arrow, conn.to, label));
        }
        
        plantuml.push_str("@enduml\n");
        
        Some(plantuml)
    }

    /// 列出所有模板
    pub fn list_templates(&self) -> Vec<&ArchitectureTemplate> {
        self.templates.values().collect()
    }

    /// SelfTest for C1 promotion
    pub fn self_test() -> Result<(), String> {
        let library = TemplateLibrary::new();
        
        // Test 1: All 8 templates registered
        assert_eq!(library.templates.len(), 8);
        
        // Test 2: Key templates present
        assert!(library.templates.contains_key(&ArchitectureType::DataLake));
        assert!(library.templates.contains_key(&ArchitectureType::RealTimeStreaming));
        assert!(library.templates.contains_key(&ArchitectureType::DataWarehouse));
        assert!(library.templates.contains_key(&ArchitectureType::EtlPipeline));
        assert!(library.templates.contains_key(&ArchitectureType::LogAnalytics));
        assert!(library.templates.contains_key(&ArchitectureType::MlFeatureStore));
        assert!(library.templates.contains_key(&ArchitectureType::CdcPipeline));
        assert!(library.templates.contains_key(&ArchitectureType::MultiSourceBi));
        
        // Test 3: Each template has nodes and connections
        for template in library.templates.values() {
            assert!(!template.nodes.is_empty(), "Template {} has no nodes", template.name);
            assert!(!template.connections.is_empty(), "Template {} has no connections", template.name);
            assert!(!template.example_file.is_empty(), "Template {} has no example file", template.name);
        }
        
        Ok(())
    }
}

impl Default for TemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_library_creation() {
        let library = TemplateLibrary::new();
        assert_eq!(library.templates.len(), 8);
    }

    #[test]
    fn test_get_template() {
        let library = TemplateLibrary::new();
        let template = library.get(ArchitectureType::DataLake).unwrap();
        assert_eq!(template.name, "Data Lake");
        assert_eq!(template.nodes.len(), 4);
        assert_eq!(template.connections.len(), 3);
    }

    #[test]
    fn test_self_test_passes() {
        assert!(TemplateLibrary::self_test().is_ok());
    }
}