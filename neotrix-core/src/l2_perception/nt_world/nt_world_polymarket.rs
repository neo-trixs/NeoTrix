#![forbid(unsafe_code)]

//! Polymarket Prediction Market 接线 — Wave6 H4 第8个情报工具 (P1, 10工具中第8个)
//!
//! - **数据源**: Polymarket Gamma API (`gamma-api.polymarket.com`, 免费无key, JSON)
//! - **端点**: `https://gamma-api.polymarket.com/markets?limit=10&active=true`
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `PolymarketBackend`
//! - **入库**: `nt_memory_kb::KnowledgeBase` (Event, domain=polymarket)
//! - **Egress**: `nt_shield_sandbox::INTEL_POLYMARKET_HOST` allow (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定

use serde::{Deserialize, Serialize};

// ── Polymarket 数据模型 ───────────────────────────────────────────

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PolymarketMarketRaw {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub outcomes: String,
    #[serde(default)]
    pub outcome_prices: String,
    #[serde(default)]
    pub volume: String,
    #[serde(default)]
    pub liquidity: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub end_date: String,
    #[serde(default)]
    pub group_item_title: Option<String>,
    #[serde(default)]
    pub market_maker_address: String,
}

/// 内部标准化结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketMarket {
    pub id: String,
    pub question: String,
    pub description: String,
    pub url: String,
    pub volume: String,
    pub end_date: String,
    pub active: bool,
}

fn extract_market(raw: &PolymarketMarketRaw) -> PolymarketMarket {
    PolymarketMarket {
        id: raw.id.clone(),
        question: raw.question.clone(),
        description: raw.description.clone(),
        url: format!("https://polymarket.com/event/{}", raw.slug),
        volume: raw.volume.clone(),
        end_date: raw.end_date.clone(),
        active: raw.active,
    }
}

// ── Fetch 层 ────────────────────────────────────────────────────

pub struct PolymarketFetcher {
    client: std::sync::OnceLock<reqwest::blocking::Client>,
}

impl Default for PolymarketFetcher {
    fn default() -> Self { Self::new() }
}

impl PolymarketFetcher {
    pub fn new() -> Self { Self { client: std::sync::OnceLock::new() } }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (Polymarket intel-watch; https://github.com/neotrix)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    pub fn build_url(&self) -> String {
        "https://gamma-api.polymarket.com/markets?limit=10&active=true".to_string()
    }

    pub fn fetch(&self) -> Result<Vec<PolymarketMarket>, String> {
        let resp = self.client().get(self.build_url()).send().map_err(|e| format!("Polymarket request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("Polymarket returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("Polymarket read body: {}", e))?;
        Self::parse_json(&text)
    }

    pub fn parse_json(json: &str) -> Result<Vec<PolymarketMarket>, String> {
        let raw: Vec<PolymarketMarketRaw> = serde_json::from_str(json).map_err(|e| format!("Polymarket parse failed: {}", e))?;
        Ok(raw.iter().map(extract_market).collect())
    }

    pub fn ingest_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
    ) -> Result<PolymarketIngestReport, String> {
        let events = Self::parse_json(json)?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
    ) -> Result<PolymarketIngestReport, String> {
        let events = self.fetch()?;
        Self::ingest_events(kb, &events)
    }

    pub fn ingest_events(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        events: &[PolymarketMarket],
    ) -> Result<PolymarketIngestReport, String> {
        let mut report = PolymarketIngestReport { events_fetched: events.len(), ..Default::default() };
        for evt in events {
            if evt.id.trim().is_empty() { report.errors.push("skip empty id".into()); continue; }
            let summary = format!("Q: {} | vol={} | end={}", evt.question, evt.volume, evt.end_date);
            let existing = kb.find_node_by_url(&evt.url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb.insert_or_get_node(&evt.question, crate::core::nt_core_kb_types::NodeType::Event, Some(&summary), Some(&evt.url), Some("polymarket"))
                .map_err(|e| format!("KB ingest failed for {}: {}", evt.id, e))?;
            if is_new { report.nodes_created += 1; } else { report.nodes_reused += 1; }
        }
        Ok(report)
    }

    pub fn to_search_results(events: &[PolymarketMarket]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        events.iter().map(|e| crate::l2_perception::nt_world::nt_world_search::SearchResult {
            title: e.question.clone(),
            url: e.url.clone(),
            snippet: format!("vol={} | end={} | active={}", e.volume, e.end_date, e.active),
            evidence: None,
        }).collect()
    }
}

// ── Egress ───────────────────────────────────────────────────

pub const POLYMARKET_HOST: &str = "gamma-api.polymarket.com";
pub fn polymarket_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(POLYMARKET_HOST, "443")
}
pub fn polymarket_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![polymarket_egress_rule()], false)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolymarketIngestReport {
    pub events_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend ────────────────────────────────────────────

pub struct PolymarketBackend {
    fetcher: PolymarketFetcher,
}

impl Default for PolymarketBackend {
    fn default() -> Self { Self { fetcher: PolymarketFetcher::new() } }
}

impl PolymarketBackend {
    pub fn new() -> Self { Self::default() }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for PolymarketBackend {
    fn name(&self) -> &str { "polymarket" }
    fn search(&self, _query: &str, count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        let events = self.fetcher.fetch()?;
        let limited: Vec<PolymarketMarket> = events.into_iter().take(count).collect();
        Ok(PolymarketFetcher::to_search_results(&limited))
    }
}

// ── Fixture ────────────────────────────────────────────────

pub const POLYMARKET_FIXTURE_JSON: &str = r#"[{"id":"1","question":"Will X happen?","description":"Desc","slug":"will-x-happen","outcomes":"[\"Yes\",\"No\"]","outcomePrices":"[\"0.6\",\"0.4\"]","volume":"1000","liquidity":"500","active":true,"closed":false,"endDate":"2024-12-31","marketMakerAddress":"0xabc"}]"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let markets = PolymarketFetcher::parse_json(POLYMARKET_FIXTURE_JSON).expect("parse");
        assert_eq!(markets.len(), 1);
        assert_eq!(markets[0].question, "Will X happen?");
        assert!(markets[0].url.contains("will-x-happen"));
    }

    #[test]
    fn test_build_url() {
        assert_eq!(PolymarketFetcher::new().build_url(), "https://gamma-api.polymarket.com/markets?limit=10&active=true");
    }

    #[test]
    fn test_egress_policy() {
        let policy = polymarket_egress_policy();
        assert!(policy.check("gamma-api.polymarket.com", 443));
        assert!(!policy.check("evil.com", 443));
    }

    #[test]
    fn test_ingest_fixture() {
        let dir = tempfile::tempdir().expect("tempdir");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(dir.path().join("test.db"))).expect("open kb");
        let report = PolymarketFetcher::new().ingest_from_json(&kb, POLYMARKET_FIXTURE_JSON).expect("ingest");
        assert_eq!(report.nodes_created, 1);
    }

    #[test]
    fn test_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        assert_eq!(SearchBackend::name(&PolymarketBackend::new()), "polymarket");
    }
}
