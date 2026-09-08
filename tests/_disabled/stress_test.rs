//! 融合架构压力测试
//!
//! 测试系统在高并发和长时间运行下的稳定性

use neotrix::core::fused_architecture::FusedArchitecture;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_high_concurrency_stress() {
    let arch = Arc::new(RwLock::new(FusedArchitecture::new()));
    
    // 初始化
    {
        let mut arch_write = arch.write().await;
        let _ = arch_write.init().await;
    }
    
    let concurrency = 100;
    let mut handles = Vec::new();
    
    let start = std::time::Instant::now();
    
    for i in 0..concurrency {
        let arch_clone = arch.clone();
        handles.push(tokio::spawn(async move {
            let mut arch_write = arch_clone.write().await;
            let result = arch_write.process_request(&format!("Stress test {}", i)).await;
            result.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    let duration = start.elapsed();
    
    // 验证所有请求都成功
    let success_count = results.iter().filter(|r| r.unwrap()).count();
    assert_eq!(success_count, concurrency);
    
    // 验证性能指标
    assert!(duration.as_millis() < 30000); // 100个并发请求应该在30秒内完成
    
    println!("✅ 高并发压力测试通过：{} 个并发请求，耗时 {:?}", concurrency, duration);
}

#[tokio::test]
async fn test_sustained_load_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    let duration = Duration::from_secs(10); // 持续10秒
    let start = std::time::Instant::now();
    let mut request_count = 0;
    
    while start.elapsed() < duration {
        let result = arch.process_request(&format!("Sustained load test {}", request_count)).await;
        assert!(result.is_ok());
        request_count += 1;
        
        // 避免CPU过载
        sleep(Duration::from_millis(10)).await;
    }
    
    let actual_duration = start.elapsed();
    let requests_per_second = request_count as f64 / actual_duration.as_secs_f64();
    
    // 验证吞吐量
    assert!(requests_per_second > 10.0); // 每秒至少10个请求
    
    println!("✅ 持续负载压力测试通过：{} 个请求，吞吐量 {:.2} req/s", request_count, requests_per_second);
}

#[tokio::test]
async fn test_memory_pressure_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 大量请求以测试内存使用
    let large_request_count = 1000;
    
    for i in 0..large_request_count {
        let result = arch.process_request(&format!("Memory pressure test {}", i)).await;
        assert!(result.is_ok());
        
        // 每100个请求验证一次状态
        if i % 100 == 0 {
            let status = arch.get_system_status().await;
            assert!(status.consciousness_health > 0.0);
            assert!(status.energy_level > 0.0);
        }
    }
    
    // 最终状态验证
    let status = arch.get_system_status().await;
    assert!(status.consciousness_health > 0.0);
    assert!(status.energy_level > 0.0);
    assert!(status.frequency_stability > 0.0);
    assert!(status.vibration_intensity > 0.0);
    assert!(status.manifestation_quality > 0.0);
    
    println!("✅ 内存压力测试通过：{} 个请求", large_request_count);
}

#[tokio::test]
async fn test_component_failure_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试组件故障恢复
    let failure_scenarios = vec![
        "组件超时",
        "资源不足",
        "配置错误",
        "依赖缺失",
    ];
    
    for scenario in failure_scenarios {
        let result = arch.process_request(scenario).await;
        // 系统应该能优雅处理各种故障场景
        assert!(result.is_ok());
    }
    
    // 验证系统仍然正常工作
    let status = arch.get_system_status().await;
    assert!(status.consciousness_health > 0.0);
    
    println!("✅ 组件故障压力测试通过");
}

#[tokio::test]
async fn test_concurrent_read_write_stress() {
    let arch = Arc::new(RwLock::new(FusedArchitecture::new()));
    
    // 初始化
    {
        let mut arch_write = arch.write().await;
        let _ = arch_write.init().await;
    }
    
    let mut handles = Vec::new();
    
    // 并发读写测试
    for i in 0..50 {
        let arch_clone = arch.clone();
        
        if i % 2 == 0 {
            // 写操作
            handles.push(tokio::spawn(async move {
                let mut arch_write = arch_clone.write().await;
                let result = arch_write.process_request(&format!("Write test {}", i)).await;
                result.is_ok()
            }));
        } else {
            // 读操作
            handles.push(tokio::spawn(async move {
                let arch_read = arch_clone.read().await;
                let status = arch_read.get_system_status().await;
                status.consciousness_health >= 0.0
            }));
        }
    }
    
    let results = futures::future::join_all(handles).await;
    
    // 验证所有操作都成功
    let success_count = results.iter().filter(|r| r.unwrap()).count();
    assert_eq!(success_count, 50);
    
    println!("✅ 并发读写压力测试通过");
}

#[tokio::test]
async fn test_rapid_initialization_stress() {
    let mut handles = Vec::new();
    
    // 快速初始化测试
    for i in 0..20 {
        handles.push(tokio::spawn(async move {
            let mut arch = FusedArchitecture::new();
            let result = arch.init().await;
            result.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    
    // 验证所有初始化都成功
    let success_count = results.iter().filter(|r| r.unwrap()).count();
    assert_eq!(success_count, 20);
    
    println!("✅ 快速初始化压力测试通过");
}

#[tokio::test]
async fn test_large_payload_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 大负载测试
    let large_payloads = vec![
        "x".repeat(1000),      // 1KB
        "y".repeat(10000),     // 10KB
        "z".repeat(100000),    // 100KB
    ];
    
    for (i, payload) in large_payloads.iter().enumerate() {
        let result = arch.process_request(payload).await;
        assert!(result.is_ok());
        
        // 验证系统状态
        let status = arch.get_system_status().await;
        assert!(status.consciousness_health > 0.0);
    }
    
    println!("✅ 大负载压力测试通过");
}

#[tokio::test]
async fn test_resource_exhaustion_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 资源耗尽测试
    let mut handles = Vec::new();
    
    for i in 0..200 {
        let mut arch_clone = FusedArchitecture::new();
        let _ = arch_clone.init().await;
        
        handles.push(tokio::spawn(async move {
            let result = arch_clone.process_request(&format!("Resource exhaustion test {}", i)).await;
            result.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    
    // 验证系统在资源压力下仍然稳定
    let success_count = results.iter().filter(|r| r.unwrap()).count();
    assert!(success_count > 150); // 至少75%的成功率
    
    println!("✅ 资源耗尽压力测试通过：{}/200 成功", success_count);
}

#[tokio::test]
async fn test_long_running_stress() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    let duration = Duration::from_secs(30); // 长时间运行30秒
    let start = std::time::Instant::now();
    let mut request_count = 0;
    let mut error_count = 0;
    
    while start.elapsed() < duration {
        let result = arch.process_request(&format!("Long running test {}", request_count)).await;
        
        if result.is_ok() {
            request_count += 1;
        } else {
            error_count += 1;
        }
        
        // 避免CPU过载
        sleep(Duration::from_millis(5)).await;
    }
    
    let actual_duration = start.elapsed();
    let requests_per_second = request_count as f64 / actual_duration.as_secs_f64();
    
    // 验证长时间运行的稳定性
    assert!(error_count < request_count / 10); // 错误率低于10%
    assert!(requests_per_second > 5.0); // 每秒至少5个请求
    
    println!("✅ 长时间运行压力测试通过：{} 个请求，{} 个错误，吞吐量 {:.2} req/s", 
             request_count, error_count, requests_per_second);
}

#[tokio::test]
async fn test_mixed_workload_stress() {
    let arch = Arc::new(RwLock::new(FusedArchitecture::new()));
    
    // 初始化
    {
        let mut arch_write = arch.write().await;
        let _ = arch_write.init().await;
    }
    
    let mut handles = Vec::new();
    
    // 混合工作负载
    for i in 0..100 {
        let arch_clone = arch.clone();
        
        handles.push(tokio::spawn(async move {
            match i % 4 {
                0 => {
                    // 写操作
                    let mut arch_write = arch_clone.write().await;
                    arch_write.process_request(&format!("Write workload {}", i)).await.is_ok()
                }
                1 => {
                    // 读操作
                    let arch_read = arch_clone.read().await;
                    let status = arch_read.get_system_status().await;
                    status.consciousness_health >= 0.0
                }
                2 => {
                    // 状态查询
                    let arch_read = arch_clone.read().await;
                    let status = arch_read.get_system_status().await;
                    !status.version.is_empty()
                }
                _ => {
                    // 混合操作
                    let mut arch_write = arch_clone.write().await;
                    arch_write.process_request(&format!("Mixed workload {}", i)).await.is_ok()
                }
            }
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    
    // 验证所有操作都成功
    let success_count = results.iter().filter(|r| r.unwrap()).count();
    assert!(success_count > 90); // 至少90%的成功率
    
    println!("✅ 混合工作负载压力测试通过：{}/100 成功", success_count);
}

#[tokio::test]
async fn test_final_stress_verification() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 最终压力验证
    let stress_duration = Duration::from_secs(5);
    let start = std::time::Instant::now();
    let mut total_requests = 0;
    
    while start.elapsed() < stress_duration {
        let result = arch.process_request("Final stress verification").await;
        assert!(result.is_ok());
        total_requests += 1;
        sleep(Duration::from_millis(10)).await;
    }
    
    // 验证系统状态
    let status = arch.get_system_status().await;
    assert_eq!(status.version, "1.0.0");
    assert!(status.consciousness_health > 0.0);
    assert!(status.energy_level > 0.0);
    assert!(status.frequency_stability > 0.0);
    assert!(status.vibration_intensity > 0.0);
    assert!(status.manifestation_quality > 0.0);
    
    println!("✅ 最终压力验证通过：{} 个请求", total_requests);
    println!("🎉 融合架构压力测试全部通过！");
}
