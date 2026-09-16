//! nt_io_mail 模块性能基准测试
//!
//! 覆盖: MIME 解析、SMTP 构建、IMAP 命令、内存占用
//!
//! NOTE: nt_io_mail module not yet migrated to 6-layer architecture.
//! Benchmark stub — all functions are no-ops.

use criterion::{criterion_group, criterion_main};

fn stub(_c: &mut criterion::Criterion) {}

criterion_group!(benches, stub);
criterion_main!(benches);
