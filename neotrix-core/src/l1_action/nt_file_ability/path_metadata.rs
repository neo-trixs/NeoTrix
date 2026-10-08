//! 路径元数据提取器 (D1+D3)
//!
//! 从文件路径/文件夹名中提取结构化元数据。
//! 通用设计: 支持多种路径命名规则，可扩展。
//!
//! 支持的命名规则:
//! - 订单文件夹: "日期 国家 客户名 订单号" (如 "4.01菲律宾 WSD-I-26031303")
//! - 文件名: "业务员 产品名 供应商 日期.xlsx"

use std::path::Path;

/// 路径元数据
#[derive(Debug, Clone, Default)]
pub struct PathMetadata {
    /// 业务员 (从父文件夹名提取)
    pub salesperson: Option<String>,
    /// 订单文件夹名
    pub order_folder: Option<String>,
    /// 订单号 (WSD-X-数字 或 WZD-X-数字)
    pub order_no: Option<String>,
    /// 订单日期 (如 "4.01", "4.21")
    pub order_date: Option<String>,
    /// 国家 (中文)
    pub country: Option<String>,
    /// 客户 (英文名)
    pub customer: Option<String>,
    /// 文件名
    pub file_name: Option<String>,
}

impl PathMetadata {
    /// 从完整路径提取元数据
    pub fn from_path(path: &Path) -> Self {
        let mut meta = Self::default();

        // 文件名
        meta.file_name = path.file_name().map(|s| s.to_string_lossy().into_owned());

        // 父文件夹 → 业务员（父级若是订单文件夹则上溯一级取业务员）
        let sales_name: Option<String> = path.parent().and_then(|parent| {
            let name = parent.file_name()?.to_string_lossy().into_owned();
            let is_order = name.contains("WSD-")
                || name.contains("WZD-")
                || name.split_whitespace().any(is_date_pattern)
                || name
                    .split_whitespace()
                    .next()
                    .map(|t| strip_leading_date(t).0.is_some())
                    .unwrap_or(false);
            if is_order {
                parent.parent()?.file_name().map(|s| s.to_string_lossy().into_owned())
            } else {
                Some(name)
            }
        });
        if let Some(name) = sales_name {
            // 业务员文件夹通常是人名 (中文，2-4字)
            if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ' ')
                && name.len() <= 20
            {
                meta.salesperson = Some(name);
            }
        }

        // 订单文件夹 → 从路径中找含 WSD-/WZD- 的部分
        if let Some(order_folder) = extract_order_folder(path) {
            meta.order_folder = Some(order_folder.clone());
            parse_order_folder(&order_folder, &mut meta);
        }

        meta
    }

    /// 从业务员+订单文件夹名提取元数据
    pub fn from_salesperson_order(salesperson: &str, order_folder: &str) -> Self {
        let mut meta = Self::default();
        meta.salesperson = Some(salesperson.to_string());
        meta.order_folder = Some(order_folder.to_string());
        parse_order_folder(order_folder, &mut meta);
        meta
    }
}

/// 从路径中提取订单文件夹名 (含 WSD-/WZD- 的部分)
fn extract_order_folder(path: &Path) -> Option<String> {
    let components: Vec<_> = path.components().collect();

    for component in &components {
        let name = component.as_os_str().to_string_lossy();
        if name.contains("WSD-") || name.contains("WZD-") {
            return Some(name.into_owned());
        }
    }

    // 如果没找到 WSD-/WZD-，取最后一个非文件名的组件
    if components.len() >= 2 {
        let parent = components[components.len() - 2];
        return Some(parent.as_os_str().to_string_lossy().into_owned());
    }

    None
}

/// 解析订单文件夹名
///
/// 格式: "日期 国家 客户名 订单号"
/// 例如: "4.01菲律宾 WSD-I-26031303" → date=4.01, country=菲律宾, order_no=WSD-I-26031303
/// 例如: "4.21印尼Alvin WSD-I-26041403" → date=4.21, country=印尼, customer=Alvin, order_no=WSD-I-26041403
fn parse_order_folder(folder: &str, meta: &mut PathMetadata) {
    let parts: Vec<&str> = folder.split_whitespace().collect();

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

    // 提取国家和客户 (跳过日期和订单号；兼容无空格粘连如 "4.01菲律宾"/"印尼Alvin")
    let mut country_parts = Vec::new();
    let mut customer_parts = Vec::new();

    for part in &parts {
        // 跳过订单号
        if part.starts_with("WSD-") || part.starts_with("WZD-") {
            break;
        }
        // 跳过纯日期；粘连日期先剥离（日期另由上方循环收录）
        let mut token = *part;
        if is_date_pattern(token) {
            continue;
        }
        if let (Some(d), rest) = strip_leading_date(token) {
            if meta.order_date.is_none() {
                meta.order_date = Some(d.to_string());
            }
            if !rest.is_empty() {
                token = rest;
            }
        }
        // 英文名 → customer；CJK 尾随 ASCII（印尼Alvin）→ 拆分
        if is_english_only(token) {
            customer_parts.push(token);
        } else {
            let (head, tail) = split_cjk_ascii(token);
            country_parts.push(head);
            if !tail.is_empty() {
                customer_parts.push(tail);
            }
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
}

/// 检查是否是日期格式 (数字.数字)
fn is_date_pattern(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].parse::<u32>().is_ok() && parts[1].parse::<u32>().is_ok()
}

/// 检查是否全是英文
fn is_english_only(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace())
}

/// 剥离开头的日期前缀（"4.01菲律宾" → (Some("4.01"), "菲律宾")；无则 (None, 原串)）。
fn strip_leading_date(token: &str) -> (Option<&str>, &str) {
    let bytes = token.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i > 0
        && bytes.get(i) == Some(&b'.')
        && bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit())
    {
        let mut j = i + 1;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        return (Some(&token[..j]), &token[j..]);
    }
    (None, token)
}

/// 拆 CJK 头＋ASCII 尾（"印尼Alvin" → ("印尼", "Alvin")；纯串原样返回，尾空）。
fn split_cjk_ascii(s: &str) -> (&str, &str) {
    let idx = s
        .char_indices()
        .find(|(_, c)| c.is_ascii_alphabetic())
        .map(|(i, _)| i);
    match idx {
        Some(0) | None => (s, ""),
        Some(i) => (&s[..i], &s[i..]),
    }
}

/// 批量提取多个路径的元数据
pub fn extract_batch(paths: &[&Path]) -> Vec<PathMetadata> {
    paths.iter().map(|p| PathMetadata::from_path(p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_philippines() {
        let meta = PathMetadata::from_salesperson_order("段留杰", "4.01菲律宾 WSD-I-26031303");
        assert_eq!(meta.salesperson.as_deref(), Some("段留杰"));
        assert_eq!(meta.order_date.as_deref(), Some("4.01"));
        assert_eq!(meta.country.as_deref(), Some("菲律宾"));
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26031303"));
    }

    #[test]
    fn test_parse_indonesia_with_customer() {
        let meta = PathMetadata::from_salesperson_order("段留杰", "4.21印尼Alvin WSD-I-26041403");
        assert_eq!(meta.country.as_deref(), Some("印尼"));
        assert_eq!(meta.customer.as_deref(), Some("Alvin"));
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26041403"));
    }

    #[test]
    fn test_parse_guatemala() {
        let meta =
            PathMetadata::from_salesperson_order("段留杰", "4.02危地马拉 Erson WSD-I-26032701");
        assert_eq!(meta.country.as_deref(), Some("危地马拉"));
        assert_eq!(meta.customer.as_deref(), Some("Erson"));
    }

    #[test]
    fn test_from_path() {
        let path = PathBuf::from("data/smb_local/段留杰/4.01菲律宾 WSD-I-26031303/合同.xlsx");
        let meta = PathMetadata::from_path(&path);
        assert_eq!(meta.salesperson.as_deref(), Some("段留杰"));
        assert_eq!(meta.country.as_deref(), Some("菲律宾"));
        assert_eq!(meta.order_no.as_deref(), Some("WSD-I-26031303"));
    }
}
