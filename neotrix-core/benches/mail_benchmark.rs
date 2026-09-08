//! nt_io_mail 模块性能基准测试
//!
//! 覆盖: MIME 解析、SMTP 构建、IMAP 命令、内存占用

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neotrix::unified::layers::action::nt_io::nt_io_mail::imap::{
    decode_mailbox_name, parse_rfc5322_headers,
};
use neotrix::unified::layers::action::nt_io::nt_io_mail::mime::MimeMessage;
use neotrix::unified::layers::action::nt_io::nt_io_mail::smtp::build_rfc5322_message;
use neotrix::unified::layers::action::nt_io::nt_io_mail::{
    AttachmentData, MailAddress, MailMessage, SendMailRequest,
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
//  Helper: 生成不同大小的 RFC 5322 邮件原始字节
// ---------------------------------------------------------------------------

/// 生成小邮件 (约 1 KB)
fn small_email_raw() -> Vec<u8> {
    let body = "Hello, this is a test email.\n".repeat(30);
    format!(
        "From: sender@example.com\r\n\
         To: receiver@example.com\r\n\
         Subject: Small Email Benchmark\r\n\
         Date: Mon, 01 Jan 2024 12:00:00 +0800\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {body}"
    )
    .into_bytes()
}

/// 生成中等邮件 (约 100 KB)
fn medium_email_raw() -> Vec<u8> {
    let line = "The quick brown fox jumps over the lazy dog. ".repeat(3);
    let body = format!("{}\n", line.repeat(50));
    format!(
        "From: sender@example.com\r\n\
         To: receiver@example.com\r\n\
         Subject: Medium Email Benchmark\r\n\
         Date: Mon, 01 Jan 2024 12:00:00 +0800\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {body}"
    )
    .into_bytes()
}

/// 生成大邮件 (约 1 MB+)
fn large_email_raw() -> Vec<u8> {
    let line = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
                Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. "
        .repeat(2);
    let body = format!("{}\n", line.repeat(300));
    format!(
        "From: sender@example.com\r\n\
         To: receiver@example.com\r\n\
         Subject: Large Email Benchmark\r\n\
         Date: Mon, 01 Jan 2024 12:00:00 +0800\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {body}"
    )
    .into_bytes()
}

/// 生成带附件的 multipart 邮件
fn email_with_attachment_raw() -> Vec<u8> {
    // 模拟 base64 编码的附件内容 (约 50 KB)
    let attachment_body: String = (0..700).map(|i| format!("{i:04x}")).collect();
    let b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        attachment_body.as_bytes(),
    );

    format!(
        "From: sender@example.com\r\n\
         To: receiver@example.com\r\n\
         Subject: Attachment Benchmark\r\n\
         Date: Mon, 01 Jan 2024 12:00:00 +0800\r\n\
         Content-Type: multipart/mixed; boundary=bench-boundary\r\n\
         \r\n\
         --bench-boundary\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         This email has an attachment.\r\n\
         --bench-boundary\r\n\
         Content-Type: application/octet-stream; name=\"data.bin\"\r\n\
         Content-Disposition: attachment; filename=\"data.bin\"\r\n\
         Content-Transfer-Encoding: base64\r\n\
         \r\n\
         {b64}\r\n\
         --bench-boundary--\r\n"
    )
    .into_bytes()
}

/// 生成带复杂头部的邮件 (用于头部解析基准)
fn complex_headers_raw() -> Vec<u8> {
    format!(
        "From: =?utf-8?B?5ZCN5bq2?= <sender@example.com>\r\n\
         To: receiver@example.com, \"Another User\" <user2@example.com>\r\n\
         Cc: cc1@example.com, cc2@example.com\r\n\
         Subject: =?utf-8?B?5L2g5aW9?=\r\n\
         Date: Mon, 01 Jan 2024 12:00:00 +0800\r\n\
         Message-ID: <abc123@neotrix>\r\n\
         In-Reply-To: <parent@neotrix>\r\n\
         References: <ref1@neotrix> <ref2@neotrix>\r\n\
         MIME-Version: 1.0\r\n\
         X-Priority: 1\r\n\
         X-Mailer: NeoTrix\r\n\
         X-Custom-Header: some-value\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         Body text"
    )
    .into_bytes()
}

/// 生成 UID set 字符串 (用于压缩算法基准)
fn uid_set_small() -> String {
    (1..=10)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn uid_set_medium() -> String {
    (1..=1000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn uid_set_large() -> String {
    (1..=10000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

// ---------------------------------------------------------------------------
//  1. MIME 解析基准
// ---------------------------------------------------------------------------

fn bench_mime_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("mime_parse");

    let small = small_email_raw();
    let medium = medium_email_raw();
    let large = large_email_raw();
    let attachment = email_with_attachment_raw();

    group.bench_function("small_1kb", |b| {
        b.iter(|| black_box(MimeMessage::parse(&small).unwrap()));
    });

    group.bench_function("medium_100kb", |b| {
        b.iter(|| black_box(MimeMessage::parse(&medium).unwrap()));
    });

    group.bench_function("large_1mb", |b| {
        b.iter(|| black_box(MimeMessage::parse(&large).unwrap()));
    });

    group.bench_function("with_attachment", |b| {
        b.iter(|| black_box(MimeMessage::parse(&attachment).unwrap()));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
//  2. SMTP 构建基准
// ---------------------------------------------------------------------------

fn bench_smtp_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("smtp_build");

    // 纯文本邮件
    let text_request = SendMailRequest {
        to: vec![MailAddress {
            name: Some("John Doe".to_string()),
            address: "john@example.com".to_string(),
        }],
        subject: "Benchmark Text Email".to_string(),
        text_body: Some("Hello, this is a benchmark email body.\n".repeat(10)),
        ..Default::default()
    };

    group.bench_function("plain_text", |b| {
        b.iter(|| black_box(build_rfc5322_message(black_box(&text_request)).unwrap()));
    });

    // HTML 邮件
    let html_request = SendMailRequest {
        to: vec![MailAddress {
            name: None,
            address: "john@example.com".to_string(),
        }],
        subject: "Benchmark HTML Email".to_string(),
        text_body: Some("Plain text fallback".to_string()),
        html_body: Some("<html><body><h1>Hello</h1><p>".repeat(50) + "</p></body></html>"),
        ..Default::default()
    };

    group.bench_function("html_with_text", |b| {
        b.iter(|| black_box(build_rfc5322_message(black_box(&html_request)).unwrap()));
    });

    // 带附件邮件
    let attach_request = SendMailRequest {
        to: vec![MailAddress {
            name: None,
            address: "john@example.com".to_string(),
        }],
        subject: "Benchmark Attachment Email".to_string(),
        text_body: Some("Please see attached file.".to_string()),
        attachments: vec![
            AttachmentData {
                filename: "document.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                data: vec![0x25; 10_000], // 10 KB fake PDF
            },
            AttachmentData {
                filename: "image.png".to_string(),
                mime_type: "image/png".to_string(),
                data: vec![0x89; 50_000], // 50 KB fake PNG
            },
        ],
        ..Default::default()
    };

    group.bench_function("with_attachments", |b| {
        b.iter(|| black_box(build_rfc5322_message(black_box(&attach_request)).unwrap()));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
//  3. IMAP 命令基准
// ---------------------------------------------------------------------------

fn bench_imap_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("imap_commands");

    // Modified UTF-7 编解码
    let simple_name = "INBOX";
    let utf7_name = "&Jjo-"; // ☺
    let mixed_name = "Sent & Received";

    group.bench_function("decode_utf7_simple", |b| {
        b.iter(|| black_box(decode_mailbox_name(black_box(simple_name))));
    });

    group.bench_function("decode_utf7_unicode", |b| {
        b.iter(|| black_box(decode_mailbox_name(black_box(utf7_name))));
    });

    group.bench_function("decode_utf7_mixed", |b| {
        b.iter(|| black_box(decode_mailbox_name(black_box(mixed_name))));
    });

    // 邮件头解析
    let simple_header =
        "Subject: Hello World\nFrom: Alice <alice@example.com>\nTo: bob@example.com";
    let complex_header = complex_headers_raw();
    let complex_header_str = String::from_utf8_lossy(&complex_header);

    group.bench_function("parse_headers_simple", |b| {
        b.iter_with_setup(
            || {
                let mut msg = MailMessage {
                    uid: 1,
                    message_id: None,
                    subject: String::new(),
                    from: vec![],
                    to: vec![],
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
                };
                msg
            },
            |mut msg| {
                black_box(parse_rfc5322_headers(black_box(simple_header), &mut msg));
            },
        );
    });

    group.bench_function("parse_headers_complex", |b| {
        b.iter_with_setup(
            || MailMessage {
                uid: 1,
                message_id: None,
                subject: String::new(),
                from: vec![],
                to: vec![],
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
            },
            |mut msg| {
                black_box(parse_rfc5322_headers(
                    black_box(&complex_header_str),
                    &mut msg,
                ));
            },
        );
    });

    // UID SET 处理基准 (模拟压缩前的解析)
    let uid_small = uid_set_small();
    let uid_medium = uid_set_medium();
    let uid_large = uid_set_large();

    group.bench_function("uid_set_parse_small_10", |b| {
        b.iter(|| {
            black_box(
                uid_small
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect::<Vec<u32>>(),
            )
        });
    });

    group.bench_function("uid_set_parse_medium_1000", |b| {
        b.iter(|| {
            black_box(
                uid_medium
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect::<Vec<u32>>(),
            )
        });
    });

    group.bench_function("uid_set_parse_large_10000", |b| {
        b.iter(|| {
            black_box(
                uid_large
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect::<Vec<u32>>(),
            )
        });
    });

    // UID SET 压缩 (连续范围合并)
    group.bench_function("uid_set_compress_medium", |b| {
        b.iter_with_setup(
            || (1..=1000u32).map(|i| i.to_string()).collect::<Vec<_>>(),
            |uids| {
                black_box(compress_uid_set(black_box(&uids)));
            },
        );
    });

    group.finish();
}

/// UID SET 压缩: 将连续 UID 合并为范围 (如 "1:10,15,20:25")
fn compress_uid_set(uids: &[String]) -> String {
    let mut nums: Vec<u32> = uids.iter().filter_map(|s| s.parse().ok()).collect();
    nums.sort_unstable();
    nums.dedup();

    if nums.is_empty() {
        return String::new();
    }

    let mut ranges: Vec<String> = Vec::new();
    let mut start = nums[0];
    let mut end = nums[0];

    for &n in &nums[1..] {
        if n == end + 1 {
            end = n;
        } else {
            if start == end {
                ranges.push(start.to_string());
            } else {
                ranges.push(format!("{start}:{end}"));
            }
            start = n;
            end = n;
        }
    }

    if start == end {
        ranges.push(start.to_string());
    } else {
        ranges.push(format!("{start}:{end}"));
    }

    ranges.join(",")
}

// ---------------------------------------------------------------------------
//  4. 内存使用基准
// ---------------------------------------------------------------------------

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // 单个邮件对象内存占用
    group.bench_function("mail_message_size", |b| {
        b.iter(|| black_box(std::mem::size_of::<MailMessage>()));
    });

    group.bench_function("send_mail_request_size", |b| {
        b.iter(|| black_box(std::mem::size_of::<SendMailRequest>()));
    });

    // 批量邮件处理内存 — 构造 1000 封 MailMessage 测量总分配
    group.bench_function("batch_1000_mail_messages", |b| {
        b.iter(|| {
            let msgs: Vec<MailMessage> = (0..1000)
                .map(|i| MailMessage {
                    uid: i,
                    message_id: Some(format!("<msg{i}@bench>")),
                    subject: format!("Subject {i}"),
                    from: vec![MailAddress {
                        name: Some(format!("Sender {i}")),
                        address: format!("sender{i}@example.com"),
                    }],
                    to: vec![MailAddress {
                        name: None,
                        address: format!("recv{i}@example.com"),
                    }],
                    cc: vec![],
                    bcc: vec![],
                    date: None,
                    seen: i % 2 == 0,
                    flagged: false,
                    draft: false,
                    deleted: false,
                    has_attachments: i % 5 == 0,
                    size: Some((i * 1024) as u32),
                    preview: Some(format!("Preview text for message {i}")),
                    keywords: vec![],
                    raw: None,
                })
                .collect();
            black_box(msgs);
        });
    });

    // MIME 解析后内存 (解析 1000 封小邮件)
    group.bench_function("batch_1000_mime_parse", |b| {
        let small = small_email_raw();
        b.iter(|| {
            let msgs: Vec<MimeMessage> = (0..1000)
                .map(|_| MimeMessage::parse(&small).unwrap())
                .collect();
            black_box(msgs);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
//  criterion 入口
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_mime_parse,
    bench_smtp_build,
    bench_imap_commands,
    bench_memory_usage,
);
criterion_main!(benches);
