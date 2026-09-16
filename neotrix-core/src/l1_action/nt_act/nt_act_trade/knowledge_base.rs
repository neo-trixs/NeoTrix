//! Knowledge Base — 外贸知识库接口定义
//!
//! 定义产品、供应商、价格等外贸知识的统一查询与管理接口。
//! 所有 trade 子模块通过 `use super::knowledge_base::*` 引用，
//! 知识库实现方需实现 `KnowledgeBase` trait。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::full_cycle::{ProductSpec, ProductType};

// ============================================================
// 1. 知识库错误类型
// ============================================================

/// 知识库操作错误
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KnowledgeBaseError {
    /// 产品/供应商未找到
    NotFound { entity: String, id: String },
    /// 查询参数无效
    InvalidQuery { reason: String },
    /// 知识库更新冲突（如并发写入）
    Conflict { message: String },
    /// 知识库不可用（连接/存储故障）
    Unavailable { reason: String },
    /// 权限不足
    PermissionDenied { operation: String },
}

impl std::fmt::Display for KnowledgeBaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(f, "{} '{}' not found", entity, id),
            Self::InvalidQuery { reason } => write!(f, "Invalid query: {}", reason),
            Self::Conflict { message } => write!(f, "Conflict: {}", message),
            Self::Unavailable { reason } => write!(f, "Knowledge base unavailable: {}", reason),
            Self::PermissionDenied { operation } => {
                write!(f, "Permission denied for operation: {}", operation)
            }
        }
    }
}

impl std::error::Error for KnowledgeBaseError {}

/// 知识库操作结果
pub type KnowledgeResult<T> = Result<T, KnowledgeBaseError>;

// ============================================================
// 2. 查询过滤器
// ============================================================

/// 产品查询过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductFilters {
    /// 产品名称关键词（模糊匹配）
    pub name_keyword: Option<String>,
    /// 产品类型筛选
    pub product_type: Option<ProductType>,
    /// HS 编码前缀匹配
    pub hs_code_prefix: Option<String>,
    /// 所需认证筛选（AND 逻辑）
    pub required_certs: Vec<String>,
    /// 最小价格范围 (USD)
    pub min_price: Option<f64>,
    /// 最大价格范围 (USD)
    pub max_price: Option<f64>,
    /// 标签筛选（OR 逻辑）
    pub tags: Vec<String>,
    /// 分页: 每页数量
    pub page_size: Option<u32>,
    /// 分页: 页码 (0-based)
    pub page: Option<u32>,
}

/// 供应商查询过滤器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupplierFilters {
    /// 供应商名称关键词（模糊匹配）
    pub name_keyword: Option<String>,
    /// 国家/地区筛选
    pub country: Option<String>,
    /// 产品类别筛选
    pub product_categories: Vec<String>,
    /// 最小评分
    pub min_rating: Option<f64>,
    /// 是否只显示已认证供应商
    pub certified_only: bool,
    /// 产能范围筛选
    pub min_capacity: Option<u64>,
    pub max_capacity: Option<u64>,
    /// 分页
    pub page_size: Option<u32>,
    pub page: Option<u32>,
}

// ============================================================
// 3. 查询结果
// ============================================================

/// 产品查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQueryResult {
    /// 匹配的产品列表
    pub items: Vec<ProductRecord>,
    /// 总匹配数
    pub total: u64,
    /// 当前页码
    pub page: u32,
    /// 每页数量
    pub page_size: u32,
}

/// 供应商查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierQueryResult {
    /// 匹配的供应商列表
    pub items: Vec<SupplierRecord>,
    /// 总匹配数
    pub total: u64,
    /// 当前页码
    pub page: u32,
    /// 每页数量
    pub page_size: u32,
}

// ============================================================
// 4. 知识记录
// ============================================================

/// 产品知识记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRecord {
    /// 产品唯一标识
    pub product_id: String,
    /// 产品名称
    pub name: String,
    /// 产品规格
    pub spec: ProductSpec,
    /// 参考单价 (USD)
    pub reference_price: f64,
    /// 最小起订量
    pub moq: u64,
    /// 交货周期 (天)
    pub lead_time_days: u32,
    /// 关联供应商 ID 列表
    pub supplier_ids: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 更新时间戳 (epoch seconds)
    pub updated_at: u64,
}

/// 供应商知识记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierRecord {
    /// 供应商唯一标识
    pub supplier_id: String,
    /// 供应商名称
    pub name: String,
    /// 国家/地区
    pub country: String,
    /// 提供的产品类别
    pub product_categories: Vec<String>,
    /// 信用评分 (0.0 - 1.0)
    pub credit_score: f64,
    /// 综合评分 (0.0 - 5.0)
    pub rating: f64,
    /// 月产能
    pub monthly_capacity: u64,
    /// 是否通过认证
    pub certified: bool,
    /// 认证列表
    pub certifications: Vec<String>,
    /// 联系方式
    pub contact_info: HashMap<String, String>,
    /// 更新时间戳 (epoch seconds)
    pub updated_at: u64,
}

// ============================================================
// 5. 匹配结果
// ============================================================

/// 产品匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchResult {
    /// 精确匹配的产品
    pub exact: Vec<ProductRecord>,
    /// 模糊匹配的产品（按相似度降序）
    pub fuzzy: Vec<ProductMatchEntry>,
    /// 匹配所用的查询条件摘要
    pub query_summary: String,
}

/// 模糊匹配条目（含相似度得分）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMatchEntry {
    /// 产品记录
    pub product: ProductRecord,
    /// 相似度得分 (0.0 - 1.0)
    pub score: f64,
    /// 匹配原因说明
    pub match_reasons: Vec<String>,
}

/// 供应商匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchResult {
    /// 精确匹配的供应商
    pub exact: Vec<SupplierRecord>,
    /// 模糊匹配的供应商（按相似度降序）
    pub fuzzy: Vec<SupplierMatchEntry>,
    /// 匹配所用的查询条件摘要
    pub query_summary: String,
}

/// 模糊匹配条目（含相似度得分）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierMatchEntry {
    /// 供应商记录
    pub supplier: SupplierRecord,
    /// 相似度得分 (0.0 - 1.0)
    pub score: f64,
    /// 匹配原因说明
    pub match_reasons: Vec<String>,
}

// ============================================================
// 6. 价格查询
// ============================================================

/// 价格查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuery {
    /// 产品 ID
    pub product_id: String,
    /// 供应商 ID（可选，查指定供应商报价）
    pub supplier_id: Option<String>,
    /// 数量
    pub quantity: u64,
    /// 目标货币
    pub currency: String,
    /// 目标国家/地区（用于计算关税/运费）
    pub destination_country: Option<String>,
}

/// 价格查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceResult {
    /// 产品 ID
    pub product_id: String,
    /// 供应商 ID
    pub supplier_id: Option<String>,
    /// 单价
    pub unit_price: f64,
    /// 总价
    pub total_price: f64,
    /// 货币
    pub currency: String,
    /// 数量
    pub quantity: u64,
    /// 交货周期 (天)
    pub lead_time_days: u32,
    /// 附加费用明细
    pub additional_costs: HashMap<String, f64>,
    /// 价格有效期（epoch seconds）
    pub valid_until: u64,
    /// 备注
    pub notes: Vec<String>,
}

// ============================================================
// 7. 知识更新操作
// ============================================================

/// 知识更新操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeOperation {
    /// 新增产品
    AddProduct(ProductRecord),
    /// 更新产品
    UpdateProduct {
        product_id: String,
        changes: HashMap<String, String>,
    },
    /// 删除产品
    DeleteProduct(String),
    /// 新增供应商
    AddSupplier(SupplierRecord),
    /// 更新供应商
    UpdateSupplier {
        supplier_id: String,
        changes: HashMap<String, String>,
    },
    /// 删除供应商
    DeleteSupplier(String),
    /// 批量导入
    BulkImport {
        products: Vec<ProductRecord>,
        suppliers: Vec<SupplierRecord>,
    },
}

/// 知识更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUpdateResult {
    /// 操作是否成功
    pub success: bool,
    /// 影响的记录数
    pub affected_count: u32,
    /// 错误信息（如有）
    pub errors: Vec<String>,
    /// 更新时间戳
    pub updated_at: u64,
}

// ============================================================
// 8. KnowledgeBase trait
// ============================================================

/// 外贸知识库统一接口
///
/// 实现方可对接数据库、向量搜索引擎、文件存储等后端。
/// 所有方法均为 async，支持异步 IO。
///
/// # 实现要求
///
/// - 不得使用 unsafe 代码
/// - 所有方法必须返回 `KnowledgeResult<T>`
/// - 实现方必须保证 `Send + Sync`
///
/// # 示例
///
/// ```ignore
/// struct PostgresKnowledgeBase { /* ... */ }
///
/// #[async_trait]
/// impl KnowledgeBase for PostgresKnowledgeBase {
///     async fn query_products(&self, filters: &ProductFilters) -> KnowledgeResult<ProductQueryResult> {
///         // 实现数据库查询...
///     }
///     // ...
/// }
/// ```
#[async_trait::async_trait]
pub trait KnowledgeBase: Send + Sync {
    /// 查询产品列表
    ///
    /// 根据过滤条件检索产品，支持关键词搜索、类型筛选、
    /// 价格区间、认证要求等多维度过滤。
    async fn query_products(
        &self,
        filters: &ProductFilters,
    ) -> KnowledgeResult<ProductQueryResult>;

    /// 查询供应商列表
    ///
    /// 根据过滤条件检索供应商，支持名称搜索、国家筛选、
    /// 产品类别、评分、产能等多维度过滤。
    async fn query_suppliers(
        &self,
        filters: &SupplierFilters,
    ) -> KnowledgeResult<SupplierQueryResult>;

    /// 产品智能匹配
    ///
    /// 根据需求描述或产品规格进行语义匹配，
    /// 返回精确匹配和模糊匹配结果。
    ///
    /// # 参数
    /// - `query`: 查询关键词或产品规格描述
    /// - `limit`: 最大返回数量
    async fn match_product(
        &self,
        query: &str,
        limit: u32,
    ) -> KnowledgeResult<ProductMatchResult>;

    /// 供应商智能匹配
    ///
    /// 根据需求描述进行语义匹配，
    /// 返回精确匹配和模糊匹配结果。
    ///
    /// # 参数
    /// - `query`: 查询关键词或需求描述
    /// - `product_category`: 产品类别（可选，用于缩小匹配范围）
    /// - `limit`: 最大返回数量
    async fn match_supplier(
        &self,
        query: &str,
        product_category: Option<&str>,
        limit: u32,
    ) -> KnowledgeResult<SupplierMatchResult>;

    /// 获取产品价格
    ///
    /// 查询指定产品在给定条件下的价格信息，
    /// 可指定供应商、数量、目标市场等参数。
    async fn get_price(&self, query: &PriceQuery) -> KnowledgeResult<PriceResult>;

    /// 更新知识库
    ///
    /// 执行知识更新操作（增删改），
    /// 支持单条和批量操作。
    async fn update_knowledge(
        &self,
        operations: &[KnowledgeOperation],
    ) -> KnowledgeResult<KnowledgeUpdateResult>;
}

// ============================================================
// 9. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_filters_default() {
        let f = ProductFilters::default();
        assert!(f.name_keyword.is_none());
        assert!(f.product_type.is_none());
        assert!(f.required_certs.is_empty());
        assert!(f.page_size.is_none());
    }

    #[test]
    fn test_supplier_filters_default() {
        let f = SupplierFilters::default();
        assert!(f.name_keyword.is_none());
        assert!(f.country.is_none());
        assert!(!f.certified_only);
    }

    #[test]
    fn test_knowledge_base_error_display() {
        let e = KnowledgeBaseError::NotFound {
            entity: "Product".into(),
            id: "P-001".into(),
        };
        assert_eq!(e.to_string(), "Product 'P-001' not found");

        let e = KnowledgeBaseError::InvalidQuery {
            reason: "page_size must be > 0".into(),
        };
        assert!(e.to_string().contains("Invalid query"));
    }

    #[test]
    fn test_product_query_result_serialize() {
        let r = ProductQueryResult {
            items: vec![],
            total: 0,
            page: 0,
            page_size: 20,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"total\":0"));
    }

    #[test]
    fn test_price_query_serialize() {
        let q = PriceQuery {
            product_id: "P-001".into(),
            supplier_id: Some("S-001".into()),
            quantity: 1000,
            currency: "USD".into(),
            destination_country: Some("DE".into()),
        };
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("P-001"));
        assert!(json.contains("USD"));
    }

    #[test]
    fn test_knowledge_operation_serialize() {
        let op = KnowledgeOperation::DeleteProduct("P-001".into());
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("DeleteProduct"));
    }

    #[test]
    fn test_match_entry_score_ordering() {
        let mut entries = vec![
            ProductMatchEntry {
                product: ProductRecord {
                    product_id: "P-002".into(),
                    name: "Widget B".into(),
                    spec: ProductSpec {
                        spec_id: "S-002".into(),
                        product_type: ProductType::Electronics,
                        bom: vec![],
                        routing: vec![],
                        packaging: super::super::full_cycle::PackagingSpec {
                            package_type: "box".into(),
                            dimensions_cm: (10.0, 10.0, 10.0),
                            gross_weight_kg: 1.0,
                            net_weight_kg: 0.8,
                            marks: vec![],
                        },
                        certifications: vec![],
                        hs_code: "8541.00".into(),
                        tax_refund_rate: 0.13,
                    },
                    reference_price: 5.0,
                    moq: 100,
                    lead_time_days: 14,
                    supplier_ids: vec![],
                    tags: vec![],
                    updated_at: 0,
                },
                score: 0.6,
                match_reasons: vec![],
            },
            ProductMatchEntry {
                product: ProductRecord {
                    product_id: "P-001".into(),
                    name: "Widget A".into(),
                    spec: ProductSpec {
                        spec_id: "S-001".into(),
                        product_type: ProductType::Electronics,
                        bom: vec![],
                        routing: vec![],
                        packaging: super::super::full_cycle::PackagingSpec {
                            package_type: "box".into(),
                            dimensions_cm: (10.0, 10.0, 10.0),
                            gross_weight_kg: 1.0,
                            net_weight_kg: 0.8,
                            marks: vec![],
                        },
                        certifications: vec![],
                        hs_code: "8541.00".into(),
                        tax_refund_rate: 0.13,
                    },
                    reference_price: 5.0,
                    moq: 100,
                    lead_time_days: 14,
                    supplier_ids: vec![],
                    tags: vec![],
                    updated_at: 0,
                },
                score: 0.9,
                match_reasons: vec![],
            },
        ];
        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        assert_eq!(entries[0].product.product_id, "P-001");
    }
}
