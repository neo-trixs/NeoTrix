//! Excel 能力树注册节点
//!
//! 将 Excel 解析能力注册到 CapabilityRegistry:
//! - L0Primitive: 基础解析 (xlsx_read, csv_read)
//! - L1Composite: 组合能力 (template_detect, parse_config, extract_metadata)
//! - L2Orchestrator: 编排能力 (auto_parse)
//!
//! 遵循 R-P42: 强化现有节点, 不平行重造枚举。
//! 遵循 R-P1: 零 unsafe。

use nt_core_capability_tree::{CapabilityNode, Domain, NodeLayer};

use super::config_parser::ConfigFields;
use super::path_metadata::PathMetadata;
use super::template_engine::{ColumnMap, TemplateType};
use super::types::TableData;
use super::{config_parser, path_metadata, tables, template_engine, xlsx_parser};

/// Excel 能力节点定义
pub struct ExcelCapability;

impl ExcelCapability {
    /// L0 基础能力: XLSX 读取
    pub fn xlsx_read(path: &str) -> Result<Vec<TableData>, String> {
        xlsx_parser::parse_xlsx(path).map_err(|e| format!("XLSX 读取失败: {e}"))
    }

    /// L0 基础能力: CSV 读取
    pub fn csv_read(path: &str) -> Result<TableData, String> {
        tables::read_csv(path).map_err(|e| format!("CSV 读取失败: {e}"))
    }

    /// L1 组合能力: 模板检测
    pub fn detect_template(grid: &[Vec<String>]) -> Option<(TemplateType, ColumnMap)> {
        template_engine::auto_detect(grid).map(|(t, m, _)| (t, m))
    }

    /// L1 组合能力: 配置解析
    pub fn parse_config(raw: &str) -> ConfigFields {
        config_parser::ConfigFields::parse(raw)
    }

    /// L1 组合能力: 路径元数据
    pub fn extract_metadata(path: &std::path::Path) -> PathMetadata {
        path_metadata::PathMetadata::from_path(path)
    }

    /// L2 编排能力: 自动解析 (读取+检测+提取)
    pub fn auto_parse(path: &str) -> Result<ExcelParseResult, String> {
        let tables = Self::xlsx_read(path)?;
        let table = tables.first().ok_or("无工作表")?;

        let (template, map) = Self::detect_template(&[table.headers.clone()])
            .unwrap_or((TemplateType::Unknown, Default::default()));

        let metadata = Self::extract_metadata(std::path::Path::new(path));

        Ok(ExcelParseResult {
            template_type: template,
            column_map: map,
            metadata,
            table: table.clone(),
        })
    }

    /// 注册 Excel 能力节点到能力树
    pub fn register_nodes(registry: &mut nt_core_capability_tree::CapabilityRegistry) {
        // L0 原语节点: 基础解析能力
        let xlsx_node = CapabilityNode::new_primitive(
            "nt_file_ability::excel::xlsx_read".into(),
            Domain::Io,
            vec!["xlsx_read".into()],
        );
        let csv_node = CapabilityNode::new_primitive(
            "nt_file_ability::excel::csv_read".into(),
            Domain::Io,
            vec!["csv_read".into()],
        );

        // L1 组合节点: 组合基础能力
        let template_node = CapabilityNode::new_composite(
            "nt_file_ability::excel::template_detect".into(),
            Domain::Io,
            NodeLayer::L1Composite,
            vec!["template_detect".into()],
            vec!["xlsx_read".into()],
        );
        let config_node = CapabilityNode::new_composite(
            "nt_file_ability::excel::parse_config".into(),
            Domain::Io,
            NodeLayer::L1Composite,
            vec!["parse_config".into()],
            vec![],
        );
        let metadata_node = CapabilityNode::new_composite(
            "nt_file_ability::excel::extract_metadata".into(),
            Domain::Io,
            NodeLayer::L1Composite,
            vec!["extract_metadata".into()],
            vec![],
        );

        // L2 编排节点: 组合 L0 + L1
        let auto_parse_node = CapabilityNode::new_composite(
            "nt_file_ability::excel::auto_parse".into(),
            Domain::Io,
            NodeLayer::L2Orchestrator,
            vec!["auto_parse".into()],
            vec![
                "xlsx_read".into(),
                "template_detect".into(),
                "extract_metadata".into(),
            ],
        );

        // 逐个注册 (跳过已存在的节点)
        for node in [
            xlsx_node,
            csv_node,
            template_node,
            config_node,
            metadata_node,
            auto_parse_node,
        ] {
            let _ = registry.register(node);
        }

        // 建立依赖关系
        let _ = registry.add_dependency(
            "nt_file_ability::excel::auto_parse",
            "nt_file_ability::excel::xlsx_read",
        );
        let _ = registry.add_dependency(
            "nt_file_ability::excel::auto_parse",
            "nt_file_ability::excel::template_detect",
        );
        let _ = registry.add_dependency(
            "nt_file_ability::excel::auto_parse",
            "nt_file_ability::excel::extract_metadata",
        );
        let _ = registry.add_dependency(
            "nt_file_ability::excel::template_detect",
            "nt_file_ability::excel::xlsx_read",
        );
    }
}

/// Excel 自动解析结果
#[derive(Debug, Clone)]
pub struct ExcelParseResult {
    pub template_type: TemplateType,
    pub column_map: ColumnMap,
    pub metadata: PathMetadata,
    pub table: TableData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config = ExcelCapability::parse_config("执行标准:美标,压力:150LB");
        assert_eq!(config.exec_std.as_deref(), Some("美标"));
        assert_eq!(config.pressure.as_deref(), Some("150LB"));
    }

    #[test]
    fn test_detect_template_purchase_request() {
        let grid = vec![vec![
            "序号".into(),
            "名称".into(),
            "口径".into(),
            "配置".into(),
            "数量".into(),
            "单位".into(),
            "单价/元".into(),
            "总价/元".into(),
            "单重kg".into(),
            "总重kg".into(),
        ]];
        let result = ExcelCapability::detect_template(&grid);
        assert!(result.is_some());
        let (template, map) = result.unwrap();
        assert_eq!(template, TemplateType::PurchaseRequest);
        assert_eq!(map.serial, Some(0));
        assert_eq!(map.name, Some(1));
        assert_eq!(map.price, Some(6));
    }

    #[test]
    fn test_extract_metadata_from_path() {
        let path = std::path::Path::new("/orders/4.01菲律宾 WSD-I-26031303/报价.xlsx");
        let meta = ExcelCapability::extract_metadata(path);
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26031303"));
    }
}
