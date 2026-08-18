// C3 benchmark 基线 — NT-ACT (行动执行者) 能力节点 (C2→C3 晋升证据)
//
// 覆盖:
// - nt_agent_mcp_gateway::fold_tool_specs — MCP 工具规格 N→4 折叠 (PTC 链路)
// - nt_act_action_cache::ActionCache      — remember/lookup 动作缓存吞吐
// - nt_act_disk_guard::DiskGuard          — check_many 批量磁盘边界裁决
//
// 目的: 建立 NT-ACT 性能基线 (C3 = benchmark 基线建立 + 无回归),
// 对比历史 (cargo bench --bench act_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::neotrix::l1_body_impl::nt_act_action_cache::ActionCache;
use neotrix::neotrix::l1_body_impl::nt_act_disk_guard::DiskGuard;
use neotrix::neotrix::l1_body_impl::nt_agent_mcp_gateway::{fold_tool_specs, ToolSpec};

/// 构造 n 个 MCP 工具规格 (混合 Search/Act/Knowledge/File 分类)。
fn make_tool_specs(n: usize) -> Vec<ToolSpec> {
    let domains = [
        ("search_web", "Search"), ("read_file", "File"), ("edit_file", "File"),
        ("store_note", "Knowledge"), ("run_shell", "Act"), ("fetch_url", "Search"),
    ];
    (0..n)
        .map(|i| {
            let (name, _) = domains[i % domains.len()];
            ToolSpec {
                name: format!("{name}_{i}"),
                description: format!("tool {i} for {name} operations with structured schema"),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" },
                        "path": { "type": "string" },
                        "limit": { "type": "integer", "default": 10 },
                    },
                    "required": ["query"],
                }),
            }
        })
        .collect()
}

fn bench_fold_tools(c: &mut Criterion) {
    let mut group = c.benchmark_group("act_fold_tools");
    for &n in &[8usize, 32, 128] {
        let specs = make_tool_specs(n);
        group.bench_function(format!("fold_{n}tools"), |b| {
            b.iter_batched(
                || specs.clone(),
                |s| {
                    let folded = fold_tool_specs(s);
                    black_box(folded.savings_percent);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_action_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("act_action_cache");
    for &n in &[64usize, 256, 1024] {
        group.bench_function(format!("remember_lookup_{n}"), |b| {
            b.iter_batched(
                || ActionCache::new(),
                |mut cache| {
                    for i in 0..n {
                        cache.remember(&format!("sig-{i}"), &format!("action-{i}"), vec![format!("sel-{i}")]);
                    }
                    let mut hits = 0usize;
                    for i in 0..n {
                        hits += cache.lookup(&format!("sig-{i}")).map(|_| 1).unwrap_or(0);
                    }
                    black_box(hits);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_disk_guard(c: &mut Criterion) {
    let mut group = c.benchmark_group("act_disk_guard");
    for &n in &[64usize, 256, 1024] {
        let root = std::env::temp_dir().join("nt_act_guard_bench");
        let mut guard = DiskGuard::new();
        guard.allow(&root);
        let targets: Vec<std::path::PathBuf> = (0..n)
            .map(|i| {
                if i % 10 == 0 {
                    std::path::PathBuf::from("/etc/passwd")
                } else {
                    root.join(format!("file-{i}.txt"))
                }
            })
            .collect();
        group.bench_function(format!("check_many_{n}"), |b| {
            b.iter_batched(
                || guard.clone(),
                |mut g| {
                    let blocked = g.check_many("write", &targets);
                    black_box(blocked.len());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    act_c3,
    bench_fold_tools,
    bench_action_cache,
    bench_disk_guard,
);
criterion_main!(act_c3);
