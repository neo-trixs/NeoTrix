//! NT-HF-BRIDGE — HuggingFace 开源训练数据 → 晶体摄入
//!
//! 用户指令：数据不足时从 huggingface.co/datasets 取开源训练数据。
//! 本桥只做三件事，全部落在**现有**摄入管线上，不碰核心：
//! 1. 解析数据集 URL → (owner, name)（口径与 nt_memory_crawl 对齐）；
//! 2. datasets-server rows API 取行（走 nt_http，享 SSRF guard + 重试）；
//! 3. 行 → `crawl_queue.jsonl` 格式（`{url,title,content,domain}`），
//!    由 `IngestionEngine::ingest_crawl_queue` 按每次 100 条消费。
//! 4. 广场目录条目 → `crawl_queue.jsonl` 行（`HfCatalogEntry`，只用 API 元数据，
//!    不取行数据；落地文件见 `datasets/hf_catalog/`，经 `neotrix-hf-catalog` 拉取）。
//!
//! 转换器是纯函数（fixture 可测）；取数是薄 IO（显式调用）。
//! 无 unwrap / expect / panic；无 `[]` 索引。

use super::consciousness::{CrystalConsciousness, MemoryType};

/// HF 行转出的记忆（先转结构，再决定落哪条管线）
#[derive(Debug, Clone)]
pub struct HfMemory {
    pub prompt: String,
    pub completion: String,
    pub thinking: Option<String>,
    pub source: String,
}

pub struct NtHfBridge;

impl NtHfBridge {
    /// 解析 `https://huggingface.co/datasets/owner/name` → (owner, name)
    pub fn parse_dataset_url(url: &str) -> Option<(String, String)> {
        let trimmed = url.trim().trim_end_matches('/');
        let path = trimmed
            .strip_prefix("https://huggingface.co/datasets/")
            .or_else(|| trimmed.strip_prefix("http://huggingface.co/datasets/"))
            .or_else(|| trimmed.strip_prefix("huggingface.co/datasets/"))?;
        let mut parts = path.split('/');
        let owner = parts.next().unwrap_or("").to_string();
        let name = parts.next().unwrap_or("").to_string();
        if owner.is_empty() || name.is_empty() {
            return None;
        }
        Some((owner, name))
    }

    /// datasets-server rows API 地址（默认取 train split 头部）
    pub fn rows_endpoint(owner: &str, name: &str, offset: u64, length: u64) -> String {
        format!(
            "https://datasets-server.huggingface.co/rows?dataset={owner}/{name}&config=default&split=train&offset={offset}&length={length}"
        )
    }

    /// 取行（薄 IO：走 nt_http，SSRF guard + 重试 + 代理全继承）
    pub fn fetch_rows(url: &str) -> Result<String, String> {
        crate::l4_emotion::nt_memory::nt_memory_kb::nt_http::fetch_safe_http_with_retry(url)
            .map(|(body, _host)| body)
    }

    /// datasets-server `{"rows":[{"row":{...}}]}` → 记忆结构。
    ///
    /// 字段映射（按优先级）：
    /// prompt ← instruction/question/problem/input/prompt；
    /// completion ← output/answer/response/completion/label；
    /// thinking ← reasoning/thought/thinking/chain_of_thought。
    /// 两者皆空的行丢弃。
    pub fn rows_to_memories(rows_json: &str, dataset_id: &str) -> Vec<HfMemory> {
        let v: serde_json::Value = match serde_json::from_str(rows_json) {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        };
        let rows = match v.get("rows").and_then(|r| r.as_array()) {
            Some(r) => r,
            None => return Vec::new(),
        };
        let mut out = Vec::new();
        for item in rows {
            let row = match item.get("row") {
                Some(r) => r,
                None => continue,
            };
            let prompt = first_str(row, &["instruction", "question", "problem", "input", "prompt", "query", "messages", "conversations"]);
            let completion = first_str(
                row,
                &["output", "answer", "response", "completion", "label", "target"],
            );
            if prompt.is_empty() && completion.is_empty() {
                continue;
            }
            let thinking = first_str(
                row,
                &["reasoning", "thought", "thinking", "chain_of_thought", "rationale"],
            );
            out.push(HfMemory {
                prompt,
                completion,
                thinking: if thinking.is_empty() {
                    None
                } else {
                    Some(thinking)
                },
                source: dataset_id.to_string(),
            });
        }
        out
    }

    /// 记忆 → crawl_queue.jsonl 行（ingestion.rs 消费格式，限 100 条/轮由消费方控制）
    pub fn to_crawl_queue_lines(mems: &[HfMemory], domain: &str) -> Vec<String> {
        mems.iter()
            .map(|m| {
                let title = if m.prompt.len() > 120 {
                    m.prompt.chars().take(120).collect::<String>()
                } else {
                    m.prompt.clone()
                };
                let mut content = if m.completion.is_empty() {
                    m.prompt.clone()
                } else {
                    format!("问：{}\n答：{}", m.prompt, m.completion)
                };
                if let Some(t) = &m.thinking {
                    let think: String = t.chars().take(500).collect();
                    content = format!("{content}\n思考：{think}");
                }
                serde_json::json!({
                    "url": format!("hf-dataset://{}", m.source),
                    "title": title,
                    "content": content,
                    "domain": domain,
                })
                .to_string()
            })
            .collect()
    }

    /// 直接灌注（小批量 pilot 用，convert-then-remember，无文件落地）
    pub fn ingest_memories(
        consciousness: &mut CrystalConsciousness,
        mems: &[HfMemory],
        domain: &str,
        confidence: f64,
    ) -> usize {
        let mut n = 0;
        for m in mems {
            let content = if m.completion.is_empty() {
                m.prompt.clone()
            } else {
                format!("问：{}\n答：{}", m.prompt, m.completion)
            };
            if content.trim().is_empty() {
                continue;
            }
            consciousness.remember(content, MemoryType::Fact, domain, confidence.clamp(0.0, 1.0));
            n += 1;
        }
        n
    }

    /// 觉醒冷启动候选（小体量推理/指令集，按 license 可用性排序使用前先查 API tags）
    pub fn curated_pilot() -> Vec<(&'static str, &'static str)> {
        vec![
            ("open-r1/OpenR1-Math-220k", "math reasoning, Apache-2.0"),
            ("open-thoughts/OpenThoughts-114k", "general reasoning traces"),
            ("GeneralReasoning/GeneralThought-323K", "general reasoning"),
            ("nvidia/OpenMathReasoning", "math reasoning"),
            ("PRM800K/process_rewards", "process supervision"),
            ("HuggingFaceH4/no_robots", "instruction, Apache-2.0"),
        ]
    }
}

/// HF 数据集广场目录条目（只用 `GET /api/datasets/{id}` 元数据，不取行数据）。
#[derive(Debug, Clone)]
pub struct HfCatalogEntry {
    /// `owner/name`
    pub id: String,
    pub likes: u64,
    pub downloads: u64,
    /// `task_categories:*` 标签去前缀后集合
    pub tasks: Vec<String>,
    /// `modality:*` 标签去前缀后集合
    pub modalities: Vec<String>,
    /// `format:*` 标签去前缀后集合
    pub formats: Vec<String>,
    /// `license:*` 首个（无则空串）
    pub license: String,
    /// 描述前 300 字符（已去首尾空白）
    pub blurb: String,
}

impl NtHfBridge {
    /// API 元数据 JSON → 目录条目。字段缺失即取默认；非 JSON / 无 id 返回 None。
    pub fn catalog_entry_from_api(api_json: &str) -> Option<HfCatalogEntry> {
        let v: serde_json::Value = serde_json::from_str(api_json).ok()?;
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        if id.is_empty() || !id.contains('/') {
            return None;
        }
        let likes = v.get("likes").and_then(|x| x.as_u64()).unwrap_or(0);
        let downloads = v.get("downloads").and_then(|x| x.as_u64()).unwrap_or(0);
        let mut tasks = Vec::new();
        let mut modalities = Vec::new();
        let mut formats = Vec::new();
        let mut license = String::new();
        if let Some(tags) = v.get("tags").and_then(|t| t.as_array()) {
            for t in tags {
                let s = match t.as_str() {
                    Some(s) => s,
                    None => continue,
                };
                if let Some(rest) = s.strip_prefix("task_categories:") {
                    tasks.push(rest.to_string());
                } else if let Some(rest) = s.strip_prefix("modality:") {
                    modalities.push(rest.to_string());
                } else if let Some(rest) = s.strip_prefix("format:") {
                    formats.push(rest.to_string());
                } else if license.is_empty() {
                    if let Some(rest) = s.strip_prefix("license:") {
                        license = rest.to_string();
                    }
                }
            }
        }
        let blurb: String = v
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .chars()
            .take(300)
            .collect();
        Some(HfCatalogEntry {
            id,
            likes,
            downloads,
            tasks,
            modalities,
            formats,
            license,
            blurb,
        })
    }

    /// 目录条目 → crawl_queue.jsonl 行（ingestion.rs 消费格式）。
    pub fn catalog_entry_to_crawl_line(entry: &HfCatalogEntry, domain: &str) -> String {
        let mut parts = Vec::new();
        if !entry.blurb.is_empty() {
            parts.push(format!("描述：{}", entry.blurb));
        }
        parts.push(format!(
            "热度：likes {} · downloads {}",
            entry.likes, entry.downloads
        ));
        if !entry.tasks.is_empty() {
            parts.push(format!("任务：{}", entry.tasks.join(",")));
        }
        if !entry.modalities.is_empty() {
            parts.push(format!("模态：{}", entry.modalities.join(",")));
        }
        if !entry.formats.is_empty() {
            parts.push(format!("格式：{}", entry.formats.join(",")));
        }
        if !entry.license.is_empty() {
            parts.push(format!("许可：{}", entry.license));
        }
        serde_json::json!({
            "url": format!("hf-dataset://{}", entry.id),
            "title": entry.id,
            "content": parts.join("\n"),
            "domain": domain,
        })
        .to_string()
    }
}

/// 按优先级取第一个非空字符串字段
fn first_str(v: &serde_json::Value, keys: &[&str]) -> String {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return s.to_string();
            }
        }
        // messages 数组（OpenAI 格式）→ 拼 user/assistant 文本
        if let Some(arr) = v.get(*k).and_then(|x| x.as_array()) {
            let mut parts = Vec::new();
            for m in arr {
                let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let text = m
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                if !text.trim().is_empty() {
                    parts.push(format!("[{role}] {text}"));
                }
            }
            if !parts.is_empty() {
                return parts.join("\n");
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{"rows":[
        {"row": {"instruction": "2+2=?", "output": "4", "reasoning": "1+1=2, double it"}},
        {"row": {"question": "empty?", "answer": ""}},
        {"row": {"foo": 1}},
        {"row": {"messages": [{"role": "user", "content": "hi"}, {"role": "assistant", "content": "hello"}]}}
    ]}"#;

    #[test]
    fn test_parse_dataset_url() {
        assert_eq!(
            NtHfBridge::parse_dataset_url("https://huggingface.co/datasets/open-r1/OpenR1-Math-220k"),
            Some(("open-r1".to_string(), "OpenR1-Math-220k".to_string()))
        );
        assert_eq!(
            NtHfBridge::parse_dataset_url("https://huggingface.co/datasets/owner-only/"),
            None
        );
        assert_eq!(NtHfBridge::parse_dataset_url("https://example.com/x"), None);
    }

    #[test]
    fn test_rows_endpoint_shape() {
        let u = NtHfBridge::rows_endpoint("o", "n", 0, 100);
        assert!(u.contains("datasets-server.huggingface.co/rows"));
        assert!(u.contains("dataset=o/n"));
    }

    #[test]
    fn test_rows_to_memories_mapping() {
        let mems = NtHfBridge::rows_to_memories(FIXTURE, "ds/test");
        // 行1（instruction+output+reasoning）+ 行2（question，无answer但有prompt）+ 行4（messages）
        assert_eq!(mems.len(), 3);
        assert_eq!(mems[0].prompt, "2+2=?");
        assert_eq!(mems[0].completion, "4");
        assert!(mems[0].thinking.is_some());
        assert!(mems[1].prompt.contains("empty?"));
        assert!(mems[2].prompt.contains("[user] hi"));
    }

    #[test]
    fn test_malformed_json_yields_empty() {
        assert!(NtHfBridge::rows_to_memories("not json", "x").is_empty());
        assert!(NtHfBridge::rows_to_memories("{\"rows\":[]}", "x").is_empty());
    }

    #[test]
    fn test_crawl_queue_lines_match_consumer() {
        let mems = NtHfBridge::rows_to_memories(FIXTURE, "ds/test");
        let lines = NtHfBridge::to_crawl_queue_lines(&mems, "hf-reasoning");
        assert_eq!(lines.len(), 3);
        for line in &lines {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(v.get("url").is_some());
            assert!(v.get("title").is_some());
            assert!(v.get("content").and_then(|c| c.as_str()).map_or(false, |s| !s.is_empty()));
            assert_eq!(v.get("domain").and_then(|d| d.as_str()), Some("hf-reasoning"));
        }
    }

    #[test]
    fn test_ingest_memories_counts() {
        let mut c = CrystalConsciousness::new("t");
        let mems = NtHfBridge::rows_to_memories(FIXTURE, "ds/test");
        let n = NtHfBridge::ingest_memories(&mut c, &mems, "hf-reasoning", 0.6);
        assert_eq!(n, 3);
        assert_eq!(c.memories.len(), 3);
    }

    const CATALOG_FIXTURE: &str = r#"{"id":"o/n","likes":12,"downloads":34,
        "tags":["task_categories:text-generation","modality:text","format:parquet","license:mit"],
        "description":"  hello world  "}"#;

    #[test]
    fn test_catalog_entry_from_api() {
        let e = NtHfBridge::catalog_entry_from_api(CATALOG_FIXTURE).unwrap();
        assert_eq!(e.id, "o/n");
        assert_eq!(e.likes, 12);
        assert_eq!(e.downloads, 34);
        assert_eq!(e.tasks, vec!["text-generation".to_string()]);
        assert_eq!(e.modalities, vec!["text".to_string()]);
        assert_eq!(e.formats, vec!["parquet".to_string()]);
        assert_eq!(e.license, "mit");
        assert_eq!(e.blurb, "hello world");
    }

    #[test]
    fn test_catalog_entry_rejects_bad_input() {
        assert!(NtHfBridge::catalog_entry_from_api("not json").is_none());
        assert!(NtHfBridge::catalog_entry_from_api("{\"likes\":1}").is_none());
        assert!(NtHfBridge::catalog_entry_from_api("{\"id\":\"nonslash\"}").is_none());
        // 缺字段即默认，不失败
        let e = NtHfBridge::catalog_entry_from_api("{\"id\":\"o/n\"}").unwrap();
        assert_eq!(e.likes, 0);
        assert!(e.tasks.is_empty());
        assert!(e.blurb.is_empty());
    }

    #[test]
    fn test_catalog_entry_to_crawl_line_match_consumer() {
        let e = NtHfBridge::catalog_entry_from_api(CATALOG_FIXTURE).unwrap();
        let line = NtHfBridge::catalog_entry_to_crawl_line(&e, "hf-catalog");
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            v.get("url").and_then(|u| u.as_str()),
            Some("hf-dataset://o/n")
        );
        assert_eq!(v.get("title").and_then(|t| t.as_str()), Some("o/n"));
        let content = v
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        assert!(content.contains("hello world"));
        assert!(content.contains("12"));
        assert_eq!(v.get("domain").and_then(|d| d.as_str()), Some("hf-catalog"));
    }
}
