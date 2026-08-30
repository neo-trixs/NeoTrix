//! KB 觉醒管线 — 纯 Rust 实现
//! 用法: cargo run --example kb_awakening -p neotrix
#![forbid(unsafe_code)]

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use neotrix::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_embed;
use neotrix::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    println!("🧠 KB Awakening Pipeline");
    println!("{}", "=".repeat(50));

    let kb = KnowledgeBase::open(None)?;
    let guard = kb.raw_conn()?;
    let conn = &*guard;

    // ── 状态 ──
    let total_nodes: i64 = conn.query_row("SELECT count(*) FROM nodes", [], |r| r.get(0))?;
    let total_edges: i64 = conn.query_row("SELECT count(*) FROM edges", [], |r| r.get(0))?;
    println!("状态: {} nodes, {} edges", total_nodes, total_edges);

    // ═══ E1: Embedding ═══
    println!("\n[E1] Embedding Generation");
    let missing = nt_memory_embed::find_nodes_missing_embeddings(conn)?;
    println!("  缺少 embedding: {}", missing.len());
    
    if !missing.is_empty() {
        let mut texts: Vec<String> = Vec::new();
        let mut ids: Vec<String> = Vec::new();
        
        for nid in &missing {
            let result: Result<(String, Option<String>, Option<String>), _> = conn.query_row(
                "SELECT title, summary, content FROM nodes WHERE id = ?1",
                [nid.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            );
            if let Ok((title, summary, content)) = result {
                let text = nt_memory_embed::build_node_text(
                    &title, summary.as_deref(), content.as_deref(),
                );
                texts.push(text);
                ids.push(nid.clone());
            }
        }
        
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        const DIM: usize = 256;
        let vectors = nt_memory_embed::local_embed_texts(&text_refs, DIM);
        
        let stored = vectors.iter()
            .zip(ids.iter())
            .filter(|(vec, nid)| {
                nt_memory_embed::store_embedding(conn, nid, vec, "hash-kernel-v1").is_ok()
            })
            .count();
        println!("  新生成: {}/{}", stored, ids.len());
    }

    let embeddings = nt_memory_embed::load_all_embeddings(conn)?;
    println!("  总 embeddings: {}", embeddings.len());

    // ═══ E2: 语义边生成 ═══
    println!("\n[E2] Semantic Edge Generation");
    
    let vec_map: HashMap<String, Vec<f32>> = embeddings.iter().cloned().collect();
    
    let mut existing: HashSet<(String, String)> = HashSet::new();
    {
        let mut stmt = conn.prepare("SELECT source_id, target_id FROM edges")?;
        let rows = stmt.query_map([], |r| Ok((
            r.get::<_, String>(0)?, r.get::<_, String>(1)?
        )))?;
        for row in rows {
            let (s, t) = row?;
            existing.insert((s.clone(), t.clone()));
            existing.insert((t, s));
        }
    }

    let nids: Vec<&String> = vec_map.keys().collect();
    let now_ts: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs() as i64;

    let mut new_edges: Vec<(String, String, String, String, f64, String, i64)> = Vec::new();
    
    for i in 0..nids.len() {
        if new_edges.len() >= 5000 { break; }
        let va = &vec_map[nids[i]];
        let dot_a: f32 = va.iter().map(|x| x * x).sum();
        let na = dot_a.sqrt();
        if na == 0.0 { continue; }
        
        for j in (i + 1)..nids.len() {
            if existing.contains(&(nids[i].clone(), nids[j].clone()))
                || existing.contains(&(nids[j].clone(), nids[i].clone())) {
                continue;
            }
            
            let vb = &vec_map[nids[j]];
            let sim = nt_memory_embed::cosine_similarity(va, vb);
            if sim <= 0.25 { continue; }
            
            let rel = if sim > 0.35 { "supports" } else { "related_to" };
            new_edges.push((
                uuid::Uuid::new_v4().to_string(),
                nids[i].clone(), nids[j].clone(),
                rel.to_string(),
                ((0.4 + sim as f64 * 0.5).min(0.95)),
                format!("emb-sim:{:.3}", sim),
                now_ts,
            ));
        }
    }

    let mut inserted: u64 = 0;
    for &(ref id, ref src, ref tgt, ref rel, w, ref desc, ts) in &new_edges {
        match conn.execute(
            "INSERT OR IGNORE INTO edges (id,source_id,target_id,relation_type,weight,description,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            rusqlite::params![id, src, tgt, rel, w, desc, ts],
        ) {
            Ok(_) => inserted += 1,
            Err(_) => {}
        }
    }
    println!("  新增边: {}/{}", inserted, new_edges.len());

    // ═══ E3: 图遍历（简化版 BFS）═══
    println!("\n[E3] Graph Traversal (BFS)");
    
    // 统计连通分量
    let mut visited: HashSet<String> = HashSet::new();
    let mut components = 0u64;
    
    {
        let mut stmt = conn.prepare("SELECT DISTINCT source_id FROM edges")?;
        let all_srcs: Vec<String> = stmt.query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        
        for start in &all_srcs {
            if visited.contains(start) { continue; }
            components += 1;
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start.clone());
            visited.insert(start.clone());
            
            while let Some(curr) = queue.pop_front() {
                let neighbors: Vec<String> = {
                    let mut stmt = conn.prepare(
                        "SELECT target_id FROM edges WHERE source_id=?1 UNION SELECT source_id FROM edges WHERE target_id=?1"
                    )?;
                    let rows: Vec<String> = stmt
                        .query_map([&curr], |r| r.get::<_, String>(0))?
                        .filter_map(|r| r.ok())
                        .collect();
                    rows
                };
                for nid in &neighbors {
                    if !visited.contains(nid) {
                        visited.insert(nid.clone());
                        queue.push_back(nid.clone());
                    }
                }
            }
        }
    }
    println!("  连通分量: {}", components);

    // ═══ 最终统计 ═══
    let elapsed = t0.elapsed().as_secs_f64();
    let final_edges: i64 = conn.query_row("SELECT count(*) FROM edges", [], |r| r.get(0))?;
    let final_emb = nt_memory_embed::embedding_count(conn)?;

    println!("\n{}", "=".repeat(50));
    println!("📊 觉醒管线完成 ({:.1}s)", elapsed);
    println!("{}", "=".repeat(50));
    println!("  节点:     {:>10}", total_nodes);
    println!("  边:       {:>10} (+{})", final_edges, final_edges.saturating_sub(total_edges));
    println!("  Emb:      {:>10}", final_emb);
    println!("  连通分量: {:>10}", components);
    println!("{}", "=".repeat(50));

    Ok(())
}
