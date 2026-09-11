//! 合同模板格式检测器
//!
//! 通过分析 Excel 行5的表头，自动识别合同模板格式:
//! - "威斯迪采购申请单": 序号|名称|口径|配置|数量|单位|单价|总价|单重|总重
//! - "采购清单": 序号|产品名称|型号|口径|配置|数量|单位|单价|总价|单重|总重

/// 合同模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractTemplate {
    /// 威斯迪采购申请单 (标准模板，有"配置"列)
    PurchaseRequest,
    /// 采购清单 (简化模板，有"产品名称"和"型号"列)
    PurchaseList,
    /// 未知模板
    Unknown,
}

/// 从表头行检测模板类型
pub fn detect_template(header_row: &[String]) -> ContractTemplate {
    // 检测"采购申请单"特征: 有"名称"列
    if header_row.iter().any(|h| h == "名称")
        && header_row.iter().any(|h| h == "配置")
        && header_row.iter().any(|h| h.contains("单价"))
    {
        return ContractTemplate::PurchaseRequest;
    }

    // 检测"采购清单"特征: 有"产品名称"和"型号"列
    if header_row.iter().any(|h| h == "产品名称")
        && header_row.iter().any(|h| h == "型号")
        && header_row.iter().any(|h| h.contains("单价"))
    {
        return ContractTemplate::PurchaseList;
    }

    ContractTemplate::Unknown
}

/// 模板对应的列映射
#[derive(Debug, Clone, Default)]
pub struct TemplateColumnMap {
    pub serial: Option<usize>,
    pub name: Option<usize>,
    pub model: Option<usize>,
    pub diameter: Option<usize>,
    pub config: Option<usize>,
    pub qty: Option<usize>,
    pub unit: Option<usize>,
    pub price: Option<usize>,
    pub amount: Option<usize>,
    pub unit_weight: Option<usize>,
    pub total_weight: Option<usize>,
}

/// 根据模板类型生成列映射
pub fn build_column_map(template: ContractTemplate, header_row: &[String]) -> TemplateColumnMap {
    let mut map = TemplateColumnMap::default();

    for (i, h) in header_row.iter().enumerate() {
        let h = h.trim();
        match h {
            "序号" => map.serial = Some(i),
            "口径" => map.diameter = Some(i),
            "配置" => map.config = Some(i),
            "单位" => map.unit = Some(i),
            h if h.starts_with("单价") => map.price = Some(i),
            h if h.starts_with("总价") => map.amount = Some(i),
            h if h.starts_with("单重") => map.unit_weight = Some(i),
            h if h.starts_with("总重") => map.total_weight = Some(i),
            _ => {}
        }

        match template {
            ContractTemplate::PurchaseRequest => {
                if h == "名称" {
                    map.name = Some(i);
                }
                if h == "数量" {
                    map.qty = Some(i);
                }
            }
            ContractTemplate::PurchaseList => {
                if h == "产品名称" {
                    map.name = Some(i);
                }
                if h == "型号" {
                    map.model = Some(i);
                }
                if h == "订单数量" {
                    map.qty = Some(i);
                }
            }
            _ => {}
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_purchase_request() {
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
        assert_eq!(detect_template(&header), ContractTemplate::PurchaseRequest);
    }

    #[test]
    fn test_detect_purchase_list() {
        let header = vec![
            "序号".into(),
            "产品名称".into(),
            "型号".into(),
            "口径".into(),
            "配置".into(),
            "数量".into(),
            "单位".into(),
            "单价/元".into(),
            "总价/元".into(),
            "单重kg".into(),
            "总重kg".into(),
        ];
        assert_eq!(detect_template(&header), ContractTemplate::PurchaseList);
    }

    #[test]
    fn test_detect_unknown() {
        let header = vec!["序号".into(), "描述".into(), "金额".into()];
        assert_eq!(detect_template(&header), ContractTemplate::Unknown);
    }
}
