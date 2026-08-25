//! batch_ingest_nomad — 批量 URL 摄取示例（存根）
//!
//! 原实现依赖 nt_http::download_to_file（已私有化）与 test_conn（已移除）。
//! 完整批量摄取功能已迁移至 NT-WORLD UnifiedCrawler 管线：
//!   cargo run --bin neotrix-kb-crawl
//!
//! 本文件保留 Cargo.toml [[example]] 注册占位。

fn main() {
    println!("batch_ingest_nomad: 已迁移至 neotrix-kb-crawl daemon");
    println!("用法: cargo run --bin neotrix-kb-crawl -- --help");
}
