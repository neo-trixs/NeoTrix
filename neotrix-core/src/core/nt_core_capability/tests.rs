//! 能力接口单元测试

#[cfg(test)]
mod capability_tests {
    use crate::core::nt_core_capability::*;
    use std::sync::Arc;

    #[test]
    fn test_capability_meta() {
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let meta = cap.meta();
        assert!(!meta.id.is_empty());
        assert!(!meta.name.is_empty());
        assert!(!meta.version.is_empty());
    }

    #[test]
    fn test_capability_health() {
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let health = cap.health();
        assert!(health.success_rate >= 0.0 && health.success_rate <= 1.0);
    }

    #[test]
    fn test_capability_supports() {
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试".into(),
            language: None,
        });
        assert!(cap.supports(&input));
    }

    #[test]
    fn test_capability_execute() {
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试文本".into(),
            language: None,
        });
        let result = cap.execute(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_registry() {
        let mut registry = CapabilityRegistry::new();
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        registry.register(cap);
        assert!(!registry.list_all().is_empty());
    }

    #[test]
    fn test_router() {
        let registry = Arc::new(init_global_registry());
        let mut router = CapabilityRouter::new(registry);
        router.add_rule(|input| match input {
            CapabilityInput::Nlp(_) => Some("nt-world-nlp".into()),
            _ => None,
        });

        let input = CapabilityInput::Nlp(NlpInput {
            task: NlpTask::Tokenize,
            text: "测试".into(),
            language: None,
        });
        let result = router.route(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_factory() {
        let caps = CapabilityFactory::create_all();
        assert!(caps.len() >= 3);
    }

    #[test]
    fn test_factory_by_domain() {
        let caps = CapabilityFactory::create_by_domain(Domain::NtWorld);
        assert!(!caps.is_empty());
    }

    #[test]
    fn test_factory_by_layer() {
        let caps = CapabilityFactory::create_by_layer(Layer::L2Perception);
        assert!(!caps.is_empty());
    }

    #[test]
    fn test_composer() {
        let registry = Arc::new(init_global_registry());
        let composer = CapabilityComposer::new(registry);
        assert!(!composer.registry().list_all().is_empty());
    }

    #[test]
    fn test_pipeline() {
        let pipeline = CapabilityComposer::create_pipeline("text_analysis");
        assert!(pipeline.is_some());
    }

    #[test]
    fn test_cache() {
        let config = CacheConfig::default();
        let mut cache = CapabilityCache::new(config);
        let key = "test".to_string();
        let value = CapabilityOutput::Text("hello".into());

        cache.set(key.clone(), value, None);
        let result = cache.get(&key);
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_capacity: 2,
            default_ttl: std::time::Duration::from_secs(300),
            enable_stats: true,
        };
        let mut cache = CapabilityCache::new(config);

        cache.set("a".into(), CapabilityOutput::Text("a".into()), None);
        cache.set("b".into(), CapabilityOutput::Text("b".into()), None);
        cache.set("c".into(), CapabilityOutput::Text("c".into()), None);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_monitor() {
        let registry = Arc::new(init_global_registry());
        let mut dashboard = MonitorDashboard::new(registry);

        dashboard.record_event(MonitorEvent {
            timestamp: std::time::Instant::now(),
            capability_id: "test".into(),
            event_type: EventType::Call,
            details: "test call".into(),
        });

        let stats = dashboard.event_stats();
        assert_eq!(stats.total_calls, 1);
    }

    #[test]
    fn test_discovery() {
        let config = DiscoveryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new());
        let mut discovery = DistributedDiscovery::new(config, registry);

        let result = discovery.discover();
        assert_eq!(result.nodes.len(), 1);
    }

    #[test]
    fn test_versioning() {
        let mut manager = VersionManager::new();
        let version = SemanticVersion::new(1, 0, 0);

        manager.register_version("test", version, "test", "Initial version");
        let current = manager.get_current_version("test");
        assert!(current.is_some());
    }

    #[test]
    fn test_orchestration() {
        let registry = Arc::new(CapabilityRegistry::new());
        let engine = OrchestrationEngine::new(registry);
        assert!(engine.get_active_flows().is_empty());
    }

    #[test]
    fn test_load_balancer() {
        let mut lb = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);
        let instance = CapabilityInstance {
            id: "inst_1".into(),
            capability_id: "cap_1".into(),
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
        let selected = lb.select_instance("cap_1");
        assert!(selected.is_some());
    }
}
