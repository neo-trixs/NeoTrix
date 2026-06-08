use neotrix::neotrix::nt_memory_kb::KnowledgeBase;
use std::time::Instant;

fn find_wiki_category(client: &reqwest::blocking::Client, topic: &str) -> Option<String> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch=Category:{}&srlimit=1&format=json",
        urlencode(topic)
    );
    let resp = client.get(&url).send().ok()?;
    let data: serde_json::Value = resp.json().ok()?;
    let title = data["query"]["search"][0]["title"].as_str()?.to_string();
    Some(format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_")))
}

fn find_wiki_page(client: &reqwest::blocking::Client, topic: &str) -> Option<String> {
    let url = format!(
        "https://en.wikipedia.org/w/api.php?action=opensearch&search={}&limit=1&namespace=0&format=json",
        urlencode(topic)
    );
    let resp = client.get(&url).send().ok()?;
    let data: serde_json::Value = resp.json().ok()?;
    let results = data.as_array()?;
    if results.len() < 3 { return None; }
    let urls = results[3].as_array()?;
    urls.first()?.as_str().map(|s| s.to_string())
}

fn urlencode(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "%20".to_string(),
        c => format!("%{:02X}", c as u8),
    }).collect()
}

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  NeoTrix 知识种子 v3 — 直接注入+搜索驱动              ║");
    println!("╚════════════════════════════════════════════════════════╝");

    let kb = KnowledgeBase::open(None).expect("Failed to open knowledge base");
    let client = reqwest::blocking::Client::builder()
        .user_agent("NeoTrix/0.18 (KnowledgeSeed)")
        .no_proxy()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("HTTP client");

    // Direction 1: GitHub repos — direct ingest (no crawl queue)
    println!("\n━━━ Direction 1: GitHub 仓库 (直接注入) ━━━");
    for (owner, repo) in &[("JY0284","zizhitongjian"), ("jbiaojerry","ebook-treasure-chest"),
        ("soulteary","books"), ("ruanyf","share"), ("diphosphane","ebooks")] {
        let start = Instant::now();
        match kb.ingest_github(owner, repo) {
            Ok(n) => println!("  ✅ {}/{} — {} [{:.1}s]", owner, repo, n, start.elapsed().as_secs_f64()),
            Err(e) => println!("  ⚠ {}/{} — {}", owner, repo, e),
        }
    }
    // GitHub search — batch ingest
    for q in &["topic:book+stars:>100", "topic:ebook+stars:>50", "topic:open+library+books",
               "topic:chinese-literature", "topic:textbooks"] {
        let start = Instant::now();
        match kb.ingest_github_search(q) {
            Ok(n) => println!("  🔍 GitHub search '{}' — {} 仓库 [{:.1}s]", q, n, start.elapsed().as_secs_f64()),
            Err(e) => println!("  ⚠ GitHub search '{}' — {}", q, e),
        }
    }

    // Direction 2: Wikipedia — direct ingest
    println!("\n━━━ Direction 2: Wikipedia 主题注入 ━━━");
    for topic in &["Zizhi Tongjian","History of China","Chinese literature","Book","Library",
                    "Digital library","Ebook","Open access","Project Gutenberg",
                    "Chinese philosophy","Four Great Classical Novels","Chinese historiography"] {
        let start = Instant::now();
        match kb.ingest_wikipedia(topic) {
            Ok(n) => println!("  ✅ {} — {} [{:.1}s]", topic, n, start.elapsed().as_secs_f64()),
            Err(e) => println!("  ⚠ {} — {}", topic, e),
        }
    }

    // Direction 3: OpenLibrary — direct ingest (no crawl queue!)
    println!("\n━━━ Direction 3: OpenLibrary 书籍注入 (直接 API) ━━━");
    for q in &["fiction","non_fiction","fantasy","history","science","ebook",
               "chinese literature","classical literature","programming","computer science"] {
        let start = Instant::now();
        match kb.ingest_openlibrary_search(q) {
            Ok(n) => println!("  🔍 OpenLibrary '{}' — {} 本书 [{:.1}s]", q, n, start.elapsed().as_secs_f64()),
            Err(e) => println!("  ⚠ OpenLibrary '{}' — {}", q, e),
        }
    }

    // Only queue actual web pages for crawling (Wikipedia category pages)
    println!("\n━━━ 爬取队列 (仅维基分类页) ━━━");
    let mut seeds: Vec<(String, i64, String)> = Vec::new();
    for cat in &["Chinese books","Chinese classic texts","Digital libraries","History of China",
                 "Chinese philosophers","Chinese novels"] {
        if let Some(url) = find_wiki_category(&client, cat) {
            seeds.push((url, 0, "en.wikipedia.org".into()));
            println!("  🔍 Wikipedia category: {} — 入队", cat);
        } else if let Some(url) = find_wiki_page(&client, cat) {
            seeds.push((url, 0, "en.wikipedia.org".into()));
            println!("  🔍 Wikipedia page (fallback): {} — 入队", cat);
        } else {
            println!("  ⚠ Wikipedia: {} — 未找到", cat);
        }
    }
    let refs: Vec<(&str, i64, &str)> = seeds.iter().map(|(u, p, d)| (u.as_str(), *p, d.as_str())).collect();
    let enqueued = kb.enqueue_seed_urls(&refs).unwrap_or(0);
    println!("  ✅ 入队 {} 个维基分类页", enqueued);

    // Dedup
    println!("\n━━━ 去重 ━━━");
    match kb.dedup_nodes() {
        Ok(n) => println!("  ✅ 去重 {} 节点", n),
        Err(e) => println!("  ⚠ 去重错误: {}", e),
    }

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║  注入完成                                          ║");
    println!("╚════════════════════════════════════════════════════════╝");
    if let Ok(stats) = kb.stats() {
        println!("  知识库: {} 节点, {} 边, {} 域", stats.total_nodes, stats.total_edges, stats.by_domain.len());
    }
}
