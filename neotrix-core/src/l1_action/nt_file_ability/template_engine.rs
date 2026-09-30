//! 合同模板检测引擎 (D2+D3)
//!
//! 自动识别 XLSX 中的合同模板格式，返回标准化列映射。
//! 设计为通用能力: 只看表头行，不绑定具体业务逻辑。
//!
//! 支持的模板:
//! - **采购申请单**: 序号|名称|口径|配置|数量|单位|单价|总价|单重|总重
//! - **采购清单**: 序号|产品名称|型号|口径|配置|数量|单位|单价|总价|单重|总重
//! - **通用表格**: 任意表头，返回原始列映射

/// 合同模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateType {
    /// 威斯迪采购申请单 (标准模板，有"名称"和"配置"列)
    PurchaseRequest,
    /// 采购清单 (简化模板，有"产品名称"和"型号"列)
    PurchaseList,
    /// 未知模板 (无法匹配已知格式)
    Unknown,
}

impl TemplateType {
    /// 模板名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::PurchaseRequest => "采购申请单",
            Self::PurchaseList => "采购清单",
            Self::Unknown => "未知",
        }
    }
}

/// 标准化列映射 (所有模板统一为同一套列名)
#[derive(Debug, Clone, Default)]
pub struct ColumnMap {
    pub serial: Option<usize>,       // 序号
    pub name: Option<usize>,         // 名称/产品名称
    pub model: Option<usize>,        // 规格型号 (仅采购清单)
    pub diameter: Option<usize>,     // 口径
    pub config: Option<usize>,       // 配置
    pub qty: Option<usize>,          // 数量/订单数量
    pub unit: Option<usize>,         // 单位
    pub price: Option<usize>,        // 单价
    pub amount: Option<usize>,       // 总价
    pub unit_weight: Option<usize>,  // 单重
    pub total_weight: Option<usize>, // 总重
}

impl ColumnMap {
    /// 从表头行自动检测模板类型并生成列映射
    pub fn detect(header_row: &[String]) -> (TemplateType, Self) {
        let template = Self::detect_template(header_row);
        let map = Self::build_map(template, header_row);
        (template, map)
    }

    /// 检测模板类型
    fn detect_template(header_row: &[String]) -> TemplateType {
        let has_name = header_row.iter().any(|h| h == "名称");
        let has_product_name = header_row.iter().any(|h| h == "产品名称");
        let has_config = header_row.iter().any(|h| h == "配置");
        let has_model = header_row.iter().any(|h| h == "型号");
        let has_price = header_row.iter().any(|h| h.contains("单价"));

        // 采购申请单: 有 "名称" + "配置" + "单价"
        if has_name && has_config && has_price {
            return TemplateType::PurchaseRequest;
        }

        // 采购清单: 有 "产品名称" + "型号" + "单价"
        if has_product_name && has_model && has_price {
            return TemplateType::PurchaseList;
        }

        TemplateType::Unknown
    }

    /// 根据模板类型构建列映射
    fn build_map(template: TemplateType, header_row: &[String]) -> Self {
        let mut map = Self::default();

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
                TemplateType::PurchaseRequest => {
                    if h == "名称" {
                        map.name = Some(i);
                    }
                    if h == "数量" {
                        map.qty = Some(i);
                    }
                }
                TemplateType::PurchaseList => {
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
                TemplateType::Unknown => {
                    // 通用匹配: 按关键词猜测
                    if h.contains("名称") || h.contains("产品") {
                        map.name = Some(i);
                    }
                    if h.contains("数量") || h.contains("台数") {
                        map.qty = Some(i);
                    }
                }
            }
        }

        map
    }

    /// 获取列值 (安全访问)
    pub fn get<'a>(&self, row: &'a [String], field: &ColumnMapField) -> Option<&'a str> {
        let idx = match field {
            ColumnMapField::Serial => self.serial?,
            ColumnMapField::Name => self.name?,
            ColumnMapField::Model => self.model?,
            ColumnMapField::Diameter => self.diameter?,
            ColumnMapField::Config => self.config?,
            ColumnMapField::Qty => self.qty?,
            ColumnMapField::Unit => self.unit?,
            ColumnMapField::Price => self.price?,
            ColumnMapField::Amount => self.amount?,
            ColumnMapField::UnitWeight => self.unit_weight?,
            ColumnMapField::TotalWeight => self.total_weight?,
        };
        row.get(idx).map(|s| s.trim()).filter(|s| !s.is_empty())
    }
}

/// 列映射字段枚举 (用于 ColumnMap::get)
#[derive(Debug, Clone, Copy)]
pub enum ColumnMapField {
    Serial,
    Name,
    Model,
    Diameter,
    Config,
    Qty,
    Unit,
    Price,
    Amount,
    UnitWeight,
    TotalWeight,
}

/// 从 XLSX grid 中找到表头行 (包含 "序号" 的行)
pub fn find_header_row(grid: &[Vec<String>]) -> Option<usize> {
    grid.iter()
        .position(|row| row.iter().any(|c| c.trim() == "序号"))
}

/// 从 XLSX grid 自动检测模板并返回 (模板类型, 列映射, 表头行索引)
pub fn auto_detect(grid: &[Vec<String>]) -> Option<(TemplateType, ColumnMap, usize)> {
    let header_idx = find_header_row(grid)?;
    let header = &grid[header_idx];
    let (template, map) = ColumnMap::detect(header);
    Some((template, map, header_idx))
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
        let (template, map) = ColumnMap::detect(&header);
        assert_eq!(template, TemplateType::PurchaseRequest);
        assert_eq!(map.serial, Some(0));
        assert_eq!(map.name, Some(1));
        assert_eq!(map.diameter, Some(2));
        assert_eq!(map.config, Some(3));
        assert_eq!(map.qty, Some(4));
        assert_eq!(map.unit, Some(5));
        assert_eq!(map.price, Some(6));
        assert_eq!(map.amount, Some(7));
        assert_eq!(map.unit_weight, Some(8));
        assert_eq!(map.total_weight, Some(9));
    }

    #[test]
    fn test_detect_purchase_list() {
        let header = vec![
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
        ];
        let (template, map) = ColumnMap::detect(&header);
        assert_eq!(template, TemplateType::PurchaseList);
        assert_eq!(map.name, Some(1));
        assert_eq!(map.model, Some(2));
        assert_eq!(map.qty, Some(5));
    }

    #[test]
    fn test_detect_unknown() {
        let header = vec!["列A".into(), "列B".into(), "列C".into()];
        let (template, _) = ColumnMap::detect(&header);
        assert_eq!(template, TemplateType::Unknown);
    }
}
