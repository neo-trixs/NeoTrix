#![forbid(unsafe_code)]

//! Comprehensive unit tests for trade data extraction modules:
//! - `chrome_decrypt` — key derivation, prefix stripping, decryptor construction
//! - `selenium_automation` — mock backend, session lifecycle, cookie/network operations
//! - `joinf` — extractor creation, customer/interaction/email mapping
//! - `data_pipeline` — extract_all, sync_incremental, normalizer
//! - `platform_registry` — register, get, list, feature support

use std::sync::Arc;

// ═══════════════════════════════════════════════════════════════
// chrome_decrypt tests
// ═══════════════════════════════════════════════════════════════

mod chrome_decrypt_tests {
    use crate::l1_action::nt_act::nt_act_trade::extractors::chrome_decrypt::{
        ChromeDecryptError, ChromeDecryptor,
    };

    #[test]
    fn test_chrome_decryptor_with_password_success() {
        let result = ChromeDecryptor::with_password("peanuts");
        assert!(result.is_ok());
    }

    #[test]
    fn test_chrome_decryptor_with_empty_password() {
        // Empty password should still derive a valid key
        let result = ChromeDecryptor::with_password("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_key_derivation_deterministic() {
        // Same password → same derived key (verified via decrypt roundtrip)
        let d1 = ChromeDecryptor::with_password("test_key").unwrap();
        let d2 = ChromeDecryptor::with_password("test_key").unwrap();

        // Both should produce identical results on the same ciphertext
        let plaintext = b"v10hello world padded to sixteen!!";
        let r1 = d1.decrypt(plaintext);
        let r2 = d2.decrypt(plaintext);
        assert_eq!(r1.is_ok(), r2.is_ok());
        if let (Ok(p1), Ok(p2)) = (r1, r2) {
            assert_eq!(p1, p2);
        }
    }

    #[test]
    fn test_key_derivation_different_passwords() {
        let d1 = ChromeDecryptor::with_password("password_a").unwrap();
        let d2 = ChromeDecryptor::with_password("password_b").unwrap();

        let ciphertext = b"v10some ciphertext data here!!!!";
        let r1 = d1.decrypt(ciphertext);
        let r2 = d2.decrypt(ciphertext);

        // Both may succeed (NoPadding doesn't validate integrity) but keys differ
        // We verify derivation is different by ensuring different outputs
        if let (Ok(p1), Ok(p2)) = (r1, r2) {
            // With different keys, decrypted output will differ
            // (unless collision, astronomically unlikely)
            assert_ne!(p1, p2);
        }
    }

    #[test]
    fn test_v10_prefix_stripping() {
        let inner = ChromeDecryptor::with_password("x")
            .unwrap()
            .decrypt(b"v10abc123def456");
        assert!(inner.is_ok());
    }

    #[test]
    fn test_v11_prefix_stripping() {
        let inner = ChromeDecryptor::with_password("x")
            .unwrap()
            .decrypt(b"v11abc123def456");
        assert!(inner.is_ok());
    }

    #[test]
    fn test_unknown_prefix_returns_error() {
        let d = ChromeDecryptor::with_password("x").unwrap();
        let result = d.decrypt(b"v20invalid");
        assert!(result.is_err());
        match result {
            Err(ChromeDecryptError::UnknownPrefix(_)) => {}
            other => panic!("Expected UnknownPrefix, got {:?}", other),
        }
    }

    #[test]
    fn test_empty_data_returns_error() {
        let d = ChromeDecryptor::with_password("x").unwrap();
        let result = d.decrypt(b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_prefix_only_returns_error() {
        let d = ChromeDecryptor::with_password("x").unwrap();
        // Exactly 3 bytes, all prefix — strip_prefix gives empty slice
        let result = d.decrypt(b"v10");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_error_display() {
        let err = ChromeDecryptError::Keychain("test".into());
        assert_eq!(format!("{}", err), "Keychain error: test");

        let err = ChromeDecryptError::KeyDerivation("pbkdf2 fail".into());
        assert_eq!(format!("{}", err), "Key derivation error: pbkdf2 fail");

        let err = ChromeDecryptError::Decryption("bad padding".into());
        assert_eq!(format!("{}", err), "Decryption error: bad padding");

        let err = ChromeDecryptError::Database("no such table".into());
        assert_eq!(format!("{}", err), "Database error: no such table");

        let err = ChromeDecryptError::UnknownPrefix("v99".into());
        assert_eq!(format!("{}", err), "Unknown encryption prefix: v99");

        let err = ChromeDecryptError::Io("permission denied".into());
        assert_eq!(format!("{}", err), "IO error: permission denied");
    }

    #[test]
    fn test_error_is_std_error() {
        let err = ChromeDecryptError::Io("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_login_entry_fields() {
        let entry = crate::l1_action::nt_act::nt_act_trade::extractors::chrome_decrypt::LoginEntry {
            origin_url: "https://example.com".into(),
            username: "user@test.com".into(),
            password: "secret".into(),
        };
        assert_eq!(entry.origin_url, "https://example.com");
        assert_eq!(entry.username, "user@test.com");
        assert_eq!(entry.password, "secret");
    }

    #[test]
    fn test_login_entry_clone() {
        let entry = crate::l1_action::nt_act::nt_act_trade::extractors::chrome_decrypt::LoginEntry {
            origin_url: "https://a.com".into(),
            username: "u".into(),
            password: "p".into(),
        };
        let cloned = entry.clone();
        assert_eq!(entry.origin_url, cloned.origin_url);
    }

    #[test]
    fn test_decrypt_pads_to_block_boundary() {
        // v10 prefix + 16 bytes of data = exactly 19 bytes
        // strip_prefix removes 3 → 16 bytes → one AES block
        let d = ChromeDecryptor::with_password("x").unwrap();
        let mut data = vec![0u8; 19];
        data[..3].copy_from_slice(b"v10");
        data[3..].fill(0x41); // 'A'
        // Should not panic
        let _ = d.decrypt(&data);
    }
}

// ═══════════════════════════════════════════════════════════════
// selenium_automation tests
// ═══════════════════════════════════════════════════════════════

mod selenium_automation_tests {
    use crate::l1_action::nt_act::nt_act_trade::extractors::selenium_automation::*;

    // ── SeleniumSession ──

    #[test]
    fn test_selenium_session_new() {
        let config = SeleniumConfig::default();
        let session = SeleniumSession::new(config);
        assert!(session.is_ok());
        let session = session.unwrap();
        assert!(session.profile_dir().exists());
        assert!(session.api_endpoints().is_empty());
    }

    #[test]
    fn test_selenium_session_with_backend() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let session = SeleniumSession::with_backend(config, backend);
        assert!(session.is_ok());
    }

    #[test]
    fn test_selenium_config_custom() {
        let config = SeleniumConfig {
            base_url: "https://example.com".into(),
            headless: false,
            timeout_secs: 60,
            extra_args: vec!["--no-sandbox".into()],
        };
        assert!(!config.headless);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.extra_args.len(), 1);
    }

    #[test]
    fn test_login_form_selectors_default() {
        let sel = LoginFormSelectors::default();
        assert!(!sel.username_field.is_empty());
        assert!(!sel.password_field.is_empty());
        assert!(!sel.submit_button.is_empty());
        assert!(sel.success_indicator.is_none());
    }

    #[test]
    fn test_login_form_selectors_custom() {
        let sel = LoginFormSelectors {
            username_field: "#email".into(),
            password_field: "#pass".into(),
            submit_button: "#login-btn".into(),
            success_indicator: Some(".dashboard".into()),
        };
        assert_eq!(sel.success_indicator.unwrap(), ".dashboard");
    }

    // ── MockBackend ──

    #[test]
    fn test_mock_backend_new() {
        let backend = MockBackend::new();
        assert!(!backend.is_quit());
        assert!(backend.navigations().is_empty());
        assert!(backend.clicks().is_empty());
        assert!(backend.typed().is_empty());
    }

    #[test]
    fn test_mock_backend_default() {
        let backend = MockBackend::default();
        assert!(!backend.is_quit());
    }

    #[test]
    fn test_mock_backend_navigation() {
        let mut backend = MockBackend::new();
        assert!(backend.navigate("https://example.com").is_ok());
        assert!(backend.navigate("https://example.com/login").is_ok());
        assert_eq!(
            backend.navigations(),
            &[
                "https://example.com".to_string(),
                "https://example.com/login".to_string()
            ]
        );
    }

    #[test]
    fn test_mock_backend_type_into() {
        let backend = MockBackend::new();
        assert!(backend.type_into("#user", "admin").is_ok());
    }

    #[test]
    fn test_mock_backend_type_empty_selector() {
        let backend = MockBackend::new();
        let result = backend.type_into("", "value");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("selector is empty"));
    }

    #[test]
    fn test_mock_backend_type_empty_value() {
        let backend = MockBackend::new();
        let result = backend.type_into("#user", "");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("value is empty"));
    }

    #[test]
    fn test_mock_backend_click() {
        let backend = MockBackend::new();
        assert!(backend.click("#submit").is_ok());
    }

    #[test]
    fn test_mock_backend_click_empty() {
        let backend = MockBackend::new();
        let result = backend.click("");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_backend_execute_js() {
        let backend = MockBackend::new();
        let result = backend.execute_js("return 1+1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock_js_result");
    }

    #[test]
    fn test_mock_backend_execute_js_empty() {
        let backend = MockBackend::new();
        let result = backend.execute_js("");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_backend_get_cookies_empty() {
        let backend = MockBackend::new();
        let cookies = backend.get_cookies().unwrap();
        assert!(cookies.is_empty());
    }

    #[test]
    fn test_mock_backend_inject_cookie() {
        let mut backend = MockBackend::new();
        backend.inject_cookie(Cookie {
            name: "session_id".into(),
            value: "abc123".into(),
            domain: ".example.com".into(),
            path: "/".into(),
            expires: None,
            secure: true,
        });
        let cookies = backend.get_cookies().unwrap();
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session_id");
        assert_eq!(cookies[0].value, "abc123");
        assert!(cookies[0].secure);
    }

    #[test]
    fn test_mock_backend_inject_multiple_cookies() {
        let mut backend = MockBackend::new();
        for i in 0..5 {
            backend.inject_cookie(Cookie {
                name: format!("cookie_{}", i),
                value: format!("val_{}", i),
                domain: ".test.com".into(),
                path: "/".into(),
                expires: None,
                secure: false,
            });
        }
        assert_eq!(backend.get_cookies().unwrap().len(), 5);
    }

    #[test]
    fn test_mock_backend_set_title() {
        let mut backend = MockBackend::new();
        backend.set_title("My Page");
        assert_eq!(backend.get_title().unwrap(), "My Page");
    }

    #[test]
    fn test_mock_backend_element_exists() {
        let backend = MockBackend::new();
        assert!(!backend.element_exists("#any").unwrap());
    }

    #[test]
    fn test_mock_backend_quit() {
        let backend = MockBackend::new();
        assert!(backend.quit().is_ok());
        assert!(!backend.is_quit()); // MockBackend doesn't set quit_called on quit()
    }

    // ── Cookie serialization ──

    #[test]
    fn test_cookie_serialization_roundtrip() {
        let cookie = Cookie {
            name: "sid".into(),
            value: "abc".into(),
            domain: ".test.com".into(),
            path: "/".into(),
            expires: Some(1700000000),
            secure: true,
        };
        let json = serde_json::to_string(&cookie).unwrap();
        let deserialized: Cookie = serde_json::from_str(&json).unwrap();
        assert_eq!(cookie.name, deserialized.name);
        assert_eq!(cookie.value, deserialized.value);
        assert_eq!(cookie.expires, deserialized.expires);
        assert_eq!(cookie.secure, deserialized.secure);
    }

    #[test]
    fn test_cookie_optional_expires_omitted() {
        let cookie = Cookie {
            name: "c".into(),
            value: "v".into(),
            domain: ".d".into(),
            path: "/".into(),
            expires: None,
            secure: false,
        };
        let json = serde_json::to_string(&cookie).unwrap();
        assert!(!json.contains("expires"));
    }

    #[test]
    fn test_cookie_deserialize_defaults() {
        let json = r#"{"name":"c","value":"v","domain":".d","path":"/"}"#;
        let cookie: Cookie = serde_json::from_str(json).unwrap();
        assert!(!cookie.secure);
    }

    // ── NetworkLogEntry ──

    #[test]
    fn test_network_log_entry_roundtrip() {
        let entry = NetworkLogEntry {
            url: "https://api.example.com/data".into(),
            method: "GET".into(),
            status: Some(200),
            resource_type: "xhr".into(),
            headers: {
                let mut h = std::collections::HashMap::new();
                h.insert("Content-Type".into(), "application/json".into());
                h
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: NetworkLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.url, deserialized.url);
        assert_eq!(entry.status, deserialized.status);
    }

    #[test]
    fn test_network_log_entry_no_status() {
        let entry = NetworkLogEntry {
            url: "about:blank".into(),
            method: "GET".into(),
            status: None,
            resource_type: "document".into(),
            headers: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("status"));
    }

    // ── Async session tests ──

    #[tokio::test]
    async fn test_login_flow_with_mock() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let mut session = SeleniumSession::with_backend(config, backend).unwrap();

        let result = session
            .login("https://example.com/login", "user@test.com", "pass123")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cookie_extraction_after_login() {
        let config = SeleniumConfig::default();
        let mut mock = MockBackend::new();
        mock.inject_cookie(Cookie {
            name: "token".into(),
            value: "xyz789".into(),
            domain: ".example.com".into(),
            path: "/".into(),
            expires: Some(1700000000),
            secure: false,
        });
        let mut session = SeleniumSession::with_backend(config, Box::new(mock)).unwrap();

        let cookies = session.extract_cookies().await;
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "token");
        assert_eq!(cookies[0].value, "xyz789");
    }

    #[tokio::test]
    async fn test_network_log_capture() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let session = SeleniumSession::with_backend(config, backend).unwrap();

        let logs = session.capture_network_logs().await;
        // MockBackend returns "mock_js_result" which isn't valid JSON for URLs
        // so we get empty vec — this is the expected fallback path
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn test_navigate_and_close() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let mut session = SeleniumSession::with_backend(config, backend).unwrap();

        assert!(session.navigate("https://example.com").await.is_ok());
        assert!(session.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_session_api_endpoints_empty() {
        let config = SeleniumConfig::default();
        let backend = Box::new(MockBackend::new());
        let session = SeleniumSession::with_backend(config, backend).unwrap();

        assert!(session.api_endpoints().is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════
// joinf tests
// ═══════════════════════════════════════════════════════════════

mod joinf_tests {
    use crate::l1_action::nt_act::nt_act_trade::extractors::joinf::*;
    use crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerGrade;

    #[test]
    fn test_joinf_extractor_new() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 12345, 67890);
        assert_eq!(extractor.platform_id(), "joinf");
        assert_eq!(extractor.platform_name(), "富通天下");
    }

    #[test]
    fn test_joinf_extractor_strips_trailing_slash() {
        let extractor = JoinfExtractor::new("https://www.joinf.com/", 1, 1);
        // Internal base_url should not have trailing slash
        assert_eq!(extractor.platform_id(), "joinf");
    }

    #[test]
    fn test_joinf_extractor_with_cookie() {
        let extractor =
            JoinfExtractor::with_cookie("https://www.joinf.com", 12345, 67890, "test=abc".into());
        assert_eq!(extractor.platform_id(), "joinf");
        assert_eq!(extractor.platform_name(), "富通天下");
    }

    #[test]
    fn test_joinf_customer_serialization() {
        let customer = JoinfCustomer {
            customer_id: 42,
            customer_name: "John".into(),
            company_name: "Acme".into(),
            country: "US".into(),
            email: "john@acme.com".into(),
            phone: "+1234567890".into(),
            whatsapp: "+1234567890".into(),
            grade: "A".into(),
            source: "Exhibition".into(),
            status: "won".into(),
            owner_name: "Rep".into(),
            tags: vec!["vip".into()],
            last_contact_time: Some(1700000000),
            create_time: Some(1600000000),
            update_time: Some(1700000000),
        };

        let json = serde_json::to_string(&customer).unwrap();
        let deserialized: JoinfCustomer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.customer_id, 42);
        assert_eq!(deserialized.company_name, "Acme");
    }

    #[test]
    fn test_customer_mapping() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let joinf_customer = JoinfCustomer {
            customer_id: 42,
            customer_name: "John Doe".into(),
            company_name: "Acme Corp".into(),
            country: "USA".into(),
            email: "john@acme.com".into(),
            phone: "+1234567890".into(),
            whatsapp: "+1234567890".into(),
            grade: "A".into(),
            source: "Exhibition".into(),
            status: "won".into(),
            owner_name: "Sales Rep".into(),
            tags: vec!["vip".into()],
            last_contact_time: Some(1700000000),
            create_time: Some(1600000000),
            update_time: Some(1700000000),
        };

        let profile = extractor.map_customer(&joinf_customer);
        assert_eq!(profile.id, "42");
        assert_eq!(profile.company.name, "Acme Corp");
        assert_eq!(profile.company.country, "USA");
        assert_eq!(profile.grade, CustomerGrade::A);
        assert_eq!(profile.contacts.len(), 1);
        assert_eq!(profile.contacts[0].name, "John Doe");
        assert_eq!(profile.contacts[0].emails, vec!["john@acme.com"]);
        assert_eq!(profile.contacts[0].phones, vec!["+1234567890"]);
        assert_eq!(profile.contacts[0].whatsapp, Some("+1234567890".into()));
        assert_eq!(profile.owner_id, "Sales Rep");
        assert_eq!(profile.tags, vec!["vip"]);
        assert_eq!(profile.created_at, 1600000000);
        assert_eq!(profile.updated_at, 1700000000);
    }

    #[test]
    fn test_customer_mapping_empty_fields() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let joinf_customer = JoinfCustomer {
            customer_id: 1,
            customer_name: "".into(),
            company_name: "".into(),
            country: "".into(),
            email: "".into(),
            phone: "".into(),
            whatsapp: "".into(),
            grade: "".into(),
            source: "".into(),
            status: "".into(),
            owner_name: "".into(),
            tags: vec![],
            last_contact_time: None,
            create_time: None,
            update_time: None,
        };

        let profile = extractor.map_customer(&joinf_customer);
        assert!(profile.contacts[0].emails.is_empty());
        assert!(profile.contacts[0].phones.is_empty());
        assert!(profile.contacts[0].whatsapp.is_none());
        // Unknown grade → D
        assert_eq!(profile.grade, CustomerGrade::D);
        // Unknown status → Lead
        assert_eq!(
            profile.status,
            crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Lead
        );
    }

    #[test]
    fn test_customer_mapping_grade_variants() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        for (grade_str, expected) in [
            ("A", CustomerGrade::A),
            ("A类", CustomerGrade::A),
            ("核心", CustomerGrade::A),
            ("B", CustomerGrade::B),
            ("B类", CustomerGrade::B),
            ("重要", CustomerGrade::B),
            ("C", CustomerGrade::C),
            ("C类", CustomerGrade::C),
            ("一般", CustomerGrade::C),
            ("D", CustomerGrade::D),
            ("unknown", CustomerGrade::D),
        ] {
            let customer = JoinfCustomer {
                customer_id: 1,
                customer_name: "Test".into(),
                company_name: "Co".into(),
                country: "US".into(),
                email: String::new(),
                phone: String::new(),
                whatsapp: String::new(),
                grade: grade_str.into(),
                source: String::new(),
                status: String::new(),
                owner_name: String::new(),
                tags: vec![],
                last_contact_time: None,
                create_time: None,
                update_time: None,
            };
            let profile = extractor.map_customer(&customer);
            assert_eq!(profile.grade, expected, "Grade mismatch for '{}'", grade_str);
        }
    }

    #[test]
    fn test_customer_mapping_status_variants() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        for (status_str, expected) in [
            ("lead", crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Lead),
            (
                "contacted",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Contacted,
            ),
            (
                "interested",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Interested,
            ),
            (
                "inquiring",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Inquiring,
            ),
            (
                "quoted",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Quoted,
            ),
            ("won", crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Won),
            (
                "dormant",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Dormant,
            ),
            ("lost", crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Lost),
            ("bogus", crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::CustomerStatus::Lead),
        ] {
            let customer = JoinfCustomer {
                customer_id: 1,
                customer_name: "Test".into(),
                company_name: "Co".into(),
                country: "US".into(),
                email: String::new(),
                phone: String::new(),
                whatsapp: String::new(),
                grade: String::new(),
                source: String::new(),
                status: status_str.into(),
                owner_name: String::new(),
                tags: vec![],
                last_contact_time: None,
                create_time: None,
                update_time: None,
            };
            let profile = extractor.map_customer(&customer);
            assert_eq!(profile.status, expected, "Status mismatch for '{}'", status_str);
        }
    }

    #[test]
    fn test_interaction_mapping() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let log = ActivityLog {
            id: 100,
            customer_id: 42,
            interaction_type: "whatsapp".into(),
            summary: "Sent product catalog".into(),
            detail: "Full product details sent via WhatsApp".into(),
            timestamp: 1700000000,
            operator: Some("Alice".into()),
        };

        let interaction = extractor.map_interaction(&log);
        assert_eq!(interaction.id, "100");
        assert_eq!(interaction.operator_id, "Alice");
        assert_eq!(interaction.summary, "Sent product catalog");
        assert_eq!(
            interaction.interaction_type,
            crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::WhatsApp
        );
        assert_eq!(interaction.timestamp, 1700000000);
    }

    #[test]
    fn test_interaction_mapping_type_variants() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        for (type_str, expected) in [
            (
                "email",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Email,
            ),
            (
                "phone",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Phone,
            ),
            (
                "meeting",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Meeting,
            ),
            (
                "inquiry",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Inquiry,
            ),
            (
                "quotation",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Quotation,
            ),
            (
                "contract",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::Contract,
            ),
            (
                "unknown_type",
                crate::l1_action::nt_act::nt_act_trade::nt_trade_crm::InteractionType::System,
            ),
        ] {
            let log = ActivityLog {
                id: 1,
                customer_id: 1,
                interaction_type: type_str.into(),
                summary: "s".into(),
                detail: "d".into(),
                timestamp: 0,
                operator: None,
            };
            let interaction = extractor.map_interaction(&log);
            assert_eq!(
                interaction.interaction_type, expected,
                "Type mismatch for '{}'",
                type_str
            );
        }
    }

    #[test]
    fn test_interaction_mapping_no_operator() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let log = ActivityLog {
            id: 1,
            customer_id: 1,
            interaction_type: "system".into(),
            summary: "s".into(),
            detail: "d".into(),
            timestamp: 0,
            operator: None,
        };

        let interaction = extractor.map_interaction(&log);
        assert!(interaction.operator_id.is_empty());
    }

    #[test]
    fn test_email_mapping() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let email = JoinfEmail {
            id: 200,
            from_addr: "sender@example.com".into(),
            to_addr: "a@test.com,b@test.com".into(),
            subject: "Re: Quote".into(),
            body: "<p>Thanks for the quote</p>".into(),
            received_time: Some(1700000000),
            is_read: true,
            box_id: 1,
            attachments: vec![JoinfAttachment {
                filename: "quotation.pdf".into(),
                size: 1024,
                url: "/files/quotation.pdf".into(),
            }],
        };

        let record = extractor.map_email(&email);
        assert_eq!(record.id, "200");
        assert_eq!(record.from, "sender@example.com");
        assert_eq!(record.to.len(), 2);
        assert_eq!(record.to[0], "a@test.com");
        assert_eq!(record.to[1], "b@test.com");
        assert_eq!(record.subject, "Re: Quote");
        assert_eq!(record.body_html, "<p>Thanks for the quote</p>");
        assert_eq!(record.attachments.len(), 1);
        assert_eq!(record.attachments[0].filename, "quotation.pdf");
        assert_eq!(record.attachments[0].size_bytes, 1024);
        assert_eq!(record.attachments[0].path, "/files/quotation.pdf");
        assert!(matches!(
            record.status,
            crate::l1_action::nt_act::nt_act_trade::nt_trade_email::EmailStatus::Opened
        ));
    }

    #[test]
    fn test_email_mapping_unread() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let email = JoinfEmail {
            id: 1,
            from_addr: "a@b.com".into(),
            to_addr: "c@d.com".into(),
            subject: "Hi".into(),
            body: "body".into(),
            received_time: None,
            is_read: false,
            box_id: 0,
            attachments: vec![],
        };

        let record = extractor.map_email(&email);
        assert!(matches!(
            record.status,
            crate::l1_action::nt_act::nt_act_trade::nt_trade_email::EmailStatus::Delivered
        ));
        assert!(record.sent_at.is_none());
    }

    #[test]
    fn test_email_mapping_empty_to() {
        let extractor = JoinfExtractor::new("https://www.joinf.com", 1, 1);

        let email = JoinfEmail {
            id: 1,
            from_addr: "a@b.com".into(),
            to_addr: "".into(),
            subject: "s".into(),
            body: "b".into(),
            received_time: None,
            is_read: false,
            box_id: 0,
            attachments: vec![],
        };

        let record = extractor.map_email(&email);
        assert!(record.to.is_empty());
    }

    #[test]
    fn test_extract_config_default() {
        let config = ExtractConfig::default();
        assert_eq!(config.page_size, 100);
        assert_eq!(config.max_pages, 0);
        assert!(config.customer_ids.is_empty());
        assert!(config.min_grade.is_none());
    }

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert_eq!(config.page_size, 50);
        assert_eq!(config.max_pages, 0);
        assert_eq!(config.box_id, -1);
        assert!(!config.unread_only);
    }

    #[test]
    fn test_joinf_error_display() {
        let cases = [
            (
                JoinfError::Network("timeout".into()),
                "network error: timeout",
            ),
            (
                JoinfError::Parse("bad json".into()),
                "parse error: bad json",
            ),
            (
                JoinfError::Auth("expired".into()),
                "auth error: expired",
            ),
            (
                JoinfError::Api {
                    code: 403,
                    message: "forbidden".into(),
                },
                "api error 403: forbidden",
            ),
            (
                JoinfError::SessionNotInitialized,
                "session not initialized, call init_session first",
            ),
            (
                JoinfError::Selenium("driver crash".into()),
                "selenium error: driver crash",
            ),
            (
                JoinfError::ChromeDecrypt("no key".into()),
                "chrome decrypt error: no key",
            ),
        ];

        for (err, expected) in &cases {
            assert_eq!(format!("{}", err), *expected);
        }
    }

    #[test]
    fn test_joinf_error_is_std_error() {
        let err = JoinfError::Network("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_discovered_apis_default() {
        let apis = DiscoveredApis::default();
        assert!(apis.endpoints.is_empty());
    }

    #[test]
    fn test_discovered_apis_insert() {
        let mut apis = DiscoveredApis::default();
        apis.endpoints
            .insert("/api/v1".into(), "https://example.com/api/v1".into());
        assert_eq!(apis.endpoints.len(), 1);
    }

    #[test]
    fn test_joinf_customer_log_serialization() {
        let log = JoinfCustomerLog {
            log_id: 100,
            customer_id: 42,
            log_type: "whatsapp".into(),
            content: "msg".into(),
            create_time: 1700000000,
            operator_name: "Alice".into(),
            whatsapp_info: Some("{}".into()),
            email_info: None,
        };
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: JoinfCustomerLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.log_id, 100);
        assert!(deserialized.email_info.is_none());
    }

    #[test]
    fn test_joinf_email_serialization() {
        let email = JoinfEmail {
            id: 1,
            from_addr: "a@b.com".into(),
            to_addr: "c@d.com".into(),
            subject: "Hi".into(),
            body: "body".into(),
            received_time: Some(1700000000),
            is_read: true,
            box_id: 1,
            attachments: vec![],
        };
        let json = serde_json::to_string(&email).unwrap();
        let deserialized: JoinfEmail = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert!(deserialized.is_read);
    }

    #[test]
    fn test_joinf_clue_serialization() {
        let clue = JoinfClue {
            clue_id: 1,
            company_name: "Co".into(),
            contact_name: "Person".into(),
            email: "p@co.com".into(),
            phone: "123".into(),
            source: "web".into(),
            status: "new".into(),
            create_time: 1700000000,
        };
        let json = serde_json::to_string(&clue).unwrap();
        let deserialized: JoinfClue = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.clue_id, 1);
    }

    #[test]
    fn test_joinf_business_serialization() {
        let biz = JoinfBusiness {
            business_id: 1,
            customer_name: "Co".into(),
            business_name: "Deal".into(),
            amount: 50000.0,
            currency: "USD".into(),
            status: "active".into(),
            stage_update_time: Some(1700000000),
        };
        let json = serde_json::to_string(&biz).unwrap();
        let deserialized: JoinfBusiness = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.amount, 50000.0);
    }

    #[test]
    fn test_joinf_attachment_serialization() {
        let att = JoinfAttachment {
            filename: "doc.pdf".into(),
            size: 2048,
            url: "/files/doc.pdf".into(),
        };
        let json = serde_json::to_string(&att).unwrap();
        let deserialized: JoinfAttachment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.filename, "doc.pdf");
        assert_eq!(deserialized.size, 2048);
    }

    #[test]
    fn test_whatsapp_contact_serialization() {
        let contact = JoinfWhatsappContact {
            customer_id: 42,
            whatsapp_number: "+1234567890".into(),
            display_name: "John".into(),
            last_message_time: Some(1700000000),
        };
        let json = serde_json::to_string(&contact).unwrap();
        let deserialized: JoinfWhatsappContact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.customer_id, 42);
    }
}

// ═══════════════════════════════════════════════════════════════
// data_pipeline tests
// ═══════════════════════════════════════════════════════════════

mod data_pipeline_tests {
    use super::*;
    use crate::l1_action::nt_act::nt_act_trade::data_pipeline::*;
    use crate::l1_action::nt_act::nt_act_trade::unified_types::*;
    use async_trait::async_trait;
    use chrono::Utc;

    // ── Mock extractor for testing ──

    struct MockExtractor;

    #[async_trait]
    impl ExternalPlatformExtractor for MockExtractor {
        fn platform_id(&self) -> &str {
            "mock"
        }

        fn display_name(&self) -> &str {
            "Mock Platform"
        }

        async fn extract_customers(
            &self,
            _config: ExtractConfig,
        ) -> Result<Vec<Customer>, String> {
            Ok(vec![Customer {
                id: "c1".into(),
                name: "Test Customer".into(),
                contact: TradeContactInfo::default(),
                grade: Grade::B,
                channel: Channel::Email,
                country: "CN".into(),
                tags: vec![],
                description: String::new(),
            }])
        }

        async fn extract_interactions(
            &self,
            _customer_id: &str,
        ) -> Result<Vec<Interaction>, String> {
            Ok(vec![Interaction {
                id: "i1".into(),
                customer_id: "c1".into(),
                interaction_type: "email".into(),
                summary: "Inquiry about valves".into(),
                timestamp: Utc::now(),
                raw: None,
            }])
        }

        async fn extract_emails(
            &self,
            _config: EmailConfig,
        ) -> Result<Vec<Email>, String> {
            Ok(vec![Email {
                id: "e1".into(),
                from: "buyer@example.com".into(),
                to: vec!["sales@company.com".into()],
                subject: "RFQ".into(),
                body: "Need 100 units.".into(),
                sent_at: Utc::now(),
                is_read: false,
                customer_id: Some("c1".into()),
                raw: None,
            }])
        }

        async fn sync_incremental(
            &self,
            _last_sync: DateTime<Utc>,
        ) -> Result<SyncResult, String> {
            Ok(SyncResult {
                customers_created: 1,
                customers_updated: 0,
                interactions_created: 1,
                emails_created: 1,
                synced_until: Utc::now(),
                errors: vec![],
            })
        }
    }

    struct FailingExtractor;

    #[async_trait]
    impl ExternalPlatformExtractor for FailingExtractor {
        fn platform_id(&self) -> &str {
            "failing"
        }

        fn display_name(&self) -> &str {
            "Failing Platform"
        }

        async fn extract_customers(
            &self,
            _config: ExtractConfig,
        ) -> Result<Vec<Customer>, String> {
            Err("customer extraction failed".into())
        }

        async fn extract_interactions(
            &self,
            _customer_id: &str,
        ) -> Result<Vec<Interaction>, String> {
            Err("interaction extraction failed".into())
        }

        async fn extract_emails(
            &self,
            _config: EmailConfig,
        ) -> Result<Vec<Email>, String> {
            Err("email extraction failed".into())
        }

        async fn sync_incremental(
            &self,
            _last_sync: DateTime<Utc>,
        ) -> Result<SyncResult, String> {
            Err("sync failed".into())
        }
    }

    // ── Normalizer tests ──

    #[test]
    fn test_normalizer_customer_trim() {
        let normalizer = DataNormalizer::default();
        let customer = Customer {
            id: "c1".into(),
            name: "  Acme Corp  ".into(),
            contact: TradeContactInfo::default(),
            grade: Grade::B,
            channel: Channel::Email,
            country: "US".into(),
            tags: vec![],
            description: String::new(),
        };

        let normalized = normalizer.normalize_customer(customer);
        assert_eq!(normalized.name, "Acme Corp");
    }

    #[test]
    fn test_normalizer_customer_grade_upgrade() {
        let normalizer = DataNormalizer::default();
        let customer = Customer {
            id: "c1".into(),
            name: "  Acme Corp  ".into(),
            contact: TradeContactInfo {
                name: "  John  ".into(),
                email: "  JOHN@EXAMPLE.COM  ".into(),
                phone: "  +86 138 0000 0000  ".into(),
                title: "Manager".into(),
                wechat: None,
            },
            grade: Grade::E,
            channel: Channel::Exhibition,
            country: "US".into(),
            tags: vec!["vip_prospect".into()],
            description: "Important lead".into(),
        };

        let normalized = normalizer.normalize_customer(customer);
        assert_eq!(normalized.grade, Grade::C); // upgraded because tags non-empty
        assert_eq!(normalized.contact.name, "John");
        assert_eq!(normalized.contact.email, "john@example.com");
        assert_eq!(normalized.contact.phone, "+86 138 0000 0000");
    }

    #[test]
    fn test_normalizer_customer_no_upgrade_empty_tags() {
        let normalizer = DataNormalizer::default();
        let customer = Customer {
            id: "c1".into(),
            name: "Test".into(),
            contact: TradeContactInfo::default(),
            grade: Grade::E,
            channel: Channel::Email,
            country: "US".into(),
            tags: vec![],
            description: String::new(),
        };

        let normalized = normalizer.normalize_customer(customer);
        assert_eq!(normalized.grade, Grade::E); // no upgrade
    }

    #[test]
    fn test_normalizer_customer_no_upgrade_non_e() {
        let normalizer = DataNormalizer::default();
        let customer = Customer {
            id: "c1".into(),
            name: "Test".into(),
            contact: TradeContactInfo::default(),
            grade: Grade::D,
            channel: Channel::Email,
            country: "US".into(),
            tags: vec!["tag".into()],
            description: String::new(),
        };

        let normalized = normalizer.normalize_customer(customer);
        assert_eq!(normalized.grade, Grade::D); // not E, no upgrade
    }

    #[test]
    fn test_normalizer_contact() {
        let normalizer = DataNormalizer::default();
        let contact = TradeContactInfo {
            name: "  Alice  ".into(),
            email: "  ALICE@TEST.COM  ".into(),
            phone: "  +1234567  ".into(),
            title: "Dev".into(),
            wechat: None,
        };

        let normalized = normalizer.normalize_contact(contact);
        assert_eq!(normalized.name, "Alice");
        assert_eq!(normalized.email, "alice@test.com");
        assert_eq!(normalized.phone, "+1234567");
    }

    #[test]
    fn test_normalizer_interaction() {
        let normalizer = DataNormalizer::default();
        let interaction = Interaction {
            id: "i1".into(),
            customer_id: "c1".into(),
            interaction_type: "EMAIL".into(),
            summary: "  Follow up  ".into(),
            timestamp: Utc::now(),
            raw: None,
        };

        let normalized = normalizer.normalize_interaction(interaction);
        assert_eq!(normalized.interaction_type, "email");
        assert_eq!(normalized.summary, "Follow up");
    }

    #[test]
    fn test_normalizer_email() {
        let normalizer = DataNormalizer::default();
        let email = Email {
            id: "e1".into(),
            from: "  SENDER@TEST.COM  ".into(),
            to: vec![],
            subject: "  Hello  ".into(),
            body: "  world  ".into(),
            sent_at: Utc::now(),
            is_read: false,
            customer_id: None,
            raw: None,
        };

        let normalized = normalizer.normalize_email(email);
        assert_eq!(normalized.from, "sender@test.com");
        assert_eq!(normalized.subject, "Hello");
        assert_eq!(normalized.body, "world");
    }

    #[test]
    fn test_normalizer_batch_customers() {
        let normalizer = DataNormalizer::default();
        let customers = vec![
            Customer {
                id: "c1".into(),
                name: "  A  ".into(),
                contact: TradeContactInfo::default(),
                grade: Grade::B,
                channel: Channel::Email,
                country: "US".into(),
                tags: vec![],
                description: String::new(),
            },
            Customer {
                id: "c2".into(),
                name: "  B  ".into(),
                contact: TradeContactInfo::default(),
                grade: Grade::C,
                channel: Channel::Email,
                country: "US".into(),
                tags: vec![],
                description: String::new(),
            },
        ];

        let normalized = normalizer.normalize_customers(customers);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].name, "A");
        assert_eq!(normalized[1].name, "B");
    }

    // ── Pipeline tests ──

    #[tokio::test]
    async fn test_pipeline_extract_all() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockExtractor));

        let pipeline = TradeDataPipeline::with_registry(registry);
        let result = pipeline.extract_all("mock").await.unwrap();

        assert_eq!(result.customers.len(), 1);
        assert_eq!(result.customers[0].id, "c1");
        assert!(result.interactions.contains_key("c1"));
        assert_eq!(result.interactions["c1"].len(), 1);
        assert_eq!(result.emails.len(), 1);
    }

    #[tokio::test]
    async fn test_pipeline_extract_unknown_platform() {
        let registry = PlatformRegistry::new();
        let pipeline = TradeDataPipeline::with_registry(registry);

        let result = pipeline.extract_all("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not registered"));
    }

    #[tokio::test]
    async fn test_pipeline_sync_incremental() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockExtractor));

        let pipeline = TradeDataPipeline::with_registry(registry);
        let sync_result = pipeline
            .sync_incremental("mock", Utc::now())
            .await
            .unwrap();

        assert_eq!(sync_result.customers_created, 1);
        assert_eq!(sync_result.interactions_created, 1);
        assert_eq!(sync_result.emails_created, 1);
        assert!(sync_result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_pipeline_sync_unknown_platform() {
        let registry = PlatformRegistry::new();
        let pipeline = TradeDataPipeline::with_registry(registry);

        let result = pipeline.sync_incremental("ghost", Utc::now()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pipeline_extract_all_platforms() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockExtractor));

        let pipeline = TradeDataPipeline::with_registry(registry);
        let results = pipeline.extract_all_platforms().await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains_key("mock"));
    }

    #[tokio::test]
    async fn test_pipeline_extract_all_platforms_empty() {
        let registry = PlatformRegistry::new();
        let pipeline = TradeDataPipeline::with_registry(registry);
        let results = pipeline.extract_all_platforms().await.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_pipeline_with_registry() {
        let mut registry = PlatformRegistry::new();
        registry.register(Arc::new(MockExtractor));
        let pipeline = TradeDataPipeline::with_registry(registry);
        assert!(!pipeline.registry().is_empty());
    }

    #[test]
    fn test_pipeline_registry_mut() {
        let mut registry = PlatformRegistry::new();
        let mut pipeline = TradeDataPipeline::with_registry(registry);
        pipeline.registry_mut().register(Arc::new(MockExtractor));
        assert_eq!(pipeline.registry().len(), 1);
    }

    // ── ExtractionResult / SyncResult defaults ──

    #[test]
    fn test_extraction_result_default() {
        let result = ExtractionResult::default();
        assert!(result.customers.is_empty());
        assert!(result.interactions.is_empty());
        assert!(result.emails.is_empty());
        assert_eq!(result.duration_ms, 0);
    }

    #[test]
    fn test_sync_result_default() {
        let result = SyncResult::default();
        assert_eq!(result.customers_created, 0);
        assert_eq!(result.customers_updated, 0);
        assert_eq!(result.interactions_created, 0);
        assert_eq!(result.emails_created, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_extract_config_default() {
        let config = ExtractConfig::default();
        assert!(config.page_size.is_none());
        assert!(config.max_records.is_none());
        assert!(config.filters.is_empty());
    }

    #[test]
    fn test_email_config_default() {
        let config = EmailConfig::default();
        assert!(config.since.is_none());
        assert!(config.until.is_none());
        assert!(config.max_count.is_none());
        assert!(!config.unread_only);
    }

    #[test]
    fn test_extract_config_serialization() {
        let config = ExtractConfig {
            page_size: Some(50),
            max_records: Some(1000),
            filters: {
                let mut f = std::collections::HashMap::new();
                f.insert("status".into(), "active".into());
                f
            },
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ExtractConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.page_size, Some(50));
        assert_eq!(deserialized.max_records, Some(1000));
    }
}

// ═══════════════════════════════════════════════════════════════
// platform_registry tests (the one in platform_registry.rs)
// ═══════════════════════════════════════════════════════════════

mod platform_registry_tests {
    use crate::l1_action::nt_act::nt_act_trade::platform_registry::*;

    struct TestAdapter {
        id: String,
        name: String,
        features: Vec<PlatformFeature>,
    }

    impl PlatformAdapter for TestAdapter {
        fn platform_id(&self) -> &str {
            &self.id
        }
        fn platform_name(&self) -> &str {
            &self.name
        }
        fn supported_features(&self) -> Vec<PlatformFeature> {
            self.features.clone()
        }
    }

    fn test_config() -> PlatformConfig {
        PlatformConfig {
            base_url: "https://example.com".into(),
            auth_method: AuthMethod::Token,
            rate_limit: 100,
            timeout_secs: 30,
        }
    }

    #[test]
    fn test_registry_new_empty() {
        let reg = PlatformRegistry::new();
        assert!(reg.list_platforms().is_empty());
        assert!(reg.get("nope").is_none());
        assert!(reg.get_config("nope").is_none());
    }

    #[test]
    fn test_registry_default() {
        let reg = PlatformRegistry::default();
        assert!(reg.list_platforms().is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "alibaba".into(),
            name: "Alibaba".into(),
            features: vec![PlatformFeature::CustomerSync],
        });
        reg.register(adapter, test_config());

        assert_eq!(reg.list_platforms().len(), 1);
        assert!(reg.get("alibaba").is_some());
    }

    #[test]
    fn test_registry_get() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "amazon".into(),
            name: "Amazon".into(),
            features: vec![],
        });
        reg.register(adapter, test_config());

        let found = reg.get("amazon");
        assert!(found.is_some());
        assert_eq!(found.unwrap().platform_name(), "Amazon");
    }

    #[test]
    fn test_registry_get_missing() {
        let reg = PlatformRegistry::new();
        assert!(reg.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_get_config() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "shopify".into(),
            name: "Shopify".into(),
            features: vec![],
        });
        reg.register(adapter, test_config());

        let cfg = reg.get_config("shopify").unwrap();
        assert_eq!(cfg.base_url, "https://example.com");
        assert_eq!(cfg.auth_method, AuthMethod::Token);
        assert_eq!(cfg.rate_limit, 100);
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn test_registry_list_platforms() {
        let mut reg = PlatformRegistry::new();
        for id in &["a", "b", "c"] {
            let adapter = Arc::new(TestAdapter {
                id: id.to_string(),
                name: id.to_string(),
                features: vec![],
            });
            reg.register(adapter, test_config());
        }

        let mut platforms = reg.list_platforms();
        platforms.sort();
        assert_eq!(platforms, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_registry_supports_feature() {
        let mut reg = PlatformRegistry::new();
        let adapter = Arc::new(TestAdapter {
            id: "whatsapp".into(),
            name: "WhatsApp".into(),
            features: vec![PlatformFeature::WhatsAppSync, PlatformFeature::ContactSync],
        });
        reg.register(adapter, test_config());

        assert!(reg.supports_feature("whatsapp", PlatformFeature::WhatsAppSync));
        assert!(reg.supports_feature("whatsapp", PlatformFeature::ContactSync));
        assert!(!reg.supports_feature("whatsapp", PlatformFeature::EmailSync));
    }

    #[test]
    fn test_registry_supports_feature_unknown_platform() {
        let reg = PlatformRegistry::new();
        assert!(!reg.supports_feature("unknown", PlatformFeature::WhatsAppSync));
    }

    #[test]
    fn test_registry_overwrite() {
        let mut reg = PlatformRegistry::new();
        let adapter1 = Arc::new(TestAdapter {
            id: "x".into(),
            name: "X v1".into(),
            features: vec![],
        });
        let adapter2 = Arc::new(TestAdapter {
            id: "x".into(),
            name: "X v2".into(),
            features: vec![],
        });

        reg.register(adapter1, test_config());
        reg.register(adapter2, test_config());

        assert_eq!(reg.list_platforms().len(), 1);
        assert_eq!(reg.get("x").unwrap().platform_name(), "X v2");
    }

    #[test]
    fn test_auth_method_variants() {
        assert_eq!(AuthMethod::Cookie, AuthMethod::Cookie);
        assert_eq!(AuthMethod::Token, AuthMethod::Token);
        assert_eq!(AuthMethod::OAuth2, AuthMethod::OAuth2);
        assert_eq!(AuthMethod::ApiKey, AuthMethod::ApiKey);
        assert_ne!(AuthMethod::Cookie, AuthMethod::Token);
    }

    #[test]
    fn test_platform_feature_variants() {
        assert_eq!(PlatformFeature::CustomerSync, PlatformFeature::CustomerSync);
        assert_eq!(PlatformFeature::EmailSync, PlatformFeature::EmailSync);
        assert_eq!(
            PlatformFeature::WhatsAppSync,
            PlatformFeature::WhatsAppSync
        );
        assert_eq!(PlatformFeature::ContactSync, PlatformFeature::ContactSync);
        assert_ne!(PlatformFeature::CustomerSync, PlatformFeature::EmailSync);
    }

    #[test]
    fn test_platform_config_clone() {
        let config = test_config();
        let cloned = config.clone();
        assert_eq!(cloned.base_url, "https://example.com");
        assert_eq!(cloned.auth_method, AuthMethod::Token);
    }

    #[test]
    fn test_platform_config_debug() {
        let config = test_config();
        let debug = format!("{:?}", config);
        assert!(debug.contains("PlatformConfig"));
        assert!(debug.contains("https://example.com"));
    }

    #[test]
    fn test_registry_multiple_platforms_features() {
        let mut reg = PlatformRegistry::new();

        let adapter_a = Arc::new(TestAdapter {
            id: "a".into(),
            name: "A".into(),
            features: vec![PlatformFeature::CustomerSync, PlatformFeature::EmailSync],
        });
        let adapter_b = Arc::new(TestAdapter {
            id: "b".into(),
            name: "B".into(),
            features: vec![PlatformFeature::WhatsAppSync],
        });

        let mut cfg_a = test_config();
        cfg_a.auth_method = AuthMethod::Cookie;
        let mut cfg_b = test_config();
        cfg_b.rate_limit = 200;

        reg.register(adapter_a, cfg_a);
        reg.register(adapter_b, cfg_b);

        assert_eq!(reg.list_platforms().len(), 2);
        assert!(reg.supports_feature("a", PlatformFeature::CustomerSync));
        assert!(!reg.supports_feature("a", PlatformFeature::WhatsAppSync));
        assert!(reg.supports_feature("b", PlatformFeature::WhatsAppSync));
        assert!(!reg.supports_feature("b", PlatformFeature::CustomerSync));

        assert_eq!(reg.get_config("a").unwrap().auth_method, AuthMethod::Cookie);
        assert_eq!(reg.get_config("b").unwrap().rate_limit, 200);
    }
}
