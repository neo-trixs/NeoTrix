#![forbid(unsafe_code)]

//! Integration tests for business capability modules:
//! - SalespersonProfiler
//! - WritingStyleAnalyzer
//! - TradeIntelligence
//! - SalesCoach

use neotrix_core::l4_emotion::nt_feel::salesperson_profiling::*;
use neotrix_core::l4_emotion::nt_feel::writing_style::*;
use neotrix_core::l5_cognition::nt_core::trade_intelligence::*;
use neotrix_core::l5_cognition::nt_mind::sales_coaching::*;

// ── Helpers ────────────────────────────────────────────────────────────────

fn make_chat(message: &str, sender: &str, ts: i64) -> ChatRecord {
    ChatRecord {
        message: message.to_string(),
        sender: sender.to_string(),
        timestamp: ts,
    }
}

fn make_email(subject: &str, body: &str, sender: &str, ts: i64) -> EmailRecord {
    EmailRecord {
        subject: subject.to_string(),
        body: body.to_string(),
        sender: sender.to_string(),
        timestamp: ts,
    }
}

fn make_customer(id: &str, name: &str) -> Customer {
    Customer {
        customer_id: id.to_string(),
        name: name.to_string(),
    }
}

fn make_intelligence_input() -> IntelligenceInput {
    IntelligenceInput {
        customer_data: CustomerData {
            customer_id: "C001".to_string(),
            name: "Acme Corp".to_string(),
            country: "US".to_string(),
            industry: "Manufacturing".to_string(),
            grade: "A".to_string(),
            source: "Exhibition".to_string(),
            total_value: 250_000.0,
            interaction_count: 8,
        },
        interaction_history: vec![
            InteractionEvent {
                event_type: "email".to_string(),
                timestamp: now_ts() - 86400,
                content: "We need bulk order pricing for 500 units".to_string(),
                sentiment: 0.6,
            },
            InteractionEvent {
                event_type: "call".to_string(),
                timestamp: now_ts() - 172800,
                content: "Can you provide a proposal with volume discount?".to_string(),
                sentiment: 0.4,
            },
        ],
        market_context: MarketContext {
            region: "North America".to_string(),
            industry_trend: 0.3,
            competitor_activity: 0.4,
            exchange_rate: 0.05,
            seasonality: Seasonality::Peak,
        },
        company_policies: vec![Policy {
            policy_type: "payment".to_string(),
            rules: vec!["Net 30".to_string()],
            exceptions: vec![],
        }],
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn make_coaching_request(situation: &str, history_len: usize) -> CoachingRequest {
    let history: Vec<InteractionHistory> = (0..history_len)
        .map(|i| InteractionHistory {
            channel: "email".to_string(),
            timestamp: i as i64,
            direction: Direction::Outbound,
            content: format!("Interaction {}", i),
            outcome: Some("qualified".to_string()),
        })
        .collect();

    CoachingRequest {
        salesperson_id: "sp_001".to_string(),
        customer_id: "cust_001".to_string(),
        situation: situation.to_string(),
        history,
        goals: vec!["close deal".to_string()],
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SalespersonProfiler tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_profiler_new() {
    let profiler = SalespersonProfiler::new();
    assert!(profiler.get_profile("any").is_none());
}

#[test]
fn test_profile_generation() {
    let mut profiler = SalespersonProfiler::new();
    let chats = vec![
        make_chat("Hello, how are you?", "sp1", 1000),
        make_chat("Hey there 😊", "sp1", 2000),
        make_chat("Any questions?", "customer1", 1500),
    ];
    let emails = vec![make_email("RE: Order", "Thanks for the order", "sp1", 3000)];
    let customers = vec![make_customer("c1", "customer1")];

    let profile = profiler.generate_profile("sp1", "Alice", &chats, &emails, &customers);
    assert_eq!(profile.salesperson_id, "sp1");
    assert_eq!(profile.name, "Alice");
    assert!(profile.writing_style.avg_message_length > 0);
    assert_eq!(profile.activity_stats.total_whatsapp_chats, 2);
    assert_eq!(profile.activity_stats.total_emails, 1);
    assert!(profile.activity_stats.activity_score > 0.0);

    // Stored and retrievable
    let stored = profiler.get_profile("sp1").unwrap();
    assert_eq!(stored.name, "Alice");
}

#[test]
fn test_writing_style_analysis() {
    let profiler = SalespersonProfiler::new();
    let messages = vec![
        "Hello, how are you?".to_string(),
        "This is a longer formal message with regards".to_string(),
        "hey lol 😂".to_string(),
    ];

    let style = profiler.analyze_writing_style(&messages);
    assert!(style.avg_message_length > 0);
    assert!(style.emoji_usage > 0.0);
    assert!(style.question_ratio > 0.0);
    assert!(!style.templates.is_empty());
}

#[test]
fn test_writing_style_empty() {
    let profiler = SalespersonProfiler::new();
    let style = profiler.analyze_writing_style(&[]);
    assert_eq!(style.avg_message_length, 0);
    assert_eq!(style.emoji_usage, 0.0);
    assert_eq!(style.question_ratio, 0.0);
    assert_eq!(style.formality_level, 0.5);
}

#[test]
fn test_language_detection() {
    let profiler = SalespersonProfiler::new();

    let en_msgs = vec![
        "Hello, how are you?".to_string(),
        "The order has been confirmed".to_string(),
    ];
    let pref = profiler.detect_language_preference(&en_msgs);
    assert_eq!(pref.primary_language, "en");

    let zh_msgs = vec![
        "你好，今天的订单怎么样？".to_string(),
        "请确认一下发货时间".to_string(),
    ];
    let pref = profiler.detect_language_preference(&zh_msgs);
    assert_eq!(pref.primary_language, "zh");
}

#[test]
fn test_language_detection_empty() {
    let profiler = SalespersonProfiler::new();
    let pref = profiler.detect_language_preference(&[]);
    assert_eq!(pref.primary_language, "unknown");
    assert_eq!(pref.multilingual_score, 0.0);
}

#[test]
fn test_language_detection_multilingual() {
    let profiler = SalespersonProfiler::new();
    let msgs = vec![
        "Hello, how are you?".to_string(),
        "你好，你好吗？".to_string(),
        "The order has been confirmed".to_string(),
        "请确认一下发货时间".to_string(),
    ];
    let pref = profiler.detect_language_preference(&msgs);
    assert!(pref.multilingual_score > 0.0);
    assert!(!pref.secondary_languages.is_empty());
}

#[test]
fn test_communication_pattern() {
    let profiler = SalespersonProfiler::new();
    let interactions = vec![
        Interaction {
            interaction_type: "whatsapp".to_string(),
            timestamp: 1000,
            customer_id: "c1".to_string(),
        },
        Interaction {
            interaction_type: "whatsapp".to_string(),
            timestamp: 2000,
            customer_id: "c2".to_string(),
        },
        Interaction {
            interaction_type: "email".to_string(),
            timestamp: 3000,
            customer_id: "c1".to_string(),
        },
    ];

    let pattern = profiler.model_communication_pattern(&interactions);
    assert_eq!(pattern.preferred_channel, "whatsapp");
    assert!(pattern.avg_interactions_per_customer > 1.0);
}

#[test]
fn test_communication_pattern_empty() {
    let profiler = SalespersonProfiler::new();
    let pattern = profiler.model_communication_pattern(&[]);
    assert_eq!(pattern.preferred_channel, "unknown");
    assert_eq!(pattern.follow_up_frequency, FollowUpFrequency::Never);
}

#[test]
fn test_profile_comparison() {
    let mut profiler = SalespersonProfiler::new();
    let chats1 = vec![make_chat("Hello!", "sp1", 1000)];
    let chats2 = vec![make_chat("Hey 😊", "sp2", 1000)];

    profiler.generate_profile("sp1", "Alice", &chats1, &[], &[]);
    profiler.generate_profile("sp2", "Bob", &chats2, &[], &[]);

    let comparison = profiler.compare_profiles("sp1", "sp2").unwrap();
    assert_eq!(comparison.salesperson1, "Alice");
    assert_eq!(comparison.salesperson2, "Bob");
    assert!(comparison.similarities.len() + comparison.differences.len() > 0);
    assert!(!comparison.recommendation.is_empty());
}

#[test]
fn test_profile_comparison_missing() {
    let profiler = SalespersonProfiler::new();
    assert!(profiler.compare_profiles("a", "b").is_none());
}

#[test]
fn test_insights_generation() {
    let profiler = SalespersonProfiler::new();
    let insights = profiler.generate_insights();
    assert!(insights.is_empty());
}

#[test]
fn test_insights_top_performer() {
    let mut profiler = SalespersonProfiler::new();

    let mut high_chats: Vec<ChatRecord> =
        (0..30).map(|i| make_chat("msg", "sp1", 1000 + i)).collect();
    high_chats.push(make_chat("hey", "customer", 2000));

    profiler.generate_profile(
        "sp1",
        "Alice",
        &high_chats,
        &[],
        &[make_customer("c1", "customer")],
    );

    let low_chats = vec![make_chat("hi", "sp2", 1000)];
    profiler.generate_profile("sp2", "Bob", &low_chats, &[], &[]);

    let insights = profiler.generate_insights();
    assert!(insights
        .iter()
        .any(|i| i.insight_type == InsightType::TopPerformer));
}

#[test]
fn test_insights_low_activity() {
    let mut profiler = SalespersonProfiler::new();
    let low_chats = vec![make_chat("hi", "sp1", 1000)];
    profiler.generate_profile("sp1", "Bob", &low_chats, &[], &[]);

    let insights = profiler.generate_insights();
    assert!(insights
        .iter()
        .any(|i| i.insight_type == InsightType::NeedsCoaching));
}

#[test]
fn test_top_customers_ranked() {
    let mut profiler = SalespersonProfiler::new();
    let chats = vec![
        make_chat("hi", "alice", 1000),
        make_chat("hi", "alice", 1100),
        make_chat("hi", "alice", 1200),
        make_chat("hi", "bob", 1300),
    ];
    let customers = vec![make_customer("c1", "alice"), make_customer("c2", "bob")];

    let profile = profiler.generate_profile("sp1", "Sales", &chats, &[], &customers);
    assert_eq!(profile.top_customers.len(), 2);
    assert_eq!(profile.top_customers[0].customer_name, "alice");
    assert_eq!(profile.top_customers[0].interaction_count, 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// WritingStyleAnalyzer tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_analyzer_new() {
    let analyzer = WritingStyleAnalyzer::new();
    assert!(!analyzer.templates.is_empty());
    assert!(!analyzer.sentiment_lexicon.is_empty());
}

#[test]
fn test_analyzer_default() {
    let analyzer = WritingStyleAnalyzer::default();
    assert!(!analyzer.templates.is_empty());
}

#[test]
fn test_email_analysis() {
    let analyzer = WritingStyleAnalyzer::new();
    let analysis = analyzer.analyze_email(
        "Meeting Update",
        "Dear Team, please find the updated schedule. Best regards, Alice",
    );
    assert!(analysis.formality_score > 0.5);
    assert!(analysis.politeness_score > 0.0);
    assert!(analysis.sentiment.overall >= -1.0);
    assert!(analysis.sentiment.overall <= 1.0);
    assert!(!analysis.readability.grade_level.is_empty());
}

#[test]
fn test_email_analysis_empty() {
    let analyzer = WritingStyleAnalyzer::new();
    let analysis = analyzer.analyze_email("", "");
    assert_eq!(analysis.avg_sentence_length, 0.0);
    assert_eq!(analysis.question_ratio, 0.0);
    assert_eq!(analysis.sentiment.neutral, 1.0);
}

#[test]
fn test_whatsapp_analysis() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "hey lol 😂".to_string(),
        "what's up?".to_string(),
        "haha that's funny".to_string(),
    ];
    let analysis = analyzer.analyze_whatsapp(&messages);
    assert!(analysis.formality_score < 0.5);
    assert!(analysis.emoji_usage > 0.0);
    assert!(analysis.question_ratio > 0.0);
}

#[test]
fn test_whatsapp_analysis_emoji_heavy() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "hello 😀😊".to_string(),
        "thanks 🙏".to_string(),
        "great work 👍".to_string(),
    ];
    let analysis = analyzer.analyze_whatsapp(&messages);
    assert!(analysis.emoji_usage > 0.5);
}

#[test]
fn test_template_extraction() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "just checking in on the status".to_string(),
        "just checking in on the project".to_string(),
        "hello how are you".to_string(),
    ];
    let templates = analyzer.extract_templates(&messages);
    assert!(templates.iter().any(|t| t.pattern.contains("checking in")));
}

#[test]
fn test_template_extraction_empty() {
    let analyzer = WritingStyleAnalyzer::new();
    let templates = analyzer.extract_templates(&[]);
    assert!(templates.is_empty());
}

#[test]
fn test_template_matching() {
    let analyzer = WritingStyleAnalyzer::new();
    let result = analyzer.match_template("best regards");
    assert!(result.is_some());
    assert_eq!(result.unwrap().category, TemplateCategory::Closing);
}

#[test]
fn test_template_matching_no_match() {
    let analyzer = WritingStyleAnalyzer::new();
    let result = analyzer.match_template("xyzxyz");
    assert!(result.is_none());
}

#[test]
fn test_sentiment_analysis_positive() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.analyze_sentiment("I love this great excellent wonderful");
    assert!(score.overall > 0.0);
    assert!(score.positive > score.negative);
}

#[test]
fn test_sentiment_analysis_negative() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.analyze_sentiment("I hate this terrible horrible worst");
    assert!(score.overall < 0.0);
    assert!(score.negative > score.positive);
}

#[test]
fn test_sentiment_analysis_neutral() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.analyze_sentiment("the report is on the table");
    assert!(score.neutral > 0.5);
}

#[test]
fn test_sentiment_analysis_empty() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.analyze_sentiment("");
    assert_eq!(score.neutral, 1.0);
    assert_eq!(score.overall, 0.0);
}

#[test]
fn test_readability_score() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.calculate_readability(
        "The quick brown fox jumps over the lazy dog. This is a simple sentence for testing.",
    );
    assert!(score.flesch_kincaid >= 0.0);
    assert!(!score.grade_level.is_empty());
    assert!(score.gunning_fog >= 0.0);
}

#[test]
fn test_readability_empty() {
    let analyzer = WritingStyleAnalyzer::new();
    let score = analyzer.calculate_readability("");
    assert!(score.flesch_kincaid >= 0.0);
    assert!(!score.grade_level.is_empty());
}

#[test]
fn test_greeting_extraction() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "Hello, how are you?".to_string(),
        "Hello, thanks for writing".to_string(),
        "Hi there!".to_string(),
    ];
    let greetings = analyzer.extract_greetings(&messages);
    assert!(!greetings.is_empty());
    assert!(greetings.iter().any(|g| g.contains("hello")));
}

#[test]
fn test_closing_extraction() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "Thanks for the update. Best regards,".to_string(),
        "Done. Best regards,".to_string(),
    ];
    let closings = analyzer.extract_closings(&messages);
    assert!(!closings.is_empty());
}

#[test]
fn test_style_comparison_similar() {
    let analyzer = WritingStyleAnalyzer::new();
    let s1 = analyzer.analyze_email("Hi", "Hello, thank you for your email. Best regards.");
    let s2 = analyzer.analyze_email("Hi", "Hello, thanks for writing. Best regards.");
    let comparison = analyzer.compare_styles(&s1, &s2);
    assert!(comparison.similarity_score > 0.5);
}

#[test]
fn test_style_comparison_different() {
    let analyzer = WritingStyleAnalyzer::new();
    let s1 = analyzer.analyze_email(
        "Formal",
        "Dear Sir, I am writing to formally request your assistance. Kind regards.",
    );
    let s2 = analyzer.analyze_whatsapp(&vec![
        "hey lol omg brb".to_string(),
        "gonna grab food".to_string(),
    ]);
    let comparison = analyzer.compare_styles(&s1, &s2);
    assert!(comparison.similarity_score < 0.8);
    assert!(!comparison.differences.is_empty());
}

#[test]
fn test_formality_high() {
    let analyzer = WritingStyleAnalyzer::new();
    let analysis = analyzer.analyze_email(
        "Request",
        "Dear Sir, I am writing to kindly request your assistance. Please advise at your earliest convenience. Regards.",
    );
    assert!(analysis.formality_score > 0.5);
}

#[test]
fn test_formality_low() {
    let analyzer = WritingStyleAnalyzer::new();
    let analysis = analyzer.analyze_whatsapp(&vec![
        "hey lol wanna hang out".to_string(),
        "omg brb gonna grab food".to_string(),
    ]);
    assert!(analysis.formality_score < 0.5);
}

#[test]
fn test_urgency_detection() {
    let analyzer = WritingStyleAnalyzer::new();
    let analysis = analyzer.analyze_email(
        "URGENT: Immediate Action Required",
        "This is critical and must be done immediately. Deadline is today.",
    );
    assert!(analysis.urgency_score > 0.0);
}

// ═══════════════════════════════════════════════════════════════════════════
// TradeIntelligence tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_intelligence_new() {
    let ti = TradeIntelligence::new();
    assert!(ti.stats().model_count == 0);
    assert_eq!(ti.stats().total_analyses, 0);
}

#[test]
fn test_intelligence_default() {
    let ti = TradeIntelligence::default();
    assert!(ti.stats().model_count == 0);
}

#[test]
fn test_intent_analysis() {
    let ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    let intent = ti.analyze_intent(&input);
    assert_eq!(intent.primary_intent, IntentType::BulkOrder);
    assert!(intent.budget_range.is_some());
    assert!(intent.decision_timeline.is_some());
}

#[test]
fn test_intent_analysis_empty_history() {
    let ti = TradeIntelligence::new();
    let mut input = make_intelligence_input();
    input.interaction_history = vec![];
    let intent = ti.analyze_intent(&input);
    assert_eq!(intent.primary_intent, IntentType::Inquiry);
}

#[test]
fn test_intent_analysis_complaint() {
    let ti = TradeIntelligence::new();
    let mut input = make_intelligence_input();
    input.interaction_history = vec![InteractionEvent {
        event_type: "email".to_string(),
        timestamp: now_ts(),
        content: "I want to file a complaint about the defective batch".to_string(),
        sentiment: -0.9,
    }];
    let intent = ti.analyze_intent(&input);
    assert_eq!(intent.primary_intent, IntentType::Complaint);
}

#[test]
fn test_opportunity_assessment() {
    let ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    let opp = ti.assess_opportunity(&input);
    assert!(opp.score > 50.0, "score should be > 50, got {}", opp.score);
    assert!(opp.estimated_value > 0.0);
    assert!(opp.conversion_probability > 0.0 && opp.conversion_probability <= 1.0);
    assert!(!opp.factors.is_empty());
    assert!(opp.time_to_close.is_some());
}

#[test]
fn test_opportunity_assessment_low_value() {
    let ti = TradeIntelligence::new();
    let mut input = make_intelligence_input();
    input.customer_data.total_value = 100.0;
    input.customer_data.grade = "C".to_string();
    input.customer_data.interaction_count = 1;
    let opp = ti.assess_opportunity(&input);
    assert!(opp.score < 70.0);
}

#[test]
fn test_risk_detection() {
    let ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    let risks = ti.detect_risks(&input);
    // High-value customer should trigger payment risk
    let payment_risks: Vec<_> = risks
        .iter()
        .filter(|r| r.risk_type == RiskType::Payment)
        .collect();
    assert!(!payment_risks.is_empty());
}

#[test]
fn test_risk_detection_negative_sentiment() {
    let ti = TradeIntelligence::new();
    let mut input = make_intelligence_input();
    input.interaction_history = vec![
        InteractionEvent {
            event_type: "email".to_string(),
            timestamp: now_ts() - 86400,
            content: "Very disappointed with quality".to_string(),
            sentiment: -0.8,
        },
        InteractionEvent {
            event_type: "email".to_string(),
            timestamp: now_ts() - 3600,
            content: "Still waiting for response".to_string(),
            sentiment: -0.5,
        },
    ];
    let risks = ti.detect_risks(&input);
    let rep_risks: Vec<_> = risks
        .iter()
        .filter(|r| r.risk_type == RiskType::Reputation)
        .collect();
    assert!(!rep_risks.is_empty());
}

#[test]
fn test_risk_detection_no_risks_low_value() {
    let ti = TradeIntelligence::new();
    let mut input = make_intelligence_input();
    input.customer_data.total_value = 1000.0;
    input.market_context.region = "Europe".to_string();
    input.market_context.exchange_rate = 0.01;
    input.interaction_history = vec![InteractionEvent {
        event_type: "email".to_string(),
        timestamp: now_ts(),
        content: "Hello".to_string(),
        sentiment: 0.5,
    }];
    let risks = ti.detect_risks(&input);
    assert!(risks.is_empty() || risks.iter().all(|r| r.severity == RiskSeverity::Low));
}

#[test]
fn test_recommendation_generation() {
    let ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    let output = ti.analyze(&input);
    assert!(
        !output.recommendations.is_empty(),
        "should produce recommendations"
    );
}

#[test]
fn test_recommendation_bulk_order() {
    let ti = TradeIntelligence::new();
    let mut output = IntelligenceOutput {
        intent: CustomerIntent {
            primary_intent: IntentType::BulkOrder,
            secondary_intents: vec![],
            urgency: UrgencyLevel::Medium,
            budget_range: None,
            decision_timeline: None,
        },
        opportunity: OpportunityAssessment {
            score: 50.0,
            stage: OpportunityStage::Lead,
            estimated_value: 0.0,
            conversion_probability: 0.5,
            time_to_close: None,
            factors: vec![],
        },
        risks: vec![],
        recommendations: vec![],
        confidence: 0.5,
    };
    output.recommendations = ti.generate_recommendations(&output);
    assert!(output
        .recommendations
        .iter()
        .any(|r| r.action.contains("volume discount")));
}

#[test]
fn test_recommendation_complaint() {
    let ti = TradeIntelligence::new();
    let output = IntelligenceOutput {
        intent: CustomerIntent {
            primary_intent: IntentType::Complaint,
            secondary_intents: vec![],
            urgency: UrgencyLevel::Immediate,
            budget_range: None,
            decision_timeline: None,
        },
        opportunity: OpportunityAssessment {
            score: 50.0,
            stage: OpportunityStage::Lead,
            estimated_value: 0.0,
            conversion_probability: 0.5,
            time_to_close: None,
            factors: vec![],
        },
        risks: vec![],
        recommendations: vec![],
        confidence: 0.5,
    };
    let recs = ti.generate_recommendations(&output);
    assert!(recs.iter().any(|r| r.priority == Priority::Urgent));
}

#[test]
fn test_full_analysis() {
    let mut ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    let output = ti.analyze(&input);
    assert!(output.confidence > 0.0);
    assert!(!output.recommendations.is_empty());
    assert!(!output.risks.is_empty());

    let stats = ti.stats();
    assert_eq!(stats.total_analyses, 1);
    assert!(stats.avg_confidence > 0.0);
}

#[test]
fn test_full_analysis_multiple_runs() {
    let mut ti = TradeIntelligence::new();
    let input = make_intelligence_input();
    ti.analyze(&input);
    ti.analyze(&input);
    let stats = ti.stats();
    assert_eq!(stats.total_analyses, 2);
}

#[test]
fn test_update_thresholds() {
    let mut ti = TradeIntelligence::new();
    let new_th = RiskThresholds {
        payment_risk: 0.9,
        delivery_risk: 0.8,
        compliance_risk: 0.9,
        market_risk: 0.8,
    };
    ti.update_thresholds(new_th);
    // After raising thresholds, fewer risks should fire
    let input = make_intelligence_input();
    let risks = ti.detect_risks(&input);
    // payment score is 0.8 (250k > 100k), threshold is now 0.9 → no payment risk
    let payment_risks: Vec<_> = risks
        .iter()
        .filter(|r| r.risk_type == RiskType::Payment)
        .collect();
    assert!(payment_risks.is_empty());
}

#[test]
fn test_risk_type_str() {
    let alert = RiskAlert {
        risk_type: RiskType::Payment,
        severity: RiskSeverity::High,
        description: "test".to_string(),
        mitigation: "test".to_string(),
        probability: 0.8,
    };
    assert_eq!(alert.risk_type_str(), "Payment");

    let alert = RiskAlert {
        risk_type: RiskType::Delivery,
        severity: RiskSeverity::Medium,
        description: "test".to_string(),
        mitigation: "test".to_string(),
        probability: 0.5,
    };
    assert_eq!(alert.risk_type_str(), "Delivery");
}

// ═══════════════════════════════════════════════════════════════════════════
// SalesCoach tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_coach_new() {
    let coach = SalesCoach::new();
    let stats = coach.stats();
    assert!(stats.total_scripts > 0);
    assert!(stats.total_strategies > 0);
}

#[test]
fn test_coach_default() {
    let coach = SalesCoach::default();
    assert!(coach.stats().total_scripts > 0);
}

#[test]
fn test_script_recommendation() {
    let coach = SalesCoach::new();
    let scripts = coach.recommend_scripts("new prospect call", &CustomerStage::NewLead);
    assert!(!scripts.is_empty());
    assert!(scripts
        .iter()
        .all(|s| s.effectiveness_score > 0.3));
}

#[test]
fn test_script_recommendation_empty_for_wrong_stage() {
    let coach = SalesCoach::new();
    let scripts = coach.recommend_scripts("closing the deal", &CustomerStage::ClosedWon);
    assert!(scripts.is_empty());
}

#[test]
fn test_script_recommendation_negotiating() {
    let coach = SalesCoach::new();
    let scripts = coach.recommend_scripts(
        "finalize the contract terms",
        &CustomerStage::Negotiating,
    );
    assert!(!scripts.is_empty());
    assert!(scripts
        .iter()
        .any(|s| s.category == ScriptCategory::Closing));
}

#[test]
fn test_strategy_generation() {
    let coach = SalesCoach::new();
    let strat = coach.generate_strategy(&CustomerStage::NewLead, &CommunicationChannel::Email);
    assert_eq!(strat.customer_stage, CustomerStage::NewLead);
    assert_eq!(strat.channel, CommunicationChannel::Email);
    assert!(strat.max_attempts > 0);
}

#[test]
fn test_strategy_generation_default_fallback() {
    let coach = SalesCoach::new();
    let strat = coach.generate_strategy(&CustomerStage::Qualified, &CommunicationChannel::Social);
    // No pre-loaded strategy for Qualified+Social → default
    assert_eq!(strat.customer_stage, CustomerStage::Qualified);
    assert_eq!(strat.channel, CommunicationChannel::Social);
    assert!(strat.max_attempts > 0);
}

#[test]
fn test_objection_handling_expensive() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("It's too expensive");
    assert!(handler.confidence > 0.5);
    assert!(!handler.alternative_responses.is_empty());
}

#[test]
fn test_objection_handling_timing() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("not the right time to buy");
    assert!(handler.confidence > 0.5);
    assert!(!handler.response.is_empty());
}

#[test]
fn test_objection_handling_think_about() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("I need to think about it");
    assert!(handler.confidence > 0.5);
}

#[test]
fn test_objection_handling_existing_solution() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("we already have a solution");
    assert!(handler.confidence > 0.5);
}

#[test]
fn test_objection_handling_budget() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("budget constraints this quarter");
    assert!(handler.confidence > 0.5);
}

#[test]
fn test_objection_handling_unknown() {
    let coach = SalesCoach::new();
    let handler = coach.handle_objection("xyzzy unknown objection xyzzy");
    assert!(handler.confidence <= 0.5);
    assert!(!handler.alternative_responses.is_empty());
}

#[test]
fn test_performance_prediction_no_history() {
    let coach = SalesCoach::new();
    let pred = coach.predict_performance("sp_unknown", &[]);
    assert!(pred.conversion_probability > 0.0);
    assert_eq!(pred.factors.len(), 1);
    assert!(pred.estimated_value == 0.0);
}

#[test]
fn test_performance_prediction_with_history() {
    let coach = SalesCoach::new();
    let history = vec![
        PerformanceRecord {
            date: "2026-01".to_string(),
            salesperson_id: "sp_001".to_string(),
            metric: "revenue".to_string(),
            value: 120.0,
            target: 100.0,
        },
        PerformanceRecord {
            date: "2026-02".to_string(),
            salesperson_id: "sp_001".to_string(),
            metric: "revenue".to_string(),
            value: 80.0,
            target: 100.0,
        },
    ];
    let pred = coach.predict_performance("sp_001", &history);
    assert!(pred.conversion_probability > 0.0 && pred.conversion_probability <= 1.0);
    assert!(pred.estimated_value > 0.0);
    assert_eq!(pred.factors.len(), 2);
}

#[test]
fn test_performance_prediction_high_attainment() {
    let coach = SalesCoach::new();
    let history = vec![
        PerformanceRecord {
            date: "2026-01".to_string(),
            salesperson_id: "sp_002".to_string(),
            metric: "revenue".to_string(),
            value: 150.0,
            target: 100.0,
        },
        PerformanceRecord {
            date: "2026-02".to_string(),
            salesperson_id: "sp_002".to_string(),
            metric: "revenue".to_string(),
            value: 200.0,
            target: 100.0,
        },
    ];
    let pred = coach.predict_performance("sp_002", &history);
    assert!(pred.conversion_probability > 0.5);
}

#[test]
fn test_talking_points_demo() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("schedule a demo for the team");
    assert!(points.len() >= 3);
    assert!(points.iter().any(|p| p.contains("feature") || p.contains("ROI")));
}

#[test]
fn test_talking_points_objection() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("handle their objection about pricing");
    assert!(!points.is_empty());
    assert!(points.iter().any(|p| p.contains("Listen") || p.contains("Acknowledge")));
}

#[test]
fn test_talking_points_closing() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("ready for closing the deal");
    assert!(!points.is_empty());
    assert!(points.iter().any(|p| p.contains("Summarize") || p.contains("urgency")));
}

#[test]
fn test_talking_points_first_call() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("first call with intro");
    assert!(points.len() >= 3);
}

#[test]
fn test_talking_points_proposal() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("send proposal and pricing");
    assert!(!points.is_empty());
    assert!(points.iter().any(|p| p.contains("value") || p.contains("ROI")));
}

#[test]
fn test_talking_points_generic() {
    let coach = SalesCoach::new();
    let points = coach.generate_talking_points("just chatting about weather");
    assert!(!points.is_empty());
    assert!(points.iter().any(|p| p.contains("pain point")));
}

#[test]
fn test_action_items() {
    let coach = SalesCoach::new();
    let response = coach.coach(&make_coaching_request("first call with new prospect", 1));
    let items = coach.generate_action_items(&response);
    assert!(!items.is_empty());
    assert!(items.iter().any(|i| i.action.contains("script")));
    assert!(items.iter().any(|i| i.action.contains("Follow up")));
}

#[test]
fn test_action_items_low_performance() {
    let coach = SalesCoach::new();
    let response = CoachingResponse {
        recommended_scripts: vec![],
        follow_up_strategy: FollowUpStrategy {
            id: "str_test".to_string(),
            name: "Test".to_string(),
            customer_stage: CustomerStage::NewLead,
            channel: CommunicationChannel::Email,
            frequency: FollowUpFrequency::Daily,
            max_attempts: 5,
            templates: vec![],
            success_rate: 0.4,
        },
        talking_points: vec![],
        objection_handlers: vec![],
        performance_prediction: PerformancePrediction {
            conversion_probability: 0.2,
            estimated_close_date: None,
            estimated_value: 0.0,
            factors: vec![],
        },
        action_items: vec![],
    };
    let items = coach.generate_action_items(&response);
    assert!(items
        .iter()
        .any(|i| i.priority == Priority::Urgent));
}

#[test]
fn test_update_script_effectiveness_success() {
    let mut coach = SalesCoach::new();
    let before = coach.stats().avg_effectiveness;
    coach.update_script_effectiveness("scr_001", true);
    let after = coach.stats().avg_effectiveness;
    assert!(after > before);
}

#[test]
fn test_update_script_effectiveness_failure() {
    let mut coach = SalesCoach::new();
    let before = coach.stats().avg_effectiveness;
    coach.update_script_effectiveness("scr_001", false);
    let after = coach.stats().avg_effectiveness;
    assert!(after < before);
}

#[test]
fn test_coach_stats() {
    let coach = SalesCoach::new();
    let stats = coach.stats();
    assert!(stats.total_scripts > 0);
    assert!(stats.total_strategies > 0);
    assert!(stats.avg_effectiveness > 0.0);
    assert!(!stats.top_performing_scripts.is_empty());
}

#[test]
fn test_coaching_flow() {
    let coach = SalesCoach::new();
    let request = make_coaching_request("demo call with potential client", 2);
    let response = coach.coach(&request);
    assert!(!response.recommended_scripts.is_empty());
    assert!(!response.talking_points.is_empty());
    assert!(response.performance_prediction.conversion_probability > 0.0);
    assert!(!response.action_items.is_empty());
    assert!(!response.follow_up_strategy.templates.is_empty());
}

#[test]
fn test_coaching_flow_empty_history() {
    let coach = SalesCoach::new();
    let request = make_coaching_request("new prospect outreach", 0);
    let response = coach.coach(&request);
    assert!(!response.recommended_scripts.is_empty());
    assert!(!response.talking_points.is_empty());
}

#[test]
fn test_coaching_flow_with_objections() {
    let coach = SalesCoach::new();
    let mut request = make_coaching_request("negotiation call", 3);
    request.history.push(InteractionHistory {
        channel: "email".to_string(),
        timestamp: 100,
        direction: Direction::Inbound,
        content: "It's too expensive for our budget".to_string(),
        outcome: None,
    });
    let response = coach.coach(&request);
    assert!(!response.objection_handlers.is_empty());
    assert!(response
        .objection_handlers
        .iter()
        .any(|h| h.confidence > 0.5));
}

#[test]
fn test_follow_up_frequency_deadline() {
    assert_eq!(FollowUpFrequency::Daily.deadline_str(), "tomorrow");
    assert_eq!(FollowUpFrequency::Weekly.deadline_str(), "within_1_week");
    assert_eq!(FollowUpFrequency::BiWeekly.deadline_str(), "within_2_weeks");
    assert_eq!(FollowUpFrequency::Monthly.deadline_str(), "within_1_month");
    assert_eq!(
        FollowUpFrequency::Quarterly.deadline_str(),
        "within_3_months"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-module integration tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_profiler_to_intelligence_pipeline() {
    let mut profiler = SalespersonProfiler::new();
    let chats = vec![
        make_chat("Need bulk order pricing", "customer1", 1000),
        make_chat("What's the best price?", "customer1", 2000),
    ];
    let customers = vec![make_customer("c1", "customer1")];

    let profile = profiler.generate_profile("sp1", "Alice", &chats, &[], &customers);
    assert!(profile.activity_stats.activity_score > 0.0);

    let ti = TradeIntelligence::new();
    let input = IntelligenceInput {
        customer_data: CustomerData {
            customer_id: "c1".to_string(),
            name: "customer1".to_string(),
            country: "US".to_string(),
            industry: "Manufacturing".to_string(),
            grade: "B".to_string(),
            source: "WhatsApp".to_string(),
            total_value: 50_000.0,
            interaction_count: profile.activity_stats.total_whatsapp_chats as u32,
        },
        interaction_history: vec![InteractionEvent {
            event_type: "whatsapp".to_string(),
            timestamp: now_ts() - 86400,
            content: "Need bulk order pricing for 100 units".to_string(),
            sentiment: 0.5,
        }],
        market_context: MarketContext {
            region: "North America".to_string(),
            industry_trend: 0.3,
            competitor_activity: 0.3,
            exchange_rate: 0.02,
            seasonality: Seasonality::Normal,
        },
        company_policies: vec![],
    };

    let output = ti.analyze(&input);
    assert_eq!(output.intent.primary_intent, IntentType::BulkOrder);
    assert!(output.confidence > 0.0);
}

#[test]
fn test_writing_style_to_coaching_pipeline() {
    let analyzer = WritingStyleAnalyzer::new();
    let messages = vec![
        "Dear Sir, I am writing to request a quotation".to_string(),
        "Kindly provide the pricing details".to_string(),
    ];
    let analysis = analyzer.analyze_whatsapp(&messages);
    assert!(analysis.formality_score > 0.5);

    let coach = SalesCoach::new();
    let situation = if analysis.formality_score > 0.5 {
        "formal proposal presentation"
    } else {
        "casual follow-up"
    };
    let scripts = coach.recommend_scripts(situation, &CustomerStage::ProposalSent);
    assert!(!scripts.is_empty());
}
