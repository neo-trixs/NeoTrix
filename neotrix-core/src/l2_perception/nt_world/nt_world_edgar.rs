#![forbid(unsafe_code)]

//! SEC EDGAR API 接线 — Wave6 H4 第2个情报工具 (P1, 10工具中第2个)
//!
//! - **数据源**: SEC EDGAR 公司文件档案 (`sec.gov`, 免费无key，速率限制 10 req/s)
//! - **端点**: `https://data.sec.gov/submissions/CIK{10位数字}.json` (公司文件索引)
//!   `https://data.sec.gov/api/xbrl/companyfacts/CIK{10位数字}.json` (XBRL 财务事实)
//! - **强化节点**: `nt_world_search::Ordered Backend Router` (R-P42) — 新增 `EdgarBackend`
//!   作为有序路由第4后端 (DDG → Wikipedia → GDELT → EDGAR)，亦可独立作为 `EdgarFetcher`
//!   直喂 intel-watch (R-P79 具名消费者)
//! - **入库**: `nt_memory_kb::KnowledgeBase` → `insert_or_get_node` (Filing, domain=edgar)
//! - **Egress**: `nt_shield_sandbox::INTEL_EDGAR_HOST` allow 登记 (deny-wins)
//! - **测试**: fixture 驱动，无真实网络依赖，CI 稳定
//!
//! 参考: SEC EDGAR API 文档 https://www.sec.gov/edgar/sec-api-documentation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── SEC EDGAR 数据模型 ────────────────────────────────────────────

/// SEC 公司文件提交记录 (submissions 端点返回的核心结构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarSubmission {
    #[serde(default)]
    pub cik: String,
    #[serde(default)]
    pub entity_type: String,
    #[serde(default)]
    pub sic: String,
    #[serde(default)]
    pub sic_description: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tickers: Vec<String>,
    #[serde(default)]
    pub exchanges: Vec<String>,
    #[serde(default)]
    pub ein: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub investor_website: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub fiscal_year_end: String,
    #[serde(default)]
    pub state_of_incorporation: String,
    #[serde(default)]
    pub state_of_incorporation_description: String,
    #[serde(default)]
    pub addresses: EdgarAddresses,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub flags: String,
    #[serde(default)]
    pub former_names: Vec<EdgarFormerName>,
    /// 近期文件列表
    #[serde(default)]
    pub filings: EdgarFilings,
}

/// 近期文件列表
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgarFilings {
    #[serde(default)]
    pub recent: EdgarRecentFilings,
    #[serde(default)]
    pub files: Vec<EdgarFilingFile>,
}

/// 最近文件详情
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgarRecentFilings {
    #[serde(default)]
    pub accession_number: Vec<String>,
    #[serde(default)]
    pub filing_date: Vec<String>,
    #[serde(default)]
    pub report_date: Vec<String>,
    #[serde(default)]
    pub acceptance_datetime: Vec<String>,
    #[serde(default)]
    pub act: Vec<String>,
    #[serde(default)]
    pub form: Vec<String>,
    #[serde(default)]
    pub file_number: Vec<String>,
    #[serde(default)]
    pub film_number: Vec<String>,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default)]
    pub size: Vec<i64>,
    #[serde(default)]
    pub is_xbrl: Vec<i64>,
    #[serde(default)]
    pub is_inline_xbrl: Vec<i64>,
    #[serde(default)]
    pub primary_document: Vec<String>,
    #[serde(default)]
    pub primary_doc_description: Vec<String>,
}

/// 历史文件索引文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarFilingFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub filing_from: String,
    #[serde(default)]
    pub filing_to: String,
}

/// 公司地址
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgarAddresses {
    #[serde(default)]
    pub mailing: EdgarAddress,
    #[serde(default)]
    pub business: EdgarAddress,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgarAddress {
    #[serde(default)]
    pub street1: String,
    #[serde(default)]
    pub street2: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub state_or_country: String,
    #[serde(default)]
    pub zip_code: String,
    #[serde(default)]
    pub state_or_country_description: String,
}

/// 前身名称
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarFormerName {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub date: String,
}

/// 单个文件条目 (用于入库)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarFiling {
    pub cik: String,
    pub accession_number: String,
    pub filing_date: String,
    pub report_date: String,
    pub acceptance_datetime: String,
    pub form: String,
    pub file_number: String,
    pub film_number: String,
    pub items: String,
    pub size: i64,
    pub is_xbrl: bool,
    pub is_inline_xbrl: bool,
    pub primary_document: String,
    pub primary_doc_description: String,
    pub company_name: String,
    pub company_tickers: Vec<String>,
    pub form_type: String,
}

/// XBRL 公司事实 (简化版，仅核心字段)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarCompanyFacts {
    #[serde(default)]
    pub cik: String,
    #[serde(default)]
    pub entity_name: String,
    #[serde(default)]
    pub facts: HashMap<String, HashMap<String, EdgarFactValue>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarFactValue {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub units: HashMap<String, Vec<EdgarFactUnit>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgarFactUnit {
    #[serde(default)]
    pub start: String,
    #[serde(default)]
    pub end: String,
    #[serde(default)]
    pub val: serde_json::Value,
    #[serde(default)]
    pub accn: String,
    #[serde(default)]
    pub fy: i32,
    #[serde(default)]
    pub fp: String,
    #[serde(default)]
    pub form: String,
    #[serde(default)]
    pub filed: String,
    #[serde(default)]
    pub frame: String,
}

// ── 解析层 ──────────────────────────────────────────────────────

/// 从 submissions JSON 提取文件列表
fn extract_filings_from_submission(sub: &EdgarSubmission) -> Vec<EdgarFiling> {
    let recent = &sub.filings.recent;
    let n = recent.accession_number.len().min(recent.filing_date.len());
    let mut filings = Vec::with_capacity(n);
    
    for i in 0..n {
        filings.push(EdgarFiling {
            cik: sub.cik.clone(),
            accession_number: recent.accession_number.get(i).cloned().unwrap_or_default(),
            filing_date: recent.filing_date.get(i).cloned().unwrap_or_default(),
            report_date: recent.report_date.get(i).cloned().unwrap_or_default(),
            acceptance_datetime: recent.acceptance_datetime.get(i).cloned().unwrap_or_default(),
            form: recent.form.get(i).cloned().unwrap_or_default(),
            file_number: recent.file_number.get(i).cloned().unwrap_or_default(),
            film_number: recent.film_number.get(i).cloned().unwrap_or_default(),
            items: recent.items.get(i).cloned().unwrap_or_default(),
            size: recent.size.get(i).copied().unwrap_or(0),
            is_xbrl: recent.is_xbrl.get(i).copied().unwrap_or(0) == 1,
            is_inline_xbrl: recent.is_inline_xbrl.get(i).copied().unwrap_or(0) == 1,
            primary_document: recent.primary_document.get(i).cloned().unwrap_or_default(),
            primary_doc_description: recent.primary_doc_description.get(i).cloned().unwrap_or_default(),
            company_name: sub.name.clone(),
            company_tickers: sub.tickers.clone(),
            form_type: recent.form.get(i).cloned().unwrap_or_default(),
        });
    }
    filings
}

// ── Fetch 层 ────────────────────────────────────────────────────

/// SEC EDGAR Fetcher — 免费无key，速率限制 10 req/s，直喂 intel-watch。
///
/// `base_url` 默认为 `https://data.sec.gov`，测试可注入本地 fixture URL。
pub struct EdgarFetcher {
    base_url: String,
    client: std::sync::OnceLock<reqwest::blocking::Client>,
    // 速率限制：简单的时间戳记录
    last_request: std::sync::Mutex<std::time::Instant>,
}

impl Default for EdgarFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgarFetcher {
    /// 生产构造 — 指向真实 SEC。
    pub fn new() -> Self {
        Self::with_base_url("https://data.sec.gov")
    }

    /// 测试/注入构造 — 指向任意 base (如 mock server)。
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: std::sync::OnceLock::new(),
            last_request: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self.client.get_or_init(|| {
            reqwest::blocking::Client::builder()
                .user_agent("NeoTrix/0.19 (SEC EDGAR intel-watch; contact: neotrix@example.com)")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::blocking::Client::new())
        })
    }

    /// 简单速率限制：确保请求间隔 ≥ 100ms (10 req/s)
    fn rate_limit(&self) {
        let mut last = self.last_request.lock().unwrap();
        let elapsed = last.elapsed();
        let min_interval = std::time::Duration::from_millis(100);
        if elapsed < min_interval {
            std::thread::sleep(min_interval - elapsed);
        }
        *last = std::time::Instant::now();
    }

    /// 构造 submissions 端点 URL (CIK 必须 10 位零填充)
    pub fn build_submissions_url(&self, cik: &str) -> String {
        let padded = format!("{:0>10}", cik.trim_start_matches('0'));
        format!("{}/submissions/CIK{}.json", self.base_url, padded)
    }

    /// 构造 companyfacts 端点 URL
    pub fn build_companyfacts_url(&self, cik: &str) -> String {
        let padded = format!("{:0>10}", cik.trim_start_matches('0'));
        format!("{}/api/xbrl/companyfacts/CIK{}.json", self.base_url, padded)
    }

    /// 抓取公司文件提交记录 (网络依赖)。
    pub fn fetch_submissions(&self, cik: &str) -> Result<EdgarSubmission, String> {
        self.rate_limit();
        let url = self.build_submissions_url(cik);
        let resp = self
            .client()
            .get(&url)
            .send()
            .map_err(|e| format!("EDGAR request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("EDGAR returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("EDGAR read body: {}", e))?;
        serde_json::from_str(&text).map_err(|e| format!("EDGAR parse failed: {}", e))
    }

    /// 抓取公司 XBRL 财务事实 (网络依赖)。
    pub fn fetch_companyfacts(&self, cik: &str) -> Result<EdgarCompanyFacts, String> {
        self.rate_limit();
        let url = self.build_companyfacts_url(cik);
        let resp = self
            .client()
            .get(&url)
            .send()
            .map_err(|e| format!("EDGAR companyfacts request failed: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("EDGAR companyfacts returned status: {}", resp.status()));
        }
        let text = resp.text().map_err(|e| format!("EDGAR companyfacts read body: {}", e))?;
        serde_json::from_str(&text).map_err(|e| format!("EDGAR companyfacts parse failed: {}", e))
    }

    /// 纯解析 submissions JSON (无网络，用于 fixture/单测)。
    pub fn parse_submissions(json: &str) -> Result<EdgarSubmission, String> {
        serde_json::from_str(json).map_err(|e| format!("EDGAR parse failed: {}", e))
    }

    /// 从给定 submissions JSON 解析并入库 (无网络，用于 fixture E2E)。
    pub fn ingest_submissions_from_json(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        json: &str,
        cik: &str,
    ) -> Result<EdgarIngestReport, String> {
        let sub = Self::parse_submissions(json)?;
        let filings = extract_filings_from_submission(&sub);
        Self::ingest_filings(kb, &filings, cik)
    }

    /// E2E 入库：fetch submissions → 解析 → KB `insert_or_get_node` (Filing, domain=edgar)。
    pub fn ingest_submissions(
        &self,
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        cik: &str,
    ) -> Result<EdgarIngestReport, String> {
        let sub = self.fetch_submissions(cik)?;
        let filings = extract_filings_from_submission(&sub);
        Self::ingest_filings(kb, &filings, cik)
    }

    /// 将已解析的 filings 入库 — 可复用 (fetch/parse 解耦)。
    pub fn ingest_filings(
        kb: &crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase,
        filings: &[EdgarFiling],
        cik: &str,
    ) -> Result<EdgarIngestReport, String> {
        let mut report = EdgarIngestReport {
            cik: cik.to_string(),
            filings_fetched: filings.len(),
            ..Default::default()
        };
        for f in filings {
            if f.accession_number.trim().is_empty() || f.form.trim().is_empty() {
                report.errors.push(format!("skip empty accession/form: {:?}", f.accession_number));
                continue;
            }
            // 去重：同 accession_number 复用
            let summary = format!(
                "{} | {} | {} | {} | {} | {}",
                f.form, f.filing_date, f.accession_number, f.company_name, f.form_type, f.items
            );
            let url = format!("https://www.sec.gov/Archives/edgar/data/{}/{}", cik.trim_start_matches('0'), f.accession_number.replace('-', ""));
            let existing = kb.find_node_by_url(&url).ok().flatten();
            let is_new = existing.is_none();
            let _id = kb
                .insert_or_get_node(&format!("{} ({})", f.form, f.accession_number), crate::core::nt_core_kb_types::NodeType::Filing, Some(&summary), Some(&url), Some("edgar"))
                .map_err(|e| format!("KB ingest failed for {}: {}", f.accession_number, e))?;
            if is_new {
                report.nodes_created += 1;
            } else {
                report.nodes_reused += 1;
            }
        }
        Ok(report)
    }

    /// 转 SearchResult (供 Ordered Backend Router 复用)。
    pub fn to_search_results(filings: &[EdgarFiling]) -> Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult> {
        filings
            .iter()
            .map(|f| crate::l2_perception::nt_world::nt_world_search::SearchResult {
                title: format!("{} ({})", f.form, f.accession_number),
                url: format!("https://www.sec.gov/Archives/edgar/data/{}/{}", f.cik.trim_start_matches('0'), f.accession_number.replace('-', "")),
                snippet: format!("{} | {} | {}", f.filing_date, f.form, f.company_name),
                evidence: None,
            })
            .collect()
    }
}

// ── Egress 登记 ─────────────────────────────────────────────────

/// SEC EDGAR Egress 主机 — 单一事实源 (P2)。
pub const EDGAR_HOST: &str = "data.sec.gov";
/// SEC EDGAR Egress allow 规则 (deny-wins 体系中的 allow 分支)。
pub fn edgar_egress_rule() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::allow(EDGAR_HOST, "443")
}
/// SEC EDGAR 专用 Egress Policy (deny_all 基线 + 单条 allow)。
pub fn edgar_egress_policy() -> crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy {
    crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressPolicy::new(vec![edgar_egress_rule()], false)
}

// ── 入库报告 ───────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgarIngestReport {
    pub cik: String,
    pub filings_fetched: usize,
    pub nodes_created: usize,
    pub nodes_reused: usize,
    pub errors: Vec<String>,
}

// ── SearchBackend 适配 (R-P42 强化 Ordered Backend Router) ──────

/// SEC EDGAR 作为有序搜索后端 (P1 第2个情报后端，免费无key)。
pub struct EdgarBackend {
    fetcher: EdgarFetcher,
}

impl Default for EdgarBackend {
    fn default() -> Self {
        Self { fetcher: EdgarFetcher::new() }
    }
}

impl EdgarBackend {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_base_url(base: &str) -> Self {
        Self { fetcher: EdgarFetcher::with_base_url(base) }
    }
}

impl crate::l2_perception::nt_world::nt_world_search::SearchBackend for EdgarBackend {
    fn name(&self) -> &str {
        "edgar"
    }
    fn search(&self, _query: &str, _count: usize) -> Result<Vec<crate::l2_perception::nt_world::nt_world_search::SearchResult>, String> {
        // EDGAR 不支持通用搜索查询，返回空结果 (仅通过 CIK 精确查询)
        // 实际使用时通过 CIK 精确获取，搜索路由不应将通用 query 发给 EDGAR
        Ok(vec![])
    }
}

// ── Fixture (测试共用) ──────────────────────────────────────────

/// 最小 submissions fixture：Apple (CIK 0000320193) 的简化版
pub const EDGAR_FIXTURE_JSON: &str = r#"{
  "cik": "0000320193",
  "entityType": "10-K",
  "sic": "3571",
  "sicDescription": "Electronic Computers",
  "name": "Apple Inc.",
  "tickers": ["AAPL"],
  "exchanges": ["NASDAQ"],
  "ein": "942404110",
  "description": "Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories.",
  "website": "https://www.apple.com",
  "investorWebsite": "https://investor.apple.com",
  "category": "Large Accelerated Filer",
  "fiscalYearEnd": "0925",
  "stateOfIncorporation": "CA",
  "stateOfIncorporationDescription": "California",
  "addresses": {
    "mailing": {
      "street1": "One Apple Park Way",
      "city": "Cupertino",
      "stateOrCountry": "CA",
      "zipCode": "95014",
      "stateOrCountryDescription": "California"
    },
    "business": {
      "street1": "One Apple Park Way",
      "city": "Cupertino",
      "stateOrCountry": "CA",
      "zipCode": "95014",
      "stateOrCountryDescription": "California"
    }
  },
  "phone": "408-996-1010",
  "flags": "",
  "formerNames": [],
  "filings": {
    "recent": {
      "accessionNumber": ["0000320193-23-000106", "0000320193-23-000077"],
      "filingDate": ["2023-11-03", "2023-08-04"],
      "reportDate": ["2023-09-30", "2023-06-30"],
      "acceptanceDateTime": ["2023-11-03T16:31:12.000Z", "2023-08-04T16:45:22.000Z"],
      "act": ["34", "34"],
      "form": ["10-Q", "10-Q"],
      "fileNumber": ["001-36743", "001-36743"],
      "filmNumber": ["231428593", "231185234"],
      "items": ["", ""],
      "size": [1234567, 987654],
      "isXBRL": [1, 1],
      "isInlineXBRL": [1, 1],
      "primaryDocument": ["aapl-20230930.htm", "aapl-20230630.htm"],
      "primaryDocDescription": ["Quarterly Report", "Quarterly Report"]
    },
    "files": []
  }
}"#;

// ── 测试 ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fixture() {
        let sub = EdgarFetcher::parse_submissions(EDGAR_FIXTURE_JSON).expect("parse fixture");
        assert_eq!(sub.cik, "0000320193");
        assert_eq!(sub.name, "Apple Inc.");
        assert_eq!(sub.tickers, vec!["AAPL"]);
        assert_eq!(sub.former_names.len(), 0);
    }

    #[test]
    fn test_extract_filings() {
        let sub = EdgarFetcher::parse_submissions(EDGAR_FIXTURE_JSON).expect("parse fixture");
        let filings = extract_filings_from_submission(&sub);
        assert_eq!(filings.len(), 2);
        assert_eq!(filings[0].accession_number, "0000320193-23-000106");
        assert_eq!(filings[0].form, "10-Q");
        assert_eq!(filings[0].filing_date, "2023-11-03");
        assert_eq!(filings[0].company_name, "Apple Inc.");
        assert_eq!(filings[0].company_tickers, vec!["AAPL"]);
        assert_eq!(filings[1].accession_number, "0000320193-23-000077");
        assert_eq!(filings[1].form, "10-Q");
    }

    #[test]
    fn test_build_submissions_url() {
        let f = EdgarFetcher::new();
        let url = f.build_submissions_url("320193");
        assert_eq!(url, "https://data.sec.gov/submissions/CIK0000320193.json");
        let url2 = f.build_submissions_url("0000320193");
        assert_eq!(url2, "https://data.sec.gov/submissions/CIK0000320193.json");
    }

    #[test]
    fn test_build_companyfacts_url() {
        let f = EdgarFetcher::new();
        let url = f.build_companyfacts_url("320193");
        assert_eq!(url, "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json");
    }

    #[test]
    fn test_egress_policy_allows_edgar_denies_other() {
        let policy = edgar_egress_policy();
        assert!(policy.check(EDGAR_HOST, 443), "edgar host should be allowed on 443");
        assert!(!policy.check(EDGAR_HOST, 80), "wrong port should be denied");
        assert!(!policy.check("evil.com", 443), "non-edgar host denied");
        // deny-wins: 叠加 deny 规则应覆盖 allow
        let mut with_deny = edgar_egress_policy();
        with_deny.rules.push(crate::l3_embodiment::nt_shield::nt_shield_sandbox::EgressRule::deny(EDGAR_HOST, "443"));
        assert!(!with_deny.check(EDGAR_HOST, 443), "explicit deny wins over allow");
    }

    #[test]
    fn test_to_search_results() {
        let sub = EdgarFetcher::parse_submissions(EDGAR_FIXTURE_JSON).unwrap();
        let filings = extract_filings_from_submission(&sub);
        let results = EdgarFetcher::to_search_results(&filings);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "10-Q (0000320193-23-000106)");
        assert!(results[0].url.contains("000032019323000106"));
        assert!(results[0].snippet.contains("Apple Inc."));
    }

    #[test]
    fn test_ingest_fixture_to_kb_and_requery() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test_edgar_kb.db");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
        let fetcher = EdgarFetcher::new();
        // 用 fixture 纯内存 ingest，无网络
        let report = fetcher.ingest_submissions_from_json(&kb, EDGAR_FIXTURE_JSON, "320193").expect("ingest");
        assert_eq!(report.filings_fetched, 2);
        assert_eq!(report.nodes_created, 2, "first ingest creates 2 nodes");
        assert_eq!(report.nodes_reused, 0);

        // 回查：URL 精确命中
        let url = "https://www.sec.gov/Archives/edgar/data/320193/000032019323000106";
        let node = kb.find_node_by_url(url).expect("find").expect("found");
        assert_eq!(node.title, "10-Q (0000320193-23-000106)");
        assert_eq!(node.domain.as_deref(), Some("edgar"));
        // FTS 搜索可回查
        let hits = kb.search_permission_aware("Apple", 10, crate::l1_action::nt_memory::nt_memory_kb::nt_memory_types::PermissionLevel::Public).expect("search");
        assert!(!hits.is_empty(), "FTS should recall ingested filing by keyword");

        // 幂等：二次 ingest 同 fixture → nodes_reused
        let report2 = fetcher.ingest_submissions_from_json(&kb, EDGAR_FIXTURE_JSON, "320193").expect("re-ingest");
        assert_eq!(report2.nodes_created, 0, "second ingest reuses nodes");
        assert_eq!(report2.nodes_reused, 2);
    }

    #[test]
    fn test_edgar_backend_name() {
        use crate::l2_perception::nt_world::nt_world_search::SearchBackend;
        let b = EdgarBackend::new();
        assert_eq!(SearchBackend::name(&b), "edgar");
    }

    #[test]
    fn test_ingest_skips_empty_accession() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test_edgar_skip.db");
        let kb = crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase::open(Some(db_path)).expect("open kb");
        let fetcher = EdgarFetcher::new();
        // 构造无 accession_number 的 bad fixture
        let bad_json = r#"{
            "cik":"0000320193","name":"Test","filings":{"recent":{"accessionNumber":[""],"filingDate":["2023-01-01"],"form":["10-K"],"acceptanceDateTime":["2023-01-01T00:00:00Z"],"fileNumber":[""],"filmNumber":[""],"items":[""],"size":[0],"isXBRL":[0],"isInlineXBRL":[0],"primaryDocument":[""],"primaryDocDescription":[""]}}
        }"#;
        let report = fetcher.ingest_submissions_from_json(&kb, bad_json, "320193").expect("ingest bad");
        assert_eq!(report.nodes_created, 0);
        assert_eq!(report.errors.len(), 1);
    }
}