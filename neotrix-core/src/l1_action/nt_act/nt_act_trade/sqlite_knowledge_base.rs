//! SQLite Knowledge Base — 基于 SQLite 的外贸知识库实现
//!
//! 使用 rusqlite 存储产品、供应商及产品配置数据，
//! 支持智能匹配（名称/类别/标签多维评分）和批量导入。

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection};
use serde_json;

use super::knowledge_base::{
    KnowledgeBase, KnowledgeBaseError, KnowledgeOperation, KnowledgeResult, KnowledgeUpdateResult,
    PriceQuery, PriceResult, ProductFilters, ProductMatchEntry, ProductMatchResult, ProductQueryResult,
    ProductRecord, SupplierFilters, SupplierMatchEntry, SupplierMatchResult, SupplierQueryResult,
    SupplierRecord,
};

// ============================================================
// 1. 错误转换
// ============================================================

impl From<rusqlite::Error> for KnowledgeBaseError {
    fn from(e: rusqlite::Error) -> Self {
        KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for KnowledgeBaseError {
    fn from(e: serde_json::Error) -> Self {
        KnowledgeBaseError::InvalidQuery {
            reason: e.to_string(),
        }
    }
}

// ============================================================
// 2. SQLiteKnowledgeBase 核心结构
// ============================================================

/// 基于 SQLite 的外贸知识库
pub struct SqliteKnowledgeBase {
    conn: Mutex<Connection>,
}

impl SqliteKnowledgeBase {
    /// 创建新的 SQLite 知识库实例
    pub fn new(db_path: impl AsRef<Path>) -> KnowledgeResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let kb = Self {
            conn: Mutex::new(conn),
        };
        kb.init_tables()?;
        Ok(kb)
    }

    /// 创建内存知识库（用于测试）
    pub fn new_in_memory() -> KnowledgeResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let kb = Self {
            conn: Mutex::new(conn),
        };
        kb.init_tables()?;
        Ok(kb)
    }

    /// 初始化数据库表结构
    fn init_tables(&self) -> KnowledgeResult<()> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS products (
                product_id  TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                spec_json   TEXT NOT NULL,
                reference_price REAL NOT NULL DEFAULT 0.0,
                moq         INTEGER NOT NULL DEFAULT 0,
                lead_time_days INTEGER NOT NULL DEFAULT 0,
                supplier_ids TEXT NOT NULL DEFAULT '[]',
                tags        TEXT NOT NULL DEFAULT '[]',
                updated_at  INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_products_name ON products(name);
            CREATE INDEX IF NOT EXISTS idx_products_price ON products(reference_price);

            CREATE TABLE IF NOT EXISTS suppliers (
                supplier_id     TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                country         TEXT NOT NULL DEFAULT '',
                product_categories TEXT NOT NULL DEFAULT '[]',
                credit_score    REAL NOT NULL DEFAULT 0.0,
                rating          REAL NOT NULL DEFAULT 0.0,
                monthly_capacity INTEGER NOT NULL DEFAULT 0,
                certified       INTEGER NOT NULL DEFAULT 0,
                certifications  TEXT NOT NULL DEFAULT '[]',
                contact_info    TEXT NOT NULL DEFAULT '{}',
                updated_at      INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_suppliers_name ON suppliers(name);
            CREATE INDEX IF NOT EXISTS idx_suppliers_country ON suppliers(country);

            CREATE TABLE IF NOT EXISTS product_configs (
                config_id   TEXT PRIMARY KEY,
                product_id  TEXT NOT NULL,
                key         TEXT NOT NULL,
                value       TEXT NOT NULL DEFAULT '',
                FOREIGN KEY (product_id) REFERENCES products(product_id) ON DELETE CASCADE
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_config_product_key
                ON product_configs(product_id, key);
            ",
        )?;

        Ok(())
    }

    // ── 导入方法 ────────────────────────────────────────────

    /// 批量导入产品数据
    pub fn import_products(&self, products: &[ProductRecord]) -> KnowledgeResult<u32> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let mut count = 0u32;
        for p in products {
            let spec_json = serde_json::to_string(&p.spec)?;
            let supplier_ids_json = serde_json::to_string(&p.supplier_ids)?;
            let tags_json = serde_json::to_string(&p.tags)?;
            conn.execute(
                "INSERT OR REPLACE INTO products
                 (product_id, name, spec_json, reference_price, moq, lead_time_days, supplier_ids, tags, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    p.product_id,
                    p.name,
                    spec_json,
                    p.reference_price,
                    p.moq as i64,
                    p.lead_time_days as i64,
                    supplier_ids_json,
                    tags_json,
                    p.updated_at as i64,
                ],
            )?;
            count += 1;
        }
        Ok(count)
    }

    /// 批量导入供应商数据
    pub fn import_suppliers(&self, suppliers: &[SupplierRecord]) -> KnowledgeResult<u32> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let mut count = 0u32;
        for s in suppliers {
            let categories_json = serde_json::to_string(&s.product_categories)?;
            let certs_json = serde_json::to_string(&s.certifications)?;
            let contact_json = serde_json::to_string(&s.contact_info)?;
            conn.execute(
                "INSERT OR REPLACE INTO suppliers
                 (supplier_id, name, country, product_categories, credit_score, rating,
                  monthly_capacity, certified, certifications, contact_info, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    s.supplier_id,
                    s.name,
                    s.country,
                    categories_json,
                    s.credit_score,
                    s.rating,
                    s.monthly_capacity as i64,
                    s.certified as i32,
                    certs_json,
                    contact_json,
                    s.updated_at as i64,
                ],
            )?;
            count += 1;
        }
        Ok(count)
    }

    /// 插入产品配置键值对
    pub fn upsert_product_config(
        &self,
        product_id: &str,
        key: &str,
        value: &str,
    ) -> KnowledgeResult<()> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let config_id = format!("{}:{}", product_id, key);
        conn.execute(
            "INSERT OR REPLACE INTO product_configs (config_id, product_id, key, value)
             VALUES (?1, ?2, ?3, ?4)",
            params![config_id, product_id, key, value],
        )?;
        Ok(())
    }

    /// 读取产品所有配置
    pub fn get_product_configs(
        &self,
        product_id: &str,
    ) -> KnowledgeResult<HashMap<String, String>> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let mut stmt = conn.prepare(
            "SELECT key, value FROM product_configs WHERE product_id = ?1",
        )?;
        let rows = stmt.query_map(params![product_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = HashMap::new();
        for r in rows {
            let (k, v) = r?;
            map.insert(k, v);
        }
        Ok(map)
    }

    /// 获取产品总数
    pub fn product_count(&self) -> KnowledgeResult<u64> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM products", [], |r| r.get(0))?;
        Ok(count as u64)
    }

    /// 获取供应商总数
    pub fn supplier_count(&self) -> KnowledgeResult<u64> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM suppliers", [], |r| r.get(0))?;
        Ok(count as u64)
    }

    // ── 内部辅助 ────────────────────────────────────────────

    fn row_to_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductRecord> {
        let spec_json: String = row.get(2)?;
        let supplier_ids_json: String = row.get(6)?;
        let tags_json: String = row.get(7)?;
        Ok(ProductRecord {
            product_id: row.get(0)?,
            name: row.get(1)?,
            spec: serde_json::from_str(&spec_json).unwrap_or_default(),
            reference_price: row.get(3)?,
            moq: row.get::<_, i64>(4)? as u64,
            lead_time_days: row.get::<_, i64>(5)? as u32,
            supplier_ids: serde_json::from_str(&supplier_ids_json).unwrap_or_default(),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            updated_at: row.get::<_, i64>(8)? as u64,
        })
    }

    fn row_to_supplier(row: &rusqlite::Row<'_>) -> rusqlite::Result<SupplierRecord> {
        let categories_json: String = row.get(3)?;
        let certs_json: String = row.get(8)?;
        let contact_json: String = row.get(9)?;
        Ok(SupplierRecord {
            supplier_id: row.get(0)?,
            name: row.get(1)?,
            country: row.get(2)?,
            product_categories: serde_json::from_str(&categories_json).unwrap_or_default(),
            credit_score: row.get(4)?,
            rating: row.get(5)?,
            monthly_capacity: row.get::<_, i64>(6)? as u64,
            certified: row.get::<_, i32>(7)? != 0,
            certifications: serde_json::from_str(&certs_json).unwrap_or_default(),
            contact_info: serde_json::from_str(&contact_json).unwrap_or_default(),
            updated_at: row.get::<_, i64>(10)? as u64,
        })
    }

    /// 简单文本相似度（bigram 重叠率）
    fn text_similarity(a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower == b_lower {
            return 1.0;
        }
        if a_lower.is_empty() || b_lower.is_empty() {
            return 0.0;
        }

        let a_bigrams: Vec<String> = a_lower
            .chars()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| w.iter().collect())
            .collect();
        let b_bigrams: Vec<String> = b_lower
            .chars()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| w.iter().collect())
            .collect();

        if a_bigrams.is_empty() || b_bigrams.is_empty() {
            return if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
                0.5
            } else {
                0.0
            };
        }

        let mut overlap = 0u32;
        for bg in &a_bigrams {
            if b_bigrams.contains(bg) {
                overlap += 1;
            }
        }
        let max_len = a_bigrams.len().max(b_bigrams.len()) as f64;
        overlap as f64 / max_len
    }

    /// 综合匹配评分（名称 + 类别 + 标签多维加权）
    fn compute_product_score(query: &str, product: &ProductRecord) -> (f64, Vec<String>) {
        let mut reasons = Vec::new();
        let mut score = 0.0f64;

        let name_sim = Self::text_similarity(query, &product.name);
        if name_sim > 0.3 {
            score += name_sim * 0.5;
            reasons.push(format!("名称匹配 ({:.0}%)", name_sim * 100.0));
        }

        let spec_desc = format!(
            "{:?} {}",
            product.spec.product_type,
            product.spec.hs_code
        );
        let spec_sim = Self::text_similarity(query, &spec_desc);
        if spec_sim > 0.2 {
            score += spec_sim * 0.3;
            reasons.push(format!("规格匹配 ({:.0}%)", spec_sim * 100.0));
        }

        for tag in &product.tags {
            let tag_sim = Self::text_similarity(query, tag);
            if tag_sim > 0.3 {
                score += tag_sim * 0.2;
                reasons.push(format!("标签匹配: {}", tag));
            }
        }

        score = score.min(1.0);
        (score, reasons)
    }

    /// 供应商综合匹配评分
    fn compute_supplier_score(
        query: &str,
        product_category: Option<&str>,
        supplier: &SupplierRecord,
    ) -> (f64, Vec<String>) {
        let mut reasons = Vec::new();
        let mut score = 0.0f64;

        let name_sim = Self::text_similarity(query, &supplier.name);
        if name_sim > 0.3 {
            score += name_sim * 0.4;
            reasons.push(format!("名称匹配 ({:.0}%)", name_sim * 100.0));
        }

        let country_sim = Self::text_similarity(query, &supplier.country);
        if country_sim > 0.3 {
            score += country_sim * 0.2;
            reasons.push(format!("国家匹配: {}", supplier.country));
        }

        if let Some(cat) = product_category {
            for c in &supplier.product_categories {
                let cat_sim = Self::text_similarity(cat, c);
                if cat_sim > 0.3 {
                    score += cat_sim * 0.2;
                    reasons.push(format!("类别匹配: {}", c));
                }
            }
        }

        if supplier.certified {
            score += 0.1;
            reasons.push("已认证供应商".into());
        }

        if supplier.rating >= 4.0 {
            score += 0.1;
            reasons.push(format!("高评分 ({:.1})", supplier.rating));
        }

        score = score.min(1.0);
        (score, reasons)
    }

    fn now_epoch() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================
// 3. KnowledgeBase trait 实现
// ============================================================

#[async_trait::async_trait]
impl KnowledgeBase for SqliteKnowledgeBase {
    async fn query_products(
        &self,
        filters: &ProductFilters,
    ) -> KnowledgeResult<ProductQueryResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref kw) = filters.name_keyword {
            conditions.push("name LIKE ?".into());
            param_values.push(Box::new(format!("%{}%", kw)));
        }
        if let Some(min_p) = filters.min_price {
            conditions.push("reference_price >= ?".into());
            param_values.push(Box::new(min_p));
        }
        if let Some(max_p) = filters.max_price {
            conditions.push("reference_price <= ?".into());
            param_values.push(Box::new(max_p));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM products {}", where_clause);
        let mut count_stmt = conn.prepare(&count_sql)?;
        let total: i64 = {
            let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();
            count_stmt.query_row(params_refs.as_slice(), |r| r.get(0))?
        };

        let page = filters.page.unwrap_or(0);
        let page_size = filters.page_size.unwrap_or(20).min(100);
        let offset = page * page_size;

        let query_sql = format!(
            "SELECT product_id, name, spec_json, reference_price, moq, lead_time_days,
                    supplier_ids, tags, updated_at
             FROM products {} ORDER BY reference_price ASC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut stmt = conn.prepare(&query_sql)?;

        param_values.push(Box::new(page_size as i64));
        param_values.push(Box::new(offset as i64));
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(params_refs.as_slice(), Self::row_to_product)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }

        Ok(ProductQueryResult {
            items,
            total: total as u64,
            page,
            page_size,
        })
    }

    async fn query_suppliers(
        &self,
        filters: &SupplierFilters,
    ) -> KnowledgeResult<SupplierQueryResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref kw) = filters.name_keyword {
            conditions.push("name LIKE ?".into());
            param_values.push(Box::new(format!("%{}%", kw)));
        }
        if let Some(ref c) = filters.country {
            conditions.push("country = ?".into());
            param_values.push(Box::new(c.clone()));
        }
        if let Some(min_r) = filters.min_rating {
            conditions.push("rating >= ?".into());
            param_values.push(Box::new(min_r));
        }
        if filters.certified_only {
            conditions.push("certified = 1".into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let count_sql = format!("SELECT COUNT(*) FROM suppliers {}", where_clause);
        let mut count_stmt = conn.prepare(&count_sql)?;
        let total: i64 = {
            let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();
            count_stmt.query_row(params_refs.as_slice(), |r| r.get(0))?
        };

        let page = filters.page.unwrap_or(0);
        let page_size = filters.page_size.unwrap_or(20).min(100);
        let offset = page * page_size;

        let query_sql = format!(
            "SELECT supplier_id, name, country, product_categories, credit_score, rating,
                    monthly_capacity, certified, certifications, contact_info, updated_at
             FROM suppliers {} ORDER BY rating DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut stmt = conn.prepare(&query_sql)?;

        param_values.push(Box::new(page_size as i64));
        param_values.push(Box::new(offset as i64));
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(params_refs.as_slice(), Self::row_to_supplier)?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }

        Ok(SupplierQueryResult {
            items,
            total: total as u64,
            page,
            page_size,
        })
    }

    async fn match_product(
        &self,
        query: &str,
        limit: u32,
    ) -> KnowledgeResult<ProductMatchResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let mut stmt = conn.prepare(
            "SELECT product_id, name, spec_json, reference_price, moq, lead_time_days,
                    supplier_ids, tags, updated_at
             FROM products",
        )?;

        let all_rows = stmt.query_map([], Self::row_to_product)?;

        let mut exact = Vec::new();
        let mut fuzzy_entries: Vec<ProductMatchEntry> = Vec::new();

        for row_result in all_rows {
            let product = row_result?;
            let query_lower = query.to_lowercase();
            let name_lower = product.name.to_lowercase();

            if name_lower == query_lower || product.product_id == query {
                exact.push(product);
            } else {
                let (score, reasons) = Self::compute_product_score(query, &product);
                if score > 0.1 {
                    fuzzy_entries.push(ProductMatchEntry {
                        product,
                        score,
                        match_reasons: reasons,
                    });
                }
            }
        }

        fuzzy_entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        fuzzy_entries.truncate(limit as usize);

        Ok(ProductMatchResult {
            exact,
            fuzzy: fuzzy_entries,
            query_summary: format!("查询: \"{}\"", query),
        })
    }

    async fn match_supplier(
        &self,
        query: &str,
        product_category: Option<&str>,
        limit: u32,
    ) -> KnowledgeResult<SupplierMatchResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let mut stmt = conn.prepare(
            "SELECT supplier_id, name, country, product_categories, credit_score, rating,
                    monthly_capacity, certified, certifications, contact_info, updated_at
             FROM suppliers",
        )?;

        let all_rows = stmt.query_map([], Self::row_to_supplier)?;

        let mut exact = Vec::new();
        let mut fuzzy_entries: Vec<SupplierMatchEntry> = Vec::new();

        for row_result in all_rows {
            let supplier = row_result?;
            let query_lower = query.to_lowercase();
            let name_lower = supplier.name.to_lowercase();

            if name_lower == query_lower || supplier.supplier_id == query {
                exact.push(supplier);
            } else {
                let (score, reasons) =
                    Self::compute_supplier_score(query, product_category, &supplier);
                if score > 0.1 {
                    fuzzy_entries.push(SupplierMatchEntry {
                        supplier,
                        score,
                        match_reasons: reasons,
                    });
                }
            }
        }

        fuzzy_entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        fuzzy_entries.truncate(limit as usize);

        Ok(SupplierMatchResult {
            exact,
            fuzzy: fuzzy_entries,
            query_summary: format!("查询: \"{}\"", query),
        })
    }

    async fn get_price(&self, query: &PriceQuery) -> KnowledgeResult<PriceResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let product: ProductRecord = conn
            .query_row(
                "SELECT product_id, name, spec_json, reference_price, moq, lead_time_days,
                        supplier_ids, tags, updated_at
                 FROM products WHERE product_id = ?1",
                params![query.product_id],
                Self::row_to_product,
            )
            .map_err(|_| KnowledgeBaseError::NotFound {
                entity: "Product".into(),
                id: query.product_id.clone(),
            })?;

        let unit_price = if query.quantity >= product.moq {
            product.reference_price
        } else {
            product.reference_price * 1.2
        };

        let total_price = unit_price * query.quantity as f64;

        Ok(PriceResult {
            product_id: query.product_id.clone(),
            supplier_id: query.supplier_id.clone(),
            unit_price,
            total_price,
            currency: query.currency.clone(),
            quantity: query.quantity,
            lead_time_days: product.lead_time_days,
            additional_costs: HashMap::new(),
            valid_until: Self::now_epoch() + 30 * 86400,
            notes: vec![],
        })
    }

    async fn update_knowledge(
        &self,
        operations: &[KnowledgeOperation],
    ) -> KnowledgeResult<KnowledgeUpdateResult> {
        let conn = self.conn.lock().map_err(|e| KnowledgeBaseError::Unavailable {
            reason: e.to_string(),
        })?;

        let mut affected = 0u32;
        let mut errors = Vec::new();

        for op in operations {
            match op {
                KnowledgeOperation::AddProduct(p) => {
                    let spec_json = serde_json::to_string(&p.spec)?;
                    let supplier_ids_json = serde_json::to_string(&p.supplier_ids)?;
                    let tags_json = serde_json::to_string(&p.tags)?;
                    match conn.execute(
                        "INSERT OR REPLACE INTO products
                         (product_id, name, spec_json, reference_price, moq, lead_time_days,
                          supplier_ids, tags, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            p.product_id,
                            p.name,
                            spec_json,
                            p.reference_price,
                            p.moq as i64,
                            p.lead_time_days as i64,
                            supplier_ids_json,
                            tags_json,
                            p.updated_at as i64,
                        ],
                    ) {
                        Ok(_) => affected += 1,
                        Err(e) => errors.push(format!("AddProduct {}: {}", p.product_id, e)),
                    }
                }
                KnowledgeOperation::UpdateProduct {
                    product_id,
                    changes,
                } => {
                    for (key, value) in changes {
                        let sql = format!(
                            "UPDATE products SET {} = ?1 WHERE product_id = ?2",
                            match key.as_str() {
                                "name" | "reference_price" | "moq" | "lead_time_days"
                                | "tags" | "supplier_ids" => key.as_str(),
                                _ => continue,
                            }
                        );
                        match conn.execute(&sql, params![value, product_id]) {
                            Ok(n) => affected += n as u32,
                            Err(e) => {
                                errors.push(format!("UpdateProduct {}: {}", product_id, e))
                            }
                        }
                    }
                }
                KnowledgeOperation::DeleteProduct(pid) => {
                    match conn.execute("DELETE FROM products WHERE product_id = ?1", params![pid])
                    {
                        Ok(n) => affected += n as u32,
                        Err(e) => errors.push(format!("DeleteProduct {}: {}", pid, e)),
                    }
                }
                KnowledgeOperation::AddSupplier(s) => {
                    let categories_json = serde_json::to_string(&s.product_categories)?;
                    let certs_json = serde_json::to_string(&s.certifications)?;
                    let contact_json = serde_json::to_string(&s.contact_info)?;
                    match conn.execute(
                        "INSERT OR REPLACE INTO suppliers
                         (supplier_id, name, country, product_categories, credit_score, rating,
                          monthly_capacity, certified, certifications, contact_info, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                        params![
                            s.supplier_id,
                            s.name,
                            s.country,
                            categories_json,
                            s.credit_score,
                            s.rating,
                            s.monthly_capacity as i64,
                            s.certified as i32,
                            certs_json,
                            contact_json,
                            s.updated_at as i64,
                        ],
                    ) {
                        Ok(_) => affected += 1,
                        Err(e) => errors.push(format!("AddSupplier {}: {}", s.supplier_id, e)),
                    }
                }
                KnowledgeOperation::UpdateSupplier {
                    supplier_id,
                    changes,
                } => {
                    for (key, value) in changes {
                        let sql = format!(
                            "UPDATE suppliers SET {} = ?1 WHERE supplier_id = ?2",
                            match key.as_str() {
                                "name" | "country" | "credit_score" | "rating"
                                | "monthly_capacity" | "certified" => key.as_str(),
                                _ => continue,
                            }
                        );
                        match conn.execute(&sql, params![value, supplier_id]) {
                            Ok(n) => affected += n as u32,
                            Err(e) => {
                                errors.push(format!("UpdateSupplier {}: {}", supplier_id, e))
                            }
                        }
                    }
                }
                KnowledgeOperation::DeleteSupplier(sid) => {
                    match conn.execute(
                        "DELETE FROM suppliers WHERE supplier_id = ?1",
                        params![sid],
                    ) {
                        Ok(n) => affected += n as u32,
                        Err(e) => errors.push(format!("DeleteSupplier {}: {}", sid, e)),
                    }
                }
                KnowledgeOperation::BulkImport {
                    products,
                    suppliers,
                } => {
                    for p in products {
                        let spec_json = serde_json::to_string(&p.spec)?;
                        let supplier_ids_json = serde_json::to_string(&p.supplier_ids)?;
                        let tags_json = serde_json::to_string(&p.tags)?;
                        if let Err(e) = conn.execute(
                            "INSERT OR REPLACE INTO products
                             (product_id, name, spec_json, reference_price, moq, lead_time_days,
                              supplier_ids, tags, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![
                                p.product_id,
                                p.name,
                                spec_json,
                                p.reference_price,
                                p.moq as i64,
                                p.lead_time_days as i64,
                                supplier_ids_json,
                                tags_json,
                                p.updated_at as i64,
                            ],
                        ) {
                            errors.push(format!("BulkImport product {}: {}", p.product_id, e));
                        } else {
                            affected += 1;
                        }
                    }
                    for s in suppliers {
                        let categories_json = serde_json::to_string(&s.product_categories)?;
                        let certs_json = serde_json::to_string(&s.certifications)?;
                        let contact_json = serde_json::to_string(&s.contact_info)?;
                        if let Err(e) = conn.execute(
                            "INSERT OR REPLACE INTO suppliers
                             (supplier_id, name, country, product_categories, credit_score, rating,
                              monthly_capacity, certified, certifications, contact_info, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                            params![
                                s.supplier_id,
                                s.name,
                                s.country,
                                categories_json,
                                s.credit_score,
                                s.rating,
                                s.monthly_capacity as i64,
                                s.certified as i32,
                                certs_json,
                                contact_json,
                                s.updated_at as i64,
                            ],
                        ) {
                            errors.push(format!("BulkImport supplier {}: {}", s.supplier_id, e));
                        } else {
                            affected += 1;
                        }
                    }
                }
            }
        }

        Ok(KnowledgeUpdateResult {
            success: errors.is_empty(),
            affected_count: affected,
            errors,
            updated_at: Self::now_epoch(),
        })
    }
}

// ============================================================
// 4. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_act::nt_act_trade::full_cycle::{BomItem, PackagingSpec, ProductSpec, ProductType, RoutingStep};
    use crate::l1_action::nt_act::nt_act_trade::knowledge_base::{ProductFilters, SupplierFilters};

    fn sample_product(id: &str, name: &str) -> ProductRecord {
        ProductRecord {
            product_id: id.into(),
            name: name.into(),
            spec: ProductSpec {
                spec_id: format!("SP-{}", id),
                product_type: ProductType::Electronics,
                bom: vec![],
                routing: vec![],
                packaging: PackagingSpec {
                    package_type: "box".into(),
                    dimensions_cm: (10.0, 10.0, 10.0),
                    gross_weight_kg: 1.0,
                    net_weight_kg: 0.8,
                    marks: vec![],
                },
                certifications: vec![],
                hs_code: "8541.00".into(),
                tax_refund_rate: 0.13,
            },
            reference_price: 5.0,
            moq: 100,
            lead_time_days: 14,
            supplier_ids: vec!["S-001".into()],
            tags: vec!["solar".into(), "panel".into()],
            updated_at: 1700000000,
        }
    }

    fn sample_supplier(id: &str, name: &str, country: &str) -> SupplierRecord {
        SupplierRecord {
            supplier_id: id.into(),
            name: name.into(),
            country: country.into(),
            product_categories: vec!["Electronics".into()],
            credit_score: 0.9,
            rating: 4.5,
            monthly_capacity: 10000,
            certified: true,
            certifications: vec!["ISO9001".into()],
            contact_info: {
                let mut m = HashMap::new();
                m.insert("email".into(), "test@example.com".into());
                m
            },
            updated_at: 1700000000,
        }
    }

    #[test]
    fn test_in_memory_creation_and_tables() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        assert_eq!(kb.product_count().unwrap(), 0);
        assert_eq!(kb.supplier_count().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_import_and_query_products() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        let products = vec![
            sample_product("P-001", "Solar Panel 300W"),
            sample_product("P-002", "Wind Turbine 5kW"),
        ];
        let count = kb.import_products(&products).unwrap();
        assert_eq!(count, 2);
        assert_eq!(kb.product_count().unwrap(), 2);

        let result = kb
            .query_products(&ProductFilters {
                name_keyword: Some("Solar".into()),
                page_size: Some(10),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].product_id, "P-001");
    }

    #[tokio::test]
    async fn test_import_and_query_suppliers() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        let suppliers = vec![
            sample_supplier("S-001", "Shenzhen Solar Co", "China"),
            sample_supplier("S-002", "Berlin Wind GmbH", "Germany"),
        ];
        let count = kb.import_suppliers(&suppliers).unwrap();
        assert_eq!(count, 2);

        let result = kb
            .query_suppliers(&SupplierFilters {
                country: Some("China".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].supplier_id, "S-001");
    }

    #[tokio::test]
    async fn test_product_match_exact() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        kb.import_products(&[sample_product("P-001", "Solar Panel 300W")])
            .unwrap();

        let result = kb.match_product("Solar Panel 300W", 10).await.unwrap();
        assert_eq!(result.exact.len(), 1);
        assert_eq!(result.exact[0].product_id, "P-001");
    }

    #[tokio::test]
    async fn test_product_match_fuzzy() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        kb.import_products(&[
            sample_product("P-001", "Solar Panel 300W"),
            sample_product("P-002", "Wind Turbine 5kW"),
        ])
        .unwrap();

        let result = kb.match_product("solar", 10).await.unwrap();
        assert!(result.fuzzy.len() >= 1);
        assert!(result.fuzzy[0].score > 0.0);
    }

    #[tokio::test]
    async fn test_supplier_match() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        kb.import_suppliers(&[
            sample_supplier("S-001", "Shenzhen Solar Co", "China"),
            sample_supplier("S-002", "Berlin Wind GmbH", "Germany"),
        ])
        .unwrap();

        let result = kb
            .match_supplier("Solar", Some("Electronics"), 10)
            .await
            .unwrap();
        assert!(result.exact.len() + result.fuzzy.len() >= 1);
    }

    #[tokio::test]
    async fn test_price_query() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        kb.import_products(&[sample_product("P-001", "Solar Panel 300W")])
            .unwrap();

        let price = kb
            .get_price(&PriceQuery {
                product_id: "P-001".into(),
                supplier_id: None,
                quantity: 200,
                currency: "USD".into(),
                destination_country: None,
            })
            .await
            .unwrap();
        assert_eq!(price.unit_price, 5.0);
        assert_eq!(price.total_price, 1000.0);
    }

    #[tokio::test]
    async fn test_update_knowledge_add_and_delete() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        let result = kb
            .update_knowledge(&[KnowledgeOperation::AddProduct(
                sample_product("P-001", "Test"),
            )])
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.affected_count, 1);
        assert_eq!(kb.product_count().unwrap(), 1);

        let result = kb
            .update_knowledge(&[KnowledgeOperation::DeleteProduct("P-001".into())])
            .await
            .unwrap();
        assert_eq!(result.affected_count, 1);
        assert_eq!(kb.product_count().unwrap(), 0);
    }

    #[test]
    fn test_product_configs() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        kb.import_products(&[sample_product("P-001", "Test")])
            .unwrap();
        kb.upsert_product_config("P-001", "color", "red").unwrap();
        kb.upsert_product_config("P-001", "size", "XL").unwrap();

        let configs = kb.get_product_configs("P-001").unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs.get("color").unwrap(), "red");
        assert_eq!(configs.get("size").unwrap(), "XL");
    }

    #[test]
    fn test_text_similarity() {
        assert_eq!(SqliteKnowledgeBase::text_similarity("", ""), 0.0);
        assert_eq!(SqliteKnowledgeBase::text_similarity("abc", "abc"), 1.0);
        assert!(SqliteKnowledgeBase::text_similarity("abc", "abd") > 0.5);
        assert!(SqliteKnowledgeBase::text_similarity("abc", "xyz") < 0.1);
    }

    #[tokio::test]
    async fn test_bulk_import_via_update() {
        let kb = SqliteKnowledgeBase::new_in_memory().unwrap();
        let result = kb
            .update_knowledge(&[KnowledgeOperation::BulkImport {
                products: vec![
                    sample_product("P-001", "A"),
                    sample_product("P-002", "B"),
                ],
                suppliers: vec![sample_supplier("S-001", "X", "CN")],
            }])
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.affected_count, 3);
        assert_eq!(kb.product_count().unwrap(), 2);
        assert_eq!(kb.supplier_count().unwrap(), 1);
    }
}
