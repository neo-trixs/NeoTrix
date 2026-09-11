//! 路径元数据提取器
//!
//! 从订单文件夹名中提取结构化元数据。
//! 文件夹命名规则: "日期 国家 客户名 订单号"
//! 例如: "4.01菲律宾 WSD-I-26031303" -> order_date="4.01", country="菲律宾", order_no="WSD-I-26031303"

/// 从订单文件夹名提取元数据
pub fn extract_order_metadata(order_folder: &str) -> OrderMetadata {
    let mut meta = OrderMetadata::default();
    meta.order_folder = order_folder.to_string();

    let parts: Vec<&str> = order_folder.split_whitespace().collect();

    // 提取订单号 (WSD-X-数字 或 WZD-X-数字)
    for part in &parts {
        if part.starts_with("WSD-") || part.starts_with("WZD-") {
            meta.order_no = Some(part.to_string());
            break;
        }
    }

    // 提取日期 (数字.数字 格式，如 4.01, 4.21)
    for part in &parts {
        if is_date_pattern(part) {
            meta.order_date = Some(part.to_string());
            break;
        }
    }

    // 提取国家 (跳过日期和英文名)
    let mut country_parts = Vec::new();
    let mut customer_parts = Vec::new();
    for part in &parts {
        // 跳过订单号
        if part.starts_with("WSD-") || part.starts_with("WZD-") {
            break;
        }
        // 跳过日期
        if is_date_pattern(part) {
            continue;
        }
        // 英文名 -> customer
        if is_english_only(part) {
            customer_parts.push(*part);
        } else {
            // 中文 -> country
            country_parts.push(*part);
        }
    }

    meta.country = if country_parts.is_empty() {
        None
    } else {
        Some(country_parts.join(" "))
    };

    meta.customer = if customer_parts.is_empty() {
        None
    } else {
        Some(customer_parts.join(" "))
    };

    meta
}

fn is_date_pattern(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
}

fn is_english_only(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}

/// 路径元数据
#[derive(Debug, Clone, Default)]
pub struct OrderMetadata {
    pub order_folder: String,
    pub order_no: Option<String>,
    pub order_date: Option<String>,
    pub country: Option<String>,
    pub customer: Option<String>,
}

impl OrderMetadata {
    /// 从业务员+文件路径提取完整元数据
    pub fn from_salesperson_path(salesperson: &str, order_folder: &str) -> Self {
        let _ = salesperson;
        extract_order_metadata(order_folder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_philippines() {
        let meta = extract_order_metadata("4.01菲律宾 WSD-I-26031303");
        assert_eq!(meta.order_date.as_deref(), Some("4.01"));
        assert_eq!(meta.country.as_deref(), Some("菲律宾"));
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26031303"));
    }

    #[test]
    fn test_extract_indonesia_with_customer() {
        let meta = extract_order_metadata("4.21印尼Alvin WSD-I-26041403");
        assert_eq!(meta.order_date.as_deref(), Some("4.21"));
        assert_eq!(meta.country.as_deref(), Some("印尼Alvin"));
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26041403"));
    }

    #[test]
    fn test_extract_guatemala() {
        let meta = extract_order_metadata("4.02危地马拉 Erson WSD-I-26032701");
        assert_eq!(meta.country.as_deref(), Some("危地马拉"));
        assert_eq!(meta.customer.as_deref(), Some("Erson"));
    }
}
