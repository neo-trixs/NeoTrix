//! `agent_eval` bench — 极简评测台冒烟基准。
//! 契约定义在 `l0_substrate::nt_agent_eval`（含完整单测），此处只验证接线可跑。

use criterion::{criterion_group, criterion_main, Criterion};
use neotrix::l0_substrate::nt_agent_eval::{contains_gold, evaluate, EvalCase};

fn bench_agent_eval_smoke(c: &mut Criterion) {
    let cases = [
        EvalCase { prompt: "1+1=?", gold: "2", scorer: contains_gold },
        EvalCase { prompt: "capital of France?", gold: "Paris", scorer: contains_gold },
    ];
    let outputs = ["2", "Paris"];
    c.bench_function("agent_eval_smoke", |b| {
        b.iter(|| {
            let (mean, rate) = evaluate(&cases, &outputs, 0.5);
            criterion::black_box((mean, rate));
        });
    });
}

criterion_group!(benches, bench_agent_eval_smoke);
criterion_main!(benches);
