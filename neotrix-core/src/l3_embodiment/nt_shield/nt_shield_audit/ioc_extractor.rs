use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IocType {
    IpAddress,
    Domain,
    Url,
    Hash,
    Email,
    FilePath,
    Mutex,
    RegistryKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IocEntry {
    pub ioc_type: IocType,
    pub value: String,
    pub context: Option<String>,
    pub confidence: f64,
}

pub struct IocExtractor {
    extractors: Vec<Box<dyn Fn(&str) -> Vec<IocEntry> + Send + Sync>>,
}

impl IocExtractor {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn extract(&self, text: &str) -> Vec<IocEntry> {
        let mut results = Vec::new();
        for extractor in &self.extractors {
            results.extend(extractor(text));
        }
        results
    }

    pub fn extract_ips(text: &str) -> Vec<IocEntry> {
        let mut entries = Vec::new();
        for word in text.split_whitespace() {
            let clean: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '.' || *c == ':')
                .collect();
            let parts: Vec<&str> = clean.split('.').collect();
            if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                entries.push(IocEntry {
                    ioc_type: IocType::IpAddress,
                    value: clean,
                    context: None,
                    confidence: 0.8,
                });
            }
        }
        entries
    }

    pub fn extract_hashes(text: &str) -> Vec<IocEntry> {
        let mut entries = Vec::new();
        for word in text.split_whitespace() {
            let clean: String = word.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            if clean.len() == 32 || clean.len() == 40 || clean.len() == 64 {
                entries.push(IocEntry {
                    ioc_type: IocType::Hash,
                    value: clean,
                    context: None,
                    confidence: 0.9,
                });
            }
        }
        entries
    }
}

impl Default for IocExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ip() {
        let entries = IocExtractor::extract_ips("connect to 192.168.1.1 now");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].value, "192.168.1.1");
    }

    #[test]
    fn test_extract_hash() {
        let entries = IocExtractor::extract_hashes("md5 is d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(entries.len(), 1);
    }
}
