// C3 benchmark 基线 — affective interface 情感交互层 (C2→C3 晋升证据)
//
// 覆盖:
// - affective_interface::lexicon_detect        — 词典情绪检测 (中/英关键词)
// - affective_interface::estimate_disclosure    — 自我披露深度启发式 (CJK 感知)
// - affective_interface::AffectiveInterface::process_user_input — 全管线 (检测+关系+共情意图+韵律镜像/引导)
// - emotion_state::EmotionEngine::observe_appraisal — OCC 事件评估 (novelty/gc/coping → PAD)
//
// 目的: 建立 NT-CORE affective 性能基线 (C3 = benchmark 基线 + 无回归),
// 对比历史 (cargo bench --bench affective_c3)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use neotrix::core::nt_core_self::affective_interface::{
    estimate_disclosure, lexicon_detect, AffectiveInterface, GuideMode, RhythmProfile,
};
use neotrix::core::nt_core_self::emotion_state::{EmotionConfig, EmotionEngine};

fn bench_lexicon(c: &mut Criterion) {
    let mut group = c.benchmark_group("affective_lexicon");
    for text in [
        "普通的一句话没有明显情绪",
        "我很难过，真的很难受，心里特别沮丧",
        "太开心了 好棒 真高兴",
        "i am so sad and depressed today",
    ] {
        group.bench_function(format!("detect_{}", text.chars().count()), |b| {
            b.iter(|| {
                let (e, s) = lexicon_detect(black_box(text));
                black_box((e, s));
            });
        });
    }
    group.finish();
}

fn bench_disclosure(c: &mut Criterion) {
    let mut group = c.benchmark_group("affective_disclosure");
    let cjk_long = "我昨天遇到一件很难过的事情，心里特别难受，很想找人说一说";
    let en_long = "I feel really sad about what happened yesterday, and I want to talk about it with someone I trust";
    for (name, text) in [("cjk", cjk_long), ("en", en_long)] {
        group.bench_function(name, |b| {
            b.iter(|| {
                let (p, d) = estimate_disclosure(black_box(text));
                black_box((p, d));
            });
        });
    }
    group.finish();
}

fn bench_process_user_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("affective_pipeline");
    for guide in [
    GuideMode::Auto,
    GuideMode::Mirror,
    GuideMode::Toward(RhythmProfile::calm()),
] {
        group.bench_function(format!("process_{guide:?}"), |b| {
            b.iter_batched(
                || AffectiveInterface::new(),
                |mut iface| {
                    let ro = iface.process_user_input(
                        "我很难过，真的很难受",
                        None,
                        guide,
                    );
                    black_box(ro.expression);
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_observe_appraisal(c: &mut Criterion) {
    let mut group = c.benchmark_group("affective_appraisal");
    group.bench_function("observe_appraisal", |b| {
        b.iter_batched(
            || EmotionEngine::new(EmotionConfig::default()),
            |mut engine| {
                for i in 0..20u32 {
                    let t = i as f64 / 20.0;
                    engine.observe_appraisal(t, 1.0 - t, t, "bench");
                }
                black_box(engine.report().valence);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_lexicon,
    bench_disclosure,
    bench_process_user_input,
    bench_observe_appraisal
);
criterion_main!(benches);