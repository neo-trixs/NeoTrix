//! 规则记忆体 (Rule Memory) — L3 KB 接口层。
//!
//! 将 core::nt_core_rule_memory 的四元组存储与查询能力暴露给 L3 KB 接口，
//! 供 NT-MIND / NT-CORE / NT-WORLD 统一调用。
//!
//! 遵循 R-P42: 强化现有 KB 节点，不建平行适配器。

#[allow(unused_imports)]
use crate::core::nt_core_kb_primitives::now;
#[allow(unused_imports)]
use crate::core::nt_core_rule_memory::{
    crystallize_scan, extract_params, gc_rules, render_template, rule_delete, rule_exec,
    rule_get, rule_list, rule_match, rule_put, verify_rule, Crystallized, GcReport, RuleRecord,
    RuleStatus, ScanConfig, ScanReport,
};
#[allow(unused_imports)]
use rusqlite::Connection;
#[allow(unused_imports)]
use std::collections::BTreeMap;

/// KnowledgeBase 扩展：规则记忆操作。
pub struct RuleMemoryInterface;

impl RuleMemoryInterface {
    /// 结晶扫描: 从 experience namespace 结晶规则 (detect→compile→verify→replace)。
    pub fn crystallize(conn: &Connection, cfg: &ScanConfig) -> Result<ScanReport, String> {
        crate::core::nt_core_rule_memory::crystallize_scan(conn, cfg)
    }

    /// 手工添加规则 (需后续 verify 晋升 Active)。
    pub fn add_rule(
        conn: &Connection,
        id: String,
        trigger_text: String,
        generator: String,
        domain: String,
    ) -> Result<RuleRecord, String> {
        let now_ts = now();
        let mut rec = crate::core::nt_core_rule_memory::RuleRecord::new(
            id, domain, trigger_text, generator, now_ts,
        );
        rec.status = RuleStatus::Quarantined;
        rule_put(conn, &rec)?;
        Ok(rec)
    }

    /// 执行规则: Jaccard 路由匹配 → 渲染生成器 → 回写计数器。
    pub fn exec_rule(
        conn: &Connection,
        query: &str,
        vars: &BTreeMap<String, String>,
        threshold: f64,
    ) -> Result<(RuleRecord, f64, String), String> {
        let m = rule_match(conn, query, threshold)?;
        let Some((mut rec, sim)) = m else {
            return Err("无匹配规则".into());
        };
        let out = rule_exec(&rec, vars)?;
        rec.exec_count += 1;
        rec.updated_at = now();
        rule_put(conn, &rec)?;
        Ok((rec, sim, out))
    }

    /// 列出规则 (可按状态过滤)。
    pub fn list_rules(
        conn: &Connection,
        status: Option<RuleStatus>,
    ) -> Result<Vec<RuleRecord>, String> {
        let rules = rule_list(conn)?;
        Ok(rules
            .into_iter()
            .filter(|r| status.as_ref().map_or(true, |s| r.status == *s))
            .collect())
    }

    /// 获取单条规则详情。
    pub fn get_rule(conn: &Connection, id: &str) -> Result<Option<RuleRecord>, String> {
        rule_get(conn, id)
    }

    /// 重验证规则 (单条或全量)。
    pub fn verify_rule(
        conn: &Connection,
        id: &str,
        sources: &std::collections::HashMap<String, String>,
    ) -> Result<Option<RuleRecord>, String> {
        verify_rule(conn, id, sources)
    }

    /// 全量验证所有规则。
    pub fn verify_all_rules(conn: &Connection) -> Result<Vec<(String, RuleStatus, u64)>, String> {
        let sources = build_sources(conn)?;
        let mut report = Vec::new();
        for rec in rule_list(conn)? {
            if let Ok(Some(r)) = verify_rule(conn, &rec.id, &sources) {
                report.push((r.id, r.status, r.fail_count));
            }
        }
        Ok(report)
    }

    /// Dark Forest GC: 清理无消费者/验证失败的规则。
    pub fn gc_rules(conn: &Connection, max_idle_days: u64) -> Result<GcReport, String> {
        gc_rules(conn, max_idle_days)
    }

    /// 统计信息。
    pub fn stats(conn: &Connection) -> Result<String, String> {
        let rules = rule_list(conn)?;
        let mut by_status = std::collections::HashMap::new();
        let mut total_exec = 0u64;
        for r in &rules {
            *by_status.entry(r.status).or_insert(0) += 1;
            total_exec += r.exec_count;
        }
        Ok(format!(
            "规则总数: {}\n按状态: {:?}\n总执行数: {}\n",
            rules.len(), by_status, total_exec
        ))
    }
}

/// 从 experience namespace 构建 sources 映射 (provenance_key → content)。
fn build_sources(
    conn: &Connection,
) -> Result<std::collections::HashMap<String, String>, String> {
#[allow(unused_imports)]
    use crate::core::nt_core_kb_primitives::kv_list;
    let mut map = std::collections::HashMap::new();
    for (key, value) in kv_list(conn, "experience")? {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&value) {
            if let Some(content) = v.get("content").and_then(|c| c.as_str()) {
                map.insert(key, content.to_string());
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
#[allow(unused_imports)]
    use crate::core::nt_core_kb_primitives::schema_initialize;
#[allow(unused_imports)]
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        schema_initialize(&c).unwrap();
        c
    }

    #[test]
    fn test_rule_memory_interface_basic() {
        let conn = mem_conn();
        let rec = RuleMemoryInterface::add_rule(
            &conn,
            "test_rule".into(),
            "部署 服务".into(),
            "部署 {{s0}} 到 {{s1}}".into(),
            "NT-ACT".into(),
        )
        .unwrap();
        assert_eq!(rec.id, "test_rule");
        assert_eq!(rec.status, crate::core::nt_core_rule_memory::RuleStatus::Quarantined);
    }
}