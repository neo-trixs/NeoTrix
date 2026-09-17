//! 薄摄入层 — 外部源 → 晶体意识
//!
//! 不做处理，只做归一化。所有外部数据通过这里进入晶体意识。

use super::consciousness::{CrystalConsciousness, MemoryType};
use std::path::PathBuf;

/// 摄入源
pub enum IngestionSource {
    /// pending-absorb.json (会话经验)
    PendingAbsorb,
    /// KB pattern 命名空间
    KBPatterns,
    /// KB theory 命名空间
    KBTheories,
    /// crawl_queue.jsonl (外部抓取)
    CrawlQueue,
    /// 任意 JSONL 文件
    CustomJSONL(PathBuf),
}

/// 薄摄入引擎 — 只做归一化，不做处理
pub struct IngestionEngine;

impl IngestionEngine {
    /// 从所有源摄入到晶体意识
    pub fn ingest_all(consciousness: &mut CrystalConsciousness) -> IngestionReport {
        let mut report = IngestionReport::default();

        report.pending_absorb = Self::ingest_pending_absorb(consciousness);
        report.kb_patterns = Self::ingest_kb_namespace(consciousness, "pattern");
        report.kb_theories = Self::ingest_kb_namespace(consciousness, "theory");
        report.crawl_queue = Self::ingest_crawl_queue(consciousness);

        report.total = report.pending_absorb + report.kb_patterns + report.kb_theories + report.crawl_queue;
        report
    }

    /// 从 pending-absorb.json 摄入
    fn ingest_pending_absorb(consciousness: &mut CrystalConsciousness) -> usize {
        let path = home_path(".neotrix/pending-absorb.json");
        if !path.exists() {
            return 0;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let items = match super::engine::CrystalEngine::parse_pending_absorb(&content) {
            Ok(i) => i,
            Err(_) => return 0,
        };

        let count = items.len();
        for item in items {
            let content = format!("{}: {} → {}", item.context, item.action, item.result);
            consciousness.remember(
                content,
                MemoryType::Experience,
                &item.domain,
                0.7,
            );
        }

        let _ = std::fs::remove_file(&path);
        count
    }

    /// 从 KB 命名空间摄入
    fn ingest_kb_namespace(consciousness: &mut CrystalConsciousness, namespace: &str) -> usize {
        let kb_path = home_path(".neotrix/knowledge.db");
        if !kb_path.exists() {
            return 0;
        }

        let ns_path = kb_path.parent()
            .unwrap_or(&kb_path)
            .join(format!("kb_{}.jsonl", namespace));

        if !ns_path.exists() {
            return 0;
        }

        let content = match std::fs::read_to_string(&ns_path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut count = 0;
        for line in content.lines() {
            if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
                let content = item.get("content")
                    .or_else(|| item.get("claim"))
                    .or_else(|| item.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let domain = item.get("domain")
                    .and_then(|v| v.as_str())
                    .unwrap_or(namespace);

                if !content.is_empty() {
                    let memory_type = match namespace {
                        "pattern" => MemoryType::Pattern,
                        "theory" => MemoryType::Fact,
                        _ => MemoryType::Fact,
                    };
                    consciousness.remember(content, memory_type, domain, 0.6);
                    count += 1;
                }
            }
        }

        count
    }

    /// 从 crawl_queue.jsonl 摄入
    fn ingest_crawl_queue(consciousness: &mut CrystalConsciousness) -> usize {
        let path = home_path(".neotrix/crawl_queue.jsonl");
        if !path.exists() {
            return 0;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut count = 0;
        for line in content.lines().take(100) { // 限制每次最多100条
            if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
                let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let text = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let domain = item.get("domain").and_then(|v| v.as_str()).unwrap_or("web");

                if !text.is_empty() {
                    let content = format!("[{}] {}", url, if title.is_empty() { text } else { title });
                    consciousness.remember(content, MemoryType::Fact, domain, 0.5);
                    count += 1;
                }
            }
        }

        count
    }

    /// 从任意 JSONL 文件摄入
    pub fn ingest_jsonl(consciousness: &mut CrystalConsciousness, path: &PathBuf, domain: &str) -> usize {
        if !path.exists() {
            return 0;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut count = 0;
        for line in content.lines() {
            if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
                let content = item.get("content")
                    .or_else(|| item.get("text"))
                    .or_else(|| item.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if !content.is_empty() {
                    consciousness.remember(content, MemoryType::Fact, domain, 0.5);
                    count += 1;
                }
            }
        }

        count
    }
}

/// 摄入报告
#[derive(Debug, Default)]
pub struct IngestionReport {
    pub pending_absorb: usize,
    pub kb_patterns: usize,
    pub kb_theories: usize,
    pub crawl_queue: usize,
    pub total: usize,
}

fn home_path(relative: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(relative)
}
