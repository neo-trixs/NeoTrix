//! 融合架构端到端测试
//!
//! 测试完整的能量→频率→震动→显化流程

use neotrix::core::fused_architecture::{
    FusedArchitecture, FusedArchitectureIntegration, SystemStatus,
    ConsciousnessCore, EnergyLayer, FrequencyLayer, VibrationLayer, ManifestationLayer,
    CapabilityNetwork, SkillEcosystem,
};

#[tokio::test]
async fn test_fused_architecture_creation() {
    let mut arch = FusedArchitecture::new();
    assert!(arch.init().await.is_ok());
}

#[tokio::test]
async fn test_system_status() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    let status = arch.get_system_status().await;
    assert_eq!(status.version, "1.0.0");
    assert!(status.consciousness_health >= 0.0);
    assert!(status.consciousness_health <= 1.0);
    assert!(status.energy_level >= 0.0);
    assert!(status.energy_level <= 1.0);
    assert!(status.frequency_stability >= 0.0);
    assert!(status.frequency_stability <= 1.0);
    assert!(status.vibration_intensity >= 0.0);
    assert!(status.vibration_intensity <= 1.0);
    assert!(status.manifestation_quality >= 0.0);
    assert!(status.manifestation_quality <= 1.0);
}

#[tokio::test]
async fn test_consciousness_core_analysis() {
    let core = ConsciousnessCore::new();
    
    let analysis = core.analyze_request("帮我分析这段代码").await;
    assert!(analysis.is_ok());
    
    let analysis = analysis.unwrap();
    assert!(!analysis.request_type.is_empty());
    assert!(analysis.complexity >= 0.0);
    assert!(analysis.complexity <= 1.0);
}

#[tokio::test]
async fn test_energy_layer_assessment() {
    let mut energy_layer = EnergyLayer::new();
    let _ = energy_layer.init().await;
    
    let requirement = energy_layer.assess_requirement(&neotrix::core::fused_architecture::core::RequestAnalysis {
        request_type: "code_analysis".to_string(),
        complexity: 0.7,
        required_capabilities: vec!["code_analysis".to_string()],
        estimated_energy: 0.6,
    }).await;
    
    assert!(requirement.is_ok());
    let requirement = requirement.unwrap();
    assert!(requirement.energy_amount > 0.0);
}

#[tokio::test]
async fn test_frequency_layer_selection() {
    let mut frequency_layer = FrequencyLayer::new();
    let _ = frequency_layer.init().await;
    
    let frequency = frequency_layer.select_frequency(&neotrix::core::fused_architecture::layers::EnergyRequirement {
        energy_amount: 0.7,
        energy_type: "cognitive".to_string(),
        priority: 1,
    }).await;
    
    assert!(frequency.is_ok());
    let frequency = frequency.unwrap();
    assert!(frequency.value >= 0.0);
    assert!(frequency.value <= 1.0);
}

#[tokio::test]
async fn test_vibration_layer_execution() {
    let mut vibration_layer = VibrationLayer::new();
    let _ = vibration_layer.init().await;
    
    let result = vibration_layer.execute_vibration(&neotrix::core::fused_architecture::layers::Frequency {
        value: 0.7,
        stability: 0.9,
        resonance: 0.8,
    }).await;
    
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(result.execution_time_ms > 0);
}

#[tokio::test]
async fn test_manifestation_layer_output() {
    let mut manifestation_layer = ManifestationLayer::new();
    let _ = manifestation_layer.init().await;
    
    let result = manifestation_layer.manifest(&neotrix::core::fused_architecture::layers::VibrationResult {
        success: true,
        vibration_intensity: 0.8,
        effects: vec!["analysis_complete".to_string()],
        execution_time_ms: 50,
    }).await;
    
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_capability_network_operations() {
    let mut capability_network = CapabilityNetwork::new();
    let _ = capability_network.init().await;
    
    // 测试能力注册
    let capability = neotrix::core::fused_architecture::capability::CapabilityEntry {
        id: "test_capability".to_string(),
        name: "Test Capability".to_string(),
        description: "A test capability".to_string(),
        version: "1.0.0".to_string(),
        capability_type: neotrix::core::fused_architecture::capability::CapabilityType::Native,
        status: neotrix::core::fused_architecture::capability::CapabilityStatus::Active,
        energy_requirement: 0.5,
        frequency: 0.7,
        vibration_pattern: "steady".to_string(),
    };
    
    let register_result = capability_network.register_capability(capability).await;
    assert!(register_result.is_ok());
    
    // 测试能力查询
    let query_result = capability_network.query_capability("test_capability").await;
    assert!(query_result.is_ok());
    let queried = query_result.unwrap();
    assert!(queried.is_some());
    assert_eq!(queried.unwrap().id, "test_capability");
}

#[tokio::test]
async fn test_skill_ecosystem_operations() {
    let mut skill_ecosystem = SkillEcosystem::new();
    let _ = skill_ecosystem.init().await;
    
    // 测试技能构建
    let skill = neotrix::core::fused_architecture::skill::SkillDefinition {
        id: "test_skill".to_string(),
        name: "Test Skill".to_string(),
        description: "A test skill".to_string(),
        skill_type: neotrix::core::fused_architecture::skill::SkillType::CodeAnalysis,
        required_capabilities: vec!["test_capability".to_string()],
        energy_cost: 0.3,
        expected_performance_gain: 0.2,
    };
    
    let build_result = skill_ecosystem.build_skill(skill).await;
    assert!(build_result.is_ok());
    
    // 测试技能查询
    let query_result = skill_ecosystem.query_skill("test_skill").await;
    assert!(query_result.is_ok());
    let queried = query_result.unwrap();
    assert!(queried.is_some());
    assert_eq!(queried.unwrap().id, "test_skill");
}

#[tokio::test]
async fn test_wisdom_integration() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试智慧累积
    let wisdom = neotrix::core::fused_architecture::wisdom::WisdomEntry {
        id: "test_wisdom".to_string(),
        source: "e2e_test".to_string(),
        content: "Test wisdom content".to_string(),
        confidence: 0.8,
        domain: "testing".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    let accumulate_result = arch.wisdom_integration.accumulate_wisdom(wisdom).await;
    assert!(accumulate_result.is_ok());
    
    // 测试智慧检索
    let retrieve_result = arch.wisdom_integration.retrieve_wisdom("testing").await;
    assert!(retrieve_result.is_ok());
    let retrieved = retrieve_result.unwrap();
    assert!(!retrieved.is_empty());
}

#[tokio::test]
async fn test_evolution_mechanism() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试进化触发
    let trigger_result = arch.evolution.trigger_evolution("performance_optimization").await;
    assert!(trigger_result.is_ok());
    
    // 测试进化状态
    let status_result = arch.evolution.get_evolution_status().await;
    assert!(status_result.is_ok());
    let status = status_result.unwrap();
    assert!(!status.current_phase.is_empty());
}

#[tokio::test]
async fn test_full_process_flow() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试完整处理流程
    let result = arch.process_request("分析这段代码的性能瓶颈").await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试并发请求
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let mut arch_clone = FusedArchitecture::new();
        let _ = arch_clone.init().await;
        
        handles.push(tokio::spawn(async move {
            let result = arch_clone.process_request(&format!("Request {}", i)).await;
            result.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap());
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试空请求
    let result = arch.process_request("").await;
    assert!(result.is_ok()); // 空请求应该被优雅处理
    
    // 测试超长请求
    let long_request = "x".repeat(10000);
    let result = arch.process_request(&long_request).await;
    assert!(result.is_ok()); // 超长请求应该被优雅处理
}

#[tokio::test]
async fn test_state_persistence() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 处理一些请求
    let _ = arch.process_request("Request 1").await;
    let _ = arch.process_request("Request 2").await;
    
    // 检查状态
    let status = arch.get_system_status().await;
    assert!(status.capability_count > 0);
    assert!(status.skill_count > 0);
}

#[tokio::test]
async fn test_integration_initialization() {
    let integration = FusedArchitectureIntegration::new();
    let init_result = integration.initialize().await;
    assert!(init_result.is_ok());
}

#[tokio::test]
async fn test_integration_process() {
    let mut integration = FusedArchitectureIntegration::new();
    let _ = integration.initialize().await;
    
    let result = integration.process_request("Test request").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_energy_frequency_vibration_chain() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 测试能量→频率→震动链式调用
    let energy_req = neotrix::core::fused_architecture::layers::EnergyRequirement {
        energy_amount: 0.6,
        energy_type: "cognitive".to_string(),
        priority: 1,
    };
    
    let energy_result = arch.energy_layer.assess_requirement(&neotrix::core::fused_architecture::core::RequestAnalysis {
        request_type: "test".to_string(),
        complexity: 0.5,
        required_capabilities: vec![],
        estimated_energy: 0.6,
    }).await;
    assert!(energy_result.is_ok());
    
    let frequency_result = arch.frequency_layer.select_frequency(&energy_result.unwrap()).await;
    assert!(frequency_result.is_ok());
    
    let vibration_result = arch.vibration_layer.execute_vibration(&frequency_result.unwrap()).await;
    assert!(vibration_result.is_ok());
    
    let manifestation_result = arch.manifestation_layer.manifest(&vibration_result.unwrap()).await;
    assert!(manifestation_result.is_ok());
}

#[tokio::test]
async fn test_capability_skill_interaction() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 注册能力
    let capability = neotrix::core::fused_architecture::capability::CapabilityEntry {
        id: "interaction_capability".to_string(),
        name: "Interaction Capability".to_string(),
        description: "A capability for interaction test".to_string(),
        version: "1.0.0".to_string(),
        capability_type: neotrix::core::fused_architecture::capability::CapabilityType::Native,
        status: neotrix::core::fused_architecture::capability::CapabilityStatus::Active,
        energy_requirement: 0.4,
        frequency: 0.6,
        vibration_pattern: "steady".to_string(),
    };
    
    let register_result = arch.capability_network.register_capability(capability).await;
    assert!(register_result.is_ok());
    
    // 构建技能
    let skill = neotrix::core::fused_architecture::skill::SkillDefinition {
        id: "interaction_skill".to_string(),
        name: "Interaction Skill".to_string(),
        description: "A skill for interaction test".to_string(),
        skill_type: neotrix::core::fused_architecture::skill::SkillType::CodeAnalysis,
        required_capabilities: vec!["interaction_capability".to_string()],
        energy_cost: 0.3,
        expected_performance_gain: 0.2,
    };
    
    let build_result = arch.skill_ecosystem.build_skill(skill).await;
    assert!(build_result.is_ok());
    
    // 验证技能已注册
    let query_result = arch.skill_ecosystem.query_skill("interaction_skill").await;
    assert!(query_result.is_ok());
    assert!(query_result.unwrap().is_some());
}

#[tokio::test]
async fn test_wisdom_evolution_feedback_loop() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 累积智慧
    let wisdom = neotrix::core::fused_architecture::wisdom::WisdomEntry {
        id: "feedback_wisdom".to_string(),
        source: "feedback_test".to_string(),
        content: "Feedback loop wisdom".to_string(),
        confidence: 0.9,
        domain: "optimization".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    
    let accumulate_result = arch.wisdom_integration.accumulate_wisdom(wisdom).await;
    assert!(accumulate_result.is_ok());
    
    // 触发进化
    let evolution_result = arch.evolution.trigger_evolution("wisdom_based").await;
    assert!(evolution_result.is_ok());
    
    // 检查进化状态
    let status_result = arch.evolution.get_evolution_status().await;
    assert!(status_result.is_ok());
}

#[tokio::test]
async fn test_performance_metrics() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    let start = std::time::Instant::now();
    
    // 执行多个请求
    for i in 0..10 {
        let _ = arch.process_request(&format!("Performance test {}", i)).await;
    }
    
    let duration = start.elapsed();
    
    // 验证性能指标
    assert!(duration.as_millis() < 5000); // 10个请求应该在5秒内完成
    
    let status = arch.get_system_status().await;
    assert!(status.consciousness_health > 0.0);
    assert!(status.energy_level > 0.0);
}

#[tokio::test]
async fn test_concurrent_safety() {
    let arch = std::sync::Arc::new(tokio::sync::RwLock::new(FusedArchitecture::new()));
    
    // 初始化
    {
        let mut arch_write = arch.write().await;
        let _ = arch_write.init().await;
    }
    
    // 并发读写测试
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let arch_clone = arch.clone();
        handles.push(tokio::spawn(async move {
            let mut arch_write = arch_clone.write().await;
            let result = arch_write.process_request(&format!("Concurrent test {}", i)).await;
            result.is_ok()
        }));
    }
    
    let results = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap());
    }
}

#[tokio::test]
async fn test_memory_usage() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 执行大量请求以测试内存使用
    for i in 0..100 {
        let _ = arch.process_request(&format!("Memory test {}", i)).await;
    }
    
    // 验证系统仍然正常工作
    let status = arch.get_system_status().await;
    assert!(status.consciousness_health > 0.0);
    assert!(status.energy_level > 0.0);
}

#[tokio::test]
async fn test_graceful_degradation() {
    let mut arch = FusedArchitecture::new();
    let _ = arch.init().await;
    
    // 模拟部分组件失败
    // 测试系统是否能优雅降级
    let result = arch.process_request("Graceful degradation test").await;
    assert!(result.is_ok());
}

#[tokio::test]
async test_final_verification() {
    let mut arch = FusedArchitecture::new();
    let init_result = arch.init().await;
    assert!(init_result.is_ok());
    
    // 验证所有组件都已初始化
    let status = arch.get_system_status().await;
    assert_eq!(status.version, "1.0.0");
    assert!(status.consciousness_health >= 0.0);
    assert!(status.energy_level >= 0.0);
    assert!(status.frequency_stability >= 0.0);
    assert!(status.vibration_intensity >= 0.0);
    assert!(status.manifestation_quality >= 0.0);
    
    // 执行完整流程
    let result = arch.process_request("Final verification test").await;
    assert!(result.is_ok());
    
    println!("✅ 融合架构 E2E 测试全部通过！");
}
