//! 159562 黄金股ETF凡华 — 真实信息推演 + KB 落盘验证
//!
//! 注入 2026-08 真实宏观因子事件流（金价、Fed、地缘、CPI），
//! 推演 159562 未来 1-2 周演化路线，落盘到 KB `forecast` namespace，
//! 并检索验证。驱动脚本仅验证用途 — 生产调用走内部任务路由，无 CLI。
use neotrix::core::nt_core_forecast::ForecastEngine;

fn main() {
    // 临时诊断：验证 LLM 池子调用是否真实成功（跑完移除）
    {
        use neotrix::core::nt_core_forecast::LlmNarrator;
        let n = LlmNarrator::new();
        // 1) 池子有哪些注册名？
        {
            let handle = n.raw_gateway_handle_for_diag();
            let names = handle.providers();
            println!("[DIAG] providers: {:?}", names);
        }
        // 2) 直调 pollinations provider 看真实错误
        {
            use neotrix::neotrix::l1_body_impl::nt_io_provider::free_providers::PollinationsProvider;
            use neotrix::neotrix::l1_body_impl::nt_io_provider::types::{LlmRequest, Message, Role};
            let p = PollinationsProvider::new();
            let req = LlmRequest::new("openai", "Reply with exactly: E2E-OK");
            match std::thread::scope(|s| {
                s.spawn(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(p.stream_complete(&req))
                })
                .join()
            }) {
                Ok(Ok(mut rx)) => {
                    let mut buf = String::new();
                    while let Ok(Some(Ok(r))) = tokio::runtime::Runtime::new().unwrap().block_on(rx.recv()) {
                        buf.push_str(&r.content);
                    }
                    println!("[DIAG] PollinationsProvider direct: OK -> {buf:?}");
                }
                Ok(Err(e)) => println!("[DIAG] PollinationsProvider direct: ERR -> {e}"),
                Err(_) => println!("[DIAG] PollinationsProvider direct: join panic"),
            }
        }
        match n.narrate_scenarios("probe: is LLM path alive?") {
            Some(s) => println!("[DIAG] LLM narrate OK: {}...", s.chars().take(100).collect::<String>()),
            None => println!("[DIAG] LLM narrate returned None (degraded)"),
        }
    }
    // 真实宏观因子事件流（2026-08 上旬，黄金板块背景）
    let events: Vec<(&str, &str, &str, f64)> = vec![
        // 1. 金价突破 4100（8/4-8/5 大涨，ETF 净值 +7.08% 至 2.1743）
        ("gold_spot", "broke_above", "4100_usd", 0.90),
        // 2. Fed 维持利率 3.50-3.75% + 鸽派引导（9 月降息预期）
        ("fed", "held_rate", "3.75pct_dovish", 0.55),
        // 3. 央行购金持续（官方储备连续 3 月净增）
        ("central_banks", "continued_purchasing", "gold_reserves", 0.45),
        // 4. 地缘紧张（美伊局势升温 → 避险需求）
        ("geopolitics", "escalated", "middle_east_tension", 0.60),
        // 5. 8/12 CPI 即将公布（核心通胀黏性 → 降息节奏不确定）
        ("cpi", "scheduled", "aug12_release", 0.35),
    ];

    let mut engine = ForecastEngine::new().with_llm_narrator(None);
    engine.abstain_threshold = 0.10;
    for (a, ac, o, i) in events {
        engine.ingest_event(a, ac, o, i);
    }
    engine.tick(1);

    // 推演 159562 演化（base_state: 0x2E 当前强势格局）
    let forecast = engine.generate_forecast("159562_gold_etf_equity", 0x2E);

    println!("=== 159562 黄金股ETF凡华 推演报告 ===");
    println!("目标     : {}", forecast.target);
    println!("弃权判定 : {}", forecast.abstain);
    println!("置信理由 : {}", forecast.confidence_reason);
    println!("情景树   :");
    for (i, n) in forecast.tree.nodes.iter().enumerate() {
        println!("  [{i}] {} prob={:.3} conf={:.3}",
            n.state, n.probability, n.confidence);
        if !n.leading_indicators.is_empty() {
            println!("      先行指标: {}", n.leading_indicators.join(", "));
        }
        if !n.invalidation.is_empty() {
            println!("      失效条件: {}", n.invalidation.join(", "));
        }
        if let Some(narr) = &n.narrative {
            println!("      叙事: {narr}");
        }
    }
    println!("叶子概率和: {:.4}", forecast.tree.leaf_probability_sum());

    // KB 落盘
    match engine.persist_to_kb(&forecast) {
        Ok(()) => println!("\n[KB] 落盘成功 -> ~/.neotrix/knowledge.db namespace=forecast"),
        Err(e) => println!("\n[KB] 落盘失败: {e}"),
    }

    // 检索验证
    let recent = ForecastEngine::recent_forecasts(Some("159562"), 3);
    println!("\n[KB] 检索到 {} 条历史推演:", recent.len());
    for r in &recent {
        println!(
            "  - {} | time={} | leaf_sum={:.3} | events={}",
            r.target,
            r.created_at,
            r.forecast.tree.leaf_probability_sum(),
            r.event_stream.len()
        );
    }
}
