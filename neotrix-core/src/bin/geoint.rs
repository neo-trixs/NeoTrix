//! mil_officer GEOINT Perception Pipeline — Rust 参考实现 (neotrix-geoint)
//!
//! 六层架构 (geoint-system.md):
//!   ①Ingest → ②Detect → ③Fuse → ④Analyze → ⑤Warn → ⑥Decide
//!
//! 设计来源 (KB experience hub):
//!   branch_214_0  变化检测复合指数 D (REACTIV/PWTT/Omnibus, 语境分流)
//!   branch_214_1  Agentic GEOINT 星上编排 (tip-and-cue, 边缘回传)
//!   branch_214_8  海事感知 (AIS 状态一致性/暗船/context-aware)
//!   branch_214_6/7 战略预警 I&W (z-score 指标跟踪, 跟踪引信非火柴)
//!   branch_215_4  Multi-INT 融合 (学科感知关联/分类传播/审计)
//!   branch_215_5  杀伤链/OODA (机器速度 vs 人类速度断点, 审批门)
//!
//! 用法:
//!   neotrix-geoint                # demo1 海上监测
//!   neotrix-geoint --scenario demo2
//!   neotrix-geoint --verbose      # 分阶段明细
//!
//! 研究/教育用途参考实现, 非操作级认证 (OSINT 标准免责声明).
//!
//! port of: mil/officer/geoint/pipeline.py (cycle 222 迁移, 零 Python 运行时依赖)

#![forbid(unsafe_code)]
use clap::Parser;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

// ─────────────────────────────────────────────────────────────────────────────
// ① 摄取层 Ingest
// ─────────────────────────────────────────────────────────────────────────────

/// 规范观测: 携带源学科元数据, 不做跨学科解释 (branch_215_4).
#[derive(Clone)]
struct Observation {
    discipline: String, // IMINT / SIGINT / OSINT / AIS / ADS-B
    otype: String,      // 观测类型: 变化图斑/轨迹/文本/状态
    lat: f64,
    lon: f64,
    confidence: f64, // 0..1
    attrs: Map<String, Value>, // 学科特定属性
    source: String, // 可追溯
    classification: String, // 分类传播 (branch_215_4)
}

/// 适配器层: 把外部事件转为规范观测 (适配器只转换, 不解释).
fn ingest(raw: &[Value]) -> Vec<Observation> {
    raw.iter()
        .map(|e| Observation {
            discipline: e["discipline"].as_str().unwrap_or("").to_string(),
            otype: e["type"].as_str().unwrap_or("").to_string(),
            lat: e["lat"].as_f64().unwrap_or(0.0),
            lon: e["lon"].as_f64().unwrap_or(0.0),
            confidence: e["confidence"].as_f64().unwrap_or(0.6),
            attrs: e.get("attrs").and_then(|a| a.as_object()).cloned().unwrap_or_default(),
            source: e.get("source").and_then(|s| s.as_str()).unwrap_or("unknown").to_string(),
            classification: e
                .get("classification")
                .and_then(|c| c.as_str())
                .unwrap_or("unclassified")
                .to_string(),
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// ② 检测层 Detect
// ─────────────────────────────────────────────────────────────────────────────

fn zscore(x: f64, mean: f64, stdev: f64) -> f64 {
    if stdev <= 1e-9 {
        0.0
    } else {
        (x - mean) / stdev
    }
}

fn fmean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn pstdev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = fmean(v);
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64;
    var.sqrt()
}

/// 复合变化指数 D (branch_214_0).
/// 城市场所: REACTIV(变异系数) + PWTT(强度变化)
/// 非城市场所: REACTIV(变异系数) + Omnibus(协方差偏移, 用方差代理)
fn change_index(sar_series: &[f64], baseline_mean: f64, baseline_stdev: f64, urban: bool) -> f64 {
    if sar_series.is_empty() {
        return 0.0;
    }
    let reactiv = pstdev(sar_series) / (fmean(sar_series).abs() + 1e-9);
    let pwtt = zscore(fmean(sar_series), baseline_mean, baseline_stdev);
    let omnibus = ((pstdev(sar_series) + 1e-9) / (baseline_stdev + 1e-9)).ln().abs();
    let d = if urban {
        0.6 * pwtt.abs() + 0.4 * reactiv
    } else {
        0.5 * omnibus + 0.5 * reactiv
    };
    (d * 10000.0).round() / 10000.0
}

/// AIS 状态一致性检测 (branch_214_8): 船型+航态 vs 运动学一致.
fn ais_status_consistency(vessel_type: &str, reported_status: &str, sog_knots: f64) -> (f64, String) {
    let score = if vessel_type == "fishing" && reported_status == "underway" {
        if sog_knots > 8.0 { 1.0 } else { 0.0 } // 渔船'在航'但慢速 → 可疑
    } else if vessel_type == "cargo" && reported_status == "fishing" {
        1.0 // 货船报'捕捞' = 明显伪造
    } else {
        0.0
    };
    let reason = if score >= 0.5 { "状态与船型矛盾" } else { "状态一致" };
    (score, reason.to_string())
}

fn detect(obs: &[Observation], context: &Value) -> Vec<Value> {
    let mut events: Vec<Value> = Vec::new();
    let mut sar_series: Vec<f64> = context["sar_series"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
        .unwrap_or_default();
    let baseline = context["sar_baseline"]
        .as_array()
        .map(|a| (a[0].as_f64().unwrap_or(1.0), a[1].as_f64().unwrap_or(0.2)))
        .unwrap_or((1.0, 0.2));
    let urban = context["urban"].as_bool().unwrap_or(false);

for o in obs {
        if o.discipline == "IMINT" && o.otype == "change" {
            // 与 Python 参考一致: 用上下文基序 + 单次回波, 不跨观测累计累加。
            let bs = o.attrs.get("backscatter").and_then(|b| b.as_f64()).unwrap_or(1.0);
            let mut series = sar_series.clone();
            series.push(bs);
            let d = change_index(&series, baseline.0, baseline.1, urban);
            let sig = if d > 1.2 { "high" } else if d > 0.7 { "medium" } else { "low" };
            events.push(json!({
                "kind": "change_event", "discipline": "IMINT",
                "lat": o.lat, "lon": o.lon, "index_d": d,
                "confidence": o.confidence, "significance": sig, "source": o.source,
            }));
        }
        if o.discipline == "AIS" && o.otype == "status" {
            let vt = o.attrs.get("vessel_type").and_then(|v| v.as_str()).unwrap_or("");
            let rs = o.attrs.get("reported_status").and_then(|v| v.as_str()).unwrap_or("");
            let sog = o.attrs.get("sog").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let (score, reason) = ais_status_consistency(vt, rs, sog);
            if score >= 0.5 {
                events.push(json!({
                    "kind": "ais_status_anomaly", "discipline": "AIS",
                    "lat": o.lat, "lon": o.lon, "score": score, "reason": reason,
                    "confidence": o.confidence, "source": o.source,
                }));
            }
        }
        if o.discipline == "SIGINT" && o.otype == "emitter" {
            events.push(json!({
                "kind": "emitter", "discipline": "SIGINT",
                "lat": o.lat, "lon": o.lon,
                "band": o.attrs.get("band").and_then(|b| b.as_str()).unwrap_or("unknown"),
                "bearing_only": true, "confidence": o.confidence, "source": o.source,
            }));
        }
        if o.discipline == "OSINT" && o.otype == "text" {
            events.push(json!({
                "kind": "osint_alert", "discipline": "OSINT",
                "lat": o.lat, "lon": o.lon,
                "summary": o.attrs.get("summary").and_then(|s| s.as_str()).unwrap_or(""),
                "confidence": o.confidence, "source": o.source,
            }));
        }
    }
    events
}

// ─────────────────────────────────────────────────────────────────────────────
// ③ 融合层 Fuse
// ─────────────────────────────────────────────────────────────────────────────

/// 独立误差结构: 学科各异才真独立 (branch_215_4: 衍生产品非独立).
fn independent(disciplines: &[String]) -> bool {
    let mut s: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for d in disciplines {
        s.insert(d.as_str());
    }
    s.len() > 1
}

fn fuse(events: &[Value]) -> Vec<Value> {
    let mut tracks: BTreeMap<String, Map<String, Value>> = BTreeMap::new();

    for e in events {
        let kind = e["kind"].as_str().unwrap_or("").to_string();
        let discipline = e["discipline"].as_str().unwrap_or("").to_string();
        let bearing_only = e["bearing_only"].as_bool().unwrap_or(false);
        let (lat, lon) = if bearing_only {
            (Value::Null, Value::Null)
        } else {
            (json!(e["lat"].as_f64().unwrap_or(0.0)), json!(e["lon"].as_f64().unwrap_or(0.0)))
        };
        let key = if bearing_only {
            format!("emitter_{}", e["band"].as_str().unwrap_or("unknown"))
        } else {
            format!(
                "{}_{}_{}",
                kind,
                (e["lat"].as_f64().unwrap_or(0.0) * 10.0).round() / 10.0,
                (e["lon"].as_f64().unwrap_or(0.0) * 10.0).round() / 10.0
            )
        };

        let entry = tracks.entry(key).or_insert_with(|| {
            let mut m = Map::new();
            m.insert("kind".into(), json!(kind));
            m.insert("lat".into(), lat);
            m.insert("lon".into(), lon);
            m.insert("disciplines".into(), json!([]));
            m.insert("confidence".into(), json!(0.0));
            m.insert("evidence".into(), json!([]));
            m.insert("conflicts".into(), json!([]));
            m.insert("classification".into(), json!("unclassified"));
            m.insert(
                "significance".into(),
                e.get("significance").cloned().unwrap_or_else(|| json!("low")),
            );
            m.insert(
                "band".into(),
                e.get("band").cloned().unwrap_or_else(|| json!("unknown")),
            );
            m
        });

        let disc_list = entry["disciplines"].as_array_mut().unwrap();
        if !disc_list.iter().any(|d| d.as_str() == Some(discipline.as_str())) {
            disc_list.push(json!(discipline));
        }
        let cur = entry["confidence"].as_f64().unwrap_or(0.0);
        entry.insert("confidence".into(), json!(1.0 - (1.0 - cur) * (1.0 - e["confidence"].as_f64().unwrap_or(0.0))));
        let ev = entry["evidence"].as_array_mut().unwrap();
        ev.push(json!(format!("{}:{}", discipline, e["source"].as_str().unwrap_or(""))));
        let cls = e.get("classification").and_then(|c| c.as_str()).unwrap_or("unclassified");
        if cls != "unclassified" {
            entry.insert("classification".into(), json!(cls));
        }
    }

    let mut fused: Vec<Value> = Vec::new();
    for (_k, mut t) in tracks {
        let conf = t["confidence"].as_f64().unwrap_or(0.0);
        let mut disc: Vec<String> = t["disciplines"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        disc.dedup();
        if independent(&disc) && conf < 1.0 {
            t.insert("confidence".into(), json!(f64::min(1.0, conf + 0.15)));
        }
        let n_disc = disc.len();
        let new_conf = t["confidence"].as_f64().unwrap_or(0.0);
        let ambiguity = n_disc >= 2 && new_conf > 0.85;
        t.insert("ambiguity".into(), json!(ambiguity));
        fused.push(json!(t));
    }
    fused
}

// ─────────────────────────────────────────────────────────────────────────────
// ④ 分析层 Analyze
// ─────────────────────────────────────────────────────────────────────────────

fn analyze(fused: &[Value], region: &str) -> Value {
    let ais_anoms = fused.iter().filter(|t| t["kind"] == "ais_status_anomaly").count();
    let emitters = fused.iter().filter(|t| t["kind"] == "emitter").count();
    let changes: Vec<&Value> = fused.iter().filter(|t| t["kind"] == "change_event").collect();
    let high_changes = changes.iter().filter(|c| c["significance"] == "high").count();
    let movement = if high_changes > 0 {
        "building/positioning"
    } else if changes.is_empty() {
        "baseline"
    } else {
        "monitoring"
    };
    json!({
        "region": region,
        "ais_anomalies": ais_anoms,
        "emitter_clusters": emitters,
        "high_significance_changes": high_changes,
        "order_of_battle": {
            "notable_targets": changes.iter().take(5).map(|c| c["kind"].clone()).collect::<Vec<_>>(),
            "change_count": changes.len(),
        },
        "movement_signal": movement,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// ⑤ 预警层 Warn
// ─────────────────────────────────────────────────────────────────────────────

/// z-score 对滚动基线 (branch_214_6/7 方法).
fn indicator_z(series: &[f64]) -> f64 {
    if series.len() < 3 {
        return 0.0;
    }
    let n = series.len().min(3);
    let recent = &series[series.len() - n..];
    let baseline = &series[..series.len() - n];
    let mean = fmean(baseline);
    let stdev = pstdev(baseline);
    zscore(fmean(recent), mean, stdev).max(0.0)
}

fn warn(indicators: &Map<String, Value>, weights: &Map<String, Value>) -> Value {
    let mut scores: BTreeMap<String, f64> = BTreeMap::new();
    for (name, series) in indicators {
        let s: Vec<f64> = series.as_array().map(|a| a.iter().filter_map(|x| x.as_f64()).collect()).unwrap_or_default();
        let z = (indicator_z(&s) * 1000.0).round() / 1000.0;
        scores.insert(name.clone(), z);
    }
    let fused: f64 = indicators
        .keys()
        .map(|n| scores.get(n).copied().unwrap_or(0.0) * weights.get(n).and_then(|w| w.as_f64()).unwrap_or(0.0))
        .sum();
    let total_w: f64 = indicators
        .keys()
        .map(|n| weights.get(n).and_then(|w| w.as_f64()).unwrap_or(0.0))
        .sum();
    let risk_score = fused / if total_w != 0.0 { total_w } else { 1.0 };

    let level = if risk_score >= 2.5 {
        "CRITICAL"
    } else if risk_score >= 1.5 {
        "HIGH"
    } else if risk_score >= 0.5 {
        "MODERATE"
    } else {
        "LOW"
    };

    // 多模型三角化 (众包视角, branch_214_6)
    let n = indicators.len();
    let agreement = if n == 0 {
        0.0
    } else {
        indicators.keys().filter(|k| scores.get(*k).copied().unwrap_or(0.0) > 0.3).count() as f64 / n as f64
    };
    let confidence = (0.5 + 0.4 * agreement) * 100.0;
    let confidence = confidence.round() / 100.0;

    let mut sorted: Vec<(String, f64)> = scores.iter().map(|(k, v)| (k.clone(), *v)).collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let top: Vec<String> = sorted.iter().take(3).map(|(k, _)| k.clone()).collect();

    let mut feats = Map::new();
    for n in indicators.keys() {
        feats.insert(n.clone(), json!((weights.get(n).and_then(|w| w.as_f64()).unwrap_or(0.0) * 1000.0).round() / 1000.0));
    }

    json!({
        "level": level,
        "risk_score": (risk_score * 1000.0).round() / 1000.0,
        "confidence": confidence,
        "indicators": scores.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<Map<_, _>>(),
        "top_signals": top,
        "feature_importances": feats,
        "warning_note": "跟踪冲突风险指标, 非精确事件预测 (跟踪引信非火柴).",
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// ⑥ 决策闸 Decide
// ─────────────────────────────────────────────────────────────────────────────

fn decide(warning: &Value, fused: &[Value]) -> Vec<Value> {
    let mut gates: Vec<Value> = Vec::new();
    let level = warning["level"].as_str().unwrap_or("");
    if level == "HIGH" || level == "CRITICAL" {
        gates.push(json!({
            "gate": "human_approval", "authority": "operator",
            "action_required": "validate hostile status before any action", "mandatory": true,
        }));
    }
    if fused.iter().any(|t| t["ambiguity"].as_bool().unwrap_or(false)) {
        gates.push(json!({
            "gate": "deconfliction", "authority": "analyst",
            "action_required": "resolve conflicting source identities", "mandatory": true,
        }));
    }
    gates.push(json!({
        "gate": "audit", "authority": "system",
        "action_required": "log all release decisions with source attribution", "mandatory": true,
    }));
    gates
}

// ─────────────────────────────────────────────────────────────────────────────
// 简报输出
// ─────────────────────────────────────────────────────────────────────────────

fn render(obs: &[Observation], events: &[Value], fused: &[Value], analysis: &Value, warning: &Value, gates: &[Value], verbose: bool) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("=".repeat(62));
    lines.push("mil_officer GEOINT 感知简报".to_string());
    lines.push("=".repeat(62));

    let mut disc_set: Vec<&str> = obs.iter().map(|o| o.discipline.as_str()).collect();
    disc_set.sort_unstable();
    disc_set.dedup();
    lines.push(format!(
        "\n[① 摄取] 观测数: {} (学科: {})",
        obs.len(),
        disc_set.join(",")
    ));
    if verbose {
        for o in obs {
            lines.push(format!(
                "  - [{}] {} @({:.2},{:.2}) conf={:.2} src={}",
                o.discipline, o.otype, o.lat, o.lon, o.confidence, o.source
            ));
        }
    }

    lines.push(format!("\n[② 检测] 事件数: {}", events.len()));
    for e in events {
        let mut m = e.as_object().cloned().unwrap_or_default();
        m.remove("kind");
        m.remove("discipline");
        lines.push(format!(
            "  - {:<22} {:<8} {}",
            e["kind"].as_str().unwrap_or(""),
            e["discipline"].as_str().unwrap_or(""),
            serde_json::to_string(&json!(m)).unwrap_or_default()
        ));
    }

    lines.push(format!("\n[③ 融合] 航迹数: {}", fused.len()));
    for t in fused {
        let disc = t["disciplines"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("/"))
            .unwrap_or_default();
        lines.push(format!(
            "  - {:<22} conf={:.2} 学科={} 歧义={}",
            t["kind"].as_str().unwrap_or(""),
            t["confidence"].as_f64().unwrap_or(0.0),
            disc,
            t["ambiguity"].as_bool().unwrap_or(false)
        ));
    }

    lines.push(format!("\n[④ 分析] {}", serde_json::to_string_pretty(analysis).unwrap_or_default()));

    lines.push(format!(
        "\n[⑤ 预警] 等级={} 风险分={:.2} 置信={:.2}",
        warning["level"].as_str().unwrap_or(""),
        warning["risk_score"].as_f64().unwrap_or(0.0),
        warning["confidence"].as_f64().unwrap_or(0.0)
    ));
    lines.push(format!(
        "  信号: {} | 最强特征: {}",
        warning["indicators"],
        warning["top_signals"]
    ));
    lines.push(format!("  提示: {}", warning["warning_note"].as_str().unwrap_or("")));

    lines.push("\n[⑥ 决策闸]".to_string());
    for g in gates {
        lines.push(format!(
            "  - [{}] {} → {} (必需={})",
            g["gate"].as_str().unwrap_or(""),
            g["action_required"].as_str().unwrap_or(""),
            g["authority"].as_str().unwrap_or(""),
            g["mandatory"].as_bool().unwrap_or(false)
        ));
    }

    lines.push("\n── 研究/教育用途参考实现, 非操作级认证 ──".to_string());
    lines.join("\n")
}

// ─────────────────────────────────────────────────────────────────────────────
// 演示场景
// ─────────────────────────────────────────────────────────────────────────────

fn scenario_demo1() -> Value {
    json!({
        "region": "演示海区 (研究用虚构)",
        "urban": false,
        "sar_baseline": [1.0, 0.15],
        "sar_series": [1.02, 0.98, 1.05, 1.55, 1.90, 2.30],
        "events": [
            {"discipline": "AIS", "type": "status", "lat": 32.1, "lon": 121.5,
             "confidence": 0.9, "source": "ais-aishub",
             "attrs": {"vessel_type": "fishing", "reported_status": "underway",
                       "sog": 14.2, "cog": 45}},
            {"discipline": "AIS", "type": "status", "lat": 32.2, "lon": 121.6,
             "confidence": 0.9, "source": "ais-aishub",
             "attrs": {"vessel_type": "cargo", "reported_status": "fishing",
                       "sog": 3.1, "cog": 90}},
            {"discipline": "SIGINT", "type": "emitter", "lat": 32.3, "lon": 121.7,
             "confidence": 0.7, "source": "sigint-sat",
             "attrs": {"band": "X"}},
            {"discipline": "IMINT", "type": "change", "lat": 32.0, "lon": 121.4,
             "confidence": 0.8, "source": "sentinel1",
             "attrs": {"backscatter": 2.1}},
            {"discipline": "OSINT", "type": "text", "lat": 32.05, "lon": 121.45,
             "confidence": 0.6, "source": "osint-news",
             "attrs": {"summary": "区域货运活动异常增多 (研究用虚构)"}},
        ],
        "indicators": {
            "mil_air_movements":  [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.9, 1.2, 1.8],
            "ais_vessel_cluster": [0.2, 0.2, 0.3, 0.3, 0.5, 0.6, 0.8, 1.0, 1.4, 2.2],
            "ews_emissions":      [0.1, 0.1, 0.2, 0.2, 0.3, 0.4, 0.5, 0.6, 0.9, 1.5],
            "news_sentiment":     [0.0, 0.1, 0.1, 0.2, 0.2, 0.3, 0.4, 0.5, 0.6, 0.9],
            "gold_price":         [0.0, 0.0, 0.1, 0.1, 0.1, 0.2, 0.2, 0.3, 0.4, 0.6],
        },
        "weights": {
            "mil_air_movements": 0.25, "ais_vessel_cluster": 0.25,
            "ews_emissions": 0.20, "news_sentiment": 0.15, "gold_price": 0.15,
        }
    })
}

fn scenario_demo2() -> Value {
    json!({
        "region": "演示机场周边 (研究用虚构)",
        "urban": true,
        "sar_baseline": [1.4, 0.10],
        "sar_series": [1.38, 1.42, 1.40, 1.45, 1.70, 2.00],
        "events": [
            {"discipline": "IMINT", "type": "change", "lat": 30.0, "lon": 115.0,
             "confidence": 0.85, "source": "sentinel1",
             "attrs": {"backscatter": 2.4}},
            {"discipline": "IMINT", "type": "change", "lat": 30.05, "lon": 115.02,
             "confidence": 0.80, "source": "sentinel1",
             "attrs": {"backscatter": 2.1}},
            {"discipline": "ADS-B", "type": "status", "lat": 30.02, "lon": 115.01,
             "confidence": 0.9, "source": "adsb-opensky",
             "attrs": {"callsign_pattern": "logistics", "surge": true}},
            {"discipline": "OSINT", "type": "text", "lat": 30.0, "lon": 115.0,
             "confidence": 0.55, "source": "osint-social",
             "attrs": {"summary": "深夜运输活动目击 (研究用虚构)"}},
        ],
        "indicators": {
            "mil_air_movements":  [0.1, 0.2, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 1.0, 1.6],
            "ais_vessel_cluster": [0.1, 0.1, 0.1, 0.2, 0.2, 0.3, 0.3, 0.4, 0.5, 0.7],
            "ews_emissions":      [0.1, 0.1, 0.1, 0.2, 0.2, 0.2, 0.3, 0.3, 0.4, 0.5],
            "news_sentiment":     [0.0, 0.0, 0.1, 0.1, 0.2, 0.2, 0.3, 0.4, 0.5, 0.8],
            "gold_price":         [0.0, 0.0, 0.1, 0.1, 0.1, 0.1, 0.2, 0.2, 0.3, 0.4],
        },
        "weights": {
            "mil_air_movements": 0.30, "ais_vessel_cluster": 0.15,
            "ews_emissions": 0.15, "news_sentiment": 0.20, "gold_price": 0.20,
        }
    })
}

fn run(scenario: &Value, verbose: bool) -> String {
    let events_raw: Vec<Value> = scenario["events"].as_array().cloned().unwrap_or_default();
    let obs = ingest(&events_raw);
    let events = detect(&obs, scenario);
    let fused = fuse(&events);
    let analysis = analyze(&fused, scenario["region"].as_str().unwrap_or(""));
    let warning = warn(
        scenario["indicators"].as_object().unwrap(),
        scenario["weights"].as_object().unwrap(),
    );
    let gates = decide(&warning, &fused);
    render(&obs, &events, &fused, &analysis, &warning, &gates, verbose)
}

#[derive(Parser, Debug)]
#[command(name = "neotrix-geoint", about = "mil_officer GEOINT 感知管线 (Rust)")]
struct Args {
    /// 演示场景
    #[arg(long, default_value = "demo1")]
    scenario: String,
    /// 分阶段明细
    #[arg(long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let scenario = if args.scenario == "demo2" {
        scenario_demo2()
    } else {
        scenario_demo1()
    };
    println!("{}", run(&scenario, args.verbose));
}
