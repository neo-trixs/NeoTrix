//! nt_memory_crawl: 通用爬虫 (wiki/arxiv/github/hf + 地理摄取 + HTML/XML 工具).
//! 由单文件拆为目录模块, 外部路径保持不变.
pub mod nt_crawl_core;
pub mod nt_crawl_geo;
pub mod nt_crawl_parse;
pub mod nt_crawl_sources;

pub use nt_crawl_core::{
    CrawlCycleReport, discover_from_seed, enqueue_seed_urls, is_safe_fetch_url,
    on_node_inserted, resolve_blocking_client, run_crawl_cycle,
};
pub use nt_crawl_geo::{
    ingest_country_boundaries, ingest_geo_airports, ingest_geo_boundaries,
    ingest_geo_cities, ingest_geo_peaks, ingest_geo_vectors,
};
pub use nt_crawl_parse::{extract_html_content, extract_links};
pub use nt_crawl_sources::{
    ingest_from_alphaxiv_feed, ingest_from_arxiv, ingest_from_github,
    ingest_from_hf_dataset, ingest_from_openlibrary, ingest_from_wikipedia,
    run_hf_queue_batch,
};

#[cfg(test)]
mod tests {
    use super::nt_crawl_core::is_safe_fetch_url;
    use super::nt_crawl_parse::extract_xml_tag;

    // arXiv export API 返回 feed, 首个 <title> 是 feed 级 "arXiv Query: ...",
    // 论文 title/summary/author 在 <entry> 内。此测试锁定 entry 截取逻辑,
    // 防止回归到 feed 级 title (R-P16 持久化验证 + R-P80 吸收纪律)。
    #[test]
    fn test_arxiv_entry_parsing_extracts_paper_title() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>arXiv Query: search_query=&amp;id_list=2608.06922</title>
  <entry>
    <title>MAGE: Safeguarding LLM Agents against Long-Horizon Threats via Shadow Memory</title>
    <summary>Defensive framework using dedicated safety-focused agentic memory.</summary>
    <author><name>Alice Zhang</name></author>
    <author><name>Bob Li</name></author>
  </entry>
</feed>"#;
        // 复刻 ingest_from_arxiv 的 entry 截取 + 提取逻辑
        let entry = xml.find("<entry>").map(|i| &xml[i..]).unwrap_or(xml);
        let title = extract_xml_tag(entry, "title").unwrap_or_else(|| "Unknown".into());
        let summary = extract_xml_tag(entry, "summary").unwrap_or_default();
        let mut authors_str = String::new();
        let mut rest = entry;
        while let Some(name_start) = rest.find("<name>") {
            let after = &rest[name_start + 6..];
            if let Some(name_end) = after.find("</name>") {
                if !authors_str.is_empty() {
                    authors_str.push_str(", ");
                }
                authors_str.push_str(after[..name_end].trim());
                rest = &after[name_end + 7..];
            } else {
                break;
            }
        }

        assert!(
            title.contains("MAGE"),
            "title should be the paper title, got: {}",
            title
        );
        assert!(title.contains("Shadow Memory"));
        assert!(!title.contains("arXiv Query"));
        assert!(summary.contains("Defensive framework"));
        assert_eq!(authors_str, "Alice Zhang, Bob Li");
    }

    #[test]
    fn test_ssrf_rejects_loopback() {
        assert!(!is_safe_fetch_url("http://127.0.0.1/"));
        assert!(!is_safe_fetch_url("http://127.0.0.1:8080/admin"));
        assert!(!is_safe_fetch_url("https://localhost/"));
        assert!(!is_safe_fetch_url("http://localhost:3000"));
        assert!(!is_safe_fetch_url("http://test.localhost/"));
        assert!(!is_safe_fetch_url("http://foo.local/"));
        assert!(!is_safe_fetch_url("http://[::1]/"));
    }

    #[test]
    fn test_ssrf_rejects_private_and_reserved() {
        assert!(!is_safe_fetch_url("http://10.0.0.1/"));
        assert!(!is_safe_fetch_url("http://172.16.0.1/"));
        assert!(!is_safe_fetch_url("http://192.168.1.1/"));
        // AWS IMDS / cloud metadata (link-local)
        assert!(!is_safe_fetch_url(
            "http://169.254.169.254/latest/meta-data/"
        ));
        assert!(!is_safe_fetch_url("http://[fc00::1]/"));
        assert!(!is_safe_fetch_url("http://[fe80::1]/"));
    }

    #[test]
    fn test_ssrf_rejects_ipv4_mapped_ipv6() {
        // `::ffff:127.0.0.1` 与 `::ffff:192.168.0.1` 曾绕过旧守卫 (is_loopback 只匹配 ::1)
        assert!(!is_safe_fetch_url("http://[::ffff:127.0.0.1]/"));
        assert!(!is_safe_fetch_url("http://[::ffff:127.0.0.2]:8080/"));
        assert!(!is_safe_fetch_url("http://[::ffff:192.168.1.1]/"));
        assert!(!is_safe_fetch_url("http://[::ffff:10.0.0.1]/"));
    }

    #[test]
    fn test_ssrf_rejects_bad_scheme_and_unparseable() {
        assert!(!is_safe_fetch_url("ftp://example.com/file"));
        assert!(!is_safe_fetch_url("file:///etc/passwd"));
        assert!(!is_safe_fetch_url("javascript:alert(1)"));
        assert!(!is_safe_fetch_url(""));
        assert!(!is_safe_fetch_url("not a url"));
    }

    #[test]
    fn test_ssrf_allows_public() {
        assert!(is_safe_fetch_url("http://8.8.8.8/"));
        assert!(is_safe_fetch_url("https://1.1.1.1/"));
    }
}
