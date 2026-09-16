//! 数据模型测试
//!
//! 测试 data_model.rs 中定义的所有核心数据结构。

#![forbid(unsafe_code)]

use super::super::data_model::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_default() {
        let product = Product::default();
        assert!(product.code.is_empty());
        assert_eq!(product.category, ProductCategory::Valve);
        assert!(product.subcategory.is_empty());
        assert!(product.model.is_empty());
        assert!(product.materials.is_empty());
        assert_eq!(product.drive_type, DriveType::Manual);
        assert_eq!(product.connection_type, ConnectionType::Flanged);
        assert!(product.standard.is_empty());
        assert_eq!(product.pressure.value, 0);
        assert!(product.price.unit_price >= 0.0);
    }

    #[test]
    fn test_supplier_default() {
        let supplier = Supplier::default();
        assert!(!supplier.id.is_empty());
        assert!(supplier.code.is_empty());
        assert!(supplier.name.is_empty());
        assert_eq!(supplier.category, ProductCategory::Valve);
        assert_eq!(supplier.status, "active");
        assert!(supplier.performance.overall_score >= 0.0);
    }

    #[test]
    fn test_inquiry_default() {
        let inquiry = Inquiry::default();
        assert!(!inquiry.id.is_empty());
        assert!(inquiry.inquiry_no.is_empty());
        assert!(inquiry.customer.is_empty());
        assert_eq!(inquiry.trade_terms, TradeTerms::ExWorks);
        assert!(inquiry.items.is_empty());
        assert_eq!(inquiry.status, InquiryStatus::Received);
    }

    #[test]
    fn test_quote_default() {
        let quote = Quote::default();
        assert!(!quote.id.is_empty());
        assert!(quote.quote_no.is_empty());
        assert!(quote.inquiry_id.is_empty());
        assert!(quote.items.is_empty());
        assert!(quote.total_amount >= 0.0);
        assert_eq!(quote.currency, "CNY");
        assert_eq!(quote.validity_days, 30);
        assert_eq!(quote.status, QuoteStatus::Draft);
    }

    #[test]
    fn test_order_default() {
        let order = Order::default();
        assert!(!order.id.is_empty());
        assert!(order.order_no.is_empty());
        assert!(order.quote_id.is_empty());
        assert!(order.customer.is_empty());
        assert!(order.items.is_empty());
        assert!(order.total_amount >= 0.0);
        assert_eq!(order.currency, "CNY");
        assert_eq!(order.status, OrderStatus::Draft);
    }

    #[test]
    fn test_product_serialization() {
        let product = Product {
            code: "P001".to_string(),
            category: ProductCategory::Valve,
            subcategory: "Ball Valve".to_string(),
            model: "BV-100".to_string(),
            materials: vec![MaterialSpec {
                name: "CF8M".to_string(),
                standard: "ASTM A351".to_string(),
                grade: "316".to_string(),
            }],
            drive_type: DriveType::Manual,
            connection_type: ConnectionType::Flanged,
            standard: "API 608".to_string(),
            pressure: PressureRating {
                value: 150,
                unit: "CLASS".to_string(),
            },
            size: SizeSpec {
                nominal: "2\"".to_string(),
                inner_diameter: Some(50.0),
                outer_diameter: Some(60.0),
                length: Some(200.0),
            },
            price: PriceInfo {
                unit_price: 1500.0,
                currency: "USD".to_string(),
                discount_rate: Some(0.1),
                tax_included: false,
            },
            supplier_id: "S001".to_string(),
            grade: "A".to_string(),
        };

        let json = serde_json::to_string(&product).unwrap();
        assert!(json.contains("P001"));
        assert!(json.contains("Ball Valve"));
        assert!(json.contains("CF8M"));

        let deserialized: Product = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, "P001");
        assert_eq!(deserialized.materials.len(), 1);
        assert_eq!(deserialized.materials[0].name, "CF8M");
    }

    #[test]
    fn test_inquiry_item_boundary() {
        let item = InquiryItem {
            seq: 1,
            product_name: "Test".to_string(),
            model: "T-001".to_string(),
            standard: "GB/T".to_string(),
            materials: vec![],
            drive_type: DriveType::Electric,
            connection_type: ConnectionType::Welded,
            pressure: PressureRating {
                value: 600,
                unit: "PN".to_string(),
            },
            size: SizeSpec::default(),
            quantity: 0,
            unit: "pcs".to_string(),
            target_price: None,
            remarks: None,
        };

        assert_eq!(item.quantity, 0);
        assert!(item.target_price.is_none());

        let item_with_max_qty = InquiryItem {
            quantity: u32::MAX,
            ..item
        };
        assert_eq!(item_with_max_qty.quantity, u32::MAX);
    }

    #[test]
    fn test_performance_metrics_boundary() {
        let metrics = PerformanceMetrics {
            on_time_delivery_rate: 0.0,
            quality_pass_rate: 1.0,
            response_time_hours: 0.0,
            cooperation_count: 0,
            overall_score: 100.0,
        };

        assert!(metrics.on_time_delivery_rate >= 0.0);
        assert!(metrics.quality_pass_rate <= 1.0);
        assert!(metrics.overall_score >= 0.0);
        assert!(metrics.overall_score <= 100.0);
    }

    #[test]
    fn test_contact_info_serialization() {
        let contact = ContactInfo {
            name: "John Doe".to_string(),
            phone: "+86-13800138000".to_string(),
            email: "john@example.com".to_string(),
            title: "Manager".to_string(),
            wechat: Some("johndoe".to_string()),
        };

        let json = serde_json::to_string(&contact).unwrap();
        assert!(json.contains("John Doe"));
        assert!(json.contains("johndoe"));

        let deserialized: ContactInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.wechat, Some("johndoe".to_string()));
    }

    #[test]
    fn test_trade_terms_variants() {
        let terms = vec![
            TradeTerms::ExWorks,
            TradeTerms::Fca,
            TradeTerms::Fob,
            TradeTerms::Cfr,
            TradeTerms::Cif,
            TradeTerms::Ddp,
            TradeTerms::Other("EXW Shanghai".to_string()),
        ];

        for term in &terms {
            let json = serde_json::to_string(term).unwrap();
            let deserialized: TradeTerms = serde_json::from_str(&json).unwrap();
            assert_eq!(&deserialized, term);
        }
    }
}
