//! exp_topo — 从 `experience.rs` 拆分 (行为零变更, 见拆分基线 smoke).
//! 由 scripts 搬运, 原文逐行, 仅补 `pub(crate)` 可见性。

use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::HashMap;
use super::{NS};
use super::exp_concept::{text_doc_vector, vsa_tokens};
use super::exp_store::{ensure_hub, scan_values};
use super::exp_util::{now_ts, truncate};
use neotrix::l2_perception::nt_core_hcube::ghrr_vsa::ghrr_similarity;
use neotrix::l2_perception::nt_core_hcube::{PersistentHomology, PointCloud};

pub(crate) fn cmd_sim(a: &str, b: &str, dim: usize) {
    let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
    let (va, _) = text_doc_vector(&a.to_lowercase(), dim, &mut memo);
    let (vb, _) = text_doc_vector(&b.to_lowercase(), dim, &mut memo);
    let sim = if va.is_empty() || vb.is_empty() {
        0.0
    } else {
        ghrr_similarity(&va, &vb)
    };
    println!("sim(a,b) = {:.6}  (dim={})", sim, dim);
    println!("  len(a)={} tokens, len(b)={} tokens", vsa_tokens(a).len(), vsa_tokens(b).len());
}

/// 记忆星系拓扑报告: 全量分支 → VSA 文档向量 (归一化) → 持续同调点云。
/// 输出 Betti 曲线 (β₀=记忆簇/组件, β₁=环路=反复出现的模式链, β₂=填充四面体≈高密度凸起)
/// + 积分估计 (Φ 代理) + 持久熵 + 选定尺度下的记忆簇成员 (凸起映射回真实分支)。
///
/// 归一化向量欧氏距离: 语义相关 ≈0.4-0.6, 无关 ≈1.0-1.4 → scale_max=0.8 已覆盖相关区。
/// O(n³) 三角形计数 → max_points 分层采样 (按 domain 均摊) 防爆炸。
#[allow(clippy::needless_range_loop)] // 矩阵双索引 (dists[i][j]) 迭代器改写不可读


pub(crate) fn cmd_topology(conn: &Connection, dim: usize, steps: usize, max_points: usize, json: bool) {
    ensure_hub(conn);
    let mut memo: HashMap<String, Vec<f64>> = HashMap::new();
    let mut entries: Vec<(String, String, String, Vec<f64>)> = Vec::new();
    for (key, value) in scan_values(conn, "branch_") {
        let Ok(v) = serde_json::from_str::<Value>(&value) else { continue };
        let content = v
            .get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_lowercase();
        if content.trim().is_empty() {
            continue;
        }
        let (raw, _) = text_doc_vector(&content, dim, &mut memo);
        let norm = l2_norm(&raw);
        let vec: Vec<f64> = if norm > 1e-12 {
            raw.iter().map(|x| x / norm).collect()
        } else {
            raw
        };
        entries.push((
            key,
            v.get("domain").and_then(|d| d.as_str()).unwrap_or("").to_string(),
            truncate(
                v.get("content").and_then(|c| c.as_str()).unwrap_or(""),
                60,
            ),
            vec,
        ));
    }
    if entries.len() < 2 {
        println!("[topology] 至少需要 2 个分支 (当前 {})", entries.len());
        return;
    }

    // 分层采样: 按 domain 均摊到 max_points, 保持域多样性
    if max_points > 0 && entries.len() > max_points {
        let mut by_domain: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            by_domain.entry(e.1.clone()).or_default().push(i);
        }
        let n_domains = by_domain.len();
        let per = (max_points / n_domains).max(1);
        let mut picked: Vec<usize> = Vec::with_capacity(max_points);
        for idxs in by_domain.values() {
            picked.extend(idxs.iter().take(per));
        }
        if picked.len() < max_points {
            let mut rest: Vec<usize> = (0..entries.len()).filter(|i| !picked.contains(i)).collect();
            rest.sort_by_key(|i| std::cmp::Reverse((entries[*i].1.len(), 0)));
            picked.extend(rest.into_iter().take(max_points - picked.len()));
        }
        let mut filtered: Vec<(String, String, String, Vec<f64>)> = Vec::with_capacity(picked.len());
        for &i in &picked {
            filtered.push(entries[i].clone());
        }
        entries = filtered;
    }

    let mut cloud = PointCloud::new("memory-galaxy");
    for e in &entries {
        cloud.add_point(e.3.clone());
    }

    let scale_max = 0.8f64;
    let ph = PersistentHomology::compute(&cloud, scale_max, steps);

    // 报告 Betti 曲线 (采样几个代表性尺度)
    let mut curve_lines = Vec::new();
    for (s, b) in &ph.betti_curves {
        curve_lines.push(format!(
            "  scale={:.2} β0={} β1={} β2={}",
            s, b.beta_0, b.beta_1, b.beta_2
        ));
    }
    let phi = ph
        .simplified_betti()
        .integration_estimate();
    let entropy = ph.persistence_entropy();

    // 记忆簇: 在 scale 0.6 (语义相近距离) 做 union-find 聚类, 输出≥2 成员的簇
    let mut parent: Vec<usize> = (0..entries.len()).collect();
    let dists = cloud_distance_matrix(&cloud);
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            if dists[i][j] <= 0.6 {
                cluster_union(&mut parent, i, j);
            }
        }
    }
    let mut roots: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..parent.len() {
        let r = cluster_find(&mut parent, i);
        roots.entry(r).or_default().push(i);
    }
    let mut clusters: Vec<Vec<usize>> = roots.into_values().filter(|c| c.len() >= 2).collect();
    clusters.sort_by_key(|c| std::cmp::Reverse(c.len()));

    if json {
        let mut cluster_json = Vec::new();
        for c in &clusters {
            let members: Vec<Value> = c
                .iter()
                .map(|&i| {
                    json!({
                        "key": entries[i].0,
                        "domain": entries[i].1,
                        "content": entries[i].2,
                    })
                })
                .collect();
            cluster_json.push(json!({ "size": c.len(), "members": members }));
        }
        let out = json!({
            "dim": dim,
            "points": entries.len(),
            "betti": ph.betti_curves.iter().map(|(s, b)| json!({
                "scale": s, "beta_0": b.beta_0, "beta_1": b.beta_1, "beta_2": b.beta_2
            })).collect::<Vec<_>>(),
            "integration_estimate": phi,
            "persistence_entropy": entropy,
            "clusters": cluster_json,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        println!("[topology] 记忆星系拓扑 — {} 个分支, dim={}", entries.len(), dim);
        println!("Betti 曲线 (scale∈[0,{}], {} 步):", scale_max, steps);
        for l in curve_lines.iter().take(6) {
            println!("{}", l);
        }
        if steps > 6 {
            println!("  ... 共 {} 步, 中间省略 ...", steps + 1);
            for l in curve_lines.iter().skip(curve_lines.len() - 2) {
                println!("{}", l);
            }
        }
        println!(
            "integration_estimate (Φ 代理) = {:.4}, persistence_entropy = {:.4}",
            phi, entropy
        );
        println!("\n记忆簇 (scale≤0.6, 语义相近 ≥2 分支): {} 个", clusters.len());
        for (ci, c) in clusters.iter().enumerate().take(10) {
            println!(" 簇 #{} ({} 分支):", ci + 1, c.len());
            for &i in c.iter().take(5) {
                println!("   · [{}] {} — {}", entries[i].1, entries[i].0, entries[i].2);
            }
            if c.len() > 5 {
                println!("   ... 其余 {} 分支", c.len() - 5);
            }
        }
    }
}


pub(crate) fn l2_norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

#[allow(clippy::needless_range_loop)] // 矩阵双索引 (dists[i][j]) 迭代器改写不可读


pub(crate) fn cloud_distance_matrix(cloud: &PointCloud) -> Vec<Vec<f64>> {
    let n = cloud.n();
    let mut dists = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = l2_distance(&cloud.points[i], &cloud.points[j]);
            dists[i][j] = d;
            dists[j][i] = d;
        }
    }
    dists
}


pub(crate) fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f64>()
        .sqrt()
}


pub(crate) fn cluster_find(parent: &mut [usize], mut i: usize) -> usize {
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}


pub(crate) fn cluster_union(parent: &mut [usize], a: usize, b: usize) {
    let ra = cluster_find(parent, a);
    let rb = cluster_find(parent, b);
    if ra != rb {
        parent[ra] = rb;
    }
}


pub(crate) fn cmd_rune(conn: &Connection, action: &str, color: Option<&str>, module: Option<&str>) {
    match action {
        "set" => {
            let color_str = color.expect("color required for rune set");
            let valid_colors = ["crimson", "indigo", "obsidian", "golden", "alabaster"];
            if !valid_colors.iter().any(|c| c == &color_str) {
                eprintln!("invalid rune color: {}. valid: crimson, indigo, obsidian, golden, alabaster", color_str);
                return;
            }
            let module_str = module.unwrap_or("default");
            let key = format!("rune:{}:{}", module_str, color_str);
            let val = json!({ "color": color_str, "module": module_str, "set_at": now_ts() });
            conn.execute(
                "INSERT OR REPLACE INTO kv_store (namespace, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
                params![NS, key, serde_json::to_string(&val).expect("JSON serialization"), now_ts()],
            )
            .expect("failed to set rune");
            println!("rune set: module={}, color={}", module_str, color_str);
        }
        "get" => {
            let module_str = module.unwrap_or("default");
            let prefix = format!("rune:{}:", module_str);
            let mut stmt = conn.prepare("SELECT count(*) FROM kv_store WHERE namespace=?1 AND key LIKE ?2").expect("failed to prepare");
            if let Ok(count) = stmt.query_row(params![NS, format!("{}%", prefix)], |row| row.get::<_, i64>(0)) {
                if count > 0 {
                    println!("rune config for module {} exists", module_str);
                } else {
                    println!("no rune config for module {}", module_str);
                }
            } else {
                println!("error querying rune config");
            }
        }
        _ => {
            eprintln!("unknown rune action: {}. use 'set' or 'get'", action);
        }
    }
}


pub(crate) fn cmd_constellation(conn: &Connection, action: &str, domain: Option<&str>) {
    let all_domains = [
        "NT-CORE", "NT-MIND", "NT-MEMORY", "NT-WORLD", "NT-ACT", "NT-IO", "NT-SHIELD", "NT-META", "NT-REPAIR", "NT-GOVERNANCE", "NT-NEXUS",
    ];
    let target = domain.unwrap_or("all");
    match action {
        "audit" => {
            println!("=== Constellation Audit ===");
            let domains: Vec<&str> = if target == "all" {
                all_domains.iter().map(|&d| d).collect()
            } else {
                vec![target]
            };
            for d in domains.iter() {
                let key = format!("maturity:{}", d);
                match conn.query_row("SELECT value FROM kv_store WHERE namespace=?1 AND key=?2", params![NS, key], |row| row.get::<_, String>(0)) {
                    Ok(val) => println!("domain {} maturity: {}", d, val),
                    Err(_) => println!("domain {}: no maturity record (C0)", d),
                }
            }
        }
        "mature" => {
            println!("=== Constellation Maturity Report ===");
            let domains: Vec<&str> = if target == "all" {
                all_domains.iter().map(|&d| d).collect()
            } else {
                vec![target]
            };
            for d in domains.iter() {
                let key = format!("maturity:{}", d);
                match conn.query_row("SELECT value FROM kv_store WHERE namespace=?1 AND key=?2", params![NS, key], |row| row.get::<_, String>(0)) {
                    Ok(val) => {
                        let maturity: i32 = val.parse().unwrap_or(0);
                        let status = if maturity >= 4 {
                            "C4+ pipeline-ready".to_string()
                        } else {
                            format!("C{}", maturity)
                        };
                        println!("domain {}: {} → {}", d, maturity, status);
                    }
                    Err(_) => println!("domain {}: no maturity record (C0)", d),
                }
            }
        }
        _ => {
            eprintln!("unknown constellation action: {}. use 'audit' or 'mature'", action);
        }
    }
}
