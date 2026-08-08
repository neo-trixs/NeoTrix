//! 临时脚本: 补跑 guji 域节点嵌入 (hash-kernel 零依赖, 只处理 domain=guji 缺嵌入节点)
//! 用法: cargo run -p neotrix --example guji_embed_fill
use std::path::PathBuf;

use neotrix::neotrix::nt_memory_kb::nt_memory_embed::{
    EmbedMode, EmbeddingConfig, build_node_text, embed_text, store_embedding,
};

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/neo".into());
    let db_path = PathBuf::from(&home).join(".neotrix").join("knowledge.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open KB");

    // guji 缺嵌入节点
    let missing: Vec<(String,)> = conn
        .prepare(
            "SELECT n.id FROM nodes n WHERE n.domain='guji' \
             AND n.id NOT IN (SELECT node_id FROM embeddings)",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?,)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    println!("guji 缺嵌入节点: {}", missing.len());

    let dim = 384usize;
    let mut done = 0usize;
    let mut failed = 0usize;
    for (id,) in &missing {
        let node: Option<(String, Option<String>, Option<String>)> = conn
            .prepare("SELECT title, summary, content FROM nodes WHERE id=?1")
            .unwrap()
            .query_row([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .ok();
        if let Some((title, summary, content)) = node {
            let text = build_node_text(&title, summary.as_deref(), content.as_deref());
            // 直接用 hash-kernel (本地零依赖)
            let cfg = EmbeddingConfig {
                mode: EmbedMode::Local,
                dimension: dim,
                ..Default::default()
            };
            match embed_text(&cfg, &text) {
                Ok(vec) => {
                    if store_embedding(&conn, id, &vec, "hash-kernel-384").is_ok() {
                        done += 1;
                    } else {
                        failed += 1;
                    }
                }
                Err(e) => {
                    failed += 1;
                    if failed <= 3 {
                        println!("embed err {}: {}", id, e);
                    }
                }
            }
        } else {
            failed += 1;
        }
    }
    println!("完成: {} 嵌入, {} 失败", done, failed);
    conn.close();
}
