//! 搜索引擎类型
//!
//! 定义通用的搜索引擎、搜索结果等类型，支持所有需要搜索能力的Agent。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 搜索引擎 — 跨域通用
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchEngine {
    /// Google
    Google,
    /// Bing
    Bing,
    /// Baidu
    Baidu,
    /// Yahoo
    Yahoo,
    /// DuckDuckGo
    DuckDuckGo,
    /// Yandex
    Yandex,
    /// 其他
    Other(String),
}

impl SearchEngine {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "google" | "Google" => Some(Self::Google),
            "bing" | "Bing" => Some(Self::Bing),
            "baidu" | "Baidu" | "百度" => Some(Self::Baidu),
            "yahoo" | "Yahoo" => Some(Self::Yahoo),
            "duckduckgo" | "DuckDuckGo" => Some(Self::DuckDuckGo),
            "yandex" | "Yandex" => Some(Self::Yandex),
            _ => Some(Self::Other(s.to_string())),
        }
    }

    /// 获取搜索引擎类型
    pub fn engine_type(&self) -> SearchEngineType {
        match self {
            Self::Google | Self::Bing | Self::Yahoo | Self::DuckDuckGo | Self::Yandex => {
                SearchEngineType::General
            }
            Self::Baidu => SearchEngineType::General,
            Self::Other(_) => SearchEngineType::Other,
        }
    }

    /// 获取默认搜索域名
    pub fn default_domain(&self) -> &str {
        match self {
            Self::Google => "google.com",
            Self::Bing => "bing.com",
            Self::Baidu => "baidu.com",
            Self::Yahoo => "yahoo.com",
            Self::DuckDuckGo => "duckduckgo.com",
            Self::Yandex => "yandex.com",
            Self::Other(_) => "",
        }
    }
}

impl std::fmt::Display for SearchEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Google => write!(f, "Google"),
            Self::Bing => write!(f, "Bing"),
            Self::Baidu => write!(f, "Baidu"),
            Self::Yahoo => write!(f, "Yahoo"),
            Self::DuckDuckGo => write!(f, "DuckDuckGo"),
            Self::Yandex => write!(f, "Yandex"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

/// 搜索引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SearchEngineType {
    /// 通用搜索引擎
    General,
    /// 代码搜索 (GitHub, GitLab)
    Code,
    /// 学术搜索 (Google Scholar, PubMed)
    Academic,
    /// 图片搜索
    Image,
    /// 视频搜索
    Video,
    /// 购物搜索
    Shopping,
    /// 其他
    Other,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 标题
    pub title: String,
    /// URL
    pub url: String,
    /// 摘要
    pub snippet: String,
    /// 排名位置
    pub position: u32,
    /// 搜索引擎
    pub engine: SearchEngine,
    /// 搜索关键词
    pub keyword: String,
    /// 是否为广告
    pub is_ad: bool,
    /// 页面权重 (如果有)
    pub page_authority: Option<f64>,
    /// 域名权重 (如果有)
    pub domain_authority: Option<f64>,
}

/// 关键词难度评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordDifficulty {
    /// 关键词
    pub keyword: String,
    /// 难度评分 (0-100)
    pub difficulty: u8,
    /// 搜索量 (月)
    pub search_volume: Option<u32>,
    /// 竞争度 (0-1)
    pub competition: f64,
    /// CPC (如果有)
    pub cpc: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_from_str() {
        assert_eq!(
            SearchEngine::from_str("Google"),
            Some(SearchEngine::Google)
        );
        assert_eq!(
            SearchEngine::from_str("百度"),
            Some(SearchEngine::Baidu)
        );
        assert_eq!(
            SearchEngine::from_str("Bing"),
            Some(SearchEngine::Bing)
        );
    }

    #[test]
    fn test_search_engine_properties() {
        assert_eq!(SearchEngine::Google.default_domain(), "google.com");
        assert_eq!(SearchEngine::Baidu.default_domain(), "baidu.com");
        assert_eq!(
            SearchEngine::Google.engine_type(),
            SearchEngineType::General
        );
    }

    #[test]
    fn test_search_result() {
        let result = SearchResult {
            title: "Example".into(),
            url: "https://example.com".into(),
            snippet: "Example snippet".into(),
            position: 1,
            engine: SearchEngine::Google,
            keyword: "example".into(),
            is_ad: false,
            page_authority: Some(50.0),
            domain_authority: Some(80.0),
        };

        assert_eq!(result.position, 1);
        assert!(!result.is_ad);
    }
}
