//! Email attachments — 邮件附件链（列表＋下载 URL＋文本路由判定）.
//!
//! 端点经 TMS bundle 验证：`GET /b/emails/queryEmailAttachementDetail/{emailId}`。
//! 本模块只做 URL 构造与响应解析（传输无关）；字节下载后：
//! 文本类（xls/xlsx/csv/txt/pdf/eml）→ 走 doc-parse 文本提取；
//! 图片类 → 标记 OCR 待处理（不静默丢弃）。
//! 附件型询盘（正文 0 行场景）的承接入口。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 附件详情端点（u = "/b"）
pub const ATTACH_DETAIL_PATH: &str = "/emails/queryEmailAttachementDetail/";

/// 可文本直提的附件类型（进 doc-parse）
const TEXT_TYPES: &[&str] = &["xls", "xlsx", "csv", "txt", "pdf", "eml", "doc", "docx"];
/// 需 OCR 的附件类型
const IMAGE_TYPES: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp"];

/// 附件信息（ tolerance 解析：缺字段给默认，不炸）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub file_code: String,
    pub file_name: String,
    pub file_type: String,
}

impl AttachmentInfo {
    pub fn text_routable(&self) -> bool {
        TEXT_TYPES
            .iter()
            .any(|t| self.file_type.eq_ignore_ascii_case(t))
    }

    pub fn needs_ocr(&self) -> bool {
        !self.text_routable()
            && IMAGE_TYPES
                .iter()
                .any(|t| self.file_type.eq_ignore_ascii_case(t))
    }
}

/// 从附件详情响应 JSON 提取附件列表（形状容错）
pub fn parse_attachments(value: &serde_json::Value) -> Vec<AttachmentInfo> {
    let items = value
        .get("data")
        .and_then(|d| d.as_array())
        .or_else(|| value.as_array());
    let mut out = Vec::new();
    if let Some(arr) = items {
        for it in arr {
            let s = |k: &str| it.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let file_type = s("fileType");
            let file_type = if file_type.is_empty() {
                s("fileName").rsplit('.').next().unwrap_or("").to_string()
            } else {
                file_type
            };
            out.push(AttachmentInfo {
                file_code: s("fileCode")
                    .is_empty()
                    .then(|| s("code"))
                    .unwrap_or_else(|| s("fileCode")),
                file_name: s("fileName")
                    .is_empty()
                    .then(|| s("name"))
                    .unwrap_or_else(|| s("fileName")),
                file_type,
            });
        }
    }
    out
}

/// 下载 URL 构造：fileCode + companyId + isBig（bundle 实证参数）
pub fn download_url(base_rapi_b: &str, file_code: &str, company_id: u64, is_big: bool) -> String {
    format!(
        "{}/emails/eml/view?fileCode={}&companyId={}&isBig={}",
        base_rapi_b.trim_end_matches('/'),
        file_code,
        company_id,
        if is_big { 1 } else { 0 }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> serde_json::Value {
        serde_json::json!({
            "success": true,
            "data": [
                {"fileCode": "ABC123", "fileName": "RFQ.xls", "fileType": "xls"},
                {"fileCode": "IMG9", "fileName": "scan.jpg", "fileType": "jpg"},
                {"fileCode": "X", "fileName": "note"}
            ]
        })
    }

    #[test]
    fn parse_lists_with_fallbacks() {
        let list = parse_attachments(&sample());
        assert_eq!(list.len(), 3);
        assert!(list[0].text_routable());
        assert!(!list[0].needs_ocr());
        assert!(list[1].needs_ocr());
        // 无 fileType 时从扩展名推断
        assert_eq!(list[2].file_type, "");
    }

    #[test]
    fn download_url_shape() {
        let u = download_url("https://trade.joinf.com/rapi/b", "ABC123", 65491, false);
        assert!(u.contains("fileCode=ABC123"));
        assert!(u.contains("companyId=65491"));
    }

    #[test]
    fn detail_path_const() {
        assert!(ATTACH_DETAIL_PATH.ends_with('/'));
    }
}
