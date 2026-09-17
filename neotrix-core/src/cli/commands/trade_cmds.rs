//! Trade CLI Commands — 外贸知识库查询命令
//!
//! 提供对导入的富通天下 CRM 数据的 CLI 查询接口。

use crate::cli::commands::types::{CliCommand, CommandOutput};

use crate::l5_cognition::nt_mind::nt_mind::SelfIteratingBrain;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// ── 工具函数 ──────────────────────────────────────────────────

fn trade_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".neotrix").join("trade_knowledge.db")
}

fn open_trade_db() -> Result<Connection, String> {
    let path = trade_db_path();
    if !path.exists() {
        return Err(format!("Trade knowledge database not found: {}", path.display()));
    }
    Connection::open(&path).map_err(|e| format!("Failed to open trade database: {}", e))
}

// ── /trade command ────────────────────────────────────────────

pub struct TradeCmd;
impl CliCommand for TradeCmd {
    fn name(&self) -> &str {
        "/trade"
    }
    fn aliases(&self) -> Vec<&str> {
        vec!["/crm", "/futong"]
    }
    fn description(&self) -> &str {
        "Trade CRM operations: /trade stats | /trade search <query> | /trade country <country> | /trade grade <grade> | /trade followup [days] | /trade operator <name>"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        if args.is_empty() {
            return CommandOutput::text(self.description());
        }

        let subcmd = args[0].as_str();
        let params = &args[1..];

        match subcmd {
            "stats" => self.cmd_stats().unwrap_or_else(|e| CommandOutput::err(&e)),
            "search" => self.cmd_search(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            "country" => self.cmd_country(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            "grade" => self.cmd_grade(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            "followup" => self.cmd_followup(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            "operator" => self.cmd_operator(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            "product" => self.cmd_product(params).unwrap_or_else(|e| CommandOutput::err(&e)),
            _ => CommandOutput::text(format!(
                "Unknown subcommand: {}. Use: stats, search, country, grade, followup, operator, product",
                subcmd
            )),
        }
    }
}

impl TradeCmd {
    fn cmd_stats(&self) -> Result<CommandOutput, String> {
        let conn = open_trade_db()?;

        let total: u64 = conn
            .query_row("SELECT COUNT(*) FROM customers", [], |r| r.get(0))
            .map_err(|e| format!("Query error: {}", e))?;

        let grade_dist: Vec<(String, u64)> = conn
            .prepare("SELECT grade, COUNT(*) FROM customers GROUP BY grade ORDER BY COUNT(*) DESC")
            .map_err(|e| format!("Prepare error: {}", e))?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let country_dist: Vec<(String, u64)> = conn
            .prepare("SELECT country, COUNT(*) FROM customers WHERE country != '' GROUP BY country ORDER BY COUNT(*) DESC LIMIT 10")
            .map_err(|e| format!("Prepare error: {}", e))?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let operator_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM operators", [], |r| r.get(0))
            .map_err(|e| format!("Query error: {}", e))?;

        let product_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM product_categories", [], |r| r.get(0))
            .map_err(|e| format!("Query error: {}", e))?;

        let country_count: u64 = conn
            .query_row("SELECT COUNT(*) FROM countries", [], |r| r.get(0))
            .map_err(|e| format!("Query error: {}", e))?;

        let mut output = String::new();
        output.push_str(&format!("📊 Trade Knowledge Base Stats\n"));
        output.push_str(&format!("═══════════════════════════════════════\n"));
        output.push_str(&format!("Customers:      {}\n", total));
        output.push_str(&format!("Operators:      {}\n", operator_count));
        output.push_str(&format!("Products:       {}\n", product_count));
        output.push_str(&format!("Countries:      {}\n", country_count));
        output.push_str(&format!("\n📈 Grade Distribution:\n"));
        for (grade, count) in &grade_dist {
            let pct = (*count as f64 / total as f64 * 100.0) as u32;
            output.push_str(&format!("  {:<4} {:<6} ({}%)\n", grade, count, pct));
        }
        output.push_str(&format!("\n🌍 Top Countries:\n"));
        for (country, count) in &country_dist {
            output.push_str(&format!("  {:<20} {}\n", country, count));
        }

        Ok(CommandOutput::text(output))
    }

    fn cmd_search(&self, params: &[String]) -> Result<CommandOutput, String> {
        if params.is_empty() {
            return Err("Usage: /trade search <query>".to_string());
        }
        let query = params.join(" ");
        let conn = open_trade_db()?;

        let mut stmt = conn
            .prepare(
                "SELECT customer_id, name, grade, country, owner, contact_name, channel
                 FROM customers
                 WHERE name LIKE ?1 OR contact_name LIKE ?1 OR description LIKE ?1
                 ORDER BY grade DESC, last_follow_at DESC
                 LIMIT 20",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", query);
        let rows: Vec<String> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let grade: String = row.get(2)?;
                let country: String = row.get(3)?;
                let owner: String = row.get(4)?;
                let contact: String = row.get(5)?;
                let channel: String = row.get(6)?;
                Ok(format!(
                    "[{}] {} | Grade:{} | {} | Contact:{} | Owner:{} | {}",
                    id, name, grade, country, contact, owner, channel
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            Ok(CommandOutput::text(format!("No customers found matching '{}'", query)))
        } else {
            let mut output = format!("🔍 Search Results for '{}' ({} found):\n\n", query, rows.len());
            for row in rows {
                output.push_str(&format!("{}\n", row));
            }
            Ok(CommandOutput::text(output))
        }
    }

    fn cmd_country(&self, params: &[String]) -> Result<CommandOutput, String> {
        if params.is_empty() {
            return Err("Usage: /trade country <country>".to_string());
        }
        let country = params.join(" ");
        let conn = open_trade_db()?;

        let mut stmt = conn
            .prepare(
                "SELECT customer_id, name, grade, contact_name, owner, channel
                 FROM customers
                 WHERE country LIKE ?1 OR region LIKE ?1
                 ORDER BY grade DESC
                 LIMIT 20",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", country.to_uppercase());
        let rows: Vec<String> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let grade: String = row.get(2)?;
                let contact: String = row.get(3)?;
                let owner: String = row.get(4)?;
                let channel: String = row.get(5)?;
                Ok(format!(
                    "[{}] {} | Grade:{} | Contact:{} | Owner:{} | {}",
                    id, name, grade, contact, owner, channel
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            Ok(CommandOutput::text(format!("No customers found in '{}'", country)))
        } else {
            let mut output = format!("🌍 Customers in '{}' ({} found):\n\n", country, rows.len());
            for row in rows {
                output.push_str(&format!("{}\n", row));
            }
            Ok(CommandOutput::text(output))
        }
    }

    fn cmd_grade(&self, params: &[String]) -> Result<CommandOutput, String> {
        if params.is_empty() {
            return Err("Usage: /trade grade <A|B|C|D|E|S|VIP>".to_string());
        }
        let grade = params[0].to_uppercase();
        let conn = open_trade_db()?;

        let mut stmt = conn
            .prepare(
                "SELECT customer_id, name, country, contact_name, owner, channel
                 FROM customers
                 WHERE grade = ?1
                 ORDER BY last_follow_at DESC
                 LIMIT 20",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let rows: Vec<String> = stmt
            .query_map(rusqlite::params![grade], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let country: String = row.get(2)?;
                let contact: String = row.get(3)?;
                let owner: String = row.get(4)?;
                let channel: String = row.get(5)?;
                Ok(format!(
                    "[{}] {} | {} | Contact:{} | Owner:{} | {}",
                    id, name, country, contact, owner, channel
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let total: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM customers WHERE grade = ?1",
                rusqlite::params![grade],
                |r| r.get(0),
            )
            .map_err(|e| format!("Query error: {}", e))?;

        let mut output = format!("📋 Grade {} Customers ({} total, showing {}):\n\n", grade, total, rows.len());
        for row in rows {
            output.push_str(&format!("{}\n", row));
        }
        Ok(CommandOutput::text(output))
    }

    fn cmd_followup(&self, params: &[String]) -> Result<CommandOutput, String> {
        let days: u64 = params
            .first()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let conn = open_trade_db()?;
        let cutoff = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64)
            - (days as i64 * 86400);

        let mut stmt = conn
            .prepare(
                "SELECT customer_id, name, grade, country, contact_name, owner,
                        last_follow_at, last_activity
                 FROM customers
                 WHERE (last_follow_at < ?1 OR last_follow_at = 0)
                 ORDER BY grade DESC, last_follow_at ASC
                 LIMIT 20",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let rows: Vec<String> = stmt
            .query_map(rusqlite::params![cutoff], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let grade: String = row.get(2)?;
                let country: String = row.get(3)?;
                let contact: String = row.get(4)?;
                let owner: String = row.get(5)?;
                let last_follow: i64 = row.get(6)?;
                let last_activity: String = row.get(7)?;

                let days_ago = if last_follow > 0 {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    (now - last_follow) / 86400
                } else {
                    -1
                };

                Ok(format!(
                    "[{}] {} | Grade:{} | {} | {} | {} | LastFollow:{}d ago | {}",
                    id, name, grade, country, contact, owner, days_ago, last_activity
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let total: u64 = conn
            .query_row(
                "SELECT COUNT(*) FROM customers WHERE last_follow_at < ?1 OR last_follow_at = 0",
                rusqlite::params![cutoff],
                |r| r.get(0),
            )
            .map_err(|e| format!("Query error: {}", e))?;

        let mut output = format!("⏰ Customers needing follow-up (>{} days, {} total):\n\n", days, total);
        for row in rows {
            output.push_str(&format!("{}\n", row));
        }
        Ok(CommandOutput::text(output))
    }

    fn cmd_operator(&self, params: &[String]) -> Result<CommandOutput, String> {
        if params.is_empty() {
            return Err("Usage: /trade operator <name>".to_string());
        }
        let name = params.join(" ");
        let conn = open_trade_db()?;

        // First find the operator
        let mut stmt = conn
            .prepare(
                "SELECT operator_id, name, english_name, login_name
                 FROM operators
                 WHERE name LIKE ?1 OR english_name LIKE ?1 OR login_name LIKE ?1
                 LIMIT 5",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", name);
        let operators: Vec<(String, String, String, String)> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                ))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if operators.is_empty() {
            return Ok(CommandOutput::text(format!("No operators found matching '{}'", name)));
        }

        let mut output = format!("👤 Operators matching '{}':\n\n", name);
        for (id, op_name, eng_name, login) in &operators {
            output.push_str(&format!("  {} ({}) - {} [{}]\n", op_name, eng_name, login, id));
        }

        // Show customer count for first operator
        if let Some((id, op_name, _, _)) = operators.first() {
            let count: u64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM customers WHERE owner_id = ?1",
                    rusqlite::params![id],
                    |r| r.get(0),
                )
                .map_err(|e| format!("Query error: {}", e))?;

            output.push_str(&format!("\n📊 {} manages {} customers\n", op_name, count));

            // Show grade distribution for this operator
            let grade_dist: Vec<(String, u64)> = conn
                .prepare(
                    "SELECT grade, COUNT(*) FROM customers WHERE owner_id = ?1 GROUP BY grade ORDER BY COUNT(*) DESC",
                )
                .map_err(|e| format!("Prepare error: {}", e))?
                .query_map(rusqlite::params![id], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| format!("Query error: {}", e))?
                .filter_map(|r| r.ok())
                .collect();

            output.push_str(&format!("   Grade distribution: "));
            for (grade, count) in grade_dist {
                output.push_str(&format!("{}:{} ", grade, count));
            }
        }

        Ok(CommandOutput::text(output))
    }

    fn cmd_product(&self, params: &[String]) -> Result<CommandOutput, String> {
        if params.is_empty() {
            return Err("Usage: /trade product <query>".to_string());
        }
        let query = params.join(" ");
        let conn = open_trade_db()?;

        let mut stmt = conn
            .prepare(
                "SELECT id, cname, ename
                 FROM product_categories
                 WHERE cname LIKE ?1 OR ename LIKE ?1
                 LIMIT 20",
            )
            .map_err(|e| format!("Prepare error: {}", e))?;

        let pattern = format!("%{}%", query);
        let rows: Vec<(i64, String, String)> = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| format!("Query error: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            Ok(CommandOutput::text(format!("No products found matching '{}'", query)))
        } else {
            let mut output = format!("📦 Products matching '{}' ({} found):\n\n", query, rows.len());
            for (id, cname, ename) in rows {
                output.push_str(&format!("  [{}] {} {}\n", id, cname, ename));
            }
            Ok(CommandOutput::text(output))
        }
    }
}
