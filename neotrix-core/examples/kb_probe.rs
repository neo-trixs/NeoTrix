//! KB 挂点探针：分段计时定位 write 链挂起位置。
//!
//! 运行: timeout 120 cargo run --example kb_probe
//! 输出每段耗时；停在哪行，哪段就是挂点（shell timeout 兜底）。
//! 只用 temp 库，不碰生产数据。
//!
//! v2 (2026-09-25): 二分法 —— 逐个调用 write_memory_entry 内部的
//! public 阶段（A~F），再跑 G（无正文全弧）/ H（有正文全弧）。
//! record_node_fact 为 pub(crate)，用排除法定位：
//!   G 通 + H 挂 + E 通 → 挂点 ∈ {record_node_fact, conflict_detect}，F 再二分。

use neotrix::l4_emotion::nt_memory::nt_memory_kb::nt_memory_curation;
use neotrix::l4_emotion::nt_memory::nt_memory_kb::nt_memory_graphrag::GraphRagConfig;
use neotrix::l5_cognition::l1_facade::KnowledgeBase;
use neotrix_types::knowledge_access::NodeType;
use std::io::Write as _;

fn stamp(label: &str, t0: std::time::Instant) {
    eprintln!(
        "[kb-probe] {:36} +{:>6.1}s",
        label,
        t0.elapsed().as_secs_f64()
    );
    let _ = std::io::stderr().flush();
}

fn main() {
    let t0 = std::time::Instant::now();
    let dir = std::env::temp_dir().join(format!("neotrix_kbprobe_{}", std::process::id()));
    stamp("start", t0);
    // phase -1: temp 子目录必须先建（rusqlite 建不了缺失父目录；
    // 全仓 temp-dir 单测同病，KB::open 遇缺失目录挂起而不报错——已定位）
    match std::fs::create_dir_all(&dir) {
        Ok(()) => stamp("mkdir ok", t0),
        Err(e) => stamp(&format!("mkdir ERR {e}"), t0),
    }
    let db_path = dir.join("kb.db");
    // phase 0: 裸 rusqlite（排除驱动/文件系统层）
    {
        let raw = dir.join("raw.db");
        match rusqlite::Connection::open(&raw) {
            Ok(c) => match c.execute("CREATE TABLE t(x)", []) {
                Ok(_) => stamp("raw-sqlite ok", t0),
                Err(e) => stamp(&format!("raw-sqlite ERR {e}"), t0),
            },
            Err(e) => stamp(&format!("raw-open ERR {e}"), t0),
        }
    }
    let kb = match KnowledgeBase::open(Some(db_path)) {
        Ok(k) => {
            stamp("open ok", t0);
            k
        }
        Err(e) => {
            stamp(&format!("open ERR {e}"), t0);
            return;
        }
    };
    // A: 主库插入（write 弧 step 1 前半）
    let node_id = match kb.insert_or_get_node(
        "probe",
        NodeType::Concept,
        Some("probe body"),
        None,
        Some("test"),
    ) {
        Ok(id) => {
            stamp("A insert_or_get_node ok", t0);
            id
        }
        Err(e) => {
            stamp(&format!("A insert ERR {e}"), t0);
            return;
        }
    };
    // B: 读回（step 1 后半 / versioned / svaf 共用）
    match kb.get_node(&node_id) {
        Ok(_) => stamp("B get_node ok", t0),
        Err(e) => stamp(&format!("B get ERR {e}"), t0),
    }
    // C: metadata 更新（versioned / svaf 共用）
    match kb.update_node_metadata(&node_id, &serde_json::json!({"probe": 1})) {
        Ok(()) => stamp("C update_node_metadata ok", t0),
        Err(e) => stamp(&format!("C update_meta ERR {e}"), t0),
    }
    // D: SVAF content-only 门禁（纯计算断言）
    {
        let ev = kb.gate_content_only("probe body", "unknown");
        stamp(&format!("D gate_content_only ok {:?}", ev.decision), t0);
    }
    // E: GraphRAG 派生（step 6）
    match kb.init_graphrag(GraphRagConfig::default()) {
        Ok(()) => stamp("E init_graphrag ok", t0),
        Err(e) => stamp(&format!("E init_graphrag ERR {e}"), t0),
    }
    match kb.graphrag_extract("probe body", &node_id) {
        Ok((ents, rels)) => stamp(
            &format!("E graphrag_extract ok e={} r={}", ents.len(), rels.len()),
            t0,
        ),
        Err(e) => stamp(&format!("E graphrag_extract ERR {e}"), t0),
    }
    // F: 冲突检测（step 5），直接拿裸 conn 调纯函数
    match kb.raw_conn() {
        Ok(conn) => {
            stamp("F raw_conn lock ok", t0);
            match nt_memory_curation::conflict_detect_for_write(&conn, &node_id, 0.4) {
                Ok(hits) => stamp(&format!("F conflict_detect ok hits={}", hits.len()), t0),
                Err(e) => stamp(&format!("F conflict_detect ERR {e}"), t0),
            }
        }
        Err(e) => stamp(&format!("F raw_conn ERR {e}"), t0),
    }
    // G: 全弧无正文（跳过 record_node_fact / conflict / graphrag）
    match kb.write_memory_entry(
        "probe-nocontent",
        NodeType::Concept,
        None,
        None,
        Some("test"),
        None,
    ) {
        Ok(id) => stamp(&format!("G full-write-nocontent ok id={id}"), t0),
        Err(e) => stamp(&format!("G full-write-nocontent ERR {e}"), t0),
    }
    // H: 全弧有正文（原挂起点）
    match kb.write_memory_entry(
        "probe2",
        NodeType::Concept,
        Some("probe body 2"),
        None,
        Some("test"),
        None,
    ) {
        Ok(id) => stamp(&format!("H full-write-content ok id={id}"), t0),
        Err(e) => stamp(&format!("H full-write-content ERR {e}"), t0),
    }
    stamp("done", t0);
}
