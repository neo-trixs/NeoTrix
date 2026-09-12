//! Excel 解析能力 SelfTest (T1-T3)
//!
//! T1: 模块存在性测试 (编译即证明)
//! T2: 注册到 SelfTestRegistry
//! T3: 生产路径测试 (实际调用检测函数)

use crate::neotrix::nt_file_ability::selftest::SelfTest;

/// Excel 解析能力自检
pub struct ExcelSelfTest;

impl SelfTest for ExcelSelfTest {
    fn name(&self) -> &str {
        "nt_file_ability::excel"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // T1: 模块存在性 (编译即证明)

        // T3: 生产路径测试
        // 1. ConfigFields::parse
        let config = crate::neotrix::nt_file_ability::config_parser::ConfigFields::parse(
            "执行标准:美标,压力:150LB",
        );
        if config.exec_std.as_deref() != Some("美标") {
            errors.push("ConfigFields::parse 执行标准解析失败".to_string());
        }
        if config.pressure.as_deref() != Some("150LB") {
            errors.push("ConfigFields::parse 压力解析失败".to_string());
        }

        // 2. ColumnMap::detect
        let header = vec![
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
        ];
        let (template, _map) =
            crate::neotrix::nt_file_ability::template_engine::ColumnMap::detect(&header);
        if template
            != crate::neotrix::nt_file_ability::template_engine::TemplateType::PurchaseRequest
        {
            errors.push(format!(
                "ColumnMap::detect 采购申请单模板检测失败, got {:?}",
                template
            ));
        }

        // 3. PathMetadata
        let meta =
            crate::neotrix::nt_file_ability::path_metadata::PathMetadata::from_salesperson_order(
                "段留杰",
                "4.01菲律宾 WSD-I-26031303",
            );
        if meta.country.as_deref() != Some("菲律宾") {
            errors.push(format!("PathMetadata 国家提取失败, got {:?}", meta.country));
        }

        // 4. TablePresenter
        let table = crate::neotrix::nt_file_ability::types::TableData {
            name: "test".into(),
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()]],
        };
        let md = crate::neotrix::nt_file_ability::table_presenter::TablePresenter::new(&table)
            .to_markdown();
        if !md.contains("| A | B |") {
            errors.push("TablePresenter Markdown 输出失败".to_string());
        }

        // 5. read_xlsx_fast 存在性 (泛型函数无法转为 fn 指针，跳过)
        // (函数存在性由编译器保证)

        // 6. chunk_planner 分块
        let chunks = crate::neotrix::nt_file_ability::chunk_planner::chunk_table(&table);
        if chunks.is_empty() {
            errors.push("chunk_planner::chunk_table 返回空分块".to_string());
        }

        // 7. excel_tool_schema Function Calling Schema
        let schema = super::excel_tool_schema::parse_excel_schema();
        if schema["name"] != "parse_excel" {
            errors.push("excel_tool_schema 名称不匹配".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 注册 Excel SelfTest 到主 registry
pub fn register_excel_self_tests(registry: &mut crate::neotrix::nt_file_ability::selftest::SelfTestRegistry) {
    registry.register(Box::new(ExcelSelfTest));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_file_ability::selftest::SelfTestRegistry;

    #[test]
    fn test_excel_selftest() {
        let test = ExcelSelfTest;
        assert_eq!(test.name(), "nt_file_ability::excel");
        assert!(test.self_test().is_ok());
    }

    #[test]
    fn test_register_excel_selftest() {
        let mut registry = SelfTestRegistry::new();
        register_excel_self_tests(&mut registry);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_chunk_planner_existence() {
        let table = crate::neotrix::nt_file_ability::types::TableData {
            name: "test".into(),
            headers: vec!["A".into()],
            rows: vec![vec!["1".into()]],
        };
        let chunks = crate::neotrix::nt_file_ability::chunk_planner::chunk_table(&table);
        assert!(!chunks.is_empty(), "chunk_table 应返回非空分块");
    }

    #[test]
    fn test_excel_tool_schema_existence() {
        let schema = super::super::excel_tool_schema::parse_excel_schema();
        assert_eq!(schema["name"], "parse_excel");
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("file_path")));
    }

    #[test]
    fn test_config_parse_robustness() {
        // 测试各种边界情况
        let cases = vec![
            "",
            " ",
            "执行标准:美标",
            "执行标准:美标,压力:150LB",
            "执行标准:美标;压力:150LB",
            "执行标准:美标；压力:150LB",
            ":",
            ":::",
            "key:value,key:value",
        ];
        for case in cases {
            let config =
                crate::neotrix::nt_file_ability::config_parser::ConfigFields::parse(case);
            // 不应 panic
            let _ = config.is_empty();
        }
    }

    #[test]
    fn test_template_detect_robustness() {
        // 测试各种表头情况
        let cases = vec![
            vec![],
            vec!["A".into()],
            vec![
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
            ],
            vec![
                "序号".into(),
                "产品名称".into(),
                "型号".into(),
                "口径".into(),
                "配置".into(),
                "订单数量".into(),
                "单位".into(),
                "单价/元".into(),
                "总价/元".into(),
                "单重kg".into(),
                "总重kg".into(),
            ],
            vec!["列A".into(), "列B".into(), "列C".into()],
        ];
        for case in cases {
            let (_, _) =
                crate::neotrix::nt_file_ability::template_engine::ColumnMap::detect(&case);
            // 不应 panic
        }
    }

    #[test]
    fn test_path_metadata_robustness() {
        // 测试各种路径情况
        let cases = vec![
            "",
            "4.01菲律宾 WSD-I-26031303",
            "4.21印尼Alvin WSD-I-26041403",
            "4.02危地马拉 Erson WSD-I-26032701",
            "WSD-I-26031303",
            "没有订单号的文件夹",
        ];
        for case in cases {
            let _ = crate::neotrix::nt_file_ability::path_metadata::PathMetadata::from_salesperson_order("测试", case);
            // 不应 panic
        }
    }
}
