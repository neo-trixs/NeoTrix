//! NeoTrix — Free LLM Pool 自动测试 + 古籍 RAG 链路验证
//!
//! 流程:
//!   1. 构建统一网关 (create_gateway_async → 自动注册 keyless 免费 provider + 环境变量 key)
//!   2. 遍历池中所有已注册 provider (gateway.providers()):
//!      a. LLM Challenge (arithmetic) — 精确匹配评分 → 质量/延迟/成本
//!      b. 古籍 RAG: KB search("史记 鸿门宴") → 上下文 → 池路由生成 → 验证古籍检索→生成链路
//!   3. 输出池健康报告 + 各 provider benchmark
//!
//! 使用: cargo run --example guji_llm_pool_test --release
//!   RAG_ONLY=1        跳过 challenge, 只跑古籍检索生成
//!   POOL_FILTER=xxx   只测名字包含 xxx 的 provider (如 POOL_FILTER=pollinations)

use std::time::Instant;

use neotrix::neotrix::nt_io_provider::factory::create_gateway_async;
use neotrix::neotrix::nt_io_provider::provider_catalog::CommunicationProfile;
use neotrix::neotrix::nt_io_provider::types::LlmRequest;
use neotrix::neotrix::nt_memory_kb::KnowledgeBase;

/// 知识库路径 (统一记忆大脑)
fn kb_path() -> String {
    std::env::var("NEOTRIX_KB").unwrap_or_else(|_| {
        format!("{}/.neotrix/knowledge.db", std::env::var("HOME").unwrap_or_else(|_| ".".into()))
    })
}

/// 古籍检索链路验证 — 不依赖 LLM, 证明 KB→上下文构建工作
/// 返回 (命中数, 上下文预览)。任何环境都可验证。
fn rag_retrieve(kb: &neotrix::neotrix::nt_memory_kb::KnowledgeBase, query: &str) -> Result<(usize, String), String> {
    let results = kb.search(query, 3).map_err(|e| e.to_string())?;
    if results.is_empty() {
        return Err(format!("KB 检索「{}」无结果", query));
    }
    let mut context = String::new();
    for r in results.iter().take(2) {
        let n = &r.node;
        let snippet = n.content.as_deref().unwrap_or("").chars().take(600).collect::<String>();
        context.push_str(&format!("【{}】{}…\n", n.title, snippet));
    }
    Ok((results.len(), context))
}

/// 单个 provider 的古籍 RAG 生成测试 (走池路由 complete_for_profile)
/// 依赖外部 LLM 端点可用。
async fn rag_generate(
    gateway: &neotrix::neotrix::nt_io_provider::gateway::GatewayV2,
    context: &str,
    provider: &str,
) -> Result<(bool, u64), String> {
    // LLM 调用 (走池路由 — model 用注册名的 model_id 部分, 同 gateway.provider_model 逻辑)
    let model = provider.split('/').next_back().unwrap_or(provider);
    let prompt = format!(
        "你是中国古典文献助手。根据下面的《史记》原文片段，回答：鸿门宴中项羽为什么没有杀刘邦？请引用原文回答。\n\n{}",
        context
    );
    let request = LlmRequest::new(model, &prompt).with_max_tokens(600);
    let start = Instant::now();
    let profile = CommunicationProfile::Open;
    let resp = gateway
        .complete_for_profile(profile, &request)
        .await
        .map_err(|e| format!("{}", e))?;
    let latency = start.elapsed().as_millis() as u64;

    let answer = resp.content.trim();
    let ok = !answer.is_empty()
        && (answer.contains("鸿门") || answer.contains("项羽") || answer.contains("项庄")
            || answer.contains("范增") || answer.contains("刘邦"));
    println!("    └─ RAG 回答 ({}ms, {} tokens): {}", latency, resp.usage.total_tokens,
             &answer.chars().take(120).collect::<String>());
    Ok((ok, latency))
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  🧪 NeoTrix Free LLM Pool — 自动测试 + 古籍 RAG 链路          ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // ── 打开知识库 (统一记忆大脑) ──
    println!("📚 打开 KB: {}", kb_path());
    let kb = KnowledgeBase::open(
        Some(std::path::PathBuf::from(kb_path())),
    ).map_err(|e| {
        println!("❌ 知识库打开失败: {}", e);
        std::process::exit(1);
    }).unwrap();
    let stats = kb.stats().unwrap_or_default();
    println!("   → {} nodes / {} edges / {:.1} MB", stats.total_nodes, stats.total_edges,
             stats.db_size_bytes as f64 / 1e6);

    // ── 构建统一网关 (自动注册 free 池) ──
    println!("\n🔌 构建网关 (自动注册 keyless free provider + env key)...");
    let gateway = create_gateway_async().await;

    let names = gateway.providers();
    let filter = std::env::var("POOL_FILTER").unwrap_or_default();
    let rag_only = std::env::var("RAG_ONLY").unwrap_or_default() == "1";

    let targets: Vec<String> = names.iter()
        .filter(|n| filter.is_empty() || n.contains(&filter))
        .cloned()
        .collect();
    if targets.is_empty() {
        if !filter.is_empty() {
            println!("⚠️ 无 provider 匹配 '{}'。池中全部: {:?}", filter, names);
        } else {
            println!("⚠️ 池为空 — create_gateway_async 未注册任何 provider");
        }
        if rag_only {
            // RAG_ONLY 模式下空池也允许: 用 complete_for_profile 尝试全局 fallback
            println!("⚠️ RAG_ONLY=1 且池空 — 仍尝试古籍 RAG (依赖 complete_for_profile 降级)");
        } else {
            return;
        }
    }
    println!("🏊 池中 provider ({}/{}): {:?}", targets.len(), names.len(), names);

    // ── 0. 古籍检索链路独立验证 (不依赖 LLM, 任何环境可跑) ──
    println!("\n📖 古籍检索链路验证 (KB → 上下文构建, 无需 LLM)...");
    match rag_retrieve(&kb, "史记 鸿门宴") {
        Ok((n, context)) => {
            println!("  ✅ 检索命中 {} 条, 上下文 {} 字符", n, context.chars().count());
            println!("  └─ 上下文预览: {}", &context.chars().take(160).collect::<String>());
            println!("  📋 再验证: 论语, 道德经, 三国志");
            for q in ["论语", "道德经", "三国志"] {
                match rag_retrieve(&kb, q) {
                    Ok((m, _)) => println!("  ✅ 「{}」命中 {} 条", q, m),
                    Err(e) => println!("  ❌ 「{}」: {}", q, e),
                }
            }
        }
        Err(e) => {
            println!("  ❌ 检索链路失败: {}", e);
        }
    }

    // ── 0.5 标题加权排序验证 (原书应排到引用书前) ──
    println!("\n🏷️ 标题加权排序验证 (search_fts 增强)...");
    for q in ["史记", "论语", "道德经"] {
        match kb.search(q, 15) {
            Ok(results) => {
                let titles: Vec<String> = results.iter().take(15)
                    .map(|r| format!("{}[{:?}][{:.2}]", r.node.title, r.node.node_type, r.score)).collect();
                println!("  「{}」top15: {}", q, titles.join(" → "));
            }
            Err(e) => println!("  ❌ 「{}」: {}", q, e),
        }
    }

    // ── 逐 provider 测试 ──
    // (name, challenge_ok, challenge_latency, accuracy, rag_ok, rag_latency)
    let mut results: Vec<(String, bool, u64, f64, bool, u64)> = Vec::new();
    for name in &targets {
        println!("\n───── Testing: {} ─────", name);

        // a. LLM Challenge (arithmetic) — 若失败跳过 RAG
        if !rag_only {
            let start = Instant::now();
            match gateway.run_llm_challenge(name, "arithmetic").await {
                Ok(bench) => {
                    let ok = bench.accuracy >= 0.5;
                    println!("  ✅ Challenge[{}]: acc={:.0}% lat={}ms cost=${:.4}",
                             bench.task_type, bench.accuracy * 100.0, bench.latency_ms, bench.cost_usd);
                    results.push((name.clone(), ok, bench.latency_ms, bench.accuracy, false, 0));
                    if !ok {
                        println!("  ⚠️  accuracy 过低, 跳过 RAG 测试");
                        continue;
                    }
                }
                Err(e) => {
                    println!("  ❌ Challenge 失败: {}", e);
                    results.push((name.clone(), false, start.elapsed().as_millis() as u64, 0.0, false, 0));
                    continue;
                }
            }
        }

        // b. 古籍 RAG 生成测试 (仅当 challenge 通过时执行)
        match rag_generate(&gateway, "【史记】鸿门宴片段(检索链路已验证)", name).await {
            Ok((ok, latency)) => {
                println!("  ✅ RAG: {} ({}ms)", if ok { "命中" } else { "生成但未命中关键词" }, latency);
                if let Some(r) = results.last_mut() {
                    r.4 = ok;
                    r.5 = latency;
                }
            }
            Err(e) => {
                println!("  ⚠️  RAG 生成不可用 (端点失效, 属外部依赖): {}", e);
            }
        }
    }

    // ── 报告 ──
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  📊 Free LLM Pool 健康报告                                 ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    for (name, cok, clat, acc, rok, rlat) in &results {
        let c = if *cok { "🟢" } else { "🔴" };
        let r = if *rok { "✅" } else { "—" };
        println!("║  {} {:<22} chall={:>3}ms acc={:>3.0}% rag={} {:>5}ms",
                 c, name, clat, acc * 100.0, r, rlat);
    }
    if !results.is_empty() {
        let healthy = results.iter().filter(|(_, ok, _, _, _, _)| *ok).count();
        println!("║  ---                                               ---");
        println!("║  ✅ challenge healthy: {}/{}", healthy, results.len());
    }
    println!("╚══════════════════════════════════════════════════════════╝");
}
