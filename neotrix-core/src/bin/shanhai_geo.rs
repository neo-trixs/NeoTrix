//! neotrix-shanhai-geo — 山海世界地理坐标系统
//!
//! Populates the KB with coordinate data and geographic mappings
//! from the nt_shanhai_geo module.
//!
//! Usage: cargo run -p neotrix --bin neotrix-shanhai-geo

use neotrix::neotrix::nt_memory_kb::nt_memory_types::*;
use neotrix::neotrix::nt_memory_kb::nt_memory_schema;
use neotrix::neotrix::nt_shanhai_geo::*;
use rusqlite::Connection;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let db_path = format!("{}/.neotrix/knowledge.db", home);
    println!("Opening KB at: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to open KB");
    nt_memory_schema::initialize(&conn).expect("Failed to init schema");

    // 1. Absorb school parameters
    println!("\n=== Absorbing School Parameters ===");
    for s in all_schools() {
        let meta = serde_json::json!({
            "type": "shanhai-school",
            "founder": s.founder,
            "scope": s.scope.as_str(),
            "li_scale": s.li_scale.name(),
            "confidence_base": s.confidence_base,
        });
        safe_insert_node(
            &conn,
            &KnowledgeNode {
                id: format!("shanhai-school:{}", s.name),
                node_type: NodeType::Theory,
                title: s.name.clone(),
                summary: Some(s.description.clone()),
                content: None,
                url: None,
                domain: None,
                language: "zh".into(),
                confidence: s.confidence_base,
                importance: 0.8,
                created_at: now(),
                updated_at: now(),
                access_count: 0,
                metadata: Some(meta),
                temporal: None,
                supersedes: None,
                source_episode: None,
            },
        )
        .expect("Failed to insert school");
        println!("  ✅ {} ({})", s.name, s.scope.as_str());
    }

    // 2. Absorb mountain peaks
    println!("\n=== Absorbing Mountain Peaks ===");
    for p in known_peaks() {
        let modern = p
            .modern_location
            .map(|g| format!("{},{}", g.lat, g.lng))
            .unwrap_or_default();
        let scholars: Vec<String> = p
            .attributed_by
            .iter()
            .map(|r| format!("{}@{}", r.scholar, r.school))
            .collect();
        let meta = serde_json::json!({
            "type": "shanhai-peak",
            "range_id": p.range_id,
            "position": p.position,
            "modern_location": modern,
            "identification_confidence": p.identification_confidence,
            "attributed_by": scholars,
        });

        let title = format!("[山] {} ({})", p.name, p.range_id);
        safe_insert_node(
            &conn,
            &KnowledgeNode {
                id: format!("shanhai-peak:{}", p.id),
                node_type: NodeType::Concept,
                title,
                summary: Some(format!(
                    "《山海经》峰峦，置信度{:.0}%，归属{:?}",
                    p.identification_confidence * 100.0,
                    p.attributed_by.iter().map(|r| r.scholar.as_str()).collect::<Vec<_>>()
                )),
                content: None,
                url: None,
                domain: None,
                language: "zh".into(),
                confidence: p.identification_confidence,
                importance: 0.7,
                created_at: now(),
                updated_at: now(),
                access_count: 0,
                metadata: Some(meta),
                temporal: None,
                supersedes: None,
                source_episode: None,
            },
        )
        .expect("Failed to insert peak");
        println!("  ✅ {} — ({})", p.name, modern);
    }

    // 3. Absorb place mappings with geographic coords
    println!("\n=== Absorbing Place Mappings ===");
    for m in all_mappings() {
        let loc = m
            .modern_location
            .map(|g| format!("{},{}", g.lat, g.lng))
            .unwrap_or_default();
        let scholars: Vec<String> = m
            .school_attribution
            .iter()
            .map(|r| format!("{}@{}:{:.0}%", r.scholar, r.school, r.confidence * 100.0))
            .collect();
        let meta = serde_json::json!({
            "type": "shanhai-mapping",
            "modern_name": m.modern_name,
            "modern_location": loc,
            "confidence": m.confidence,
            "attributed_by": scholars,
            "evidence": m.evidence_summary,
        });

        safe_insert_node(
            &conn,
            &KnowledgeNode {
                id: format!("shanhai-map:{}", m.shanhai_name),
                node_type: NodeType::Concept,
                title: m.relation_type.clone(),
                summary: Some(format!(
                    "{} → {} (置信度{:.0}%, {})",
                    m.shanhai_name, m.modern_name, m.confidence * 100.0, m.evidence_summary
                )),
                content: None,
                url: None,
                domain: None,
                language: "zh".into(),
                confidence: m.confidence,
                importance: 0.8,
                created_at: now(),
                updated_at: now(),
                access_count: 0,
                metadata: Some(meta),
                temporal: None,
                supersedes: None,
                source_episode: None,
            },
        )
        .expect("Failed to insert mapping");
        println!("  ✅ {} → {} ({})", m.shanhai_name, m.modern_name, loc);
    }

    // 4. Create cross-reference edges
    println!("\n=== Creating Cross-Reference Edges ===");
    let world_school = "shanhai-school:世界圈说——宫玉海学术体系";
    let china_school = "shanhai-school:华夏说——谭其骧学术体系";

    for m in all_mappings() {
        let mapping_id = format!("shanhai-map:{}", m.shanhai_name);
        let has_global = m
            .school_attribution
            .iter()
            .any(|r| r.school.contains("世界圈"));
        let has_china = m
            .school_attribution
            .iter()
            .any(|r| r.school.contains("华夏"));

        if has_global {
            let edge = KnowledgeEdge {
                id: format!("shanhai-edge:{}->world", m.shanhai_name),
                source_id: mapping_id.clone(),
                target_id: world_school.to_string(),
                relation_type: RelationType::Supports,
                weight: m.confidence,
                description: Some(format!("世界圈说归因: {}", m.evidence_summary)),
                created_at: now(),
                metadata: None,
            };
            let _ = safe_insert_edge(&conn, &edge);
        }
        if has_china {
            let edge = KnowledgeEdge {
                id: format!("shanhai-edge:{}->china", m.shanhai_name),
                source_id: mapping_id,
                target_id: china_school.to_string(),
                relation_type: RelationType::Supports,
                weight: m.confidence,
                description: Some(format!("华夏说归因: {}", m.evidence_summary)),
                created_at: now(),
                metadata: None,
            };
            let _ = safe_insert_edge(&conn, &edge);
        }
    }

    println!("\n✅ 地理坐标系统数据吸收完成!");
    println!("   山峰: {} 座", known_peaks().len());
    println!("   学派人: {} 个", all_schools().len());
    println!("   全球映射: {} 个", all_mappings().len());
}
