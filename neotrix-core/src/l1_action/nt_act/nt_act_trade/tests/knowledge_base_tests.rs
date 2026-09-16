//! 知识库测试
//!
//! 测试 knowledge_base.rs 中定义的过滤器、查询结果、匹配结果等。

#![forbid(unsafe_code)]

use super::super::full_cycle::{PackagingSpec, ProductSpec, ProductType};
use super::super::knowledge_base::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_filters_default() {
        let filters = ProductFilters::default();
        assert!(filters.name_keyword.is_none());
        assert!(filters.product_type.is_none());
        assert!(filters.hs_code_prefix.is_none());
        assert!(filters.required_certs.is_empty());
        assert!(filters.min_price.is_none());
        assert!(filters.max_price.is_none());
        assert!(filters.tags.is_empty());
        assert!(filters.page_size.is_none());
        assert!(filters.page.is_none());
    }

    #[test]
    fn test_supplier_filters_default() {
        let filters = SupplierFilters::default();
        assert!(filters.name_keyword.is_none());
        assert!(filters.country.is_none());
        assert!(filters.product_categories.is_empty());
        assert!(filters.min_rating.is_none());
        assert!(!filters.certified_only);
        assert!(filters.min_capacity.is_none());
        assert!(filters.max_capacity.is_none());
        assert!(filters.page_size.is_none());
        assert!(filters.page.is_none());
    }

    #[test]
    fn test_knowledge_base_error_display() {
        let errors = vec![
            KnowledgeBaseError::NotFound {
                entity: "Product".to_string(),
                id: "P001".to_string(),
            },
            KnowledgeBaseError::InvalidQuery {
                reason: "Invalid price range".to_string(),
            },
            KnowledgeBaseError::Conflict {
                message: "Concurrent update".to_string(),
            },
            KnowledgeBaseError::Unavailable {
                reason: "Database connection failed".to_string(),
            },
            KnowledgeBaseError::PermissionDenied {
                operation: "delete".to_string(),
            },
        ];

        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_product_query_result_serialization() {
        let result = ProductQueryResult {
            items: vec![],
            total: 0,
            page: 0,
            page_size: 10,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total\":0"));
        assert!(json.contains("\"page\":0"));

        let deserialized: ProductQueryResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total, 0);
        assert_eq!(deserialized.page_size, 10);
    }

    #[test]
    fn test_supplier_query_result_serialization() {
        let result = SupplierQueryResult {
            items: vec![],
            total: 5,
            page: 1,
            page_size: 20,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"total\":5"));
        assert!(json.contains("\"page\":1"));

        let deserialized: SupplierQueryResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total, 5);
    }

    #[test]
    fn test_price_query_serialization() {
        let query = PriceQuery {
            product_id: "P001".to_string(),
            supplier_id: Some("S001".to_string()),
            quantity: 100,
            currency: "USD".to_string(),
            destination_country: Some("US".to_string()),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("P001"));
        assert!(json.contains("S001"));
        assert!(json.contains("USD"));

        let deserialized: PriceQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.quantity, 100);
        assert_eq!(deserialized.supplier_id, Some("S001".to_string()));
    }

    #[test]
    fn test_knowledge_operation_serialization() {
        let operations = vec![
            KnowledgeOperation::AddProduct(ProductRecord {
                product_id: "P001".to_string(),
                name: "Test Product".to_string(),
                spec: ProductSpec {
                    spec_id: "SP001".to_string(),
                    product_type: ProductType::Electronics,
                    bom: vec![],
                    routing: vec![],
                    packaging: PackagingSpec {
                        package_type: "box".to_string(),
                        dimensions_cm: (10.0, 10.0, 10.0),
                        gross_weight_kg: 1.0,
                        net_weight_kg: 0.8,
                        marks: vec![],
                    },
                    certifications: vec![],
                    hs_code: "8541.00".to_string(),
                    tax_refund_rate: 0.13,
                },
                reference_price: 100.0,
                moq: 10,
                lead_time_days: 7,
                supplier_ids: vec!["S001".to_string()],
                tags: vec!["electronics".to_string()],
                updated_at: 0,
            }),
            KnowledgeOperation::DeleteProduct("P002".to_string()),
        ];

        let json = serde_json::to_string(&operations).unwrap();
        assert!(json.contains("AddProduct"));
        assert!(json.contains("DeleteProduct"));
    }

    #[test]
    fn test_match_entry_score_ordering() {
        let mut entries = vec![
            ProductMatchEntry {
                product: ProductRecord {
                    product_id: "P002".to_string(),
                    name: "Low Score".to_string(),
                    spec: ProductSpec {
                        spec_id: "SP002".to_string(),
                        product_type: ProductType::Electronics,
                        bom: vec![],
                        routing: vec![],
                        packaging: PackagingSpec {
                            package_type: "box".to_string(),
                            dimensions_cm: (10.0, 10.0, 10.0),
                            gross_weight_kg: 1.0,
                            net_weight_kg: 0.8,
                            marks: vec![],
                        },
                        certifications: vec![],
                        hs_code: "8541.00".to_string(),
                        tax_refund_rate: 0.13,
                    },
                    reference_price: 50.0,
                    moq: 100,
                    lead_time_days: 14,
                    supplier_ids: vec![],
                    tags: vec![],
                    updated_at: 0,
                },
                score: 0.3,
                match_reasons: vec!["Partial match".to_string()],
            },
            ProductMatchEntry {
                product: ProductRecord {
                    product_id: "P001".to_string(),
                    name: "High Score".to_string(),
                    spec: ProductSpec {
                        spec_id: "SP001".to_string(),
                        product_type: ProductType::Electronics,
                        bom: vec![],
                        routing: vec![],
                        packaging: PackagingSpec {
                            package_type: "box".to_string(),
                            dimensions_cm: (10.0, 10.0, 10.0),
                            gross_weight_kg: 1.0,
                            net_weight_kg: 0.8,
                            marks: vec![],
                        },
                        certifications: vec![],
                        hs_code: "8541.00".to_string(),
                        tax_refund_rate: 0.13,
                    },
                    reference_price: 100.0,
                    moq: 50,
                    lead_time_days: 7,
                    supplier_ids: vec!["S001".to_string()],
                    tags: vec!["premium".to_string()],
                    updated_at: 0,
                },
                score: 0.95,
                match_reasons: vec!["Exact match".to_string()],
            },
        ];

        // Sort by score descending
        entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        assert_eq!(entries[0].product.product_id, "P001");
        assert_eq!(entries[1].product.product_id, "P002");
    }

    #[test]
    fn test_price_result_serialization() {
        let mut additional_costs = std::collections::HashMap::new();
        additional_costs.insert("shipping".to_string(), 50.0);
        additional_costs.insert("insurance".to_string(), 10.0);

        let result = PriceResult {
            product_id: "P001".to_string(),
            supplier_id: Some("S001".to_string()),
            unit_price: 100.0,
            total_price: 10000.0,
            currency: "USD".to_string(),
            quantity: 100,
            lead_time_days: 14,
            additional_costs,
            valid_until: 1735689600,
            notes: vec!["Bulk discount applied".to_string()],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("P001"));
        assert!(json.contains("shipping"));

        let deserialized: PriceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.additional_costs.len(), 2);
        assert_eq!(deserialized.notes.len(), 1);
    }
}
