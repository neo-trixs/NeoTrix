//! 能力接口集成测试

#[cfg(test)]
mod integration_tests {
    use crate::core::nt_core_capability::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_full_pipeline() {
        // 1. 创建能力
        let nlp_cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let asset_cap = crate::l2_perception::nt_world::asset_map::asset_map_capability::create_asset_map_capability();
        let shield_cap =
            crate::l3_embodiment::nt_shield::shield_capability::create_shield_capabilities();

        // 2. 注册到注册中心
        let mut registry = CapabilityRegistry::new();
        registry.register(nlp_cap);
        registry.register(asset_cap);

        // 3. 创建路由器
        let mut router = CapabilityRouter::new(Arc::new(registry));
        router.add_rule(|input| match input {
            CapabilityInput::Nlp(_) => Some("nt-world-nlp".into()),
            CapabilityInput::Asset(_) => Some("nt-world-asset-map".into()),
            _ => None,
        });

        // 4. 执行路由
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试路由".into(),
            language: None,
        });

        let result = router.route(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_integration() {
        // 1. 创建能力
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();

        // 2. 创建缓存包装器
        let config = CacheConfig {
            max_capacity: 100,
            default_ttl: Duration::from_secs(60),
            enable_stats: true,
        };
        let cached = CachedCapability::new(cap, config);

        // 3. 执行两次相同输入
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "缓存测试".into(),
            language: None,
        });

        let result1 = cached.execute_cached(input.clone());
        let result2 = cached.execute_cached(input);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // 4. 检查缓存统计
        let stats = cached.cache_stats();
        assert!(stats.hits > 0);
    }

    #[test]
    fn test_composer_integration() {
        // 1. 创建能力注册中心
        let registry = Arc::new(init_global_registry());

        // 2. 创建组合器
        let composer = CapabilityComposer::new(registry);

        // 3. 创建管道
        let pipeline = CapabilityComposer::create_pipeline("text_analysis").unwrap();

        // 4. 执行管道
        let input = CapabilityInput::Text("集成测试".into());
        let result = composer.execute_pipeline(&pipeline, input);

        assert!(result.is_ok());
    }

    #[test]
    fn test_monitoring_integration() {
        // 1. 创建监控收集器
        let config = MonitoringConfig::default();
        let collector = Arc::new(std::sync::Mutex::new(MonitoringCollector::new(config)));

        // 2. 创建监控包装器
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let monitored = MonitoredCapability::new(cap, collector.clone());

        // 3. 执行多次调用
        for i in 0..5 {
            let input = CapabilityInput::Nlp(NlpInput {
                task: NlpTask::Tokenize,
                text: format!("测试{}", i),
                language: None,
            });
            let _ = monitored.execute_monitored(input);
        }

        // 4. 检查监控数据
        let collector = collector.lock().unwrap();
        let metrics = collector.get_metrics("nt-world-nlp");
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().call_count, 5);
    }

    #[test]
    fn test_security_integration() {
        // 1. 创建安全管理器
        let policy = SecurityPolicy::default();
        let security = Arc::new(std::sync::Mutex::new(SecurityManager::new(policy)));

        // 2. 添加令牌
        let token = AccessToken {
            id: "token_1".into(),
            user_id: "user_1".into(),
            permissions: vec!["capability:nt-world-nlp".into()],
            expires_at: std::time::Instant::now() + Duration::from_secs(3600),
            signature: "sig".into(),
        };
        security.lock().unwrap().add_token(token);

        // 3. 创建安全包装器
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let secure = SecureCapability::new(cap, security);

        // 4. 执行安全调用
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "安全测试".into(),
            language: None,
        });

        let result = secure.execute_secure(input, "token_1", "127.0.0.1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_balancer_integration() {
        // 1. 创建负载均衡器
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);

        // 2. 添加实例
        for i in 0..3 {
            let instance = CapabilityInstance {
                id: format!("inst_{}", i),
                capability_id: "nt-world-nlp".into(),
                weight: 1,
                current_connections: 0,
                max_connections: 100,
                avg_response_time_ms: 0,
                success_rate: 1.0,
                total_calls: 0,
                last_called: None,
                status: InstanceStatus::Healthy,
            };
            lb.add_instance(instance);
        }

        // 3. 选择实例
        let selected = lb.select_instance("nt-world-nlp");
        assert!(selected.is_some());

        // 4. 更新统计
        if let Some(inst) = selected {
            lb.update_instance_stats(&inst.id, 100, true);
        }

        assert_eq!(lb.stats().total_requests, 1);
    }

    #[test]
    fn test_versioning_integration() {
        // 1. 创建版本管理器
        let mut manager = VersionManager::new();

        // 2. 注册版本
        let v1 = SemanticVersion::new(1, 0, 0);
        manager.register_version("nt-world-nlp", v1, "system", "初始版本");

        // 3. 升级版本
        let v2 = manager.upgrade("nt-world-nlp", UpgradeType::Minor);
        assert!(v2.is_some());
        assert_eq!(v2.unwrap().to_string(), "1.1.0");

        // 4. 检查版本
        let current = manager.get_current_version("nt-world-nlp");
        assert!(current.is_some());
        assert_eq!(current.unwrap().to_string(), "1.1.0");
    }

    #[test]
    fn test_discovery_integration() {
        // 1. 创建发现器
        let config = DiscoveryConfig::default();
        let registry = Arc::new(init_global_registry());
        let mut discovery = DistributedDiscovery::new(config, registry);

        // 2. 执行发现
        let result = discovery.discover();
        assert_eq!(result.nodes.len(), 1);

        // 3. 获取在线节点
        let online = discovery.get_online_nodes();
        assert!(!online.is_empty());

        // 4. 查找能力节点
        let nodes = discovery.find_capability_nodes("nt-world-nlp");
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_orchestration_integration() {
        // 1. 创建能力注册中心
        let mut registry = CapabilityRegistry::new();
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        registry.register(cap);

        // 2. 创建编排引擎
        let mut engine = OrchestrationEngine::new(Arc::new(registry));

        // 3. 创建编排流程
        let flow = OrchestrationFlow {
            id: "flow_1".into(),
            name: "测试流程".into(),
            mode: OrchestrationMode::Sequential,
            steps: vec![OrchestrationStep {
                id: "step1".into(),
                name: "步骤1".into(),
                capability_id: "nt-world-nlp".into(),
                input_mapping: "passthrough".into(),
                output_mapping: "passthrough".into(),
                condition: None,
                max_retries: 3,
                timeout_ms: 5000,
            }],
            global_timeout_ms: 30000,
            max_parallelism: 1,
        };

        // 4. 执行流程
        let input = CapabilityInput::Text("编排测试".into());
        let result = engine.execute_flow(&flow, input);

        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_integration() {
        // 1. 创建性能优化器
        let pool_config = PoolConfig::default();
        let cache_config = CacheConfig::default();
        let optimizer = Arc::new(std::sync::Mutex::new(PerformanceOptimizer::new(
            pool_config,
            cache_config,
        )));

        // 2. 创建优化包装器
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let optimized = OptimizedCapability::new(cap, optimizer.clone());

        // 3. 执行多次调用
        for i in 0..10 {
            let input = CapabilityInput::Nlp(NlpInput {
                task: NlpTask::Tokenize,
                text: format!("性能测试{}", i),
                language: None,
            });
            let _ = optimized.execute_optimized(input);
        }

        // 4. 检查性能统计
        let optimizer = optimizer.lock().unwrap();
        assert_eq!(optimizer.stats().total_requests, 10);
    }
}
