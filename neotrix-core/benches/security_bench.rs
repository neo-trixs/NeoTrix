use criterion::{criterion_group, criterion_main, Criterion, black_box};
use neotrix::l3_embodiment::nt_shield::compliance::framework::ComplianceFramework;
use neotrix::l3_embodiment::nt_shield::compliance::evaluator::{self, ComplianceStatus};
use neotrix::l3_embodiment::nt_shield::scanners::secret_scanner::{SecretScanner, SecretDetector};
use std::io::Write;
use tempfile::tempdir;

fn bench_secret_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_scan");

    group.bench_function("scan_file_with_secrets", |b| {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.env");
        let mut f = std::fs::File::create(&file_path).unwrap();
        writeln!(
            f,
            "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"\n\
             GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456\n\
             DATABASE_URL=postgres://admin:pass@db.example.com/prod\n\
             token = \"eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc123\"\n\
             sk-proj1234567890abcdefghijklmnop"
        )
        .unwrap();
        let scanner = SecretScanner::new();

        b.iter(|| {
            black_box(scanner.scan_file(black_box(&file_path)));
        });
    });

    group.bench_function("scan_clean_file", |b| {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut f = std::fs::File::create(&file_path).unwrap();
        for i in 0..100 {
            writeln!(f, "fn function_{i}() -> i32 {{ {i} }}").unwrap();
        }
        let scanner = SecretScanner::new();

        b.iter(|| {
            black_box(scanner.scan_file(black_box(&file_path)));
        });
    });

    group.bench_function("scan_directory_mixed", |b| {
        b.iter_batched(
            || {
                let dir = tempdir().unwrap();
                for i in 0..20 {
                    let path = dir.path().join(format!("file_{i}.toml"));
                    let mut f = std::fs::File::create(&path).unwrap();
                    if i % 5 == 0 {
                        writeln!(
                            f,
                            "key = \"AKIAIOSFODNN7EXAMPLE{}\"",
                            "X".repeat(12)
                        )
                        .unwrap();
                    } else {
                        writeln!(f, "setting = \"value_{i}\"").unwrap();
                    }
                }
                let scanner = SecretScanner::new();
                (scanner, dir)
            },
            |(scanner, dir)| {
                black_box(scanner.scan_directory(black_box(dir.path()), &["toml"]));
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("detector_detect_inline", |b| {
        let detector = SecretDetector::new();
        let content = "aws_access_key_id = \"AKIAIOSFODNN7EXAMPLE\"\n\
                       GITHUB_TOKEN=ghp_abcdefghijklmnopqrstuvwxyz123456\n\
                       simple text without secrets\n\
                       DATABASE_URL=postgres://admin:pass@localhost/db";
        b.iter(|| {
            black_box(detector.detect(black_box(content), "test.txt"));
        });
    });

    group.finish();
}

fn bench_llm_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("llm_scan");

    group.bench_function("detector_patterns_compiled", |b| {
        let content = "api_key = \"sk-projabcdefghijklmnopqrstuvwxyz1234567890\"\n\
                       password = \"super_secret_password_123\"\n\
                       Authorization: Bearer eyJhbGciOiJSUzI1NiJ9\n\
                       -----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAK...";
        b.iter(|| {
            let detector = SecretDetector::new();
            black_box(detector.detect(black_box(content), "llm_output.txt"));
        });
    });

    group.bench_function("scan_large_content", |b| {
        let mut content = String::with_capacity(50_000);
        for i in 0..1000 {
            content.push_str(&format!("Line {i}: some text with data value_{i}\n"));
            if i % 100 == 0 {
                content.push_str(&format!(
                    "secret_key = \"AKIA{}\"\n",
                    "B".repeat(16)
                ));
            }
        }
        let detector = SecretDetector::new();

        b.iter(|| {
            black_box(detector.detect(black_box(&content), "large.txt"));
        });
    });

    group.finish();
}

fn bench_compliance_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("compliance_check");

    group.bench_function("owasp_top10_full_eval", |b| {
        let framework = ComplianceFramework::owasp_top10();
        let checks: Vec<(String, ComplianceStatus, String)> = vec![
            ("OWASP-A01".into(), ComplianceStatus::Pass, "Enforced".into()),
            ("OWASP-A02".into(), ComplianceStatus::Pass, "TLS 1.3".into()),
            ("OWASP-A03".into(), ComplianceStatus::Pass, "Sanitized".into()),
            ("OWASP-A04".into(), ComplianceStatus::Partial, "Partial review".into()),
            ("OWASP-A05".into(), ComplianceStatus::Fail, "Default creds".into()),
            ("OWASP-A06".into(), ComplianceStatus::Pass, "Updated".into()),
            ("OWASP-A07".into(), ComplianceStatus::Pass, "MFA enabled".into()),
            ("OWASP-A08".into(), ComplianceStatus::Partial, "Sig verification".into()),
            ("OWASP-A09".into(), ComplianceStatus::Pass, "Logging OK".into()),
            ("OWASP-A10".into(), ComplianceStatus::Pass, "No SSRF".into()),
        ];

        b.iter(|| {
            black_box(evaluator::evaluate(
                black_box(&framework),
                black_box(&checks),
            ));
        });
    });

    group.bench_function("asvs_full_eval", |b| {
        let framework = ComplianceFramework::asvs();
        let checks: Vec<(String, ComplianceStatus, String)> = framework
            .requirements
            .iter()
            .map(|r| {
                (
                    r.id.clone(),
                    ComplianceStatus::Pass,
                    "Verified".to_string(),
                )
            })
            .collect();

        b.iter(|| {
            black_box(evaluator::evaluate(
                black_box(&framework),
                black_box(&checks),
            ));
        });
    });

    group.bench_function("owasp_framework_construction", |b| {
        b.iter(|| {
            black_box(ComplianceFramework::owasp_top10());
        });
    });

    group.bench_function("asvs_framework_construction", |b| {
        b.iter(|| {
            black_box(ComplianceFramework::asvs());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_secret_scan,
    bench_llm_scan,
    bench_compliance_check,
);
criterion_main!(benches);
