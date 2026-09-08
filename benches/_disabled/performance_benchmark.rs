//! 融合架构性能基准测试
//!
//! 测试能量→频率→震动→显化流程的性能指标

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use neotrix::core::fused_architecture::FusedArchitecture;

fn bench_fused_architecture_creation(c: &mut Criterion) {
    c.bench_function("fused_architecture_creation", |b| {
        b.iter(|| {
            let arch = FusedArchitecture::new();
            arch
        })
    });
}

fn bench_fused_architecture_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("fused_architecture_initialization");
    
    for size in [1, 5, 10, 50, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut arch = FusedArchitecture::new();
                for _ in 0..size {
                    let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
                }
                arch
            })
        });
    }
    
    group.finish();
}

fn bench_process_request(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_request");
    
    let requests = vec![
        "分析这段代码",
        "优化性能瓶颈",
        "检查安全漏洞",
        "重构这个模块",
        "生成测试用例",
    ];
    
    for request in &requests {
        group.bench_with_input(BenchmarkId::from_parameter(request), request, |b, request| {
            b.iter(|| {
                let mut arch = FusedArchitecture::new();
                let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
                tokio::runtime::Runtime::new().unwrap().block_on(arch.process_request(request))
            })
        });
    }
    
    group.finish();
}

fn bench_energy_layer(c: &mut Criterion) {
    let mut group = c.benchmark_group("energy_layer");
    
    group.bench_function("assess_requirement", |b| {
        b.iter(|| {
            let mut energy_layer = neotrix::core::fused_architecture::EnergyLayer::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(energy_layer.init());
            
            let requirement = neotrix::core::fused_architecture::layers::EnergyRequirement {
                energy_amount: 0.7,
                energy_type: "cognitive".to_string(),
                priority: 1,
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(energy_layer.assess_requirement(&neotrix::core::fused_architecture::core::RequestAnalysis {
                request_type: "test".to_string(),
                complexity: 0.5,
                required_capabilities: vec![],
                estimated_energy: 0.7,
            }))
        })
    });
    
    group.finish();
}

fn bench_frequency_layer(c: &mut Criterion) {
    let mut group = c.benchmark_group("frequency_layer");
    
    group.bench_function("select_frequency", |b| {
        b.iter(|| {
            let mut frequency_layer = neotrix::core::fused_architecture::FrequencyLayer::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(frequency_layer.init());
            
            let requirement = neotrix::core::fused_architecture::layers::EnergyRequirement {
                energy_amount: 0.6,
                energy_type: "cognitive".to_string(),
                priority: 1,
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(frequency_layer.select_frequency(&requirement))
        })
    });
    
    group.finish();
}

fn bench_vibration_layer(c: &mut Criterion) {
    let mut group = c.benchmark_group("vibration_layer");
    
    group.bench_function("execute_vibration", |b| {
        b.iter(|| {
            let mut vibration_layer = neotrix::core::fused_architecture::VibrationLayer::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(vibration_layer.init());
            
            let frequency = neotrix::core::fused_architecture::layers::Frequency {
                value: 0.7,
                stability: 0.9,
                resonance: 0.8,
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(vibration_layer.execute_vibration(&frequency))
        })
    });
    
    group.finish();
}

fn bench_manifestation_layer(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifestation_layer");
    
    group.bench_function("manifest", |b| {
        b.iter(|| {
            let mut manifestation_layer = neotrix::core::fused_architecture::ManifestationLayer::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(manifestation_layer.init());
            
            let vibration_result = neotrix::core::fused_architecture::layers::VibrationResult {
                success: true,
                vibration_intensity: 0.8,
                effects: vec!["test".to_string()],
                execution_time_ms: 50,
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(manifestation_layer.manifest(&vibration_result))
        })
    });
    
    group.finish();
}

fn bench_capability_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_network");
    
    group.bench_function("register_capability", |b| {
        b.iter(|| {
            let mut capability_network = neotrix::core::fused_architecture::CapabilityNetwork::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(capability_network.init());
            
            let capability = neotrix::core::fused_architecture::capability::CapabilityEntry {
                id: "bench_capability".to_string(),
                name: "Benchmark Capability".to_string(),
                description: "A benchmark capability".to_string(),
                version: "1.0.0".to_string(),
                capability_type: neotrix::core::fused_architecture::capability::CapabilityType::Native,
                status: neotrix::core::fused_architecture::capability::CapabilityStatus::Active,
                energy_requirement: 0.5,
                frequency: 0.7,
                vibration_pattern: "steady".to_string(),
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(capability_network.register_capability(capability))
        })
    });
    
    group.bench_function("query_capability", |b| {
        b.iter(|| {
            let mut capability_network = neotrix::core::fused_architecture::CapabilityNetwork::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(capability_network.init());
            
            let capability = neotrix::core::fused_architecture::capability::CapabilityEntry {
                id: "bench_capability".to_string(),
                name: "Benchmark Capability".to_string(),
                description: "A benchmark capability".to_string(),
                version: "1.0.0".to_string(),
                capability_type: neotrix::core::fused_architecture::capability::CapabilityType::Native,
                status: neotrix::core::fused_architecture::capability::CapabilityStatus::Active,
                energy_requirement: 0.5,
                frequency: 0.7,
                vibration_pattern: "steady".to_string(),
            };
            
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(capability_network.register_capability(capability));
            tokio::runtime::Runtime::new().unwrap().block_on(capability_network.query_capability("bench_capability"))
        })
    });
    
    group.finish();
}

fn bench_skill_ecosystem(c: &mut Criterion) {
    let mut group = c.benchmark_group("skill_ecosystem");
    
    group.bench_function("build_skill", |b| {
        b.iter(|| {
            let mut skill_ecosystem = neotrix::core::fused_architecture::SkillEcosystem::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(skill_ecosystem.init());
            
            let skill = neotrix::core::fused_architecture::skill::SkillDefinition {
                id: "bench_skill".to_string(),
                name: "Benchmark Skill".to_string(),
                description: "A benchmark skill".to_string(),
                skill_type: neotrix::core::fused_architecture::skill::SkillType::CodeAnalysis,
                required_capabilities: vec![],
                energy_cost: 0.3,
                expected_performance_gain: 0.2,
            };
            
            tokio::runtime::Runtime::new().unwrap().block_on(skill_ecosystem.build_skill(skill))
        })
    });
    
    group.bench_function("query_skill", |b| {
        b.iter(|| {
            let mut skill_ecosystem = neotrix::core::fused_architecture::SkillEcosystem::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(skill_ecosystem.init());
            
            let skill = neotrix::core::fused_architecture::skill::SkillDefinition {
                id: "bench_skill".to_string(),
                name: "Benchmark Skill".to_string(),
                description: "A benchmark skill".to_string(),
                skill_type: neotrix::core::fused_architecture::skill::SkillType::CodeAnalysis,
                required_capabilities: vec![],
                energy_cost: 0.3,
                expected_performance_gain: 0.2,
            };
            
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(skill_ecosystem.build_skill(skill));
            tokio::runtime::Runtime::new().unwrap().block_on(skill_ecosystem.query_skill("bench_skill"))
        })
    });
    
    group.finish();
}

fn bench_concurrent_requests(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_requests");
    
    for concurrency in [1, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut handles = Vec::new();
                        
                        for i in 0..concurrency {
                            handles.push(tokio::spawn(async move {
                                let mut arch = FusedArchitecture::new();
                                let _ = arch.init().await;
                                arch.process_request(&format!("Concurrent test {}", i)).await
                            }));
                        }
                        
                        let results = futures::future::join_all(handles).await;
                        for result in results {
                            let _ = result.unwrap();
                        }
                    })
                })
            },
        );
    }
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("process_100_requests", |b| {
        b.iter(|| {
            let mut arch = FusedArchitecture::new();
            let _ = tokio::runtime::Runtime::new().unwrap().block_on(arch.init());
            
            for i in 0..100 {
                let _ = tokio::runtime::Runtime::new().unwrap().block_on(
                    arch.process_request(&format!("Memory test {}", i))
                );
            }
            
            arch
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_fused_architecture_creation,
    bench_fused_architecture_initialization,
    bench_process_request,
    bench_energy_layer,
    bench_frequency_layer,
    bench_vibration_layer,
    bench_manifestation_layer,
    bench_capability_network,
    bench_skill_ecosystem,
    bench_concurrent_requests,
    bench_memory_usage,
);

criterion_main!(benches);
