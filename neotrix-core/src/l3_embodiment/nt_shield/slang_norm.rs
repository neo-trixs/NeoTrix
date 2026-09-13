//! Slang Norm - 黑话规范化引擎
//!
//! Trie最长匹配 + 双关裸词屏蔽 + 域安研框架

use std::collections::HashMap;

/// 域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    Security,
    Reverse,
    Pentest,
    Crypto,
    Malware,
    General,
}

/// 转换结果
#[derive(Debug, Clone)]
pub struct _ConversionResult {
    pub original: String,
    pub converted: String,
    pub domain: Domain,
    pub matched_terms: Vec<_MatchedTerm>,
}

#[derive(Debug, Clone)]
pub struct _MatchedTerm {
    pub slang: String,
    pub professional: String,
    pub confidence: f64,
}

/// 黑话规范化引擎
pub struct SlangNormEngine {
    trie: Trie,
    _domain_rules: HashMap<Domain, Vec<ConversionRule>>,
}

#[derive(Debug, Clone)]
struct ConversionRule {
    slang: String,
    professional: String,
    domain: Domain,
}

struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
    professional: Option<String>,
    domain: Option<Domain>,
}

struct Trie {
    root: TrieNode,
}

impl Trie {
    fn new() -> Self {
        Self {
            root: TrieNode {
                children: HashMap::new(),
                is_end: false,
                professional: None,
                domain: None,
            },
        }
    }

    fn insert(&mut self, slang: &str, professional: &str, domain: Domain) {
        let mut node = &mut self.root;
        for char in slang.chars() {
            node = node.children.entry(char).or_insert_with(|| TrieNode {
                children: HashMap::new(),
                is_end: false,
                professional: None,
                domain: None,
            });
        }
        node.is_end = true;
        node.professional = Some(professional.to_string());
        node.domain = Some(domain);
    }

    fn search(&self, text: &str) -> Option<(String, Domain)> {
        let mut node = &self.root;
        let mut last_match = None;

        for (_i, char) in text.char_indices() {
            if let Some(child) = node.children.get(&char) {
                node = child;
                if node.is_end {
                    last_match = Some((node.professional.clone().expect("Some value"), node.domain.expect("has domain")));
                }
            } else {
                break;
            }
        }

        last_match
    }
}

impl SlangNormEngine {
    pub fn new() -> Self {
        let mut trie = Trie::new();
        let mut domain_rules: HashMap<Domain, Vec<ConversionRule>> = HashMap::new();

        // 安全域
        let security_rules = vec![
            ConversionRule { slang: "杀猪盘".to_string(), professional: "投资诈骗".to_string(), domain: Domain::Security },
            ConversionRule { slang: "注册机".to_string(), professional: "许可证密钥生成器".to_string(), domain: Domain::Security },
            ConversionRule { slang: "越狱".to_string(), professional: "安全绕过".to_string(), domain: Domain::Security },
            ConversionRule { slang: "破甲".to_string(), professional: "安全评估".to_string(), domain: Domain::Security },
            ConversionRule { slang: "免杀".to_string(), professional: "检测规避".to_string(), domain: Domain::Security },
            ConversionRule { slang: "webshell".to_string(), professional: "Web后门".to_string(), domain: Domain::Security },
            ConversionRule { slang: "提权".to_string(), professional: "权限提升".to_string(), domain: Domain::Security },
            ConversionRule { slang: "内网穿透".to_string(), professional: "网络横向移动".to_string(), domain: Domain::Security },
        ];

        // 逆向域
        let reverse_rules = vec![
            ConversionRule { slang: "脱壳".to_string(), professional: "加壳程序解包".to_string(), domain: Domain::Reverse },
            ConversionRule { slang: "爆破".to_string(), professional: "暴力破解".to_string(), domain: Domain::Reverse },
            ConversionRule { slang: "dump".to_string(), professional: "内存转储".to_string(), domain: Domain::Reverse },
            ConversionRule { slang: "hook".to_string(), professional: "函数拦截".to_string(), domain: Domain::Reverse },
            ConversionRule { slang: "patch".to_string(), professional: "二进制修改".to_string(), domain: Domain::Reverse },
        ];

        // 渗透域
        let pentest_rules = vec![
            ConversionRule { slang: "打点".to_string(), professional: "初始访问".to_string(), domain: Domain::Pentest },
            ConversionRule { slang: "横向".to_string(), professional: "横向移动".to_string(), domain: Domain::Pentest },
            ConversionRule { slang: "隧道".to_string(), professional: "网络隧道".to_string(), domain: Domain::Pentest },
            ConversionRule { slang: "代理".to_string(), professional: "代理服务器".to_string(), domain: Domain::Pentest },
        ];

        // 恶意软件域
        let malware_rules = vec![
            ConversionRule { slang: "木马".to_string(), professional: "特洛伊木马".to_string(), domain: Domain::Malware },
            ConversionRule { slang: "蠕虫".to_string(), professional: "网络蠕虫".to_string(), domain: Domain::Malware },
            ConversionRule { slang: "勒索".to_string(), professional: "勒索软件".to_string(), domain: Domain::Malware },
            ConversionRule { slang: "挖矿".to_string(), professional: "加密货币挖矿恶意软件".to_string(), domain: Domain::Malware },
        ];

        // 插入到Trie
        for rule in security_rules.iter().chain(reverse_rules.iter()).chain(pentest_rules.iter()).chain(malware_rules.iter()) {
            trie.insert(&rule.slang, &rule.professional, rule.domain);
        }

        domain_rules.insert(Domain::Security, security_rules);
        domain_rules.insert(Domain::Reverse, reverse_rules);
        domain_rules.insert(Domain::Pentest, pentest_rules);
        domain_rules.insert(Domain::Malware, malware_rules);

        Self { trie, _domain_rules: domain_rules }
    }

    /// 转换黑话
    pub fn convert(&self, input: &str) -> _ConversionResult {
        let mut converted = input.to_string();
        let mut matched_terms = Vec::new();
        let mut detected_domain = Domain::General;

        // Trie最长匹配
        if let Some((professional, domain)) = self.trie.search(input) {
            converted = professional.clone();
            matched_terms.push(_MatchedTerm {
                slang: input.to_string(),
                professional,
                confidence: 0.9,
            });
            detected_domain = domain;
        }

        _ConversionResult {
            original: input.to_string(),
            converted,
            domain: detected_domain,
            matched_terms,
        }
    }

    /// 检测域
    pub fn detect_domain(&self, input: &str) -> Domain {
        let (_, domain) = self.trie.search(input).unwrap_or((String::new(), Domain::General));
        domain
    }

    /// 批量转换
    pub fn _convert_batch(&self, inputs: &[&str]) -> Vec<_ConversionResult> {
        inputs.iter().map(|input| self.convert(input)).collect()
    }
}

impl Default for SlangNormEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_slang() {
        let engine = SlangNormEngine::new();
        let result = engine.convert("杀猪盘");
        assert_eq!(result.converted, "投资诈骗");
        assert_eq!(result.domain, Domain::Security);
    }

    #[test]
    fn test_detect_domain() {
        let engine = SlangNormEngine::new();
        assert_eq!(engine.detect_domain("杀猪盘"), Domain::Security);
        assert_eq!(engine.detect_domain("脱壳"), Domain::Reverse);
        assert_eq!(engine.detect_domain("打点"), Domain::Pentest);
    }

    #[test]
    fn test_batch_convert() {
        let engine = SlangNormEngine::new();
        let results = engine._convert_batch(&["杀猪盘", "脱壳", "打点"]);
        assert_eq!(results.len(), 3);
    }
}
