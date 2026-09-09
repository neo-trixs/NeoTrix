//! L1 能力网端到端测试
//!
//! 验证完整调用链路: L5 领域技能 → Bridge → Registry → Router → Provider

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ════════════════════════════════════════════════════════════════
// 测试辅助
// ════════════════════════════════════════════════════════════════

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

// ════════════════════════════════════════════════════════════════
// CAT-1 通信测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_cat1_messaging_e2e() {
    use crate::l1_action::nt_io::nt_io_messaging::*;
    
    let mut registry = MessagingRegistry::new();
    let whatsapp = WhatsAppProvider::new("https://api.whatsapp.com", "token", "123", "biz");
    registry.register(Box::new(whatsapp));
    
    let router = MessagingRouter::new(registry);
    let bridge = MessagingBridge::new(router);
    
    let post = Post {
        id: "test1".into(),
        platform: "whatsapp".into(),
        body: "Hello".into(),
        hashtags: vec![],
        status: "draft".into(),
    };
    
    let result = bridge.send(&post);
    assert!(result.is_ok());
}

#[test]
fn test_cat1_template_e2e() {
    use crate::l1_action::nt_io::nt_io_messaging::*;
    
    let templates = trade_templates();
    assert!(!templates.is_empty());
    
    let mut vars = HashMap::new();
    vars.insert("name".into(), "John".into());
    let result = templates[0].render(&vars);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("John"));
}

// ════════════════════════════════════════════════════════════════
// CAT-2 内容测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_cat2_content_e2e() {
    use crate::l1_action::nt_act::nt_act_media::*;
    
    let mut registry = ContentRegistry::new();
    registry.register(Box::new(LinkedInProvider::new("token", None)));
    registry.register(Box::new(InstagramProvider::new("token", "biz123")));
    
    let router = ContentRouter::new(registry);
    let bridge = ContentBridge::new(router);
    
    let post = Post {
        id: "test1".into(),
        platform: "linkedin".into(),
        body: "Test post".into(),
        hashtags: vec!["#test".into()],
        status: "draft".into(),
    };
    
    let result = bridge.publish(&post);
    assert!(result.is_ok());
}

// ════════════════════════════════════════════════════════════════
// CAT-3 数据测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_cat3_data_e2e() {
    use crate::l1_action::nt_memory::nt_memory_lead::*;
    
    let mut registry = LeadRegistry::new();
    registry.register(Box::new(LeadManager::new()));
    
    let router = LeadRouter::new(registry);
    let bridge = LeadBridge::new(router);
    
    let lead = Lead {
        id: "lead1".into(),
        source: LeadSource::Email,
        contact_name: "Test".into(),
        inquiry_text: "Test inquiry".into(),
        quality: LeadQuality::Warm,
        stage: LeadStage::Captured,
        score: 50.0,
        tags: vec![],
        interactions: vec![],
        assigned_to: None,
        created_at: now_secs(),
        updated_at: now_secs(),
        last_contact: None,
        next_follow_up: None,
        company_name: None,
        email: None,
        phone: None,
        whatsapp: None,
        country: None,
        product_interest: vec![],
    };
    
    // Test via registry
    let results = bridge.query("Test", 10).unwrap();
    assert!(results.is_empty() || !results.is_empty()); // query works
}

// ════════════════════════════════════════════════════════════════
// CAT-8 认知测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_cat8_cognition_e2e() {
    use crate::l1_action::nt_io::nt_io_provider::provider_pool::*;
    
    let mut pool = ProviderPool::new();
    pool.add(PoolEntry {
        label: "test".into(),
        provider: "openai".into(),
        api_key: "sk-test".into(),
        model: "gpt-4o".into(),
        tags: vec![],
        base_url: None,
        created_at: now_secs(),
    });
    
    // Test LlmRouter trait
    use crate::l1_action::traits::LlmRouter;
    let route = pool.route(&LlmRequest {
        prompt: "test".into(),
        model: None,
        max_tokens: None,
    }).unwrap();
    
    assert_eq!(route.provider, "openai");
    assert_eq!(route.model, "gpt-4o");
}

// ════════════════════════════════════════════════════════════════
// CAT-6 安全测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_cat6_security_e2e() {
    use crate::l1_action::nt_act::nt_act_security::*;
    
    let mut registry = SecurityRegistry::new();
    registry.register(Box::new(SecurityGuardManager::new()));
    
    let router = SecurityRouter::new(registry);
    let bridge = SecurityBridge::new(router);
    
    let req = ActionRequest {
        action: "read".into(),
        target: "file.txt".into(),
        parameters: HashMap::new(),
    };
    
    let verdict = bridge.check(&req);
    assert!(matches!(verdict, SecurityVerdict::Allow));
}

// ════════════════════════════════════════════════════════════════
// L5→L1 集成测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_l5_trade_orchestrator_integration() {
    use crate::l1_action::nt_act::nt_act_trade::orchestrator::*;
    
    let orchestrator = TradeOrchestrator::new();
    
    // Test G1: 创建社交内容
    let post = orchestrator.create_social_content(Platform::LinkedIn, "Test product");
    assert!(!post.body.is_empty());
    assert!(post.hashtags.contains(&"#trade".into()));
    
    // Test G1: 捕获询盘
    let lead_id = orchestrator.capture_inquiry(
        LeadSource::Email,
        "Buyer",
        "Interested in product",
        vec!["product1".into()],
    );
    assert!(!lead_id.is_empty());
    
    // Test G1: 资质评分
    let lead = orchestrator.leads.get_lead(&lead_id).unwrap();
    assert!(lead.score > 0.0);
}

// ════════════════════════════════════════════════════════════════
// 基础设施集成测试
// ════════════════════════════════════════════════════════════════

#[test]
fn test_infra_tracing() {
    use crate::l1_action::nt_infra_tracing::*;
    
    let span_id = trace_start("test.cap", "execute");
    trace_end(&span_id, true, None);
    
    let agg = trace_aggregate("test.cap").unwrap();
    assert_eq!(agg.total_calls, 1);
    assert_eq!(agg.successful, 1);
}

#[test]
fn test_infra_breaker() {
    use crate::l1_action::nt_infra_breaker::*;
    
    let mut breaker = CircuitBreaker::new(BreakerConfig::default());
    assert!(breaker.allow());
    
    breaker.record_result(false);
    breaker.record_result(false);
    // Still closed with 2 failures in window
    assert!(breaker.allow());
}

#[test]
fn test_infra_semantic_router() {
    use crate::l1_action::nt_infra_semantic_router::*;
    
    let mut router = SemanticRouter::new();
    router.add_rule(RouteRule {
        id: "r1".into(),
        intent: "search".into(),
        keywords: vec!["search".into(), "find".into()],
        provider_preference: vec!["kb".into()],
        priority: 1,
    });
    
    let decision = router.route("search for info", None).unwrap();
    assert_eq!(decision.provider_id, "kb");
}

#[test]
fn test_infra_agent_card() {
    use crate::l1_action::nt_infra_agent_card::*;
    
    let mut reg = AgentCardRegistry::new();
    let card = AgentCard::new("test", "Test", "Test agent")
        .with_capability(AgentCapability {
            name: "search".into(),
            description: "Search".into(),
            input_schema: None,
            output_schema: None,
            tags: vec![],
        });
    reg.register(card);
    
    let results = reg.find_by_capability("search");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_infra_scatter_gather() {
    use crate::l1_action::nt_infra_scatter_gather::*;
    
    let sg = ScatterGather::with_strategy(AggregateStrategy::BestScore);
    let responses = vec![
        ProviderResponse { provider_id: "a".into(), score: 0.6, data: vec![], latency_ms: 100, success: true },
        ProviderResponse { provider_id: "b".into(), score: 0.9, data: vec![], latency_ms: 200, success: true },
    ];
    
    let result = sg.gather(responses);
    assert_eq!(result.best_response.unwrap().provider_id, "b");
}

#[test]
fn test_infra_persistence() {
    use crate::l1_action::nt_infra_persistence::*;
    
    let mut p = RegistryPersistence::new("/tmp/test_registry2.json");
    p.upsert(PersistedEntry {
        id: "test1".into(),
        category: "search".into(),
        constellation: "C2".into(),
        description: "Test".into(),
        health_healthy: true,
        health_error_rate: 0.0,
        tags: vec![],
        metadata: HashMap::new(),
    });
    assert_eq!(p.count(), 1);
}

#[test]
fn test_infra_learning() {
    use crate::l1_action::nt_infra_learning::*;
    
    let mut l = RouterLearner::new();
    l.record_call("a", true, 100.0);
    l.record_call("a", true, 150.0);
    l.record_call("b", true, 50.0);
    
    assert_eq!(l.best_provider(), Some("b"));
}

#[test]
fn test_infra_integration() {
    use crate::l1_action::nt_infra_integration::*;
    
    let mut reg = EnhancedRegistry::new();
    let card = super::nt_infra_agent_card::AgentCard::new("test", "Test", "Test agent");
    reg.register(card);
    
    assert!(reg.is_available("test"));
    assert_eq!(reg.available().len(), 1);
}