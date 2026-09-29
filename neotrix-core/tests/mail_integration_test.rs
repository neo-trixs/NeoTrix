#![forbid(unsafe_code)]
#![cfg(feature = "mail")]

//! nt_io_mail 集成测试 — 需要 mail feature (nt_io_mail 模块尚未实现)
//!
//! 端到端测试覆盖:
//! 1. SMTP 发送流程 (RFC 5322 构建 + 格式验证)
//! 2. MIME 解析集成 (复杂 multipart + charset + 附件)
//! 3. 同步引擎集成 (增量同步 + uid_validity + 离线队列)
//! 4. Error Recovery (重试策略 + 超时 + 认证失败)

#[cfg(test)]
mod tests {
    use neotrix::unified::layers::action::nt_io::nt_io_mail::imap::FolderInfo as ImapFolderInfo;
    use neotrix::unified::layers::action::nt_io::nt_io_mail::mime::{MimeBody, MimeMessage};
    use neotrix::unified::layers::action::nt_io::nt_io_mail::sync::{
        IdleConfig, MailSyncEngine, OfflineOperation, OfflineQueue, ReplayResult, RetryConfig,
        SyncConfig, SyncEvent, SyncResult, SyncState,
    };
    use neotrix::unified::layers::action::nt_io::nt_io_mail::types::{
        AccountConfig, AttachmentData, AuthMethod, MailAddress, MailAttachment, MailContent,
        MailFolder, MailMessage, SearchQuery, SendMailRequest, ServerConfig,
    };
    use neotrix::unified::layers::action::nt_io::nt_io_mail::MailError;
    use neotrix::unified::layers::action::nt_io::EmailProvider;

    use std::collections::HashMap;
    use std::time::Duration;

    // ========================================================================
    // Helper: 创建测试用 AccountConfig
    // ========================================================================

    fn test_config() -> AccountConfig {
        AccountConfig {
            id: "test_account".to_string(),
            display_name: "Test User".to_string(),
            email: "test@neotrix.dev".to_string(),
            imap: ServerConfig {
                host: "imap.neotrix.dev".to_string(),
                port: 993,
                tls: true,
            },
            smtp: ServerConfig {
                host: "smtp.neotrix.dev".to_string(),
                port: 587,
                tls: true,
            },
            auth: AuthMethod::Password {
                password: "test_password".to_string(),
            },
        }
    }

    fn test_config_oauth() -> AccountConfig {
        AccountConfig {
            id: "oauth_account".to_string(),
            display_name: "OAuth User".to_string(),
            email: "oauth@neotrix.dev".to_string(),
            imap: ServerConfig {
                host: "imap.neotrix.dev".to_string(),
                port: 993,
                tls: true,
            },
            smtp: ServerConfig {
                host: "smtp.neotrix.dev".to_string(),
                port: 465,
                tls: true,
            },
            auth: AuthMethod::OAuth2 {
                access_token: "ya29.test_token".to_string(),
                refresh_token: Some("1//test_refresh".to_string()),
                client_id: "client_id_123".to_string(),
                client_secret: "client_secret_456".to_string(),
            },
        }
    }

    fn test_mail_message(uid: u32) -> MailMessage {
        MailMessage {
            uid,
            message_id: None,
            subject: format!("Test {uid}"),
            from: vec![MailAddress {
                name: Some("Sender".to_string()),
                address: "sender@test.com".to_string(),
            }],
            to: vec![MailAddress {
                name: None,
                address: "receiver@test.com".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            date: None,
            seen: false,
            flagged: false,
            draft: false,
            deleted: false,
            has_attachments: false,
            size: None,
            preview: None,
            keywords: vec![],
            raw: None,
        }
    }

    // ========================================================================
    // 1. Mock SMTP 发送流程测试
    // ========================================================================

    #[test]
    fn test_smtp_send_flow_build_message_simple() {
        let request = SendMailRequest {
            to: vec![MailAddress {
                name: Some("Recipient".to_string()),
                address: "recipient@example.com".to_string(),
            }],
            subject: "Integration Test".to_string(),
            text_body: Some("Hello from integration test".to_string()),
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let content = String::from_utf8_lossy(&raw);
        assert!(content.contains("MIME-Version: 1.0"));
        assert!(content.contains("Subject: Integration Test"));
        assert!(content.contains("To: recipient@example.com"));
        assert!(content.contains("Hello from integration test"));
        assert!(content.contains("text/plain; charset=UTF-8"));
        assert!(content.contains("Message-ID:"));
        assert!(content.contains("Date:"));
    }

    #[test]
    fn test_smtp_send_flow_with_attachments() {
        let request = SendMailRequest {
            to: vec![MailAddress {
                name: None,
                address: "user@test.com".to_string(),
            }],
            subject: "Attachment Test".to_string(),
            text_body: Some("See attached file".to_string()),
            html_body: Some("<p>See attached file</p>".to_string()),
            attachments: vec![
                AttachmentData {
                    filename: "report.pdf".to_string(),
                    mime_type: "application/pdf".to_string(),
                    data: vec![0x25, 0x50, 0x44, 0x46], // %PDF
                },
                AttachmentData {
                    filename: "data.csv".to_string(),
                    mime_type: "text/csv".to_string(),
                    data: b"id,name\n1,foo\n2,bar".to_vec(),
                },
            ],
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let content = String::from_utf8_lossy(&raw);
        assert!(content.contains("multipart/mixed"));
        assert!(content.contains("multipart/alternative"));
        assert!(content.contains("report.pdf"));
        assert!(content.contains("data.csv"));
        assert!(content.contains("Content-Transfer-Encoding: base64"));
        assert!(content.contains("Content-Disposition: attachment"));
    }

    #[test]
    fn test_smtp_send_flow_with_reply_headers() {
        let request = SendMailRequest {
            to: vec![MailAddress {
                name: None,
                address: "reply@test.com".to_string(),
            }],
            subject: "Re: Original".to_string(),
            text_body: Some("My reply".to_string()),
            in_reply_to: Some("<original@neotrix>".to_string()),
            references: vec!["<ref1@neotrix>".to_string(), "<ref2@neotrix>".to_string()],
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let content = String::from_utf8_lossy(&raw);
        assert!(content.contains("In-Reply-To: <original@neotrix>"));
        assert!(content.contains("References: <ref1@neotrix> <ref2@neotrix>"));
    }

    #[test]
    fn test_smtp_send_flow_with_custom_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Priority".to_string(), "1".to_string());
        headers.insert("X-Mailer".to_string(), "NeoTrix/1.0".to_string());
        headers.insert("X-Custom-ID".to_string(), "abc-123".to_string());

        let request = SendMailRequest {
            to: vec![MailAddress {
                name: None,
                address: "a@b.com".to_string(),
            }],
            subject: "Custom Headers".to_string(),
            text_body: Some("body".to_string()),
            headers,
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let content = String::from_utf8_lossy(&raw);
        assert!(content.contains("X-Priority: 1"));
        assert!(content.contains("X-Mailer: NeoTrix/1.0"));
        assert!(content.contains("X-Custom-ID: abc-123"));
    }

    #[test]
    fn test_smtp_send_flow_cc_bcc() {
        let request = SendMailRequest {
            to: vec![MailAddress {
                name: Some("To User".to_string()),
                address: "to@test.com".to_string(),
            }],
            cc: vec![MailAddress {
                name: Some("CC User".to_string()),
                address: "cc@test.com".to_string(),
            }],
            bcc: vec![MailAddress {
                name: None,
                address: "bcc@test.com".to_string(),
            }],
            subject: "CC/BCC Test".to_string(),
            text_body: Some("body".to_string()),
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let content = String::from_utf8_lossy(&raw);
        assert!(content.contains("To: to@test.com"));
        assert!(content.contains("Cc: cc@test.com"));
    }

    // ========================================================================
    // 2. MIME 解析集成测试
    // ========================================================================

    #[test]
    fn test_mime_parse_simple_text_email() {
        let raw = b"From: sender@example.com\r\n\
                     To: receiver@example.com\r\n\
                     Subject: Simple Text\r\n\
                     Date: Mon, 01 Jan 2024 12:00:00 +0000\r\n\
                     Message-ID: <test-001@example>\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     Hello, World!";

        let msg = MimeMessage::parse(raw).unwrap();
        assert_eq!(msg.headers.from.len(), 1);
        assert_eq!(msg.headers.from[0].address, "sender@example.com");
        assert_eq!(msg.headers.to.len(), 1);
        assert_eq!(msg.headers.to[0].address, "receiver@example.com");
        assert_eq!(msg.headers.subject, Some("Simple Text".to_string()));
        assert_eq!(
            msg.headers.message_id,
            Some("<test-001@example>".to_string())
        );

        if let MimeBody::Single(part) = &msg.body {
            assert_eq!(part.content_type, "text/plain");
            assert_eq!(String::from_utf8_lossy(&part.content), "Hello, World!");
        } else {
            panic!("expected single text part");
        }
    }

    #[test]
    fn test_mime_parse_multipart_mixed_with_attachment() {
        let raw = b"From: sender@example.com\r\n\
                     To: receiver@example.com\r\n\
                     Subject: Multipart Mixed\r\n\
                     Content-Type: multipart/mixed; boundary=boundary_001\r\n\
                     \r\n\
                     --boundary_001\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     Body text here\r\n\
                     --boundary_001\r\n\
                     Content-Type: application/pdf\r\n\
                     Content-Disposition: attachment; filename=\"doc.pdf\"\r\n\
                     Content-Transfer-Encoding: base64\r\n\
                     \r\n\
                     JVBERi0xLjQK\r\n\
                     --boundary_001--\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        let content = msg.to_mail_content();
        assert!(content.text.is_some());
        assert_eq!(content.text.unwrap(), "Body text here");
        assert_eq!(content.attachments.len(), 1);
        assert_eq!(content.attachments[0].filename, "doc.pdf");
        assert_eq!(content.attachments[0].mime_type, "application/pdf");
        assert!(content.attachments[0].size > 0);
    }

    #[test]
    fn test_mime_parse_multipart_alternative_html_and_text() {
        let raw = b"From: sender@example.com\r\n\
                     To: receiver@example.com\r\n\
                     Subject: Alternative Test\r\n\
                     Content-Type: multipart/alternative; boundary=alt_boundary\r\n\
                     \r\n\
                     --alt_boundary\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     Plain text version\r\n\
                     --alt_boundary\r\n\
                     Content-Type: text/html; charset=utf-8\r\n\
                     \r\n\
                     <p>HTML version</p>\r\n\
                     --alt_boundary--\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        let content = msg.to_mail_content();
        assert!(content.text.is_some());
        assert!(content.html.is_some());
        assert_eq!(content.text.unwrap(), "Plain text version");
        assert_eq!(content.html.unwrap(), "<p>HTML version</p>");
        assert!(content.attachments.is_empty());
    }

    #[test]
    fn test_mime_parse_nested_multipart_with_inline_image() {
        let raw = b"From: sender@example.com\r\n\
                     To: receiver@example.com\r\n\
                     Subject: Nested Test\r\n\
                     Content-Type: multipart/mixed; boundary=mixed_outer\r\n\
                     \r\n\
                     --mixed_outer\r\n\
                     Content-Type: multipart/alternative; boundary=alt_inner\r\n\
                     \r\n\
                     --alt_inner\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     Plain text body\r\n\
                     --alt_inner\r\n\
                     Content-Type: text/html; charset=utf-8\r\n\
                     \r\n\
                     <h1>HTML body</h1>\r\n\
                     --alt_inner--\r\n\
                     \r\n\
                     --mixed_outer\r\n\
                     Content-Type: image/png\r\n\
                     Content-Disposition: inline; name=\"logo.png\"\r\n\
                     Content-ID: <logo@neotrix>\r\n\
                     Content-Transfer-Encoding: base64\r\n\
                     \r\n\
                     iVBORw0KGgoAAAANSUhEUg==\r\n\
                     --mixed_outer\r\n\
                     Content-Type: text/plain\r\n\
                     Content-Disposition: attachment; filename=\"notes.txt\"\r\n\
                     Content-Transfer-Encoding: base64\r\n\
                     \r\n\
                     SGVsbG8gV29ybGQ=\r\n\
                     --mixed_outer--\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        let content = msg.to_mail_content();
        assert!(content.text.is_some());
        assert!(content.html.is_some());
        assert_eq!(content.text.unwrap(), "Plain text body");
        assert_eq!(content.html.unwrap(), "<h1>HTML body</h1>");
        assert_eq!(content.inline_images.len(), 1);
        assert_eq!(content.inline_images[0].content_id, "logo@neotrix");
        assert_eq!(content.inline_images[0].mime_type, "image/png");
        assert_eq!(content.attachments.len(), 1);
        assert_eq!(content.attachments[0].filename, "notes.txt");
    }

    #[test]
    fn test_mime_parse_base64_encoded_content() {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"Encoded body content");
        let raw = format!(
            "From: test@example.com\r\n\
             To: user@example.com\r\n\
             Subject: Base64 Test\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\
             Content-Transfer-Encoding: base64\r\n\
             \r\n\
             {}\r\n",
            encoded
        );

        let msg = MimeMessage::parse(raw.as_bytes()).unwrap();
        if let MimeBody::Single(part) = &msg.body {
            assert_eq!(
                String::from_utf8_lossy(&part.content),
                "Encoded body content"
            );
        } else {
            panic!("expected single part");
        }
    }

    #[test]
    fn test_mime_parse_quoted_printable_content() {
        let raw = b"From: test@example.com\r\n\
                     To: user@example.com\r\n\
                     Subject: QP Test\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     Content-Transfer-Encoding: quoted-printable\r\n\
                     \r\n\
                     Hello=20World=21\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        if let MimeBody::Single(part) = &msg.body {
            assert_eq!(String::from_utf8_lossy(&part.content), "Hello World!");
        } else {
            panic!("expected single part");
        }
    }

    #[test]
    fn test_mime_parse_rfc2047_encoded_subject() {
        let raw = b"From: test@example.com\r\n\
                     To: user@example.com\r\n\
                     Subject: =?utf-8?B?SGVsbG8gV29ybGQ=?=\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     body\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        assert_eq!(msg.headers.subject, Some("Hello World".to_string()));
    }

    #[test]
    fn test_mime_parse_multiple_attachments() {
        let raw = b"From: sender@example.com\r\n\
                     To: receiver@example.com\r\n\
                     Subject: Multi Attach\r\n\
                     Content-Type: multipart/mixed; boundary=multi_bound\r\n\
                     \r\n\
                     --multi_bound\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     See files\r\n\
                     --multi_bound\r\n\
                     Content-Type: image/jpeg\r\n\
                     Content-Disposition: attachment; filename=\"photo.jpg\"\r\n\
                     Content-Transfer-Encoding: base64\r\n\
                     \r\n\
                     /9j/4AAQ\r\n\
                     --multi_bound\r\n\
                     Content-Type: application/zip\r\n\
                     Content-Disposition: attachment; filename=\"archive.zip\"\r\n\
                     Content-Transfer-Encoding: base64\r\n\
                     \r\n\
                     UEsDBBQ\r\n\
                     --multi_bound\r\n\
                     Content-Type: text/csv\r\n\
                     Content-Disposition: attachment; filename=\"data.csv\"\r\n\
                     Content-Transfer-Encoding: 7bit\r\n\
                     \r\n\
                     id,name\r\n1,Alice\r\n2,Bob\r\n\
                     --multi_bound--\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        let content = msg.to_mail_content();
        assert!(content.text.is_some());
        assert_eq!(content.attachments.len(), 3);
        assert_eq!(content.attachments[0].filename, "photo.jpg");
        assert_eq!(content.attachments[0].mime_type, "image/jpeg");
        assert_eq!(content.attachments[1].filename, "archive.zip");
        assert_eq!(content.attachments[2].filename, "data.csv");
    }

    #[test]
    fn test_mime_to_mail_message_conversion() {
        let raw = b"From: alice@example.com\r\n\
                     To: bob@example.com\r\n\
                     Cc: carol@example.com\r\n\
                     Subject: Conversion Test\r\n\
                     Date: Tue, 15 Mar 2024 10:30:00 +0000\r\n\
                     Message-ID: <msg-42@example>\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     Test body content";

        let msg = MimeMessage::parse(raw).unwrap();
        let mail_msg = msg.to_mail_message(99);

        assert_eq!(mail_msg.uid, 99);
        assert_eq!(mail_msg.subject, "Conversion Test");
        assert_eq!(mail_msg.from.len(), 1);
        assert_eq!(mail_msg.from[0].address, "alice@example.com");
        assert_eq!(mail_msg.to.len(), 1);
        assert_eq!(mail_msg.to[0].address, "bob@example.com");
        assert_eq!(mail_msg.cc.len(), 1);
        assert_eq!(mail_msg.cc[0].address, "carol@example.com");
        assert!(mail_msg.date.is_some());
        assert!(!mail_msg.has_attachments);
        assert_eq!(mail_msg.message_id, Some("<msg-42@example>".to_string()));
    }

    #[test]
    fn test_mime_parse_empty_body() {
        let raw = b"From: test@example.com\r\n\
                     To: user@example.com\r\n\
                     Subject: Empty\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        if let MimeBody::Single(part) = &msg.body {
            assert!(part.content.is_empty());
        } else {
            panic!("expected single part");
        }
    }

    // ========================================================================
    // 3. 同步引擎集成测试
    // ========================================================================

    #[tokio::test]
    async fn test_sync_engine_initial_state() {
        let engine = MailSyncEngine::new(SyncConfig::default());
        assert!(!engine.is_connected().await);
        assert!(engine.get_state("INBOX").await.is_none());
        assert_eq!(engine.queue_depth().await, 0);
    }

    #[tokio::test]
    async fn test_sync_engine_set_and_get_state() {
        let engine = MailSyncEngine::new(SyncConfig::default());
        let state = SyncState {
            folder: "INBOX".to_string(),
            last_uid: 100,
            uid_validity: 42,
            uid_next: 101,
            last_sync: chrono::Utc::now(),
        };

        engine.set_state("INBOX", state.clone()).await;
        let retrieved = engine.get_state("INBOX").await;
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.last_uid, 100);
        assert_eq!(retrieved.uid_validity, 42);
        assert_eq!(retrieved.uid_next, 101);
    }

    #[tokio::test]
    async fn test_sync_engine_state_per_folder() {
        let engine = MailSyncEngine::new(SyncConfig::default());

        engine
            .set_state(
                "INBOX",
                SyncState {
                    folder: "INBOX".to_string(),
                    last_uid: 50,
                    uid_validity: 1,
                    uid_next: 51,
                    last_sync: chrono::Utc::now(),
                },
            )
            .await;
        engine
            .set_state(
                "Sent",
                SyncState {
                    folder: "Sent".to_string(),
                    last_uid: 200,
                    uid_validity: 3,
                    uid_next: 201,
                    last_sync: chrono::Utc::now(),
                },
            )
            .await;

        assert!(engine.get_state("INBOX").await.is_some());
        assert!(engine.get_state("Sent").await.is_some());
        assert!(engine.get_state("Drafts").await.is_none());

        assert_eq!(engine.get_state("INBOX").await.unwrap().last_uid, 50);
        assert_eq!(engine.get_state("Sent").await.unwrap().last_uid, 200);
    }

    #[tokio::test]
    async fn test_sync_engine_offline_queue_operations() {
        let engine = MailSyncEngine::new(SyncConfig::default());
        assert_eq!(engine.queue_depth().await, 0);

        let id1 = engine
            .enqueue_offline(OfflineOperation::MarkSeen {
                folder: "INBOX".to_string(),
                uid: 1,
            })
            .await
            .unwrap();
        let id2 = engine
            .enqueue_offline(OfflineOperation::Delete {
                folder: "INBOX".to_string(),
                uid: 2,
            })
            .await
            .unwrap();
        let id3 = engine
            .enqueue_offline(OfflineOperation::Send(SendMailRequest {
                to: vec![MailAddress {
                    name: None,
                    address: "a@b.com".to_string(),
                }],
                subject: "Offline Send".to_string(),
                text_body: Some("queued".to_string()),
                ..Default::default()
            }))
            .await
            .unwrap();

        assert_eq!(engine.queue_depth().await, 3);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[tokio::test]
    async fn test_sync_config_defaults() {
        let config = SyncConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.offline_queue_capacity, 500);
        assert_eq!(config.retry.max_retries, 5);
        assert_eq!(config.retry.base_delay, Duration::from_secs(1));
        assert_eq!(config.idle.poll_interval, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_sync_config_custom() {
        let config = SyncConfig {
            batch_size: 200,
            retry: RetryConfig {
                max_retries: 10,
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(30),
                multiplier: 1.5,
            },
            idle: IdleConfig {
                poll_interval: Duration::from_secs(15),
                keepalive_interval: Duration::from_secs(120),
                max_poll_failures: 5,
            },
            offline_queue_capacity: 1000,
        };
        assert_eq!(config.batch_size, 200);
        assert_eq!(config.retry.max_retries, 10);
        assert_eq!(config.idle.poll_interval, Duration::from_secs(15));
    }

    // ========================================================================
    // 4. OfflineQueue 集成测试
    // ========================================================================

    #[test]
    fn test_offline_queue_enqueue_and_drain() {
        let mut queue = OfflineQueue::new(100);
        assert!(queue.is_empty());

        queue
            .enqueue(OfflineOperation::MarkSeen {
                folder: "INBOX".to_string(),
                uid: 1,
            })
            .unwrap();
        queue
            .enqueue(OfflineOperation::MarkFlagged {
                folder: "INBOX".to_string(),
                uid: 5,
            })
            .unwrap();
        queue
            .enqueue(OfflineOperation::Move {
                folder: "INBOX".to_string(),
                uid: 10,
                target: "Archive".to_string(),
            })
            .unwrap();

        assert_eq!(queue.len(), 3);

        let ops = queue.drain_all();
        assert_eq!(ops.len(), 3);
        assert!(queue.is_empty());

        assert!(matches!(
            ops[0].operation,
            OfflineOperation::MarkSeen { uid: 1, .. }
        ));
        assert!(matches!(
            ops[1].operation,
            OfflineOperation::MarkFlagged { uid: 5, .. }
        ));
        assert!(matches!(
            ops[2].operation,
            OfflineOperation::Move { uid: 10, .. }
        ));
    }

    #[test]
    fn test_offline_queue_capacity_eviction() {
        let mut queue = OfflineQueue::new(3);
        for i in 1..=5 {
            queue
                .enqueue(OfflineOperation::MarkSeen {
                    folder: "INBOX".to_string(),
                    uid: i,
                })
                .unwrap();
        }
        assert_eq!(queue.len(), 3);
        let ops = queue.drain_all();
        let uids: Vec<u32> = ops
            .iter()
            .map(|op| match &op.operation {
                OfflineOperation::MarkSeen { uid, .. } => *uid,
                _ => 0,
            })
            .collect();
        assert_eq!(uids, vec![3, 4, 5]);
    }

    #[test]
    fn test_offline_queue_requeue_increments_retries() {
        let mut queue = OfflineQueue::new(10);
        let id = queue
            .enqueue(OfflineOperation::Delete {
                folder: "INBOX".to_string(),
                uid: 99,
            })
            .unwrap();

        let ops = queue.drain_all();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].id, id);
        assert_eq!(ops[0].retries, 0);

        queue.requeue(ops.into_iter().next().unwrap());
        let ops = queue.drain_all();
        assert_eq!(ops[0].retries, 1);
    }

    #[test]
    fn test_offline_queue_requeue_max_retries_drops() {
        let mut queue = OfflineQueue::new(10);
        let mut op = {
            queue
                .enqueue(OfflineOperation::MarkSeen {
                    folder: "INBOX".to_string(),
                    uid: 1,
                })
                .unwrap();
            queue.drain_all().into_iter().next().unwrap()
        };
        op.retries = 4;
        queue.requeue(op);
        let op = queue.drain_all().into_iter().next().unwrap();
        queue.requeue(op);
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_offline_queue_clear() {
        let mut queue = OfflineQueue::new(10);
        for i in 1..=5 {
            queue
                .enqueue(OfflineOperation::MarkSeen {
                    folder: "INBOX".to_string(),
                    uid: i,
                })
                .unwrap();
        }
        assert_eq!(queue.len(), 5);
        queue.clear();
        assert!(queue.is_empty());
    }

    // ========================================================================
    // 5. Error Recovery 测试
    // ========================================================================

    #[test]
    fn test_retry_config_delay_exponential_backoff() {
        let config = RetryConfig {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_retries: 5,
        };
        assert_eq!(config.delay_for(0), Duration::from_secs(1));
        assert_eq!(config.delay_for(1), Duration::from_secs(2));
        assert_eq!(config.delay_for(2), Duration::from_secs(4));
        assert_eq!(config.delay_for(3), Duration::from_secs(8));
        assert_eq!(config.delay_for(4), Duration::from_secs(16));
    }

    #[test]
    fn test_retry_config_delay_cap_at_max() {
        let config = RetryConfig {
            base_delay: Duration::from_secs(10),
            max_delay: Duration::from_secs(30),
            multiplier: 3.0,
            max_retries: 5,
        };
        assert_eq!(config.delay_for(0), Duration::from_secs(10));
        assert_eq!(config.delay_for(1), Duration::from_secs(30));
        assert_eq!(config.delay_for(2), Duration::from_secs(30));
        assert_eq!(config.delay_for(3), Duration::from_secs(30));
    }

    #[test]
    fn test_retry_config_default_values() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert!((config.multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_mail_error_display_messages() {
        let cases: Vec<(MailError, &str)> = vec![
            (MailError::Imap("bad".into()), "IMAP error: bad"),
            (MailError::Smtp("relay".into()), "SMTP error: relay"),
            (MailError::Tls("handshake".into()), "TLS error: handshake"),
            (
                MailError::Auth("bad pass".into()),
                "Authentication failed: bad pass",
            ),
            (
                MailError::MimeParse("malformed".into()),
                "MIME parse error: malformed",
            ),
            (MailError::OAuth("expired".into()), "OAuth error: expired"),
            (
                MailError::Timeout("connect".into()),
                "Connection timeout: connect",
            ),
            (
                MailError::AccountNotFound("u@x".into()),
                "Account not found: u@x",
            ),
            (
                MailError::FolderNotFound("Sent".into()),
                "Folder not found: Sent",
            ),
            (MailError::MessageNotFound(42), "Message not found: 42"),
            (
                MailError::Sync("uid mismatch".into()),
                "Sync error: uid mismatch",
            ),
            (MailError::Queue("full".into()), "Queue error: full"),
            (
                MailError::Config("missing".into()),
                "Configuration error: missing",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn test_mail_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let mail_err: MailError = io_err.into();
        assert!(matches!(mail_err, MailError::Io(_)));
        assert!(mail_err.to_string().contains("refused"));
    }

    #[test]
    fn test_mail_error_serde_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let mail_err: MailError = json_err.into();
        assert!(matches!(mail_err, MailError::Serialization(_)));
    }

    #[test]
    fn test_mail_error_anyhow_conversion() {
        let err = anyhow::anyhow!("generic error");
        let mail_err: MailError = err.into();
        assert!(matches!(mail_err, MailError::Other(_)));
    }

    #[test]
    fn test_mail_error_capability_conversion_auth() {
        let mail_err = MailError::Auth("xoauth2 failed".into());
        let cap_err: neotrix::unified::layers::action::traits::CapabilityError = mail_err.into();
        assert!(matches!(
            cap_err,
            neotrix::unified::layers::action::traits::CapabilityError::Authentication(_)
        ));
    }

    #[test]
    fn test_mail_error_capability_conversion_timeout() {
        let mail_err = MailError::Timeout("30s".into());
        let cap_err: neotrix::unified::layers::action::traits::CapabilityError = mail_err.into();
        assert!(matches!(
            cap_err,
            neotrix::unified::layers::action::traits::CapabilityError::Timeout(_)
        ));
    }

    #[test]
    fn test_mail_error_capability_conversion_not_found() {
        let cap_err: neotrix::unified::layers::action::traits::CapabilityError =
            MailError::AccountNotFound("u@x".into()).into();
        assert!(matches!(
            cap_err,
            neotrix::unified::layers::action::traits::CapabilityError::NotFound(_)
        ));
    }

    #[test]
    fn test_mail_error_capability_conversion_imap_fallback() {
        let cap_err: neotrix::unified::layers::action::traits::CapabilityError =
            MailError::Imap("server error".into()).into();
        assert!(matches!(
            cap_err,
            neotrix::unified::layers::action::traits::CapabilityError::Internal(_)
        ));
    }

    // ========================================================================
    // 6. AccountConfig 序列化集成测试
    // ========================================================================

    #[test]
    fn test_account_config_password_roundtrip() {
        let config = test_config();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let decoded: AccountConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, config.id);
        assert_eq!(decoded.email, config.email);
        assert_eq!(decoded.imap.port, 993);
        assert_eq!(decoded.smtp.port, 587);
        match decoded.auth {
            AuthMethod::Password { password } => assert_eq!(password, "test_password"),
            _ => panic!("expected Password auth"),
        }
    }

    #[test]
    fn test_account_config_oauth_roundtrip() {
        let config = test_config_oauth();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let decoded: AccountConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, "oauth_account");
        assert_eq!(decoded.smtp.port, 465);
        match decoded.auth {
            AuthMethod::OAuth2 {
                access_token,
                refresh_token,
                client_id,
                client_secret,
            } => {
                assert_eq!(access_token, "ya29.test_token");
                assert_eq!(refresh_token.as_deref(), Some("1//test_refresh"));
                assert_eq!(client_id, "client_id_123");
                assert_eq!(client_secret, "client_secret_456");
            }
            _ => panic!("expected OAuth2 auth"),
        }
    }

    // ========================================================================
    // 7. SearchQuery 集成测试
    // ========================================================================

    #[test]
    fn test_search_query_roundtrip() {
        let query = SearchQuery {
            keyword: Some("invoice".to_string()),
            from: Some("vendor@corp.com".to_string()),
            to: None,
            subject: Some("Payment Due".to_string()),
            after: Some(chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap()),
            before: Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            seen: Some(false),
            has_attachments: Some(true),
            folder: Some("INBOX".to_string()),
            limit: Some(25),
        };
        let json = serde_json::to_string(&query).unwrap();
        let decoded: SearchQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.keyword.as_deref(), Some("invoice"));
        assert_eq!(decoded.from.as_deref(), Some("vendor@corp.com"));
        assert_eq!(decoded.limit, Some(25));
        assert_eq!(decoded.seen, Some(false));
    }

    #[test]
    fn test_search_query_default_is_empty() {
        let q = SearchQuery::default();
        assert!(q.keyword.is_none());
        assert!(q.from.is_none());
        assert!(q.to.is_none());
        assert!(q.subject.is_none());
        assert!(q.after.is_none());
        assert!(q.before.is_none());
        assert!(q.seen.is_none());
        assert!(q.has_attachments.is_none());
        assert!(q.folder.is_none());
        assert!(q.limit.is_none());
    }

    // ========================================================================
    // 8. MailMessage / MailContent 序列化集成测试
    // ========================================================================

    #[test]
    fn test_mail_message_serialization_roundtrip() {
        let msg = MailMessage {
            uid: 42,
            message_id: Some("<abc@neotrix>".to_string()),
            subject: "Test Subject".to_string(),
            from: vec![MailAddress {
                name: Some("Alice".to_string()),
                address: "alice@neotrix.dev".to_string(),
            }],
            to: vec![MailAddress {
                name: None,
                address: "bob@neotrix.dev".to_string(),
            }],
            cc: vec![],
            bcc: vec![],
            date: Some(chrono::Utc::now()),
            seen: true,
            flagged: false,
            draft: false,
            deleted: false,
            has_attachments: true,
            size: Some(12345),
            preview: Some("Preview text".to_string()),
            keywords: vec!["important".to_string()],
            raw: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: MailMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.uid, 42);
        assert_eq!(decoded.subject, "Test Subject");
        assert!(decoded.seen);
        assert!(decoded.has_attachments);
        assert_eq!(decoded.size, Some(12345));
        assert_eq!(decoded.keywords.len(), 1);
        assert!(decoded.raw.is_none());
    }

    #[test]
    fn test_mail_content_serialization_roundtrip() {
        let content = MailContent {
            text: Some("plain text body".to_string()),
            html: Some("<p>html body</p>".to_string()),
            attachments: vec![MailAttachment {
                filename: "doc.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 2048,
                content_id: None,
                inline: false,
                section: "2".to_string(),
            }],
            inline_images: vec![],
        };

        let json = serde_json::to_string(&content).unwrap();
        let decoded: MailContent = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.text.as_deref(), Some("plain text body"));
        assert_eq!(decoded.html.as_deref(), Some("<p>html body</p>"));
        assert_eq!(decoded.attachments.len(), 1);
        assert_eq!(decoded.attachments[0].filename, "doc.pdf");
    }

    // ========================================================================
    // 9. SyncResult / ReplayResult 测试
    // ========================================================================

    #[test]
    fn test_sync_result_construction() {
        let result = SyncResult {
            folder: "INBOX".to_string(),
            new_messages: vec![test_mail_message(1)],
            validity_reset: false,
            old_validity: 1,
            new_validity: 1,
        };
        assert_eq!(result.folder, "INBOX");
        assert_eq!(result.new_messages.len(), 1);
        assert!(!result.validity_reset);
    }

    #[test]
    fn test_replay_result_construction() {
        let result = ReplayResult {
            succeeded: 10,
            failed: 2,
            remaining: 3,
        };
        assert_eq!(result.succeeded, 10);
        assert_eq!(result.failed, 2);
        assert_eq!(result.remaining, 3);
    }

    // ========================================================================
    // 10. EmailProvider 集成测试
    // ========================================================================

    #[test]
    fn test_email_provider_creation() {
        let provider = EmailProvider::new(
            "smtp.neotrix.dev",
            587,
            "imap.neotrix.dev",
            993,
            "test@neotrix.dev",
            "password",
            "test@neotrix.dev",
        );
        assert!(!provider.smtp_host.is_empty());
        assert!(!provider.imap_host.is_empty());
    }

    #[test]
    fn test_email_provider_smtp_only() {
        let provider = EmailProvider::new_smtp_only(
            "smtp.neotrix.dev",
            587,
            "test@neotrix.dev",
            "password",
            "test@neotrix.dev",
        );
        assert_eq!(provider.smtp_port, 587);
    }

    // ========================================================================
    // 11. SyncEvent 枚举测试
    // ========================================================================

    #[test]
    fn test_sync_event_variants() {
        let msg = test_mail_message(1);

        let events = vec![
            SyncEvent::MessagesArrived {
                folder: "INBOX".to_string(),
                messages: vec![msg.clone()],
            },
            SyncEvent::FlagsChanged {
                folder: "INBOX".to_string(),
                uid: 1,
                flags: vec!["\\Seen".to_string()],
            },
            SyncEvent::StateUpdated {
                folder: "INBOX".to_string(),
                state: SyncState {
                    folder: "INBOX".to_string(),
                    last_uid: 1,
                    uid_validity: 1,
                    uid_next: 2,
                    last_sync: chrono::Utc::now(),
                },
            },
            SyncEvent::ValidityReset {
                folder: "INBOX".to_string(),
                old_validity: 1,
                new_validity: 2,
            },
            SyncEvent::SyncError {
                folder: "INBOX".to_string(),
                error: "timeout".to_string(),
                will_retry: true,
            },
            SyncEvent::Disconnected {
                reason: "network".to_string(),
            },
            SyncEvent::Reconnected,
            SyncEvent::IdleStatus { active: true },
        ];

        assert_eq!(events.len(), 8);
        assert!(matches!(&events[0], SyncEvent::MessagesArrived { .. }));
        assert!(matches!(
            &events[4],
            SyncEvent::SyncError {
                will_retry: true,
                ..
            }
        ));
        assert!(matches!(&events[6], SyncEvent::Reconnected));
        assert!(matches!(&events[7], SyncEvent::IdleStatus { active: true }));
    }

    // ========================================================================
    // 12. ImapFolderInfo 测试
    // ========================================================================

    #[test]
    fn test_imap_folder_info_construction() {
        let info = ImapFolderInfo {
            uid_validity: Some(42),
            uid_next: Some(1001),
            exists: 500,
            recent: 3,
        };
        assert_eq!(info.uid_validity, Some(42));
        assert_eq!(info.uid_next, Some(1001));
        assert_eq!(info.exists, 500);
        assert_eq!(info.recent, 3);
    }

    #[test]
    fn test_imap_folder_info_none_validity() {
        let info = ImapFolderInfo {
            uid_validity: None,
            uid_next: None,
            exists: 0,
            recent: 0,
        };
        assert!(info.uid_validity.is_none());
        assert!(info.uid_next.is_none());
    }

    // ========================================================================
    // 13. 完整 MIME 往返测试 (构建 → 解析 → 验证)
    // ========================================================================

    #[test]
    fn test_mime_roundtrip_build_and_parse() {
        let request = SendMailRequest {
            to: vec![MailAddress {
                name: Some("Test User".to_string()),
                address: "test@neotrix.dev".to_string(),
            }],
            subject: "Roundtrip Test".to_string(),
            text_body: Some("Plain text body".to_string()),
            html_body: Some("<p>HTML body</p>".to_string()),
            attachments: vec![AttachmentData {
                filename: "test.txt".to_string(),
                mime_type: "text/plain".to_string(),
                data: b"attachment content".to_vec(),
            }],
            ..Default::default()
        };

        let raw = neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message(
            &request,
        )
        .unwrap();
        let msg = MimeMessage::parse(&raw).unwrap();
        let content = msg.to_mail_content();

        assert!(content.text.is_some());
        assert!(content.html.is_some());
        assert_eq!(content.attachments.len(), 1);
        assert_eq!(content.attachments[0].filename, "test.txt");
    }

    // ========================================================================
    // 14. Encoding 测试 (UTF-8, ISO-8859-1)
    // ========================================================================

    #[test]
    fn test_mime_parse_utf8_subject() {
        let raw = b"From: test@example.com\r\n\
                     To: user@example.com\r\n\
                     Subject: =?utf-8?B?5L2g5aW977yaVGVzdA==?=\r\n\
                     Content-Type: text/plain; charset=utf-8\r\n\
                     \r\n\
                     body\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        assert!(msg.headers.subject.is_some());
        let subject = msg.headers.subject.unwrap();
        assert!(!subject.is_empty());
    }

    #[test]
    fn test_mime_parse_iso_8859_1_content() {
        let raw = b"From: test@example.com\r\n\
                     To: user@example.com\r\n\
                     Subject: Latin\r\n\
                     Content-Type: text/plain; charset=iso-8859-1\r\n\
                     Content-Transfer-Encoding: 7bit\r\n\
                     \r\n\
                     Caf\xe9\r\n";

        let msg = MimeMessage::parse(raw).unwrap();
        if let MimeBody::Single(part) = &msg.body {
            let text = String::from_utf8_lossy(&part.content);
            assert!(text.contains("Caf"));
        } else {
            panic!("expected single part");
        }
    }

    // ========================================================================
    // 15. SendMailRequest 默认值测试
    // ========================================================================

    #[test]
    fn test_send_mail_request_default_is_empty() {
        let req = SendMailRequest::default();
        assert!(req.to.is_empty());
        assert!(req.cc.is_empty());
        assert!(req.bcc.is_empty());
        assert!(req.subject.is_empty());
        assert!(req.text_body.is_none());
        assert!(req.html_body.is_none());
        assert!(req.attachments.is_empty());
        assert!(req.in_reply_to.is_none());
        assert!(req.references.is_empty());
        assert!(req.headers.is_empty());
    }

    // ========================================================================
    // 16. MailFolder 序列化测试
    // ========================================================================

    #[test]
    fn test_mail_folder_serialization() {
        let folder = MailFolder {
            name: "INBOX".to_string(),
            delimiter: Some(".".to_string()),
            attributes: vec!["\\hasnochildren".to_string(), "\\inbox".to_string()],
            unread_count: 12,
            total_count: 500,
        };
        let json = serde_json::to_string(&folder).unwrap();
        let decoded: MailFolder = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "INBOX");
        assert_eq!(decoded.unread_count, 12);
        assert_eq!(decoded.total_count, 500);
        assert_eq!(decoded.attributes.len(), 2);
    }
}
