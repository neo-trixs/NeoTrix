#[cfg(feature = "integration_tests")]
mod basic_cycle_e2e {
    use std::sync::Once;

    fn check_env() {
        static CHECKED: Once = Once::new();
        CHECKED.call_once(|| {
            if std::env::var("NEOTRIX_TEST").is_err() {
                panic!(
                    "E2E tests require NEOTRIX_TEST env var.\n\
                     Set it via: NEOTRIX_TEST=1 cargo test --features integration_tests -p neotrix"
                );
            }
        });
    }

    #[tokio::test]
    async fn ci_creation_with_auto_config() {
        check_env();
        let ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        assert_eq!(ci.cycle, 0, "cycle should start at 0");
        assert!(
            ci.text_buffer.is_empty(),
            "text_buffer should be empty initially"
        );
        assert_eq!(ci.text_feed_count, 0, "feed_count should start at 0");
    }

    #[tokio::test]
    async fn ci_single_cycle_no_panic() {
        check_env();
        let mut ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        let result = ci.handle_consciousness_batch_async().await;
        assert_eq!(ci.cycle, 1, "cycle should be 1 after first batch");
        assert!(!result.is_empty(), "cycle result should not be empty");
    }

    #[tokio::test]
    async fn ci_text_buffer_operations() {
        check_env();
        let mut ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        assert!(ci.text_buffer.is_empty());

        ci.feed_consciousness_text("hello from e2e test");
        assert_eq!(ci.text_buffer.len(), 1);
        assert_eq!(ci.text_feed_count, 1);

        ci.feed_consciousness_text("second message");
        assert_eq!(ci.text_buffer.len(), 2);
        assert_eq!(ci.text_feed_count, 2);

        // Run one cycle — text buffer should be consumed
        let _result = ci.handle_consciousness_batch_async().await;
        // After batch, text buffer is drained during processing
        assert!(
            ci.text_buffer.is_empty(),
            "text_buffer should be drained after batch"
        );
    }

    #[tokio::test]
    async fn ci_negentropy_computation() {
        check_env();
        use neotrix::core::nt_core_negentropy::{NegentropyComponents, NegentropyFlux};

        let mut metric = neotrix::core::nt_core_negentropy::NegentropyMetric::default();
        assert_eq!(metric.total, 0.0, "initial negentropy should be 0");

        let components = NegentropyComponents {
            phi: 0.5,
            vsa_coherence: 0.3,
            kb_order: 0.8,
            prediction_acc: 0.6,
            attention_focus: 0.4,
            strategy_diff: 0.2,
            temporal_coherence: 0.7,
        };
        let flux = NegentropyFlux {
            import_rate: 0.1,
            export_rate: 0.2,
            net_flux: 0.3,
            efficiency: 0.8,
            operational_cost: 0.4,
        };

        metric.record(components.clone(), flux.clone());
        assert!(
            metric.total > 0.0,
            "negentropy should be positive after record"
        );
        assert_eq!(metric.history.len(), 1, "history should have 1 entry");

        // Second record builds trend
        metric.record(components, flux);
        assert_eq!(metric.history.len(), 2, "history should have 2 entries");
        assert!(metric.total > 0.0);
    }
}
