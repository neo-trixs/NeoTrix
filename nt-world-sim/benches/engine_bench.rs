use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nt_world_sim::core::UniversalWorld;
use nt_world_sim::core::scheduler::{ParallelScheduler, UniversalSystem, SystemDependency};
use nt_world_sim::codegen::{GameDefinition, CodeGenerator, EntityDef, SystemDef};

#[derive(Clone)]
struct BenchPos { x: f32, y: f32 }
impl nt_world_sim::core::Component for BenchPos {}

#[derive(Clone)]
struct BenchVel { x: f32, y: f32 }
impl nt_world_sim::core::Component for BenchVel {}

struct BenchSys;
impl UniversalSystem for BenchSys {
    fn name(&self) -> &str { "bench" }
    fn update(&mut self, world: &mut UniversalWorld, dt: f32) {
        let entities: Vec<_> = world.entities();
        for e in entities {
            let vel = world.get_component::<BenchVel>(e).cloned();
            if let (Some(vel), Some(pos)) = (vel, world.get_component_mut::<BenchPos>(e)) {
                pos.x += vel.x * dt;
                pos.y += vel.y * dt;
            }
        }
    }
}

fn bench_spawn(c: &mut Criterion) {
    c.bench_function("spawn_1000", |b| {
        b.iter(|| {
            let mut world = UniversalWorld::new();
            for _ in 0..1000 {
                let e = world.spawn();
                world.insert_component(e, BenchPos { x: 0.0, y: 0.0 });
                world.insert_component(e, BenchVel { x: 1.0, y: 1.0 });
            }
            black_box(world.entity_count());
        });
    });
}

fn bench_query(c: &mut Criterion) {
    let mut world = UniversalWorld::new();
    for _ in 0..1000 {
        let e = world.spawn();
        world.insert_component(e, BenchPos { x: 0.0, y: 0.0 });
        world.insert_component(e, BenchVel { x: 1.0, y: 1.0 });
    }

    c.bench_function("query_1000", |b| {
        b.iter(|| {
            let result = world.query::<(BenchPos, BenchVel)>();
            black_box(result.len());
        });
    });
}

fn bench_scheduler(c: &mut Criterion) {
    c.bench_function("scheduler_1000_entities", |b| {
        b.iter_batched(
            || {
                let mut world = UniversalWorld::new();
                for _ in 0..1000 {
                    let e = world.spawn();
                    world.insert_component(e, BenchPos { x: 0.0, y: 0.0 });
                    world.insert_component(e, BenchVel { x: 1.0, y: 1.0 });
                }
                let mut sched = ParallelScheduler::new();
                sched.add_system(Box::new(BenchSys), SystemDependency::new());
                sched.build_schedule();
                (world, sched)
            },
            |(mut world, mut sched)| {
                sched.run(&mut world, 0.016);
                black_box(world.entity_count());
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_codegen(c: &mut Criterion) {
    let mut def = GameDefinition::default();
    for i in 0..10 {
        def.entities.insert(
            format!("Entity{}", i),
            EntityDef {
                components: vec!["pos".to_string(), "vel".to_string()],
                systems: vec![format!("sys{}", i)],
            },
        );
        def.systems.insert(
            format!("sys{}", i),
            SystemDef { priority: 0, read: vec![], write: vec![] },
        );
    }
    let gen = CodeGenerator::new(def);

    c.bench_function("codegen_bevy_10_entities", |b| {
        b.iter(|| {
            black_box(gen.generate_bevy());
        });
    });
}

criterion_group!(benches, bench_spawn, bench_query, bench_scheduler, bench_codegen);
criterion_main!(benches);
