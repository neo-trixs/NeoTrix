//! Trade Knowledge Query — 外贸知识库查询接口
//!
//! 提供对导入的富通天下 CRM 数据的查询能力，供外贸 Agent 使用。

#![forbid(unsafe_code)]

use std::path::Path;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

// ── 数据结构 ──────────────────────────────────────────────────

/// 客户查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerQueryResult {
    /// 客户 ID
    pub customer_id: String,
    /// 客户名称
    pub name: String,
    /// 客户编码
    pub code: String,
    /// 等级
    pub grade: String,
    /// 来源渠道
    pub channel: String,
    /// 国家
    pub country: String,
    /// 地区
    pub region: String,
    /// 联系人
    pub contact_name: String,
    /// 负责业务员
    pub owner: String,
    /// 描述
    pub description: String,
    /// 业务类型
    pub business_type: String,
    /// 最后活动
    pub last_activity: String,
    /// 最后跟进时间
    pub last_follow_at: u64,
}

/// 客户统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerStats {
    /// 总客户数
    pub total: u64,
    /// 各等级分布
    pub by_grade: Vec<(String, u64)>,
    /// 各国家分布 (Top 10)
    pub by_country: Vec<(String, u64)>,
    /// 各渠道分布
    pub by_channel: Vec<(String, u64)>,
}

// ── 查询接口 ──────────────────────────────────────────────────

/// 外贸知识库查询器
pub struct TradeKnowledgeQuery {
    conn: Connection,
}

impl TradeKnowledgeQuery {
    /// 创建查询器
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(db_path.as_ref())
            .map_err(|e| format!("Failed to open database: {}", e))?;
        Ok(Self { conn })
    }

    /// 创建默认查询器 (使用 ~/.neotrix/trade_knowledge.db)
    pub fn default_path() -> Result<Self, String> {
        let db_path = dirs::home_dir()
            .ok_or("Cannot find home directory")?
            .join(".neotrix")
            .join("trade_knowledge.db");
        Self::new(db_path)
    }

    /// 按名称搜索客户
    pub fn search_by_name(&self, keyword: &str, limit: u32) -> Result<Vec<CustomerQueryResult>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT customer_id, name, code, grade, channel, country, region,
                    contact_name, owner, description, business_type,
                    last_activity, last_follow_at
             FROM customers
             WHERE name LIKE ?1 OR contact_name LIKE ?1
             ORDER BY last_follow_at DESC
             LIMIT ?2"
        ).map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", keyword);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(CustomerQueryResult {
                customer_id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                grade: row.get(3)?,
                channel: row.get(4)?,
                country: row.get(5)?,
                region: row.get(6)?,
                contact_name: row.get(7)?,
                owner: row.get(8)?,
                description: row.get(9)?,
                business_type: row.get(10)?,
                last_activity: row.get(11)?,
                last_follow_at: row.get::<_, i64>(12)? as u64,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }

    /// 按国家搜索客户
    pub fn search_by_country(&self, country: &str, limit: u32) -> Result<Vec<CustomerQueryResult>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT customer_id, name, code, grade, channel, country, region,
                    contact_name, owner, description, business_type,
                    last_activity, last_follow_at
             FROM customers
             WHERE country LIKE ?1 OR region LIKE ?1
             ORDER BY grade DESC, last_follow_at DESC
             LIMIT ?2"
        ).map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", country);
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(CustomerQueryResult {
                customer_id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                grade: row.get(3)?,
                channel: row.get(4)?,
                country: row.get(5)?,
                region: row.get(6)?,
                contact_name: row.get(7)?,
                owner: row.get(8)?,
                description: row.get(9)?,
                business_type: row.get(10)?,
                last_activity: row.get(11)?,
                last_follow_at: row.get::<_, i64>(12)? as u64,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }

    /// 按等级搜索客户
    pub fn search_by_grade(&self, grade: &str, limit: u32) -> Result<Vec<CustomerQueryResult>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT customer_id, name, code, grade, channel, country, region,
                    contact_name, owner, description, business_type,
                    last_activity, last_follow_at
             FROM customers
             WHERE grade = ?1
             ORDER BY last_follow_at DESC
             LIMIT ?2"
        ).map_err(|e| format!("Prepare error: {}", e))?;

        let rows = stmt.query_map(params![grade, limit], |row| {
            Ok(CustomerQueryResult {
                customer_id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                grade: row.get(3)?,
                channel: row.get(4)?,
                country: row.get(5)?,
                region: row.get(6)?,
                contact_name: row.get(7)?,
                owner: row.get(8)?,
                description: row.get(9)?,
                business_type: row.get(10)?,
                last_activity: row.get(11)?,
                last_follow_at: row.get::<_, i64>(12)? as u64,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }

    /// 获取客户统计
    pub fn get_stats(&self) -> Result<CustomerStats, String> {
        // 总数
        let total: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM customers", [], |row| row.get(0)
        ).map_err(|e| format!("Count error: {}", e))?;

        // 按等级
        let mut stmt = self.conn.prepare(
            "SELECT grade, COUNT(*) FROM customers GROUP BY grade ORDER BY COUNT(*) DESC"
        ).map_err(|e| format!("Prepare error: {}", e))?;
        let by_grade: Vec<(String, u64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok()).collect();

        // 按国家 (Top 10)
        let mut stmt = self.conn.prepare(
            "SELECT country, COUNT(*) FROM customers WHERE country != '' GROUP BY country ORDER BY COUNT(*) DESC LIMIT 10"
        ).map_err(|e| format!("Prepare error: {}", e))?;
        let by_country: Vec<(String, u64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok()).collect();

        // 按渠道
        let mut stmt = self.conn.prepare(
            "SELECT channel, COUNT(*) FROM customers WHERE channel != '' GROUP BY channel ORDER BY COUNT(*) DESC"
        ).map_err(|e| format!("Prepare error: {}", e))?;
        let by_channel: Vec<(String, u64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok()).collect();

        Ok(CustomerStats {
            total,
            by_grade,
            by_country,
            by_channel,
        })
    }

    /// 获取需要跟进的客户 (超过 N 天未联系)
    pub fn get_needs_followup(&self, days: u32) -> Result<Vec<CustomerQueryResult>, String> {
        let cutoff = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64) - (days as i64 * 86400);

        let mut stmt = self.conn.prepare(
            "SELECT customer_id, name, code, grade, channel, country, region,
                    contact_name, owner, description, business_type,
                    last_activity, last_follow_at
             FROM customers
             WHERE last_follow_at < ?1 OR last_follow_at = 0
             ORDER BY grade DESC, last_follow_at ASC
             LIMIT 50"
        ).map_err(|e| format!("Prepare error: {}", e))?;

        let rows = stmt.query_map(params![cutoff], |row| {
            Ok(CustomerQueryResult {
                customer_id: row.get(0)?,
                name: row.get(1)?,
                code: row.get(2)?,
                grade: row.get(3)?,
                channel: row.get(4)?,
                country: row.get(5)?,
                region: row.get(6)?,
                contact_name: row.get(7)?,
                owner: row.get(8)?,
                description: row.get(9)?,
                business_type: row.get(10)?,
                last_activity: row.get(11)?,
                last_follow_at: row.get::<_, i64>(12)? as u64,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Row error: {}", e))?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path() {
        let result = TradeKnowledgeQuery::default_path();
        assert!(result.is_ok(), "Should create query with default path");
    }
}
