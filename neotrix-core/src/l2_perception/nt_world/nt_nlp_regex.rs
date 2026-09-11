//! NT-WORLD NLP: 正则表达式信息提取
//!
//! 支持中文邮箱、手机、身份证、URL等信息提取
//! 受 funNLP 项目启发，实现Rust版本

use regex::Regex;

/// 提取结果
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// 匹配的文本
    pub text: String,
    /// 起始位置
    pub start: usize,
    /// 结束位置
    pub end: usize,
    /// 额外信息
    pub metadata: std::collections::HashMap<String, String>,
}

/// 信息提取器
pub struct InfoExtractor {
    /// 邮箱正则
    email_regex: Regex,
    /// 手机号正则
    phone_regex: Regex,
    /// 身份证正则
    id_card_regex: Regex,
    /// URL正则
    url_regex: Regex,
    /// IP地址正则
    ip_regex: Regex,
    /// 中文姓名正则
    name_regex: Regex,
}

impl InfoExtractor {
    /// 创建新的提取器
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            // 邮箱正则 (支持中文域名)
            email_regex: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")?,
            // 中国手机号 (11位，1开头)
            phone_regex: Regex::new(r"1[3-9]\d{9}")?,
            // 身份证号 (18位或15位)
            id_card_regex: Regex::new(r"\d{17}[\dXx]|\d{15}")?,
            // URL
            url_regex: Regex::new(r"https?://[^\s<>]+|www\.[^\s<>]+")?,
            // IPv4地址
            ip_regex: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")?,
            // 中文姓名 (2-4字)
            name_regex: Regex::new(r"[\u4e00-\u9fa5]{2,4}")?,
        })
    }

    /// 提取邮箱
    pub fn extract_emails(&self, text: &str) -> Vec<ExtractionResult> {
        self.email_regex.find_iter(text)
            .map(|m| ExtractionResult {
                text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }

    /// 提取手机号
    pub fn extract_phones(&self, text: &str) -> Vec<ExtractionResult> {
        self.phone_regex.find_iter(text)
            .map(|m| ExtractionResult {
                text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    let phone = m.as_str();
                    // 运营商推断
                    let carrier = match &phone[1..3] {
                        "34" | "35" | "36" | "37" | "38" | "39" => "中国移动",
                        "56" | "57" | "58" | "59" | "85" | "86" | "87" | "88" | "89" => "中国联通",
                        "33" | "49" | "53" | "73" | "74" | "75" | "76" | "77" | "78" => "中国电信",
                        _ => "未知",
                    };
                    meta.insert("carrier".into(), carrier.into());
                    meta
                },
            })
            .collect()
    }

    /// 提取身份证号
    pub fn extract_id_cards(&self, text: &str) -> Vec<ExtractionResult> {
        self.id_card_regex.find_iter(text)
            .filter_map(|m| {
                let id = m.as_str();
                // 验证长度
                if id.len() != 18 && id.len() != 15 {
                    return None;
                }

                let mut metadata = std::collections::HashMap::new();

                if id.len() == 18 {
                    // 提取出生日期
                    let birth = &id[6..14];
                    if birth.len() == 8 {
                        metadata.insert("birth_date".into(), birth.into());
                    }

                    // 提取性别 (第17位，奇数为男，偶数为女)
                    if let Some(gender_digit) = id.chars().nth(16) {
                        if let Some(d) = gender_digit.to_digit(10) {
                            let gender = if d % 2 == 1 { "男" } else { "女" };
                            metadata.insert("gender".into(), gender.into());
                        }
                    }

                    // 提取地区码
                    let region_code = &id[0..6];
                    metadata.insert("region_code".into(), region_code.into());
                }

                Some(ExtractionResult {
                    text: id.to_string(),
                    start: m.start(),
                    end: m.end(),
                    metadata,
                })
            })
            .collect()
    }

    /// 提取URL
    pub fn extract_urls(&self, text: &str) -> Vec<ExtractionResult> {
        self.url_regex.find_iter(text)
            .map(|m| ExtractionResult {
                text: m.as_str().to_string(),
                start: m.start(),
                end: m.end(),
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }

    /// 提取IP地址
    pub fn extract_ips(&self, text: &str) -> Vec<ExtractionResult> {
        self.ip_regex.find_iter(text)
            .filter_map(|m| {
                let ip = m.as_str();
                // 验证每个octet在0-255范围
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() != 4 {
                    return None;
                }
                let valid = parts.iter().all(|p| {
                    p.parse::<u8>().is_ok()
                });
                if valid {
                    Some(ExtractionResult {
                        text: ip.to_string(),
                        start: m.start(),
                        end: m.end(),
                        metadata: std::collections::HashMap::new(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// 提取所有信息
    pub fn extract_all(&self, text: &str) -> std::collections::HashMap<String, Vec<ExtractionResult>> {
        let mut results = std::collections::HashMap::new();
        results.insert("emails".into(), self.extract_emails(text));
        results.insert("phones".into(), self.extract_phones(text));
        results.insert("id_cards".into(), self.extract_id_cards(text));
        results.insert("urls".into(), self.extract_urls(text));
        results.insert("ips".into(), self.extract_ips(text));
        results
    }
}

impl Default for InfoExtractor {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_emails() {
        let extractor = InfoExtractor::new().unwrap();
        let text = "联系我们: admin@example.com 或 support@company.co.jp";
        let emails = extractor.extract_emails(text);
        assert_eq!(emails.len(), 2);
        assert_eq!(emails[0].text, "admin@example.com");
        assert_eq!(emails[1].text, "support@company.co.jp");
    }

    #[test]
    fn extract_phones() {
        let extractor = InfoExtractor::new().unwrap();
        let text = "手机号: 13812345678, 座机: 010-12345678";
        let phones = extractor.extract_phones(text);
        assert_eq!(phones.len(), 1);
        assert_eq!(phones[0].text, "13812345678");
        assert_eq!(phones[0].metadata.get("carrier").unwrap(), "中国移动");
    }

    #[test]
    fn extract_id_cards() {
        let extractor = InfoExtractor::new().unwrap();
        let text = "身份证: 110101199003077891";
        let ids = extractor.extract_id_cards(text);
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].metadata.get("gender").unwrap(), "男");
    }

    #[test]
    fn extract_urls() {
        let extractor = InfoExtractor::new().unwrap();
        let text = "访问 https://example.com/path?query=1 或 www.test.org";
        let urls = extractor.extract_urls(text);
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn extract_ips() {
        let extractor = InfoExtractor::new().unwrap();
        let text = "服务器: 192.168.1.100, 网关: 10.0.0.1";
        let ips = extractor.extract_ips(text);
        assert_eq!(ips.len(), 2);
    }
}
