#![forbid(unsafe_code)]

//! OSINT 军火库索引 — 外部吸收批次 (rawfilejson/awesome-osint-arsenal).
//!
//! - **数据源**: `rawfilejson/awesome-osint-arsenal` 开源情报(OSINT)工具/源聚合清单.
//! - **落点**: NT-WORLD 感知层 — 情报源目录索引. 提供按类别(分类)检索情报源的能力 stub
//!   与基础列表结构, 作为 `nt_world_osint` 域的目录入口 (R-P42 强化现有节点而非平行适配器).
//! - **成熟度**: C1 — 单元测 + SelfTest 存在级. 真实抓取接线推迟至 C2 (KB 接入).
//! - **Egress**: 仅索引元数据, 无主动外联; 具体源拉取沿用 `nt_shield_sandbox` Egress Policy.

use crate::core::nt_core_self_test::SelfTest;

/// 情报源分类 (OSINT 军火库顶层维度).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum _OsintCategory {
    People,
    Domains,
    Networks,
    Geospatial,
    Social,
    Archives,
    ThreatIntel,
    Other,
}

/// 单个情报源条目 (目录索引的最小单元).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OsintSource {
    pub name: String,
    pub url: String,
    pub category: _OsintCategory,
    pub description: String,
    #[serde(default)]
    pub requires_api_key: bool,
}

impl OsintSource {
    pub fn new(
        name: &str,
        url: &str,
        category: _OsintCategory,
        description: &str,
        requires_api_key: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            category,
            description: description.to_string(),
            requires_api_key,
        }
    }

    /// 轻量校验: 名称/URL 非空且 URL 形如 http(s). 供 SelfTest 离线验证.
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty() || self.url.is_empty() {
            return false;
        }
        self.url.starts_with("http://") || self.url.starts_with("https://")
    }
}

/// OSINT 军火库目录索引 — 管理情报源列表与分类检索.
#[derive(Debug, Clone, Default)]
pub struct _OsintArsenal {
    sources: Vec<OsintSource>,
}

impl _OsintArsenal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, source: OsintSource) {
        self.sources.push(source);
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// 按分类检索情报源 (返回该类别下全部条目).
    pub fn by_category(&self, category: _OsintCategory) -> Vec<&OsintSource> {
        self.sources
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// 按名称子串检索 (大小写不敏感).
    pub fn search(&self, keyword: &str) -> Vec<&OsintSource> {
        let kw = keyword.to_lowercase();
        self.sources
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&kw) || s.description.to_lowercase().contains(&kw)
            })
            .collect()
    }

    /// 返回全部无效 (is_valid 失败) 的条目, 供 SelfTest 暴露脏数据.
    pub fn _invalid_sources(&self) -> Vec<&OsintSource> {
        self.sources.iter().filter(|s| !s.is_valid()).collect()
    }
}

/// 基线目录 — 从 awesome-osint-arsenal 抽取的代表性条目 (stub 种子).
pub fn _baseline_arsenal() -> _OsintArsenal {
    let mut a = _OsintArsenal::new();
    a.add(OsintSource::new(
        "BGPview",
        "https://bgpview.io",
        _OsintCategory::Networks,
        "BGP/ASN/路由情报检索",
        false,
    ));
    a.add(OsintSource::new(
        "OpenCorporates",
        "https://opencorporates.com",
        _OsintCategory::People,
        "全球公司注册数据库",
        false,
    ));
    a.add(OsintSource::new(
        "Shodan",
        "https://www.shodan.io",
        _OsintCategory::Networks,
        "联网设备/端口搜索引擎",
        true,
    ));
    a.add(OsintSource::new(
        "Wayback Machine",
        "https://web.archive.org",
        _OsintCategory::Archives,
        "网页历史快照归档",
        false,
    ));
    a
}

pub struct _OsintArsenalSelfTest;
impl SelfTest for _OsintArsenalSelfTest {
    fn name(&self) -> &str {
        "world:osint_arsenal"
    }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let a = _baseline_arsenal();
        if a.is_empty() {
            return Err(vec!["osint arsenal baseline empty".into()]);
        }
        if !a._invalid_sources().is_empty() {
            return Err(vec!["osint arsenal contains invalid sources".into()]);
        }
        if a.by_category(_OsintCategory::Networks).len() < 2 {
            return Err(vec!["osint arsenal networks category sparse".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_nonempty_and_valid() {
        let a = _baseline_arsenal();
        assert!(!a.is_empty());
        assert!(a._invalid_sources().is_empty());
    }

    #[test]
    fn category_and_search_retrieval() {
        let a = _baseline_arsenal();
        assert!(a.by_category(_OsintCategory::Networks).len() >= 2);
        assert!(!a.search("bgp").is_empty());
        assert!(a.search("zzz-no-such-source").is_empty());
    }

    #[test]
    fn source_validation_flags_bad_url() {
        let mut a = _OsintArsenal::new();
        a.add(OsintSource::new(
            "broken",
            "not-a-url",
            _OsintCategory::Other,
            "bad entry",
            false,
        ));
        assert_eq!(a._invalid_sources().len(), 1);
    }
}
