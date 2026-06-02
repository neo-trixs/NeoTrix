fn main() {
    use neotrix::neotrix::nt_mind::core::{
        KnowledgeSource, FIELD_NAMES,
    };
    use neotrix::neotrix::nt_mind::self_iterating::ReasoningBrain;
    use neotrix::neotrix::nt_mind::attention_router::AttentionRouter;
    use neotrix::core::memory::ReasoningBank;

    const N: usize = 200;

    let sources = KnowledgeSource::all();
    let n_src = sources.len();
    let field_names: &[&str] = &FIELD_NAMES;

    let mut brain = ReasoningBrain::new();
    let mut _bank = ReasoningBank::new(10000);
    let mut router = AttentionRouter::new();
    router.seed_knowledge();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║      外部数据探索循环 — 200 次知识吸收 + 推理意识内核              ║");
    println!("║      来源池: {} 个 KnowledgeSource  | 超立方体种子: {} 条推理知识     ║",
        n_src, router.bridge.hypercube.cell_count());
    println!("║      能力维度: {} 维 | 学习率: {:.2}                               ║",
        field_names.len(), brain.learning_rate);
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    let mut total_dim_delta = 0.0f64;

    let route_contexts = [
        "find patterns in system logs to detect anomalies",
        "design a novel UI framework composable architecture",
        "plan next week's security review milestone",
        "reflect on past failures and suggest improvements",
        "brainstorm creative solutions for data visualization",
    ];

    for i in 0..N {
        let idx = i % n_src;
        let source = sources[idx];

        let cap_before = brain.capability.to_vector();
        brain.absorb(source);
        let cap_after = brain.capability.to_vector();

        let deltas: Vec<(usize, f64)> = cap_before.iter().enumerate()
            .map(|(j, &before)| (j, cap_after[j] - before))
            .filter(|(_, d)| d.abs() > 0.0001)
            .collect();
        let abs_sum: f64 = deltas.iter().map(|(_, d)| d.abs()).sum();
        total_dim_delta += abs_sum;
        let max5: Vec<(&str, f64)> = deltas.iter()
            .take(5).map(|(j, d)| (field_names[*j], *d)).collect();

        print!("[{:>3}/{}] {:<35} Δ={:.4}  ",
            i + 1, N, source.name(), abs_sum);
        for (name, d) in &max5 {
            print!("{}{:+.3} ", name, d);
        }
        println!();

        // Every 33 iterations, demonstrate AttentionRouter reasoning
        if i > 0 && i % 33 == 0 {
            let ctx_idx = (i / 33 - 1) % route_contexts.len();
            let ctx = route_contexts[ctx_idx];
            let routed = router.route(ctx);
            println!("       ── AttentionRouter ──");
            println!("       context: \"{}\"", ctx);
            println!("       winner: {}  |  activated: {}",
                routed.winning_topic,
                routed.active_specialists.iter()
                    .map(|s| s.short_name()).collect::<Vec<_>>().join(","));
            if !routed.knowledge_lines.is_empty() {
                let snippet: Vec<&str> = routed.knowledge_lines.iter()
                    .map(|s| s.as_str()).collect();
                println!("       knowledge[{}]: {}",
                    snippet.len(), snippet.join(" | "));
            }
            let prompt = router.build_knowledge_prompt_suffix(&routed);
            if !prompt.is_empty() {
                let line = prompt.lines().nth(1).unwrap_or("");
                let truncated = if line.len() > 80 { &line[..80] } else { line };
                println!("       prompt: {}", truncated);
            }
            println!();
        }
    }

    let cap_final = brain.capability.to_vector();
    println!();
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                          最终能力向量                               ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    for (j, name) in field_names.iter().enumerate() {
        let val = cap_final[j];
        let bar_len = (val * 25.0) as usize;
        let bar: String = std::iter::repeat('█').take(bar_len.min(25))
            .chain(std::iter::repeat('░').take((25usize).saturating_sub(bar_len)))
            .collect();
        println!("║  {:<28} {:>7.4}  {}  ║", name, val, bar);
    }
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║  吸收次数: {:>4}    历史记录: {:>4} 条     总维度漂移: {:.4}      ║",
        brain.total_absorb_count, brain.absorption_history.len(), total_dim_delta);
    println!("║  超立方体知识: {} 条  学习率: {:.2}                                 ║",
        router.bridge.hypercube.cell_count(), brain.learning_rate);
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}
