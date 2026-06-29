#[cfg(feature = "integration_tests")]
mod consciousness_e2e {
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
    async fn consciousness_creation() {
        check_env();
        let ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        assert_eq!(ci.cycle, 0, "consciousness cycle should start at 0");
    }

    #[tokio::test]
    async fn consciousness_single_cycle() {
        check_env();
        let mut ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        let result = ci.handle_consciousness_batch_async().await;
        assert_eq!(ci.cycle, 1, "cycle should be 1 after first batch");
        assert!(!result.is_empty(), "cycle result should not be empty");
    }

    #[tokio::test]
    async fn consciousness_three_cycles_no_panic() {
        check_env();
        let mut ci = neotrix::neotrix::nt_mind_background_loop::ConsciousnessIntegration::new();
        for expected in 1..=3u64 {
            let result = ci.handle_consciousness_batch_async().await;
            assert_eq!(ci.cycle, expected, "cycle mismatch at count {}", expected);
            assert!(
                !result.is_empty(),
                "cycle {} produced empty result",
                expected
            );
        }
    }
}
